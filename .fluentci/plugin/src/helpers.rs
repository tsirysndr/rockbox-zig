use anyhow::Error;
use fluentci_pdk::dag;

pub fn detect_system() -> Result<(String, String), Error> {
    let os = match dag().get_os()?.as_str() {
        "linux" => "linux",
        "macos" => "darwin",
        "windows" => "windows",
        _ => return Err(Error::msg("unsupported os")),
    };
    let arch = match dag().get_arch()?.as_str() {
        "x86_64" => "amd64",
        "aarch64" => "arm64",
        "arm" => "arm-v7",
        _ => return Err(Error::msg("unsupported arch")),
    };

    Ok((os.into(), arch.into()))
}
