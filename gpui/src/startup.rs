use std::net::TcpStream;
use std::time::Duration;

const LOCALHOST_GRPC: &str = "127.0.0.1:6061";
const CONNECT_TIMEOUT: Duration = Duration::from_millis(500);
const MDNS_SCAN_TIMEOUT: Duration = Duration::from_secs(3);

#[derive(Clone, Copy, Debug)]
pub enum StartupError {
    /// `rockboxd` binary not found anywhere in PATH or common locations.
    NotInstalled,
    /// Binary found but no daemon is reachable (localhost or network).
    NotRunning,
}

/// Run all pre-flight checks. Returns `None` when everything is ready.
///
/// Priority:
///   1. localhost:6061 — fastest, no mDNS overhead.
///   2. mDNS scan — discovers remote instances on the local network (blocks up
///      to MDNS_SCAN_TIMEOUT); sets the active server via `crate::server::set_server`.
pub fn check() -> Option<StartupError> {
    if !is_installed() {
        return Some(StartupError::NotInstalled);
    }
    if is_running() {
        return None;
    }
    let discovered = crate::server::scan_mdns(MDNS_SCAN_TIMEOUT);
    if let Some(server) = discovered.into_iter().next() {
        crate::server::set_server(server);
        return None;
    }
    Some(StartupError::NotRunning)
}

pub fn is_installed() -> bool {
    if let Ok(path_var) = std::env::var("PATH") {
        for dir in path_var.split(':') {
            if std::path::Path::new(dir).join("rockboxd").exists() {
                return true;
            }
        }
    }
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
    TcpStream::connect_timeout(&LOCALHOST_GRPC.parse().unwrap(), CONNECT_TIMEOUT).is_ok()
}
