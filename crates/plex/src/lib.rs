use mdns_sd::{ServiceDaemon, ServiceEvent};
use std::time::Duration;

const PLEX_SERVICE: &str = "_plexmediasvr._tcp.local.";

#[derive(Debug, Clone)]
pub struct PlexServer {
    pub name: String,
    pub base_url: String,
}

#[derive(Debug, Clone)]
pub struct PlexEntry {
    pub key: String,
    pub title: String,
    pub is_container: bool,
    /// Direct HTTP stream URL for audio tracks.
    pub stream_url: Option<String>,
}

/// Discover Plex Media Servers on the local network via mDNS (_plex._tcp).
pub async fn discover_plex_servers() -> Vec<PlexServer> {
    tokio::task::spawn_blocking(|| {
        let mdns = match ServiceDaemon::new() {
            Ok(d) => d,
            Err(e) => {
                tracing::warn!("Plex discovery: mDNS daemon unavailable: {e}");
                return vec![];
            }
        };
        let receiver = match mdns.browse(PLEX_SERVICE) {
            Ok(r) => r,
            Err(e) => {
                tracing::warn!("Plex discovery: browse failed: {e}");
                return vec![];
            }
        };
        let deadline = std::time::Instant::now() + Duration::from_secs(3);
        let mut servers = vec![];
        loop {
            let remaining = deadline.saturating_duration_since(std::time::Instant::now());
            if remaining.is_zero() {
                break;
            }
            match receiver.recv_timeout(remaining) {
                Ok(ServiceEvent::ServiceResolved(info)) => {
                    let full = info.get_fullname();
                    let instance = full
                        .trim_end_matches(PLEX_SERVICE)
                        .trim_end_matches('.')
                        .to_string();
                    for ip in info.get_addresses() {
                        let base_url = format!("http://{}:{}", ip, info.get_port());
                        let name = if instance.is_empty() {
                            format!("Plex ({})", ip)
                        } else {
                            instance.clone()
                        };
                        servers.push(PlexServer { name, base_url });
                        break;
                    }
                }
                Ok(_) => {}
                Err(_) => break,
            }
        }
        servers
    })
    .await
    .unwrap_or_default()
}

/// List all library sections on a Plex server.
pub async fn list_sections(base_url: &str, token: Option<&str>) -> Vec<PlexEntry> {
    let xml = match fetch_plex_xml(base_url, token, "/library/sections").await {
        Some(x) => x,
        None => return vec![],
    };
    parse_directories(&xml)
        .into_iter()
        .map(|(key, title)| {
            let nav_key = if key.contains('/') {
                key
            } else {
                format!("/library/sections/{}/all", key)
            };
            PlexEntry {
                key: nav_key,
                title,
                is_container: true,
                stream_url: None,
            }
        })
        .collect()
}

/// Browse Plex content at a given API path (e.g. `/library/sections/1/all`).
pub async fn browse_plex(base_url: &str, token: Option<&str>, api_path: &str) -> Vec<PlexEntry> {
    let xml = match fetch_plex_xml(base_url, token, api_path).await {
        Some(x) => x,
        None => return vec![],
    };
    let mut entries: Vec<PlexEntry> = vec![];

    for (key, title) in parse_directories(&xml) {
        entries.push(PlexEntry {
            key,
            title,
            is_container: true,
            stream_url: None,
        });
    }

    for (key, title, part_key) in parse_tracks(&xml) {
        let stream_url = build_stream_url(base_url, token, &part_key);
        entries.push(PlexEntry {
            key,
            title,
            is_container: false,
            stream_url: Some(stream_url),
        });
    }

    entries
}

async fn fetch_plex_xml(base_url: &str, token: Option<&str>, api_path: &str) -> Option<String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .ok()?;

    let url = format!("{}{}", base_url.trim_end_matches('/'), api_path);
    let mut req = client
        .get(&url)
        .header("X-Plex-Product", "Rockbox")
        .header("X-Plex-Version", env!("CARGO_PKG_VERSION"))
        .header("X-Plex-Client-Identifier", "rockbox-plex-client")
        .header("Accept", "application/xml");

    if let Some(t) = token {
        if !t.is_empty() {
            req = req.header("X-Plex-Token", t);
        }
    }

    match req.send().await {
        Ok(resp) => {
            let status = resp.status();
            match resp.text().await {
                Ok(body) => {
                    if !status.is_success() {
                        tracing::warn!("Plex {url}: HTTP {status}");
                        return None;
                    }
                    tracing::debug!("Plex response from {url}: {} bytes", body.len());
                    Some(body)
                }
                Err(e) => {
                    tracing::warn!("Plex {url}: failed to read response: {e}");
                    None
                }
            }
        }
        Err(e) => {
            tracing::warn!("Plex {url}: request failed: {e}");
            None
        }
    }
}

