use std::net::UdpSocket;
use std::time::Duration;

use serde::Deserialize;

/// Jellyfin-compatible HTTP API server. Opt in via the `server` feature.
#[cfg(feature = "server")]
pub mod server;

#[derive(Debug, Clone)]
pub struct JellyfinServer {
    pub name: String,
    pub base_url: String,
}

#[derive(Debug, Clone)]
pub struct JellyfinEntry {
    pub id: String,
    pub name: String,
    pub is_container: bool,
    /// Direct HTTP stream URL for audio tracks.
    pub stream_url: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct DiscoveryResponse {
    address: String,
    name: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct AuthResponse {
    access_token: String,
    user: AuthUser,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct AuthUser {
    id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct UserInfo {
    id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct ItemsResponse {
    items: Vec<JellyfinItem>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct JellyfinItem {
    id: String,
    name: String,
    #[serde(rename = "Type")]
    item_type: Option<String>,
}

/// Discover Jellyfin servers on the local network via UDP broadcast.
pub async fn discover_jellyfin_servers() -> Vec<JellyfinServer> {
    tokio::task::spawn_blocking(|| {
        let socket = match UdpSocket::bind("0.0.0.0:0") {
            Ok(s) => s,
            Err(e) => {
                tracing::warn!("Jellyfin discovery: failed to bind UDP socket: {e}");
                return vec![];
            }
        };
        if let Err(e) = socket.set_broadcast(true) {
            tracing::warn!("Jellyfin discovery: failed to set broadcast: {e}");
            return vec![];
        }
        if let Err(e) = socket.set_read_timeout(Some(Duration::from_secs(3))) {
            tracing::warn!("Jellyfin discovery: failed to set read timeout: {e}");
            return vec![];
        }

        let msg = b"Who is JellyfinServer?";
        if let Err(e) = socket.send_to(msg, "255.255.255.255:7359") {
            tracing::warn!("Jellyfin discovery: UDP send failed: {e}");
            return vec![];
        }

        let deadline = std::time::Instant::now() + Duration::from_secs(3);
        let mut servers = vec![];
        let mut buf = [0u8; 4096];

        loop {
            let remaining = deadline.saturating_duration_since(std::time::Instant::now());
            if remaining.is_zero() {
                break;
            }
            if let Err(e) = socket.set_read_timeout(Some(remaining)) {
                tracing::warn!("Jellyfin discovery: set_read_timeout failed: {e}");
                break;
            }
            match socket.recv_from(&mut buf) {
                Ok((len, _addr)) => {
                    let data = &buf[..len];
                    match serde_json::from_slice::<DiscoveryResponse>(data) {
                        Ok(resp) => {
                            let name = resp
                                .name
                                .filter(|n| !n.is_empty())
                                .unwrap_or_else(|| format!("Jellyfin ({})", resp.address));
                            servers.push(JellyfinServer {
                                name,
                                base_url: resp.address,
                            });
                        }
                        Err(e) => {
                            tracing::warn!("Jellyfin discovery: failed to parse response: {e}");
                        }
                    }
                }
                Err(_) => break,
            }
        }
        servers
    })
    .await
    .unwrap_or_default()
}

/// Authenticate with a Jellyfin server.
/// Returns `(access_token, user_id)` on success or `None` on failure.
pub async fn authenticate(
    base_url: &str,
    username: &str,
    password: &str,
) -> Option<(String, String)> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .ok()?;

    let url = format!(
        "{}/Users/AuthenticateByName",
        base_url.trim_end_matches('/')
    );
    let body = serde_json::json!({
        "Username": username,
        "Pw": password,
    });

    let resp = client
        .post(&url)
        .header(
            "X-Emby-Authorization",
            r#"MediaBrowser Client="Rockbox", Device="Rockbox", DeviceId="rockbox-jellyfin", Version="1.0""#,
        )
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await;

    match resp {
        Ok(r) => {
            let status = r.status();
            match r.json::<AuthResponse>().await {
                Ok(auth) => {
                    if !status.is_success() {
                        tracing::warn!("Jellyfin authenticate {url}: HTTP {status}");
                        return None;
                    }
                    Some((auth.access_token, auth.user.id))
                }
                Err(e) => {
                    tracing::warn!("Jellyfin authenticate {url}: failed to parse response: {e}");
                    None
                }
            }
        }
        Err(e) => {
            tracing::warn!("Jellyfin authenticate {url}: request failed: {e}");
            None
        }
    }
}

/// Authenticate using a Jellyfin API key.
/// Calls GET /Users to get the first user's ID, returns (api_key, user_id).
pub async fn authenticate_with_api_key(base_url: &str, api_key: &str) -> Option<(String, String)> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .ok()?;
    let url = format!("{}/Users", base_url.trim_end_matches('/'));
    let resp = client
        .get(&url)
        .header("X-Emby-Token", api_key)
        .send()
        .await
        .ok()?;
    if !resp.status().is_success() {
        tracing::warn!("Jellyfin API key auth {url}: HTTP {}", resp.status());
        return None;
    }
    let users: Vec<UserInfo> = resp.json().await.ok()?;
    users
        .into_iter()
        .next()
        .map(|u| (api_key.to_string(), u.id))
}

/// List all views (libraries) for a user.
pub async fn list_views(base_url: &str, token: &str, user_id: &str) -> Vec<JellyfinEntry> {
    let url = format!("{}/Users/{}/Views", base_url.trim_end_matches('/'), user_id);
    let items = fetch_jellyfin_items(base_url, token, &url).await;
    items
        .into_iter()
        .map(|item| JellyfinEntry {
            id: item.id,
            name: item.name,
            is_container: true,
            stream_url: None,
        })
        .collect()
}

/// Browse items under a parent folder.
pub async fn browse_items(
    base_url: &str,
    token: &str,
    user_id: &str,
    parent_id: &str,
) -> Vec<JellyfinEntry> {
    let url = format!(
        "{}/Users/{}/Items?ParentId={}&SortBy=SortName&SortOrder=Ascending",
        base_url.trim_end_matches('/'),
        user_id,
        parent_id
    );
    let items = fetch_jellyfin_items(base_url, token, &url).await;
    items
        .into_iter()
        .map(|item| {
            let is_audio = item
                .item_type
                .as_deref()
                .map(|t| t == "Audio")
                .unwrap_or(false);
            let stream_url = if is_audio {
                Some(format!(
                    "{}/Items/{}/Download?api_key={}",
                    base_url.trim_end_matches('/'),
                    item.id,
                    token
                ))
            } else {
                None
            };
            JellyfinEntry {
                id: item.id,
                name: item.name,
                is_container: !is_audio,
                stream_url,
            }
        })
        .collect()
}

async fn fetch_jellyfin_items(base_url: &str, token: &str, url: &str) -> Vec<JellyfinItem> {
    let client = match reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!("Jellyfin {url}: failed to build client: {e}");
            return vec![];
        }
    };

    let _ = base_url; // used for context in error messages

    match client.get(url).header("X-Emby-Token", token).send().await {
        Ok(resp) => {
            let status = resp.status();
            match resp.json::<ItemsResponse>().await {
                Ok(body) => {
                    if !status.is_success() {
                        tracing::warn!("Jellyfin {url}: HTTP {status}");
                        return vec![];
                    }
                    tracing::debug!("Jellyfin response from {url}: {} items", body.items.len());
                    body.items
                }
                Err(e) => {
                    tracing::warn!("Jellyfin {url}: failed to parse response: {e}");
                    vec![]
                }
            }
        }
        Err(e) => {
            tracing::warn!("Jellyfin {url}: request failed: {e}");
            vec![]
        }
    }
}

/// Split `http://host:port?X-Jellyfin-Token=tok&userId=id` into
/// `("http://host:port", Some("tok"), Some("id"))`.
/// Returns `(url, None, None)` when no query params are present.
pub fn parse_base_url(url: &str) -> (String, Option<String>, Option<String>) {
    if let Some(idx) = url.find('?') {
        let base = url[..idx].to_string();
        let query = &url[idx + 1..];
        let token = query
            .split('&')
            .find(|p| p.starts_with("X-Jellyfin-Token="))
            .and_then(|p| p.strip_prefix("X-Jellyfin-Token="))
            .filter(|v| !v.is_empty())
            .map(|v| v.to_string());
        let user_id = query
            .split('&')
            .find(|p| p.starts_with("userId="))
            .and_then(|p| p.strip_prefix("userId="))
            .filter(|v| !v.is_empty())
            .map(|v| v.to_string());
        (base, token, user_id)
    } else {
        (url.to_string(), None, None)
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
