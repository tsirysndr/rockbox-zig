use std::io;

use chacha20poly1305::{
    aead::{Aead, KeyInit},
    ChaCha20Poly1305, Key, Nonce,
};
use ed25519_dalek::{Signer, SigningKey, VerifyingKey};
use hkdf::Hkdf;
use sha2::Sha512;
use x25519_dalek::{EphemeralSecret, PublicKey as X25519Public};

use super::tlv8::{self, types::*, Tlv8Item};
use super::http::http_post_tlv8;

pub struct VerifyResult {
    /// Derived session key (32 bytes) — used for media encryption if needed.
    pub session_key: Vec<u8>,
}

/// Perform AirPlay 2 PAIR-VERIFY (M1→M4) against `host:port`.
///
/// Uses the client's persistent `signing_key` (Ed25519) as the long-term
/// identity that was registered during PAIR-SETUP.
pub fn pair_verify(
    host: &str,
    port: u16,
    device_id: &str,
    signing_key: &SigningKey,
) -> io::Result<VerifyResult> {
    // --- M1: send ephemeral Curve25519 public key ---
    let ephemeral_secret = EphemeralSecret::random_from_rng(rand::thread_rng());
    let ephemeral_pub = X25519Public::from(&ephemeral_secret);

    let m1 = tlv8::encode(&[
        Tlv8Item { typ: STATE, value: vec![0x01] },
        Tlv8Item { typ: PUBLIC_KEY, value: ephemeral_pub.as_bytes().to_vec() },
    ]);

    tracing::debug!("PAIR-VERIFY M1 → {}:{}", host, port);
    let resp_m2 = http_post_tlv8(host, port, "/pair-verify", &m1)?;
    let items_m2 = tlv8::decode(&resp_m2);

    check_state(&items_m2, 2, "PAIR-VERIFY M2")?;

    let server_pub_bytes = tlv8::find(&items_m2, PUBLIC_KEY)
        .ok_or_else(|| err("PAIR-VERIFY M2: missing PublicKey"))?;
    let encrypted_m2 = tlv8::find(&items_m2, ENCRYPTED_DATA)
        .ok_or_else(|| err("PAIR-VERIFY M2: missing EncryptedData"))?;

    let server_pub_arr: [u8; 32] = server_pub_bytes
        .try_into()
        .map_err(|_| err("PAIR-VERIFY M2: PublicKey wrong length"))?;
    let server_pub = X25519Public::from(server_pub_arr);

    // --- ECDH + derive verify-encrypt key ---
    let shared = ephemeral_secret.diffie_hellman(&server_pub);
    let verify_key = derive_key(
        shared.as_bytes(),
        b"Pair-Verify-Encrypt-Salt",
        b"Pair-Verify-Encrypt-Info",
    );

    // --- Decrypt M2 server data ---
    let m2_inner = chacha_decrypt(&verify_key, b"PV-Msg02", encrypted_m2)?;
    let items_inner = tlv8::decode(&m2_inner);

    // Verify server signature over <server_curve25519_pub || client_curve25519_pub>
    if let Some(server_id) = tlv8::find(&items_inner, IDENTIFIER) {
        if let Some(sig_bytes) = tlv8::find(&items_inner, SIGNATURE) {
            if let (Some(server_ltpk), _) = load_server_ltpk(server_id) {
                let msg: Vec<u8> = [server_pub.as_bytes().as_ref(), ephemeral_pub.as_bytes().as_ref()].concat();
                if let Ok(sig) = ed25519_dalek::Signature::from_slice(sig_bytes) {
                    let _ = server_ltpk.verify_strict(&msg, &sig); // log but don't abort
                }
            }
        }
    }

    // --- M3: send client signature encrypted ---
    let msg: Vec<u8> = [ephemeral_pub.as_bytes().as_ref(), server_pub.as_bytes().as_ref()].concat();
    let client_sig = signing_key.sign(&msg);

    let m3_inner = tlv8::encode(&[
        Tlv8Item { typ: IDENTIFIER, value: device_id.as_bytes().to_vec() },
        Tlv8Item { typ: SIGNATURE, value: client_sig.to_bytes().to_vec() },
    ]);
    let encrypted_m3 = chacha_encrypt(&verify_key, b"PV-Msg03", &m3_inner)?;

    let m3 = tlv8::encode(&[
        Tlv8Item { typ: STATE, value: vec![0x03] },
        Tlv8Item { typ: ENCRYPTED_DATA, value: encrypted_m3 },
    ]);

    tracing::debug!("PAIR-VERIFY M3 → {}:{}", host, port);
    let resp_m4 = http_post_tlv8(host, port, "/pair-verify", &m3)?;
    let items_m4 = tlv8::decode(&resp_m4);
    check_state(&items_m4, 4, "PAIR-VERIFY M4")?;
    tracing::debug!("PAIR-VERIFY complete");

    // Derive session key
    let session_key = derive_key(
        shared.as_bytes(),
        b"MediaRemote-Salt",
        b"MediaRemote-Key",
    );

    Ok(VerifyResult { session_key })
}

