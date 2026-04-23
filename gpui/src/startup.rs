use std::net::TcpStream;
use std::time::Duration;

const GRPC_ADDR: &str = "127.0.0.1:6061";
const CONNECT_TIMEOUT: Duration = Duration::from_millis(500);

#[derive(Clone, Copy, Debug)]
pub enum StartupError {
    /// `rockboxd` binary not found anywhere in PATH or common locations.
    NotInstalled,
    /// Binary found but daemon is not listening on the gRPC port.
    NotRunning,
}

/// Run all pre-flight checks.  Returns `None` when everything is ready.
pub fn check() -> Option<StartupError> {
    if !is_installed() {
        return Some(StartupError::NotInstalled);
    }
    if !is_running() {
        return Some(StartupError::NotRunning);
    }
    None
}

pub fn is_installed() -> bool {
    // Walk PATH explicitly — app bundles have a stripped environment.
    if let Ok(path_var) = std::env::var("PATH") {
        for dir in path_var.split(':') {
            if std::path::Path::new(dir).join("rockboxd").exists() {
                return true;
            }
        }
    }
    // Fallback: common macOS install locations regardless of PATH.
    [
        "/usr/local/bin/rockboxd",
        "/opt/homebrew/bin/rockboxd",
        "/usr/bin/rockboxd",
        "/usr/local/sbin/rockboxd",
    ]
    .iter()
    .any(|p| std::path::Path::new(p).exists())
}

pub fn is_running() -> bool {
    TcpStream::connect_timeout(&GRPC_ADDR.parse().unwrap(), CONNECT_TIMEOUT).is_ok()
}
