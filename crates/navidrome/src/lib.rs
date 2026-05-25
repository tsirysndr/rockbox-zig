pub mod server;

use serde::Deserialize;
use std::time::Duration;

const API_VERSION: &str = "1.16.1";
const CLIENT_NAME: &str = "rockbox";

#[derive(Debug, Clone)]
pub struct NavidromeEntry {
    pub id: String,
    pub name: String,
    pub is_container: bool,
    pub stream_url: Option<String>,
}

// ── Subsonic JSON response wrappers ──────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct SubsonicEnvelope {
    #[serde(rename = "subsonic-response")]
    response: SubsonicBody,
}

#[derive(Debug, Deserialize)]
struct SubsonicBody {
    status: String,
    #[serde(rename = "musicFolders")]
    music_folders: Option<MusicFoldersWrapper>,
    indexes: Option<IndexesWrapper>,
    directory: Option<Directory>,
}

#[derive(Debug, Deserialize)]
struct MusicFoldersWrapper {
    #[serde(rename = "musicFolder")]
    music_folder: Vec<MusicFolder>,
}

#[derive(Debug, Deserialize)]
struct MusicFolder {
    id: serde_json::Value, // int or string depending on server
    name: Option<String>,
}

impl MusicFolder {
    fn id_str(&self) -> String {
        match &self.id {
            serde_json::Value::Number(n) => n.to_string(),
            serde_json::Value::String(s) => s.clone(),
            v => v.to_string(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct IndexesWrapper {
    index: Option<Vec<IndexLetter>>,
}

#[derive(Debug, Deserialize)]
struct IndexLetter {
    artist: Option<Vec<ArtistEntry>>,
}

#[derive(Debug, Deserialize)]
struct ArtistEntry {
    id: String,
    name: String,
}

#[derive(Debug, Deserialize)]
struct Directory {
    child: Option<Vec<DirectoryChild>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DirectoryChild {
    id: String,
    title: Option<String>,
    name: Option<String>,
    is_dir: Option<bool>,
}

impl DirectoryChild {
    fn display_name(&self) -> String {
        self.title
            .clone()
            .or_else(|| self.name.clone())
            .unwrap_or_else(|| self.id.clone())
    }
}

// ── Auth helpers ──────────────────────────────────────────────────────────────

/// Compute the Subsonic token: md5(password + salt).
pub fn compute_token(password: &str, salt: &str) -> String {
    let digest = md5::compute(format!("{}{}", password, salt));
    format!("{:x}", digest)
}

/// Build the Subsonic auth query string fragment.
fn auth_params(user: &str, token: &str, salt: &str) -> String {
    format!(
        "u={}&t={}&s={}&v={}&c={}&f=json",
        user, token, salt, API_VERSION, CLIENT_NAME
    )
}

/// Verify credentials by calling /rest/ping.view.
/// Returns `true` if the server responds with `status: "ok"`.
pub async fn ping(base_url: &str, user: &str, token: &str, salt: &str) -> bool {
    let url = format!(
        "{}/rest/ping.view?{}",
        base_url.trim_end_matches('/'),
        auth_params(user, token, salt)
    );
    let client = match reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
    {
        Ok(c) => c,
        Err(_) => return false,
    };
    match client.get(&url).send().await {
        Ok(resp) => {
            if !resp.status().is_success() {
                return false;
            }
            match resp.json::<SubsonicEnvelope>().await {
                Ok(env) => env.response.status == "ok",
                Err(e) => {
                    tracing::warn!("Navidrome ping {url}: failed to parse response: {e}");
                    false
                }
            }
        }
        Err(e) => {
            tracing::warn!("Navidrome ping {url}: request failed: {e}");
            false
        }
    }
}

/// List music library root folders via /rest/getMusicFolders.view.
pub async fn list_music_folders(
    base_url: &str,
    user: &str,
    token: &str,
    salt: &str,
) -> Vec<NavidromeEntry> {
    let url = format!(
        "{}/rest/getMusicFolders.view?{}",
        base_url.trim_end_matches('/'),
        auth_params(user, token, salt)
    );
    match fetch_subsonic(&url).await {
        Ok(body) => {
            let folders = body
                .music_folders
                .map(|f| f.music_folder)
                .unwrap_or_default();
            folders
                .into_iter()
                .map(|f| {
                    let id = f.id_str();
                    NavidromeEntry {
                        id: id.clone(),
                        name: f.name.unwrap_or(id),
                        is_container: true,
                        stream_url: None,
                    }
                })
                .collect()
        }
        Err(e) => {
            tracing::warn!("Navidrome getMusicFolders {url}: {e}");
            vec![]
        }
    }
}

/// Browse artists via /rest/getIndexes.view (optionally scoped to a music folder).
pub async fn list_indexes(
    base_url: &str,
    user: &str,
    token: &str,
    salt: &str,
    music_folder_id: Option<&str>,
) -> Vec<NavidromeEntry> {
    let mut url = format!(
        "{}/rest/getIndexes.view?{}",
        base_url.trim_end_matches('/'),
        auth_params(user, token, salt)
    );
    if let Some(fid) = music_folder_id {
        url.push_str(&format!("&musicFolderId={}", fid));
    }
    match fetch_subsonic(&url).await {
        Ok(body) => {
            let letters = body.indexes.and_then(|i| i.index).unwrap_or_default();
            let mut entries = vec![];
            for letter in letters {
                for artist in letter.artist.unwrap_or_default() {
                    entries.push(NavidromeEntry {
                        id: artist.id,
                        name: artist.name,
                        is_container: true,
                        stream_url: None,
                    });
                }
            }
            entries
        }
        Err(e) => {
            tracing::warn!("Navidrome getIndexes {url}: {e}");
            vec![]
        }
    }
}

/// Browse a directory via /rest/getMusicDirectory.view?id=X.
pub async fn browse_directory(
    base_url: &str,
    user: &str,
    token: &str,
    salt: &str,
    dir_id: &str,
) -> Vec<NavidromeEntry> {
    let url = format!(
        "{}/rest/getMusicDirectory.view?id={}&{}",
        base_url.trim_end_matches('/'),
        dir_id,
        auth_params(user, token, salt)
    );
    match fetch_subsonic(&url).await {
        Ok(body) => {
            let children = body.directory.and_then(|d| d.child).unwrap_or_default();
            children
                .into_iter()
                .map(|c| {
                    let is_dir = c.is_dir.unwrap_or(false);
                    let name = c.display_name();
                    let stream_url = if !is_dir {
                        Some(format!(
                            "{}/rest/stream.view?id={}&{}",
                            base_url.trim_end_matches('/'),
                            c.id,
                            auth_params(user, token, salt)
                        ))
                    } else {
                        None
                    };
                    NavidromeEntry {
                        id: c.id,
                        name,
                        is_container: is_dir,
                        stream_url,
                    }
                })
                .collect()
        }
        Err(e) => {
            tracing::warn!("Navidrome getMusicDirectory {url}: {e}");
            vec![]
        }
    }
}

async fn fetch_subsonic(url: &str) -> Result<SubsonicBody, anyhow::Error> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()?;
    let resp = client.get(url).send().await?;
    let status = resp.status();
    let env: SubsonicEnvelope = resp.json().await?;
    if !status.is_success() {
        anyhow::bail!("HTTP {status}");
    }
    if env.response.status != "ok" {
        anyhow::bail!("Subsonic status: {}", env.response.status);
    }
    Ok(env.response)
}

/// Split `http://host:port?nd_user=u&nd_token=t&nd_salt=s` into its components.
/// Returns `(base_url, Option<user>, Option<token>, Option<salt>)`.
pub fn parse_base_url(url: &str) -> (String, Option<String>, Option<String>, Option<String>) {
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
        (base, get("nd_user="), get("nd_token="), get("nd_salt="))
    } else {
        (url.to_string(), None, None, None)
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
