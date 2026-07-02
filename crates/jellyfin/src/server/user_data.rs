//! Per-item user-data store for the Jellyfin API.
//!
//! Backs the `UserItemDataDto` fields Jellyfin clients round-trip through
//! GET/POST `/UserItems/{itemId}/UserData` — playback progress, play
//! count, likes, and rating. `IsFavorite` still lives in [`favorites`].
//!
//! For tracks we merge with rockbox-playlists' `track_stats` on read so
//! playback counters from the audio engine appear on the Jellyfin side
//! without an explicit sync. Writes go only to `jf_user_data` so
//! Jellyfin edits never disturb the engine's own bookkeeping.

use anyhow::Result;
use chrono::{DateTime, TimeZone, Utc};
use sqlx::{Pool, Row, Sqlite};

use super::mapping::KIND_TRACK;

/// Rolled-up user data for one item, ready to be serialized as
/// `UserItemDataDto`.
#[derive(Debug, Clone, Default)]
pub struct UserData {
    pub played: bool,
    pub play_count: i32,
    pub playback_position_ticks: i64,
    pub last_played_date: Option<String>,
    pub likes: Option<bool>,
    pub rating: Option<f64>,
}

/// Partial patch — every field is `None` = leave unchanged. Present
/// values overwrite the stored row (matches Jellyfin's spec: unset
/// fields on the request should not clobber stored state).
#[derive(Debug, Default)]
pub struct UserDataPatch {
    pub played: Option<bool>,
    pub play_count: Option<i32>,
    pub playback_position_ticks: Option<i64>,
    pub last_played_date: Option<String>,
    pub likes: Option<bool>,
    pub rating: Option<f64>,
}

fn row_to_user_data(row: &sqlx::sqlite::SqliteRow) -> UserData {
    UserData {
        played: row.try_get::<i64, _>("played").unwrap_or(0) != 0,
        play_count: row.try_get::<i64, _>("play_count").unwrap_or(0) as i32,
        playback_position_ticks: row
            .try_get::<i64, _>("playback_position_ticks")
            .unwrap_or(0),
        last_played_date: row
            .try_get::<Option<String>, _>("last_played_at")
            .unwrap_or(None),
        likes: row
            .try_get::<Option<i64>, _>("likes")
            .unwrap_or(None)
            .map(|v| v != 0),
        rating: row.try_get::<Option<f64>, _>("rating").unwrap_or(None),
    }
}

/// Read the rolled-up user data for `(kind, native_id)`. For tracks the
/// result also reflects `track_stats` — whichever counter is higher
/// wins so a Jellyfin-side manual set doesn't get overwritten by the
/// engine's next play tick.
pub async fn get(pool: &Pool<Sqlite>, kind: &str, native_id: &str) -> UserData {
    let mut ud = sqlx::query(
        "SELECT played, play_count, playback_position_ticks, last_played_at, likes, rating
         FROM jf_user_data WHERE kind = ?1 AND native_id = ?2",
    )
    .bind(kind)
    .bind(native_id)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten()
    .as_ref()
    .map(row_to_user_data)
    .unwrap_or_default();

    // Merge with rockbox-playlists' track_stats for tracks.
    if kind == KIND_TRACK {
        if let Ok(Some(row)) =
            sqlx::query("SELECT play_count, last_played FROM track_stats WHERE track_id = ?1")
                .bind(native_id)
                .fetch_optional(pool)
                .await
        {
            let ts_play_count = row.try_get::<i64, _>("play_count").unwrap_or(0) as i32;
            let ts_last_played = row.try_get::<Option<i64>, _>("last_played").unwrap_or(None);
            if ts_play_count > ud.play_count {
                ud.play_count = ts_play_count;
                if !ud.played {
                    ud.played = ts_play_count > 0;
                }
            }
            if ud.last_played_date.is_none() {
                if let Some(secs) = ts_last_played {
                    if let Some(dt) = Utc.timestamp_opt(secs, 0).single() {
                        ud.last_played_date = Some(iso_naive(dt));
                    }
                }
            }
        }
    }

    ud
}

/// Apply `patch` to the stored row, creating it if missing. Fields set
/// to `None` are preserved; fields set to `Some(v)` overwrite.
pub async fn update(
    pool: &Pool<Sqlite>,
    kind: &str,
    native_id: &str,
    patch: UserDataPatch,
) -> Result<UserData> {
    let existing = sqlx::query(
        "SELECT played, play_count, playback_position_ticks, last_played_at, likes, rating
         FROM jf_user_data WHERE kind = ?1 AND native_id = ?2",
    )
    .bind(kind)
    .bind(native_id)
    .fetch_optional(pool)
    .await?
    .as_ref()
    .map(row_to_user_data)
    .unwrap_or_default();

    let played = patch.played.unwrap_or(existing.played);
    let play_count = patch.play_count.unwrap_or(existing.play_count);
    let playback_position_ticks = patch
        .playback_position_ticks
        .unwrap_or(existing.playback_position_ticks);
    let last_played_date = patch
        .last_played_date
        .clone()
        .or(existing.last_played_date.clone());
    let likes = patch.likes.or(existing.likes);
    let rating = patch.rating.or(existing.rating);
    let updated_at = Utc::now().to_rfc3339();

    sqlx::query(
        "INSERT INTO jf_user_data
             (kind, native_id, played, play_count, playback_position_ticks,
              last_played_at, likes, rating, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
         ON CONFLICT(kind, native_id) DO UPDATE SET
             played = excluded.played,
             play_count = excluded.play_count,
             playback_position_ticks = excluded.playback_position_ticks,
             last_played_at = excluded.last_played_at,
             likes = excluded.likes,
             rating = excluded.rating,
             updated_at = excluded.updated_at",
    )
    .bind(kind)
    .bind(native_id)
    .bind(played as i64)
    .bind(play_count as i64)
    .bind(playback_position_ticks)
    .bind(&last_played_date)
    .bind(likes.map(|b| b as i64))
    .bind(rating)
    .bind(&updated_at)
    .execute(pool)
    .await?;

    Ok(UserData {
        played,
        play_count,
        playback_position_ticks,
        last_played_date,
        likes,
        rating,
    })
}

/// Naive 7-digit fractional-second ISO format that jellyfin-sdk-kotlin
/// deserializes via `LocalDateTime` — same format used elsewhere in the
/// handler layer.
fn iso_naive(dt: DateTime<Utc>) -> String {
    let ticks = dt.timestamp_subsec_nanos() / 100;
    format!("{}.{ticks:07}", dt.format("%Y-%m-%dT%H:%M:%S"))
}
