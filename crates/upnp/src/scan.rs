use std::collections::HashSet;
use std::net::{Ipv4Addr, SocketAddr, UdpSocket};
use std::time::Duration;

const SSDP_ADDR: &str = "239.255.255.250:1900";
const MSEARCH: &str = "M-SEARCH * HTTP/1.1\r\n\
    HOST: 239.255.255.250:1900\r\n\
    MAN: \"ssdp:discover\"\r\n\
    MX: 3\r\n\
    ST: urn:schemas-upnp-org:device:MediaRenderer:1\r\n\
    \r\n";

#[derive(Debug, Clone)]
pub struct RendererInfo {
    pub friendly_name: String,
    pub location: String,
    pub av_transport_url: String,
    pub ip: String,
    pub port: u16,
    pub udn: String,
}

/// Send SSDP M-SEARCH and probe every discovered location.
/// Blocks for up to `timeout_secs` while collecting SSDP replies.
pub async fn scan_renderers(timeout_secs: u64) -> Vec<RendererInfo> {
    let locations = ssdp_discover(timeout_secs);
    tracing::debug!("upnp scan: {} location(s) found", locations.len());

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .unwrap();

    let mut renderers = vec![];
    for location in locations {
        match probe_renderer(&client, &location).await {
            Some(r) => {
                tracing::info!(
                    "upnp scan: found renderer \"{}\" av={} ({}:{})",
                    r.friendly_name,
                    r.av_transport_url,
                    r.ip,
                    r.port
                );
                renderers.push(r);
            }
            None => tracing::debug!("upnp scan: {location} skipped (not a renderer)"),
        }
    }
    renderers
}

// ---------------------------------------------------------------------------
// SSDP
// ---------------------------------------------------------------------------

fn ssdp_discover(timeout_secs: u64) -> HashSet<String> {
    let socket = match UdpSocket::bind(SocketAddr::from((Ipv4Addr::UNSPECIFIED, 0))) {
        Ok(s) => s,
        Err(e) => {
            tracing::warn!("upnp scan: bind failed: {e}");
            return HashSet::new();
        }
    };
    socket
        .set_read_timeout(Some(Duration::from_secs(timeout_secs)))
        .ok();

    let dest: SocketAddr = SSDP_ADDR.parse().unwrap();
    if let Err(e) = socket.send_to(MSEARCH.as_bytes(), dest) {
        tracing::warn!("upnp scan: M-SEARCH send failed: {e}");
        return HashSet::new();
    }

    let mut locations = HashSet::new();
    let mut buf = vec![0u8; 4096];
    loop {
        match socket.recv_from(&mut buf) {
            Ok((len, _)) => {
                if let Ok(msg) = std::str::from_utf8(&buf[..len]) {
                    if let Some(loc) = header_value(msg, "LOCATION") {
                        locations.insert(loc.to_string());
                    }
                }
            }
            Err(_) => break,
        }
    }
    locations
}

// ---------------------------------------------------------------------------
// Device description probe
// ---------------------------------------------------------------------------

async fn probe_renderer(client: &reqwest::Client, location: &str) -> Option<RendererInfo> {
    let xml = client.get(location).send().await.ok()?.text().await.ok()?;

    if !xml.contains("MediaRenderer") {
        return None;
    }

    let friendly_name = tag_value(&xml, "friendlyName")?.to_string();
    let udn = tag_value(&xml, "UDN").unwrap_or("").to_string();

    let av_control_path = find_service_control_url(&xml, "AVTransport")?;
    let base = base_url_of(location)?;
    let av_transport_url = if av_control_path.starts_with("http") {
        av_control_path.to_string()
    } else {
        format!(
            "{}/{}",
            base.trim_end_matches('/'),
            av_control_path.trim_start_matches('/')
        )
    };

    let (ip, port) = ip_port_of(location)?;

    Some(RendererInfo {
        friendly_name,
        location: location.to_string(),
        av_transport_url,
        ip,
        port,
        udn,
    })
}

fn find_service_control_url<'a>(xml: &'a str, service_type: &str) -> Option<&'a str> {
    let mut search_from = 0;
    while let Some(rel) = xml[search_from..].find("<service>") {
        let start = search_from + rel;
        let end = xml[start..].find("</service>").map(|e| start + e)?;
        let block = &xml[start..end];
        if block.contains(service_type) {
            if let Some(url) = tag_value(block, "controlURL") {
                return Some(url);
            }
        }
        search_from = end + "</service>".len();
    }
    None
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn tag_value<'a>(xml: &'a str, tag: &str) -> Option<&'a str> {
    let open = format!("<{tag}>");
    let close = format!("</{tag}>");
    let start = xml.find(&open)? + open.len();
    let end = xml[start..].find(&close)?;
    Some(xml[start..start + end].trim())
}

fn header_value<'a>(msg: &'a str, name: &str) -> Option<&'a str> {
    let prefix_upper = format!("{}:", name.to_ascii_uppercase());
    msg.lines()
        .find(|l| l.to_ascii_uppercase().starts_with(&prefix_upper))
        .map(|l| l[name.len() + 1..].trim())
}

fn base_url_of(url: &str) -> Option<String> {
    let after_scheme = url.find("://")?;
    let authority_start = after_scheme + 3;
    let path_start = url[authority_start..]
        .find('/')
        .map(|p| authority_start + p)
        .unwrap_or(url.len());
    Some(url[..path_start].to_string())
}

fn ip_port_of(url: &str) -> Option<(String, u16)> {
    let after_scheme = url.find("://")?;
    let authority = url[after_scheme + 3..].split('/').next()?;
    if let Some(colon) = authority.rfind(':') {
        let ip = authority[..colon].to_string();
        let port: u16 = authority[colon + 1..].parse().ok()?;
        Some((ip, port))
    } else {
        Some((authority.to_string(), 80))
    }
}
