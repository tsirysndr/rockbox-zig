//! Cached Last.fm enrichment for artist DTOs.
//!
//! Reads populate `Overview` / `Genres` on every `artist_to_dto` call
//! (SQLite cache lookup — no network). Detail handlers
//! (`/Items/{id}`, `/Users/{uid}/Items/{id}`, `/Artists/{name}`) call
//! [`refresh_artist`] to fetch fresh data from Last.fm when the cache
//! is missing or older than [`ARTIST_TTL_SECS`]. Stale rows are still
//! served to avoid blocking on network jank.

use chrono::Utc;
use sqlx::{Pool, Sqlite};

use super::lastfm::LastFm;

/// 7 days — long enough that a hot library isn't hammering Last.fm,
/// short enough that a corrected bio propagates within a week.
pub const ARTIST_TTL_SECS: i64 = 7 * 24 * 60 * 60;

#[derive(Debug, Clone, Default)]
pub struct ArtistEnrichment {
    pub bio: Option<String>,
    pub tags: Vec<String>,
    pub image_url: Option<String>,
    pub fetched_at: i64,
}

impl ArtistEnrichment {
    pub fn is_stale(&self) -> bool {
        Utc::now().timestamp() - self.fetched_at >= ARTIST_TTL_SECS
    }
}

/// Cache-only read. Returns `None` when the artist has never been
/// enriched — call [`refresh_artist`] to populate it.
pub async fn get_artist(pool: &Pool<Sqlite>, native_id: &str) -> Option<ArtistEnrichment> {
    let row: Option<(Option<String>, Option<String>, Option<String>, i64)> = sqlx::query_as(
        "SELECT bio, tags, image_url, fetched_at
         FROM jf_artist_enrichment WHERE artist_id = ?1",
    )
    .bind(native_id)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();
    let (bio, tags_json, image_url, fetched_at) = row?;
    let tags: Vec<String> = tags_json
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default();
    Some(ArtistEnrichment {
        bio,
        tags,
        image_url,
        fetched_at,
    })
}

/// Fetch fresh data from Last.fm and upsert the cache. When Last.fm
/// is not configured (`lastfm` is `None`), returns whatever the cache
/// holds — including `None`.
pub async fn refresh_artist(
    pool: &Pool<Sqlite>,
    lastfm: Option<&LastFm>,
    native_id: &str,
    artist_name: &str,
) -> Option<ArtistEnrichment> {
    if let Some(cached) = get_artist(pool, native_id).await {
        if !cached.is_stale() {
            return Some(cached);
        }
    }
    let Some(lastfm) = lastfm else {
        // No plugin — return whatever (stale) cache we might already have.
        return get_artist(pool, native_id).await;
    };
    let info = match lastfm.artist_info(Some(artist_name), None).await {
        Ok(i) => i,
        Err(e) => {
            tracing::debug!("lastfm artist.getInfo({artist_name}): {e}");
            // Serve any stale cache we have on error so a network blip
            // doesn't wipe the enrichment.
            return get_artist(pool, native_id).await;
        }
    };
    let now = Utc::now().timestamp();
    let tags_json = serde_json::to_string(&info.tags).unwrap_or_else(|_| "[]".to_string());
    let _ = sqlx::query(
        "INSERT INTO jf_artist_enrichment (artist_id, mbid, bio, tags, image_url, fetched_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)
         ON CONFLICT(artist_id) DO UPDATE SET
             mbid = excluded.mbid,
             bio = excluded.bio,
             tags = excluded.tags,
             image_url = excluded.image_url,
             fetched_at = excluded.fetched_at",
    )
    .bind(native_id)
    .bind(&info.mbid)
    .bind(&info.bio)
    .bind(&tags_json)
    .bind(&info.image_url)
    .bind(now)
    .execute(pool)
    .await;

    Some(ArtistEnrichment {
        bio: info.bio,
        tags: info.tags,
        image_url: info.image_url,
        fetched_at: now,
    })
}
