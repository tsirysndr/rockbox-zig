use crate::{device_uuid, get_local_ip, CONFIG};
use socket2::{Domain, Protocol, Socket, Type};
use std::net::{Ipv4Addr, SocketAddr, UdpSocket as StdUdpSocket};
use std::time::Duration;
use tokio::net::UdpSocket;
use tokio::time;

const SSDP_ADDR: Ipv4Addr = Ipv4Addr::new(239, 255, 255, 250);
const SSDP_PORT: u16 = 1900;

/// Run the SSDP server: send initial advertisements, respond to M-SEARCH,
/// and re-advertise every 15 minutes.
pub async fn run(http_port: u16) {
    let std_sock = match create_ssdp_socket() {
        Ok(s) => s,
        Err(e) => {
            tracing::warn!(
                "UPnP SSDP: could not bind port 1900 \
                 (another SSDP listener may already be running): {e}"
            );
            return;
        }
    };
    let socket = match UdpSocket::from_std(std_sock) {
        Ok(s) => s,
        Err(e) => {
            tracing::error!("UPnP SSDP: tokio socket error: {e}");
            return;
        }
    };

    // Send initial alive notifications.
    send_notify_alive(&socket, http_port).await;
    tracing::info!("UPnP SSDP: advertising on 239.255.255.250:1900");

    let mut interval = time::interval(Duration::from_secs(900)); // re-advertise every 15 min
    interval.tick().await; // consume the immediate first tick
    let mut buf = vec![0u8; 2048];

    loop {
        tokio::select! {
            result = socket.recv_from(&mut buf) => {
                match result {
                    Ok((len, from)) => {
                        if let Ok(msg) = std::str::from_utf8(&buf[..len]) {
                            handle_message(msg, from, &socket, http_port).await;
                        }
                    }
                    Err(e) => tracing::warn!("UPnP SSDP: recv error: {e}"),
                }
            }
            _ = interval.tick() => {
                send_notify_alive(&socket, http_port).await;
            }
        }
    }
}

async fn handle_message(msg: &str, from: SocketAddr, socket: &UdpSocket, http_port: u16) {
    if !msg.starts_with("M-SEARCH") {
        return;
    }
    let st = msg
        .lines()
        .find(|l| l.to_ascii_uppercase().starts_with("ST:"))
        .map(|l| l[3..].trim())
        .unwrap_or("");

    let want = matches!(
        st,
        "ssdp:all"
            | "upnp:rootdevice"
            | "urn:schemas-upnp-org:device:MediaServer:1"
            | "urn:schemas-upnp-org:service:ContentDirectory:1"
    ) || st.starts_with("uuid:");

    if !want {
        return;
    }

    // Small delay to avoid overwhelming the control point.
    tokio::time::sleep(Duration::from_millis(100)).await;

    let ip = get_local_ip();
    let uuid = device_uuid();
    let friendly = CONFIG
        .lock()
        .map(|c| c.friendly_name.clone())
        .unwrap_or_default();
    let location = format!("http://{}:{}/desc.xml", ip, http_port);

    for (nt, usn) in nt_usn_pairs(uuid) {
        let response = format!(
            "HTTP/1.1 200 OK\r\n\
             CACHE-CONTROL: max-age=1800\r\n\
             EXT:\r\n\
             LOCATION: {location}\r\n\
             SERVER: Linux/1.0 UPnP/1.0 Rockbox/1.0\r\n\
             ST: {nt}\r\n\
             USN: {usn}\r\n\
             X-Friendly-Name: {friendly}\r\n\
             \r\n"
        );
        if let Err(e) = socket.send_to(response.as_bytes(), from).await {
            tracing::debug!("UPnP SSDP: M-SEARCH response error: {e}");
        }
    }
}

async fn send_notify_alive(socket: &UdpSocket, http_port: u16) {
    let ip = get_local_ip();
    let uuid = device_uuid();
    let location = format!("http://{}:{}/desc.xml", ip, http_port);
    let dest: SocketAddr = SocketAddr::from((SSDP_ADDR, SSDP_PORT));

    for (nt, usn) in nt_usn_pairs(uuid) {
        let notify = format!(
            "NOTIFY * HTTP/1.1\r\n\
             HOST: 239.255.255.250:1900\r\n\
             CACHE-CONTROL: max-age=1800\r\n\
             LOCATION: {location}\r\n\
             NT: {nt}\r\n\
             NTS: ssdp:alive\r\n\
             SERVER: Linux/1.0 UPnP/1.0 Rockbox/1.0\r\n\
             USN: {usn}\r\n\
             \r\n"
        );
        // Send three times for reliability (UPnP spec recommendation).
        for _ in 0..3 {
            if let Err(e) = socket.send_to(notify.as_bytes(), dest).await {
                tracing::debug!("UPnP SSDP: notify error: {e}");
            }
        }
    }
}

/// Returns (NT, USN) pairs for all UPnP types we advertise.
fn nt_usn_pairs(uuid: &str) -> Vec<(String, String)> {
    let device_nt = "urn:schemas-upnp-org:device:MediaServer:1";
    let cd_nt = "urn:schemas-upnp-org:service:ContentDirectory:1";
    let cm_nt = "urn:schemas-upnp-org:service:ConnectionManager:1";
    vec![
        (
            "upnp:rootdevice".to_string(),
            format!("uuid:{uuid}::upnp:rootdevice"),
        ),
        (format!("uuid:{uuid}"), format!("uuid:{uuid}")),
        (device_nt.to_string(), format!("uuid:{uuid}::{device_nt}")),
        (cd_nt.to_string(), format!("uuid:{uuid}::{cd_nt}")),
        (cm_nt.to_string(), format!("uuid:{uuid}::{cm_nt}")),
    ]
}

fn create_ssdp_socket() -> std::io::Result<StdUdpSocket> {
    let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;
    socket.set_reuse_address(true)?;
    #[cfg(unix)]
    socket.set_reuse_port(true)?;
    socket.bind(&SocketAddr::from((Ipv4Addr::UNSPECIFIED, SSDP_PORT)).into())?;
    socket.join_multicast_v4(&SSDP_ADDR, &Ipv4Addr::UNSPECIFIED)?;
    socket.set_multicast_loop_v4(true)?;
    socket.set_nonblocking(true)?;
    Ok(socket.into())
}
