use std::{
    fs,
    process::{Command, Stdio},
};
use tracing::{info, warn};

pub mod client;
pub mod types;

pub fn setup() -> Result<(), anyhow::Error> {
    let homedir =
        dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?;

    let path = format!(
        "{}:{}/{}",
        std::env::var("PATH").unwrap_or_default(),
        homedir.display(),
        ".rockbox/bin"
    );

    info!("Checking for typesense-server binary (PATH includes ~/.rockbox/bin)...");
    let mut cmd = Command::new("sh")
        .arg("-c")
        .arg("command -v typesense-server")
        .env("PATH", &path)
        .stderr(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .spawn()?;

    let data_dir = homedir.join(".config/rockbox.org/typesense");
    info!("Ensuring Typesense data directory: {}", data_dir.display());
    fs::create_dir_all(&data_dir)?;

    if !data_dir.join("api-key").exists() {
        let api_key = uuid::Uuid::new_v4().to_string();
        fs::write(data_dir.join("api-key"), &api_key)?;
        info!(
            "Generated new Typesense API key (saved to {})",
            data_dir.join("api-key").display()
        );
        if std::env::var("RB_TYPESENSE_API_KEY").is_err() {
            std::env::set_var("RB_TYPESENSE_API_KEY", &api_key);
        }
    } else {
        let api_key = fs::read_to_string(data_dir.join("api-key"))?;
        info!("Loaded existing Typesense API key from disk");
        if std::env::var("RB_TYPESENSE_API_KEY").is_err() {
            std::env::set_var("RB_TYPESENSE_API_KEY", &api_key);
        }
    }

    if cmd.wait()?.success() {
        info!("typesense-server already installed; skipping download.");
        return Ok(());
    }

    let os = if cfg!(target_os = "linux") {
        "linux"
    } else if cfg!(target_os = "macos") {
        "darwin"
    } else {
        return Err(anyhow::anyhow!("Unsupported platform"));
    };

    let arch = if cfg!(target_arch = "x86_64") {
        "amd64"
    } else if cfg!(target_arch = "aarch64") {
        "arm64"
    } else {
        return Err(anyhow::anyhow!("Unsupported architecture"));
    };

    let version = std::env::var("RB_TYPESENSE_VERSION").unwrap_or_else(|_| "30.1".to_string());
    let url = format!(
        "https://dl.typesense.org/releases/{version}/typesense-server-{version}-{os}-{arch}.tar.gz"
    );
    let filename = format!("typesense-server-{version}-{os}-{arch}.tar.gz");

    info!(
        "typesense-server not found. Downloading v{} for {}/{}...",
        version, os, arch
    );
    info!("Download URL: {}", url);

    let status = Command::new("curl")
        .arg("-L")
        .arg("--progress-bar")
        .arg(&url)
        .arg("-o")
        .arg(&filename)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;

    if !status.success() {
        return Err(anyhow::anyhow!("curl exited with {}", status));
    }
    info!("Download complete: {}", filename);

    info!("Extracting {}...", filename);
    let status = Command::new("tar")
        .arg("xzf")
        .arg(&filename)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;

    if !status.success() {
        return Err(anyhow::anyhow!("tar exited with {}", status));
    }
    info!("Extraction complete.");

    info!("Installing typesense-server to ~/.rockbox/bin/...");
    let status = Command::new("sh")
        .arg("-c")
        .arg("mkdir -p ~/.rockbox/bin && cp typesense-server ~/.rockbox/bin && chmod +x ~/.rockbox/bin/typesense-server && rm -f typesense-server typesense-server-*.tar.gz typesense-server.md5.txt")
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;

    if !status.success() {
        warn!(
            "Install script exited with {}; binary may not be in place",
            status
        );
    } else {
        info!("typesense-server installed to ~/.rockbox/bin/typesense-server");
    }

    Ok(())
}
