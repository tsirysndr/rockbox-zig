//! Favorites store for the Jellyfin sidecar.
//!
//! `jf_favorites` is the source of truth for `IsFavorite` across tracks,
//! albums, artists, and playlists. For the two kinds that also exist in
//! rockbox-library — tracks and albums — we mirror writes to the shared
//! `favourites` table so smart-playlist rules (`is_liked`) and the
//! Subsonic bridge see the same state. Reads accept a row from either
//! table so likes added elsewhere still surface as favorites here.

use anyhow::Result;
use chrono::Utc;
use sqlx::{Pool, Sqlite};
use uuid::Uuid;

use super::mapping::{KIND_ALBUM, KIND_ARTIST, KIND_PLAYLIST, KIND_TRACK};

pub async fn is_favorite(pool: &Pool<Sqlite>, kind: &str, native_id: &str) -> bool {
    let jf: Option<(i64,)> =
        sqlx::query_as("SELECT 1 FROM jf_favorites WHERE kind = ?1 AND native_id = ?2 LIMIT 1")
            .bind(kind)
            .bind(native_id)
            .fetch_optional(pool)
            .await
            .unwrap_or(None);
    if jf.is_some() {
        return true;
    }
    // Fall back to the shared library `favourites` table so likes added via
    // the Subsonic bridge / GraphQL are honoured here without an explicit
    // sync step. Only tracks/albums live in that table.
    match kind {
        KIND_TRACK => {
            sqlx::query_as::<_, (i64,)>("SELECT 1 FROM favourites WHERE track_id = ?1 LIMIT 1")
                .bind(native_id)
                .fetch_optional(pool)
                .await
                .unwrap_or(None)
                .is_some()
        }
        KIND_ALBUM => {
            sqlx::query_as::<_, (i64,)>("SELECT 1 FROM favourites WHERE album_id = ?1 LIMIT 1")
                .bind(native_id)
                .fetch_optional(pool)
                .await
                .unwrap_or(None)
                .is_some()
        }
        _ => false,
    }
}

pub async fn mark(pool: &Pool<Sqlite>, kind: &str, native_id: &str) -> Result<()> {
    let now = Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT INTO jf_favorites (kind, native_id, favorited_at) VALUES (?1, ?2, ?3)
         ON CONFLICT(kind, native_id) DO NOTHING",
    )
    .bind(kind)
    .bind(native_id)
    .bind(&now)
    .execute(pool)
    .await?;

    // Mirror to the shared `favourites` table (tracks/albums only).
    match kind {
        KIND_TRACK => {
            let existing: Option<(String,)> =
                sqlx::query_as("SELECT id FROM favourites WHERE track_id = ?1 LIMIT 1")
                    .bind(native_id)
                    .fetch_optional(pool)
                    .await?;
            if existing.is_none() {
                sqlx::query(
                    "INSERT INTO favourites (id, track_id, album_id, created_at)
                     VALUES (?1, ?2, NULL, CURRENT_TIMESTAMP)",
                )
                .bind(Uuid::new_v4().to_string())
                .bind(native_id)
                .execute(pool)
                .await?;
            }
        }
        KIND_ALBUM => {
            let existing: Option<(String,)> =
                sqlx::query_as("SELECT id FROM favourites WHERE album_id = ?1 LIMIT 1")
                    .bind(native_id)
                    .fetch_optional(pool)
                    .await?;
            if existing.is_none() {
                sqlx::query(
                    "INSERT INTO favourites (id, track_id, album_id, created_at)
                     VALUES (?1, NULL, ?2, CURRENT_TIMESTAMP)",
                )
                .bind(Uuid::new_v4().to_string())
                .bind(native_id)
                .execute(pool)
                .await?;
            }
        }
        _ => {}
    }
    Ok(())
}

pub async fn unmark(pool: &Pool<Sqlite>, kind: &str, native_id: &str) -> Result<()> {
    sqlx::query("DELETE FROM jf_favorites WHERE kind = ?1 AND native_id = ?2")
        .bind(kind)
        .bind(native_id)
        .execute(pool)
        .await?;
    // Mirror-delete from the shared table too so both stay in sync.
    match kind {
        KIND_TRACK => {
            sqlx::query("DELETE FROM favourites WHERE track_id = ?1")
                .bind(native_id)
                .execute(pool)
                .await?;
        }
        KIND_ALBUM => {
            sqlx::query("DELETE FROM favourites WHERE album_id = ?1")
                .bind(native_id)
                .execute(pool)
                .await?;
        }
        _ => {}
    }
    Ok(())
}

/// Return the native ids of every favorited item of `kind`. Combines
/// rows from `jf_favorites` with the shared `favourites` table where
/// applicable — deduplicated.
pub async fn favorite_native_ids(pool: &Pool<Sqlite>, kind: &str) -> Vec<String> {
    let mut out: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    if let Ok(rows) =
        sqlx::query_as::<_, (String,)>("SELECT native_id FROM jf_favorites WHERE kind = ?1")
            .bind(kind)
            .fetch_all(pool)
            .await
    {
        out.extend(rows.into_iter().map(|(s,)| s));
    }
    let mirror_col = match kind {
        KIND_TRACK => Some("track_id"),
        KIND_ALBUM => Some("album_id"),
        _ => None,
    };
    if let Some(col) = mirror_col {
        let sql = format!("SELECT {col} FROM favourites WHERE {col} IS NOT NULL AND {col} != ''");
        if let Ok(rows) = sqlx::query_as::<_, (String,)>(&sql).fetch_all(pool).await {
            out.extend(rows.into_iter().map(|(s,)| s));
        }
    }
    out.into_iter().collect()
}

/// Convenience helper for callers that don't want to import kind constants.
pub fn all_kinds() -> [&'static str; 4] {
    [KIND_TRACK, KIND_ALBUM, KIND_ARTIST, KIND_PLAYLIST]
}
