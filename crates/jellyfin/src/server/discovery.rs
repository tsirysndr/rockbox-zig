//! Jellyfin "client discovery" UDP protocol on port 7359. Clients broadcast
//! the literal string `"Who is JellyfinServer?"` and the server replies with
//! a JSON identifier.
//!
//! See: https://jellyfin.org/docs/general/networking/index.html#auto-discovery

use anyhow::{Context, Result};
use serde_json::json;
use tokio::net::UdpSocket;

pub const DISCOVERY_PORT: u16 = 7359;
const PROBE: &str = "Who is JellyfinServer?";

pub async fn run(server_name: String, server_id: String, http_port: u16) -> Result<()> {
    let socket = UdpSocket::bind(("0.0.0.0", DISCOVERY_PORT))
        .await
        .with_context(|| format!("binding UDP {DISCOVERY_PORT} for jellyfin discovery"))?;
    socket
        .set_broadcast(true)
        .context("enabling SO_BROADCAST on jellyfin discovery socket")?;

    tracing::info!(
        "jellyfin: discovery listener on udp/0.0.0.0:{DISCOVERY_PORT} as {server_name} ({server_id})"
    );

    let mut buf = vec![0u8; 1024];
    loop {
        let (n, peer) = match socket.recv_from(&mut buf).await {
            Ok(v) => v,
            Err(e) => {
                tracing::warn!("jellyfin discovery recv: {e}");
                continue;
            }
        };
        let msg = String::from_utf8_lossy(&buf[..n]);
        if msg.trim() != PROBE {
            continue;
        }
        let local_ip = local_ip_for_peer(peer.ip()).unwrap_or_else(|| peer.ip().to_string());
        let address = format!("http://{}:{}", local_ip, http_port);
        let reply = json!({
            "Address": address,
            "Id": server_id,
            "Name": server_name,
            "EndpointAddress": null,
        });
        if let Err(e) = socket.send_to(reply.to_string().as_bytes(), peer).await {
            tracing::warn!("jellyfin discovery reply to {peer}: {e}");
        }
    }
}

fn local_ip_for_peer(peer: std::net::IpAddr) -> Option<String> {
    let ifaces = if_addrs::get_if_addrs().ok()?;
    let peer_v4 = match peer {
        std::net::IpAddr::V4(v) => Some(v),
        std::net::IpAddr::V6(_) => None,
    };

    let mut best: Option<std::net::Ipv4Addr> = None;
    for iface in ifaces {
        if iface.is_loopback() {
            continue;
        }
        let std::net::IpAddr::V4(v4) = iface.ip() else {
            continue;
        };
        if !v4.is_private() {
            continue;
        }
        if let Some(peer_v4) = peer_v4 {
            let a = peer_v4.octets();
            let b = v4.octets();
            if a[0] == b[0] && a[1] == b[1] && a[2] == b[2] {
                return Some(v4.to_string());
            }
        }
        best.get_or_insert(v4);
    }
    best.map(|v| v.to_string())
}

/// Register `_jellyfin._tcp.local.` over mDNS. Best-effort — if the daemon
/// can't be created we just log and skip.
pub fn register_mdns(instance_name: &str, http_port: u16, server_id: &str) {
    let Ok(daemon) = mdns_sd::ServiceDaemon::new() else {
        tracing::warn!("jellyfin: mDNS responder unavailable");
        return;
    };
    let hostname = format!("{instance_name}.local.");
    let ips: Vec<std::net::IpAddr> = if_addrs::get_if_addrs()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|i| match i.ip() {
            std::net::IpAddr::V4(v) if v.is_private() && !v.is_loopback() => {
                Some(std::net::IpAddr::V4(v))
            }
            _ => None,
        })
        .collect();
    if ips.is_empty() {
        tracing::debug!("jellyfin: no LAN IPs to advertise");
        return;
    }
    let mut txt = std::collections::HashMap::new();
    txt.insert("ID".to_string(), server_id.to_string());
    let info = match mdns_sd::ServiceInfo::new(
        "_jellyfin._tcp.local.",
        instance_name,
        &hostname,
        ips.as_slice(),
        http_port,
        Some(txt),
    ) {
        Ok(i) => i,
        Err(e) => {
            tracing::warn!("jellyfin: mDNS ServiceInfo build failed: {e}");
            return;
        }
    };
    if let Err(e) = daemon.register(info) {
        tracing::warn!("jellyfin: mDNS register failed: {e}");
    } else {
        tracing::info!(
            "jellyfin: advertising _jellyfin._tcp.local. on {ips:?}:{http_port} (ID={server_id})"
        );
        // Daemon lives for the program lifetime — leak the handle so it doesn't drop.
        std::mem::forget(daemon);
    }
}
