use std::{
    fs,
    process::{Command, Stdio},
};

pub mod client;
pub mod types;

pub fn setup() -> Result<(), anyhow::Error> {
    let path = format!(
        "{}:{}",
        std::env::var("PATH").unwrap_or_default(),
        "~/.rockbox/bin"
    );
    let mut cmd = Command::new("command")
        .arg("-v")
        .arg("typesense-server")
        .env("PATH", &path)
        .stderr(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .spawn()?;

    let homedir =
        dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?;
    let data_dir = homedir.join(".config/rockbox.org/typesense");
    fs::create_dir_all(&data_dir)?;

    if !data_dir.join("api-key").exists() {
        let api_key = uuid::Uuid::new_v4().to_string();
        fs::write(data_dir.join("api-key"), &api_key)?;
        println!("Generated new Typesense API key: {}", api_key);
        if std::env::var("RB_TYPESENSE_API_KEY").is_err() {
            std::env::set_var("RB_TYPESENSE_API_KEY", &api_key);
        }
    } else {
        let api_key = fs::read_to_string(data_dir.join("api-key"))?;
        println!("Using existing Typesense API key: {}", api_key);
        if std::env::var("RB_TYPESENSE_API_KEY").is_err() {
            std::env::set_var("RB_TYPESENSE_API_KEY", &api_key);
        }
    }

    if cmd.wait()?.success() {
        println!("Typesense server is already installed and available in PATH.");
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

    Command::new("curl")
        .arg("-L")
        .arg(&url)
        .arg("-o")
        .arg(&filename)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;

    Command::new("tar")
        .arg("xzf")
        .arg(&filename)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;

    Command::new("sh")
        .arg("-c")
        .arg("mkdir -p ~/.rockbox/bin && mv typesense-server ~/.rockbox/bin && chmod +x ~/.rockbox/bin/typesense-server && rm -f typesense-server-*.tar.gz typesense-server.md5.txt")
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;

    Ok(())
}
