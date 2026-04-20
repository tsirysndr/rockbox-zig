/// AirPlay 2 PAIR-SETUP (HAP SRP6a, M1-M6).
///
/// This is only needed once per device; after completion the server stores
/// the client's Ed25519 long-term public key and subsequent connections only
/// need PAIR-VERIFY.

use std::io;

use chacha20poly1305::{
    aead::{Aead, KeyInit},
    ChaCha20Poly1305, Key, Nonce,
};
use ed25519_dalek::{Signer, SigningKey};
use hkdf::Hkdf;
use sha2::Sha512;

use super::srp::SrpClient;
use super::tlv8::{self, types::*, Tlv8Item};
use super::http::http_post_tlv8;

pub fn pair_setup(
    host: &str,
    port: u16,
    pin: &str,
    device_id: &str,
    signing_key: &SigningKey,
) -> io::Result<()> {
    // --- M1: Start SRP ---
    let m1 = tlv8::encode(&[
        Tlv8Item { typ: STATE, value: vec![0x01] },
        Tlv8Item { typ: METHOD, value: vec![0x00] },
    ]);
    tracing::debug!("PAIR-SETUP M1 → {}:{}", host, port);
    let resp_m2 = http_post_tlv8(host, port, "/pair-setup", &m1)?;
    let items_m2 = tlv8::decode(&resp_m2);
    check_state(&items_m2, 2, "PAIR-SETUP M2")?;

    let server_pub = tlv8::find(&items_m2, PUBLIC_KEY)
        .ok_or_else(|| err("M2: missing PublicKey"))?
        .to_vec();
    let salt = tlv8::find(&items_m2, SALT)
        .ok_or_else(|| err("M2: missing Salt"))?
        .to_vec();

    // --- M3: Send SRP client key + proof ---
    let srp = SrpClient::new();
    let (m1_proof, session_key) = srp
        .compute("Pair-Setup", pin, &salt, &server_pub)
        .ok_or_else(|| err("SRP compute failed"))?;

    let m3 = tlv8::encode(&[
        Tlv8Item { typ: STATE, value: vec![0x03] },
        Tlv8Item { typ: PUBLIC_KEY, value: srp.a_pub.clone() },
        Tlv8Item { typ: PROOF, value: m1_proof.clone() },
    ]);
    tracing::debug!("PAIR-SETUP M3 → {}:{}", host, port);
    let resp_m4 = http_post_tlv8(host, port, "/pair-setup", &m3)?;
    let items_m4 = tlv8::decode(&resp_m4);
    check_state(&items_m4, 4, "PAIR-SETUP M4")?;

    let server_proof = tlv8::find(&items_m4, PROOF)
        .ok_or_else(|| err("M4: missing server Proof"))?;
    if !SrpClient::verify_server(&srp.a_pub, &m1_proof, &session_key, server_proof) {
        return Err(err("M4: server proof verification failed"));
    }
    tracing::debug!("SRP verified");

    // --- M5: Send encrypted client identity ---
    let encrypt_key = derive_key(&session_key, b"Pair-Setup-Encrypt-Salt", b"Pair-Setup-Encrypt-Info");

    let lt_pub = signing_key.verifying_key();
    let sig_msg: Vec<u8> = [
        derive_key(&session_key, b"Pair-Setup-Controller-Sign-Salt", b"Pair-Setup-Controller-Sign-Info").as_slice(),
        device_id.as_bytes(),
        lt_pub.as_bytes(),
    ]
    .concat();
    let signature = signing_key.sign(&sig_msg);

    let m5_inner = tlv8::encode(&[
        Tlv8Item { typ: IDENTIFIER, value: device_id.as_bytes().to_vec() },
        Tlv8Item { typ: PUBLIC_KEY, value: lt_pub.as_bytes().to_vec() },
        Tlv8Item { typ: SIGNATURE, value: signature.to_bytes().to_vec() },
    ]);
    let encrypted_m5 = chacha_encrypt(&encrypt_key, b"PS-Msg05", &m5_inner)?;

    let m5 = tlv8::encode(&[
        Tlv8Item { typ: STATE, value: vec![0x05] },
        Tlv8Item { typ: ENCRYPTED_DATA, value: encrypted_m5 },
    ]);
    tracing::debug!("PAIR-SETUP M5 → {}:{}", host, port);
    let resp_m6 = http_post_tlv8(host, port, "/pair-setup", &m5)?;
    let items_m6 = tlv8::decode(&resp_m6);
    check_state(&items_m6, 6, "PAIR-SETUP M6")?;

    let encrypted_m6 = tlv8::find(&items_m6, ENCRYPTED_DATA)
        .ok_or_else(|| err("M6: missing EncryptedData"))?;
    let m6_inner = chacha_decrypt(&encrypt_key, b"PS-Msg06", encrypted_m6)?;
    let items_m6_inner = tlv8::decode(&m6_inner);

    // Store server's long-term public key for future PAIR-VERIFY signature checks
    if let (Some(server_id), Some(server_ltpk)) = (
        tlv8::find(&items_m6_inner, IDENTIFIER),
        tlv8::find(&items_m6_inner, PUBLIC_KEY),
    ) {
        save_server_ltpk(server_id, server_ltpk);
    }

    tracing::info!("PAIR-SETUP complete — device paired");
    Ok(())
}

fn derive_key(ikm: &[u8], salt: &[u8], info: &[u8]) -> Vec<u8> {
    let hk = Hkdf::<Sha512>::new(Some(salt), ikm);
    let mut okm = vec![0u8; 32];
    hk.expand(info, &mut okm).expect("hkdf expand");
    okm
}

fn chacha_encrypt(key: &[u8], nonce_prefix: &[u8; 8], plaintext: &[u8]) -> io::Result<Vec<u8>> {
    let cipher = ChaCha20Poly1305::new(Key::from_slice(key));
    let mut nonce_bytes = [0u8; 12];
    nonce_bytes[4..].copy_from_slice(nonce_prefix);
    let nonce = Nonce::from_slice(&nonce_bytes);
    cipher.encrypt(nonce, plaintext).map_err(|_| err("encrypt failed"))
}

fn chacha_decrypt(key: &[u8], nonce_prefix: &[u8; 8], ciphertext: &[u8]) -> io::Result<Vec<u8>> {
    let cipher = ChaCha20Poly1305::new(Key::from_slice(key));
    let mut nonce_bytes = [0u8; 12];
    nonce_bytes[4..].copy_from_slice(nonce_prefix);
    let nonce = Nonce::from_slice(&nonce_bytes);
    cipher.decrypt(nonce, ciphertext).map_err(|_| err("decrypt failed"))
}

fn check_state(items: &[tlv8::Tlv8Item], expected: u8, ctx: &str) -> io::Result<()> {
    if let Some(state) = tlv8::find(items, STATE) {
        if state.first() == Some(&expected) {
            return Ok(());
        }
    }
    if let Some(e) = tlv8::find(items, ERROR) {
        return Err(err(&format!("{}: error code {:?}", ctx, e)));
    }
    Err(err(&format!("{}: unexpected state", ctx)))
}

fn err(msg: &str) -> io::Error {
    io::Error::new(io::ErrorKind::Other, msg)
}

fn save_server_ltpk(server_id: &[u8], ltpk: &[u8]) {
    let id_hex: String = server_id.iter().map(|b| format!("{:02x}", b)).collect();
    let dir = dirs::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join(".config/rockbox");
    let _ = std::fs::create_dir_all(&dir);
    let path = dir.join(format!("airplay_server_{}.ltpk", id_hex));
    let _ = std::fs::write(&path, ltpk);
}
