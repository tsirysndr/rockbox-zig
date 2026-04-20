pub mod http;
pub mod pairing;
pub mod srp;
pub mod tlv8;
pub mod verify;

use std::io;
use std::path::PathBuf;

use ed25519_dalek::SigningKey;

use pairing::pair_setup;
use verify::pair_verify;

/// Load or generate the client's persistent Ed25519 identity key.
pub fn load_or_create_identity() -> io::Result<(SigningKey, String)> {
    let path = identity_path();
    let signing_key = if path.exists() {
        let bytes = std::fs::read(&path)?;
        if bytes.len() != 32 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "bad identity key length"));
        }
        let arr: [u8; 32] = bytes.try_into().unwrap();
        SigningKey::from_bytes(&arr)
    } else {
        let key = SigningKey::generate(&mut rand::thread_rng());
        let dir = path.parent().unwrap();
        std::fs::create_dir_all(dir)?;
        std::fs::write(&path, key.to_bytes())?;
        tracing::info!("generated new identity key → {}", path.display());
        key
    };

    // Device ID: hex encoding of the verifying key (first 16 bytes)
    let vk = signing_key.verifying_key();
    let device_id: String = vk.as_bytes()[..8]
        .iter()
        .map(|b| format!("{:02X}", b))
        .collect();

    Ok((signing_key, device_id))
}

/// Attempt AirPlay 2 handshake:
/// 1. PAIR-VERIFY (fast path for already-paired devices / Allow Everyone).
/// 2. If PAIR-VERIFY fails with an authentication error, try PAIR-SETUP with `pin`
///    and then PAIR-VERIFY again.
///
/// Returns `Ok(())` on success; caller proceeds with RTSP ANNOUNCE/SETUP/RECORD.
pub fn connect(host: &str, port: u16, pin: Option<&str>) -> io::Result<()> {
    let (signing_key, device_id) = load_or_create_identity()?;

    tracing::debug!("device_id={}", device_id);

    // Fast path: just PAIR-VERIFY (works if already paired or server allows everyone)
    match pair_verify(host, port, &device_id, &signing_key) {
        Ok(_result) => {
            tracing::info!("PAIR-VERIFY succeeded");
            return Ok(());
        }
        Err(e) => {
            tracing::debug!("PAIR-VERIFY failed: {} — trying PAIR-SETUP", e);
        }
    }

    // Full pair-setup if we have a PIN (or try with empty string for no-password mode)
    let pin_str = pin.unwrap_or("00000000");
    pair_setup(host, port, pin_str, &device_id, &signing_key)?;

    // Retry verify with the freshly obtained credentials
    pair_verify(host, port, &device_id, &signing_key)?;
    tracing::info!("connected after PAIR-SETUP + PAIR-VERIFY");
    Ok(())
}

fn identity_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".config/rockbox/airplay_identity.key")
}