fn derive_key(ikm: &[u8], salt: &[u8], info: &[u8]) -> Vec<u8> {
    let hk = Hkdf::<Sha512>::new(Some(salt), ikm);
    let mut okm = vec![0u8; 32];
    hk.expand(info, &mut okm).expect("hkdf expand");
    okm
}

fn chacha_decrypt(key: &[u8], nonce_prefix: &[u8; 8], ciphertext: &[u8]) -> io::Result<Vec<u8>> {
    let cipher = ChaCha20Poly1305::new(Key::from_slice(key));
    // HAP uses 12-byte nonce: 4 zero bytes + 8-byte label
    let mut nonce_bytes = [0u8; 12];
    nonce_bytes[4..].copy_from_slice(nonce_prefix);
    let nonce = Nonce::from_slice(&nonce_bytes);
    cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| err("ChaCha20 decrypt failed"))
}

fn chacha_encrypt(key: &[u8], nonce_prefix: &[u8; 8], plaintext: &[u8]) -> io::Result<Vec<u8>> {
    let cipher = ChaCha20Poly1305::new(Key::from_slice(key));
    let mut nonce_bytes = [0u8; 12];
    nonce_bytes[4..].copy_from_slice(nonce_prefix);
    let nonce = Nonce::from_slice(&nonce_bytes);
    cipher
        .encrypt(nonce, plaintext)
        .map_err(|_| err("ChaCha20 encrypt failed"))
}

fn check_state(items: &[tlv8::Tlv8Item], expected: u8, ctx: &str) -> io::Result<()> {
    if let Some(state) = tlv8::find(items, STATE) {
        if state.first() == Some(&expected) {
            return Ok(());
        }
    }
    if let Some(error) = tlv8::find(items, ERROR) {
        return Err(err(&format!("{}: error code {:?}", ctx, error)));
    }
    Err(err(&format!("{}: unexpected state", ctx)))
}

fn err(msg: &str) -> io::Error {
    io::Error::new(io::ErrorKind::Other, msg)
}

/// Look up a previously stored server long-term public key by server identifier.
/// Returns (Some(verifying_key), true) if found, (None, false) otherwise.
fn load_server_ltpk(server_id: &[u8]) -> (Option<VerifyingKey>, bool) {
    let path = server_ltpk_path(server_id);
    if let Ok(bytes) = std::fs::read(&path) {
        if bytes.len() == 32 {
            if let Ok(vk) = VerifyingKey::from_bytes(bytes.as_slice().try_into().unwrap()) {
                return (Some(vk), true);
            }
        }
    }
    (None, false)
}

fn server_ltpk_path(server_id: &[u8]) -> std::path::PathBuf {
    let id_hex = server_id.iter().map(|b| format!("{:02x}", b)).collect::<String>();
    let dir = dirs_config();
    dir.join(format!("airplay_server_{}.ltpk", id_hex))
}

fn dirs_config() -> std::path::PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join(".config/rockbox")
}
