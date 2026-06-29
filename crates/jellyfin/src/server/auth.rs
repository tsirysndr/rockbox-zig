//! Jellyfin auth: X-Emby-Authorization header parsing, opaque-token store
//! backed by sqlite, FromRequest extractor for protected endpoints.

use actix_web::{dev::Payload, web, FromRequest, HttpRequest};
use anyhow::Result;
use futures::future::LocalBoxFuture;
use sha2::{Digest, Sha256};
use sqlx::{Pool, Sqlite};
use std::collections::HashMap;

use super::JellyfinState;

/// Parsed `X-Emby-Authorization` / `Authorization: MediaBrowser …` header.
/// Jellyfin clients send a comma-separated list of `Key="Value"` pairs.
#[derive(Debug, Default, Clone)]
pub struct EmbyAuth {
    pub client: Option<String>,
    pub device: Option<String>,
    pub device_id: Option<String>,
    pub version: Option<String>,
    pub token: Option<String>,
}

pub fn parse_emby_auth_header(value: &str) -> EmbyAuth {
    let body = value
        .strip_prefix("MediaBrowser ")
        .or_else(|| value.strip_prefix("Emby "))
        .unwrap_or(value);

    let mut out = EmbyAuth::default();
    let mut buf = String::new();
    let mut in_quotes = false;
    let chars = body.chars();
    let mut parts: Vec<String> = Vec::new();
    for c in chars {
        match c {
            '"' => in_quotes = !in_quotes,
            ',' if !in_quotes => {
                parts.push(std::mem::take(&mut buf));
            }
            _ => buf.push(c),
        }
    }
    if !buf.is_empty() {
        parts.push(buf);
    }

    let pairs: HashMap<String, String> = parts
        .into_iter()
        .filter_map(|p| {
            let trimmed = p.trim();
            let eq = trimmed.find('=')?;
            let key = trimmed[..eq].trim().to_string();
            let val = trimmed[eq + 1..].trim().trim_matches('"').to_string();
            if key.is_empty() {
                None
            } else {
                Some((key, val))
            }
        })
        .collect();

    out.client = pairs.get("Client").cloned();
    out.device = pairs.get("Device").cloned();
    out.device_id = pairs.get("DeviceId").cloned();
    out.version = pairs.get("Version").cloned();
    out.token = pairs.get("Token").cloned();
    out
}

/// Pull the access token from any header/query position Jellyfin clients use.
pub fn extract_token(req: &HttpRequest) -> Option<String> {
    let headers = req.headers();
    for name in ["x-emby-token", "x-mediabrowser-token"] {
        if let Some(v) = headers.get(name) {
            if let Ok(s) = v.to_str() {
                let t = s.trim();
                if !t.is_empty() {
                    return Some(t.to_string());
                }
            }
        }
    }
    for name in ["x-emby-authorization", "authorization"] {
        if let Some(v) = headers.get(name) {
            if let Ok(s) = v.to_str() {
                if let Some(t) = parse_emby_auth_header(s).token {
                    if !t.is_empty() {
                        return Some(t);
                    }
                }
            }
        }
    }
    // Some clients pass it as a query param on streaming URLs.
    for pair in req.query_string().split('&') {
        let mut it = pair.splitn(2, '=');
        let k = it.next().unwrap_or("");
        let v = it.next().unwrap_or("");
        if k.eq_ignore_ascii_case("api_key") || k.eq_ignore_ascii_case("apikey") {
            if !v.is_empty() {
                return Some(
                    urlencoding::decode(v)
                        .map(|s| s.into_owned())
                        .unwrap_or_else(|_| v.to_string()),
                );
            }
        }
    }
    None
}

pub async fn token_valid(pool: &Pool<Sqlite>, token: &str) -> bool {
    let row: Option<(String,)> =
        sqlx::query_as("SELECT user_id FROM jellyfin_tokens WHERE token = ?1")
            .bind(token)
            .fetch_optional(pool)
            .await
            .unwrap_or(None);
    row.is_some()
}

pub async fn store_token(
    pool: &Pool<Sqlite>,
    token: &str,
    user_id: &str,
    auth: &EmbyAuth,
    now: &str,
) -> Result<()> {
    sqlx::query(
        "INSERT INTO jellyfin_tokens (token, user_id, device_id, device_name, client, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)
         ON CONFLICT(token) DO NOTHING",
    )
    .bind(token)
    .bind(user_id)
    .bind(auth.device_id.as_deref())
    .bind(auth.device.as_deref())
    .bind(auth.client.as_deref())
    .bind(now)
    .execute(pool)
    .await?;
    Ok(())
}

pub fn random_hex(bytes: usize) -> String {
    let mut buf = vec![0u8; bytes];
    let ok = std::fs::File::open("/dev/urandom")
        .and_then(|mut f| std::io::Read::read_exact(&mut f, &mut buf))
        .is_ok();
    if !ok {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        let mut h = Sha256::new();
        h.update(nanos.to_le_bytes());
        h.update(n.to_le_bytes());
        let digest = h.finalize();
        let take = bytes.min(digest.len());
        buf[..take].copy_from_slice(&digest[..take]);
    }
    hex::encode(buf)
}

/// Look up or generate the server's stable Jellyfin id (dashed UUID).
/// Persisted in `jellyfin_meta`; old un-dashed values are upgraded in place.
pub async fn ensure_server_id(pool: &Pool<Sqlite>) -> Result<String> {
    let existing: Option<(String,)> =
        sqlx::query_as("SELECT value FROM jellyfin_meta WHERE key = 'server_id'")
            .fetch_optional(pool)
            .await?;
    if let Some((v,)) = existing {
        if v.len() == 32 && v.chars().all(|c| c.is_ascii_hexdigit()) {
            let upgraded = super::mapping::guid_dashed(&v);
            sqlx::query("UPDATE jellyfin_meta SET value = ?1 WHERE key = 'server_id'")
                .bind(&upgraded)
                .execute(pool)
                .await?;
            return Ok(upgraded);
        }
        return Ok(v);
    }
    let id = super::mapping::guid_dashed(&random_hex(16));
    sqlx::query(
        "INSERT INTO jellyfin_meta (key, value) VALUES ('server_id', ?1)
         ON CONFLICT(key) DO NOTHING",
    )
    .bind(&id)
    .execute(pool)
    .await?;
    let (val,): (String,) =
        sqlx::query_as("SELECT value FROM jellyfin_meta WHERE key = 'server_id'")
            .fetch_one(pool)
            .await?;
    Ok(val)
}

// ── FromRequest extractor for protected endpoints ─────────────────────────────

pub struct AuthedUser {
    #[allow(dead_code)]
    pub user_id: String,
}

impl FromRequest for AuthedUser {
    type Error = actix_web::Error;
    type Future = LocalBoxFuture<'static, Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let token = extract_token(req);
        let state = req.app_data::<web::Data<JellyfinState>>().cloned();
        Box::pin(async move {
            let Some(token) = token else {
                return Err(actix_web::error::ErrorUnauthorized("missing access token"));
            };
            let Some(state) = state else {
                return Err(actix_web::error::ErrorInternalServerError("missing state"));
            };
            if token_valid(&state.pool, &token).await {
                Ok(AuthedUser {
                    user_id: state.user_id.as_str().to_string(),
                })
            } else {
                Err(actix_web::error::ErrorUnauthorized("invalid token"))
            }
        })
    }
}
