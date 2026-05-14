use mdns_sd::{ServiceDaemon, ServiceEvent};
use std::time::Duration;

const KODI_SERVICE: &str = "_xbmc-jsonrpc-h._tcp.local.";

#[derive(Debug, Clone)]
pub struct KodiServer {
    pub name: String,
    pub base_url: String,
}

#[derive(Debug, Clone)]
pub struct KodiEntry {
    pub id: String,
    pub label: String,
    pub is_container: bool,
    pub stream_url: Option<String>,
}

/// Discover Kodi/XBMC servers on the local network via mDNS (_xbmc-jsonrpc-h._tcp.local.).
pub async fn discover_kodi_servers() -> Vec<KodiServer> {
    tokio::task::spawn_blocking(|| {
        let mdns = match ServiceDaemon::new() {
            Ok(d) => d,
            Err(e) => {
                tracing::warn!("Kodi discovery: mDNS daemon unavailable: {e}");
                return vec![];
            }
        };
        let receiver = match mdns.browse(KODI_SERVICE) {
            Ok(r) => r,
            Err(e) => {
                tracing::warn!("Kodi discovery: browse failed: {e}");
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
                        .trim_end_matches(KODI_SERVICE)
                        .trim_end_matches('.')
                        .to_string();
                    for ip in info.get_addresses() {
                        let base_url = format!("http://{}:{}", ip, info.get_port());
                        let name = if instance.is_empty() {
                            format!("Kodi ({})", ip)
                        } else {
                            instance.clone()
                        };
                        servers.push(KodiServer { name, base_url });
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

/// List music library root folders via Kodi JSON-RPC Files.GetSources.
pub async fn browse_sources(
    base_url: &str,
    user: Option<&str>,
    pass: Option<&str>,
) -> Vec<KodiEntry> {
    let body = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "Files.GetSources",
        "params": { "media": "music" },
        "id": 1
    });

    let resp = match post_jsonrpc(base_url, user, pass, &body).await {
        Some(v) => v,
        None => return vec![],
    };

    let sources = match resp
        .get("result")
        .and_then(|r| r.get("sources"))
        .and_then(|s| s.as_array())
    {
        Some(a) => a.clone(),
        None => {
            tracing::warn!("Kodi GetSources: unexpected response shape");
            return vec![];
        }
    };

    sources
        .into_iter()
        .filter_map(|s| {
            let file = s.get("file")?.as_str()?.to_string();
            let label = s
                .get("label")
                .and_then(|l| l.as_str())
                .unwrap_or(&file)
                .to_string();
            Some(KodiEntry {
                id: file,
                label,
                is_container: true,
                stream_url: None,
            })
        })
        .collect()
}

/// List directory entries via Kodi JSON-RPC Files.GetDirectory.
pub async fn browse_directory(
    base_url: &str,
    user: Option<&str>,
    pass: Option<&str>,
    directory: &str,
) -> Vec<KodiEntry> {
    // `file`, `filetype`, `label`, and `type` are base fields always included in every
    // Files.GetDirectory response — they are NOT valid Files.Fields.File property names.
    // Passing them in `properties` causes Kodi to return -32602 Invalid params, hence empty results.
    let body = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "Files.GetDirectory",
        "params": {
            "directory": directory,
            "media": "music",
            "properties": []
        },
        "id": 1
    });

    let resp = match post_jsonrpc(base_url, user, pass, &body).await {
        Some(v) => v,
        None => return vec![],
    };

    if let Some(err) = resp.get("error") {
        tracing::warn!("Kodi GetDirectory {directory}: JSON-RPC error: {err}");
        return vec![];
    }

    let files = match resp
        .get("result")
        .and_then(|r| r.get("files"))
        .and_then(|f| f.as_array())
    {
        Some(a) => a.clone(),
        None => {
            tracing::warn!("Kodi GetDirectory {directory}: no files in result");
            return vec![];
        }
    };

    // Parse host:port from base_url for building VFS stream URLs.
    let host_port = base_url
        .strip_prefix("http://")
        .unwrap_or(base_url)
        .split('/')
        .next()
        .unwrap_or("")
        .to_string();

    files
        .into_iter()
        .filter_map(|f| {
            let file = f.get("file")?.as_str()?.to_string();
            let label = f
                .get("label")
                .and_then(|l| l.as_str())
                .unwrap_or(&file)
                .to_string();
            let filetype = f.get("filetype").and_then(|t| t.as_str()).unwrap_or("file");
            let is_container = filetype == "directory";

            let stream_url = if !is_container {
                let encoded = percent_encode(&file);
                let url = match (
                    user.filter(|u| !u.is_empty()),
                    pass.filter(|p| !p.is_empty()),
                ) {
                    (Some(u), Some(p)) => {
                        format!("http://{}:{}@{}/vfs/{}", u, p, host_port, encoded)
                    }
                    _ => format!("http://{}/vfs/{}", host_port, encoded),
                };
                Some(url)
            } else {
                None
            };

            Some(KodiEntry {
                id: file,
                label,
                is_container,
                stream_url,
            })
        })
        .collect()
}

/// Split `http://host:port?kodi_user=u&kodi_pass=p` into its components.
/// Returns `(base_url, Option<user>, Option<pass>)`.
pub fn parse_base_url(url: &str) -> (String, Option<String>, Option<String>) {
    if let Some(idx) = url.find('?') {
        let base = url[..idx].to_string();
        let query = &url[idx + 1..];
        let get = |key: &str| -> Option<String> {
            query
                .split('&')
                .find(|p| p.starts_with(key))
                .and_then(|p| p.strip_prefix(key))
                .filter(|v| !v.is_empty())
                .map(|v| v.to_string())
        };
        (base, get("kodi_user="), get("kodi_pass="))
    } else {
        (url.to_string(), None, None)
    }
}

async fn post_jsonrpc(
    base_url: &str,
    user: Option<&str>,
    pass: Option<&str>,
    body: &serde_json::Value,
) -> Option<serde_json::Value> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .ok()?;

    let url = format!("{}/jsonrpc", base_url.trim_end_matches('/'));
    let mut req = client
        .post(&url)
        .header("Content-Type", "application/json")
        .json(body);

    if let Some(u) = user.filter(|u| !u.is_empty()) {
        req = req.basic_auth(u, pass.filter(|p| !p.is_empty()));
    }

    match req.send().await {
        Ok(resp) => {
            let status = resp.status();
            match resp.json::<serde_json::Value>().await {
                Ok(v) => {
                    if !status.is_success() {
                        tracing::warn!("Kodi {url}: HTTP {status}");
                        return None;
                    }
                    tracing::debug!("Kodi response from {url}");
                    Some(v)
                }
                Err(e) => {
                    tracing::warn!("Kodi {url}: failed to parse response: {e}");
                    None
                }
            }
        }
        Err(e) => {
            tracing::warn!("Kodi {url}: request failed: {e}");
            None
        }
    }
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