/// Split `http://host:port?X-Plex-Token=xxx` into `("http://host:port", Some("xxx"))`.
/// Returns `(url, None)` when no token query param is present.
pub fn parse_base_url(url: &str) -> (String, Option<String>) {
    if let Some(idx) = url.find("?X-Plex-Token=") {
        let base = url[..idx].to_string();
        let rest = &url[idx + "?X-Plex-Token=".len()..];
        let token = rest.split('&').next().unwrap_or(rest).to_string();
        (base, if token.is_empty() { None } else { Some(token) })
    } else {
        (url.to_string(), None)
    }
}

/// Parse `<Directory key="..." title="..."/>` elements from Plex XML.
fn parse_directories(xml: &str) -> Vec<(String, String)> {
    let mut out = vec![];
    let mut pos = 0;
    while let Some(rel) = xml[pos..].find("<Directory ") {
        let start = pos + rel;
        let end = match xml[start..].find('>') {
            Some(p) => start + p + 1,
            None => break,
        };
        let element = &xml[start..end];
        let key = attr_value(element, "key").unwrap_or_default();
        let title = attr_value(element, "title").unwrap_or_default();
        if !key.is_empty() && !title.is_empty() {
            out.push((key, title));
        }
        pos = end;
    }
    out
}

/// Parse `<Track ...><Media><Part key="..."/></Media></Track>` elements.
fn parse_tracks(xml: &str) -> Vec<(String, String, String)> {
    let mut out = vec![];
    let mut pos = 0;
    while let Some(rel) = xml[pos..].find("<Track ") {
        let start = pos + rel;
        let end = match xml[start..].find("</Track>") {
            Some(p) => start + p + "</Track>".len(),
            None => break,
        };
        let element = &xml[start..end];
        let key = attr_value(element, "key").unwrap_or_default();
        let title = attr_value(element, "title").unwrap_or_default();
        let part_key = extract_part_key(element).unwrap_or_default();
        if !title.is_empty() && !part_key.is_empty() {
            out.push((key, title, part_key));
        }
        pos = end;
    }
    out
}

fn extract_part_key(element: &str) -> Option<String> {
    let part_start = element.find("<Part ")?;
    attr_value(&element[part_start..], "key")
}

fn build_stream_url(base_url: &str, token: Option<&str>, part_key: &str) -> String {
    let base = base_url.trim_end_matches('/');
    let path = if part_key.starts_with('/') {
        part_key.to_string()
    } else {
        format!("/{}", part_key)
    };
    match token {
        Some(t) if !t.is_empty() => format!("{}{}?X-Plex-Token={}", base, path, t),
        _ => format!("{}{}", base, path),
    }
}

fn attr_value(element: &str, name: &str) -> Option<String> {
    for quote in ['"', '\''] {
        let needle = format!("{}={}", name, quote);
        if let Some(start) = element.find(&needle) {
            let after = start + needle.len();
            if let Some(end) = element[after..].find(quote) {
                return Some(element[after..after + end].to_string());
            }
        }
    }
    None
}

pub fn percent_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len() * 3 / 2);
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char);
            }
            _ => {
                out.push('%');
                out.push(char::from_digit((b >> 4) as u32, 16).unwrap());
                out.push(char::from_digit((b & 0xf) as u32, 16).unwrap());
            }
        }
    }
    out
}

pub fn percent_decode(s: &str) -> String {
    let mut out = Vec::with_capacity(s.len());
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let (Some(hi), Some(lo)) = (
                (bytes[i + 1] as char).to_digit(16),
                (bytes[i + 2] as char).to_digit(16),
            ) {
                out.push(((hi << 4) | lo) as u8);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}
