//! GUID translation between rockbox-library's native string ids and the
//! dashed-UUID format Jellyfin clients require.

use rockbox_library::entity::{album::Album, artist::Artist, track::Track};
use sha2::{Digest, Sha256};
use sqlx::{Pool, Sqlite};

pub const KIND_ARTIST: &str = "artist";
pub const KIND_ALBUM: &str = "album";
pub const KIND_TRACK: &str = "track";
pub const KIND_PLAYLIST: &str = "playlist";
pub const KIND_GENRE: &str = "genre";
pub const KIND_LIBRARY: &str = "library";
pub const KIND_USER: &str = "user";

/// Stable per-(kind, native_id) GUID formatted as a dashed UUID. Jellyfin
/// clients built on the official Kotlin/Java SDKs parse this via
/// `UUID.fromString()`, which rejects un-dashed 32-char hex.
pub fn guid(kind: &str, native_id: &str) -> String {
    let mut h = Sha256::new();
    h.update(kind.as_bytes());
    h.update(b":");
    h.update(native_id.as_bytes());
    let digest = h.finalize();
    format_as_uuid(&hex::encode(&digest[..16]))
}

pub fn guid_dashed(hex32: &str) -> String {
    format_as_uuid(hex32)
}

fn format_as_uuid(hex32: &str) -> String {
    if hex32.len() != 32 {
        return hex32.to_string();
    }
    format!(
        "{}-{}-{}-{}-{}",
        &hex32[0..8],
        &hex32[8..12],
        &hex32[12..16],
        &hex32[16..20],
        &hex32[20..32],
    )
}

/// Accept either dashed or un-dashed input from clients and return the
/// canonical dashed form we store in `jf_guids`.
pub fn normalize_guid(input: &str) -> String {
    let stripped: String = input
        .chars()
        .filter(|c| *c != '-')
        .flat_map(|c| c.to_lowercase())
        .collect();
    if stripped.len() == 32 && stripped.chars().all(|c| c.is_ascii_hexdigit()) {
        format_as_uuid(&stripped)
    } else {
        input.to_ascii_lowercase()
    }
}

pub async fn remember(pool: &Pool<Sqlite>, kind: &str, native_id: &str) -> anyhow::Result<String> {
    let g = guid(kind, native_id);
    sqlx::query(
        "INSERT INTO jf_guids (guid, kind, native_id) VALUES (?1, ?2, ?3)
         ON CONFLICT(guid) DO NOTHING",
    )
    .bind(&g)
    .bind(kind)
    .bind(native_id)
    .execute(pool)
    .await?;
    Ok(g)
}

pub async fn lookup(pool: &Pool<Sqlite>, guid: &str) -> anyhow::Result<Option<(String, String)>> {
    let g = normalize_guid(guid);
    let row: Option<(String, String)> =
        sqlx::query_as("SELECT kind, native_id FROM jf_guids WHERE guid = ?1")
            .bind(&g)
            .fetch_optional(pool)
            .await?;
    Ok(row)
}

pub async fn remember_artist(pool: &Pool<Sqlite>, a: &Artist) -> anyhow::Result<String> {
    remember(pool, KIND_ARTIST, &a.id).await
}

pub async fn remember_album(pool: &Pool<Sqlite>, a: &Album) -> anyhow::Result<String> {
    remember(pool, KIND_ALBUM, &a.id).await
}

pub async fn remember_track(pool: &Pool<Sqlite>, t: &Track) -> anyhow::Result<String> {
    remember(pool, KIND_TRACK, &t.id).await
}

pub fn library_guid() -> String {
    guid(KIND_LIBRARY, "music")
}

pub fn playlists_library_guid() -> String {
    guid(KIND_LIBRARY, "playlists")
}

pub fn user_guid(username: &str) -> String {
    guid(KIND_USER, username)
}

pub async fn remember_genre(pool: &Pool<Sqlite>, native_id: &str) -> anyhow::Result<String> {
    remember(pool, KIND_GENRE, native_id).await
}

pub async fn remember_playlist(pool: &Pool<Sqlite>, native_id: &str) -> anyhow::Result<String> {
    remember(pool, KIND_PLAYLIST, native_id).await
}

/// Deterministic per-entry GUID for `PlaylistItemId`. Emby / Jellyfin clients
/// pass this back in `EntryIds` when removing or moving items. Reversible on
/// the server side by iterating the playlist once and matching the GUID.
pub fn playlist_entry_guid(playlist_native_id: &str, position: i64) -> String {
    guid(
        "playlist_entry",
        &format!("{playlist_native_id}:{position}"),
    )
}
