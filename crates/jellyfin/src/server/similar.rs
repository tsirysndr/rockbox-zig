//! Similar-item orchestrator.
//!
//! Ties the Last.fm and MusicBrainz plugins to the local rockbox
//! library. The public entry point [`similar`] takes a seed
//! (kind, native_id) plus a `limit` and returns local `Track` / `Album`
//! / `Artist` records ordered by how well they match the seed.
//!
//! Both plugins are optional; when the config carries neither we short
//! circuit with an empty result — that matches the "only enabled if
//! tokens are present" requirement in the settings.

use rockbox_library::entity::{album::Album, artist::Artist, track::Track};
use rockbox_library::repo;
use sqlx::{Pool, Sqlite};

use super::lastfm::LastFm;
use super::mapping::{KIND_ALBUM, KIND_ARTIST, KIND_TRACK};
use super::musicbrainz::MusicBrainz;

/// Union return type so a single `/Items/{id}/Similar` handler can
/// dispatch on `kind`. Only the variant matching the seed will be
/// populated — the others are empty.
#[derive(Default)]
pub struct SimilarResult {
    pub tracks: Vec<Track>,
    pub albums: Vec<Album>,
    pub artists: Vec<Artist>,
}

impl SimilarResult {
    pub fn is_empty(&self) -> bool {
        self.tracks.is_empty() && self.albums.is_empty() && self.artists.is_empty()
    }
}

/// Compute similar items for the given seed. Returns an empty
/// [`SimilarResult`] when Last.fm is not configured — the whole
/// endpoint is intentionally a no-op without a token.
pub async fn similar(
    pool: &Pool<Sqlite>,
    lastfm: Option<&LastFm>,
    mb: Option<&MusicBrainz>,
    kind: &str,
    native_id: &str,
    limit: usize,
) -> SimilarResult {
    let Some(lastfm) = lastfm else {
        return SimilarResult::default();
    };
    let limit = limit.max(1);
    match kind {
        KIND_ARTIST => similar_artists(pool, lastfm, mb, native_id, limit).await,
        KIND_ALBUM => similar_albums(pool, lastfm, mb, native_id, limit).await,
        KIND_TRACK => similar_tracks(pool, lastfm, mb, native_id, limit).await,
        _ => SimilarResult::default(),
    }
}

// ── Artists ─────────────────────────────────────────────────────────────────

async fn similar_artists(
    pool: &Pool<Sqlite>,
    lastfm: &LastFm,
    mb: Option<&MusicBrainz>,
    native_id: &str,
    limit: usize,
) -> SimilarResult {
    let Ok(Some(seed)) = repo::artist::find(pool.clone(), native_id).await else {
        return SimilarResult::default();
    };
    let suggestions = lastfm
        .similar_artists(Some(&seed.name), None, limit * 2)
        .await
        .unwrap_or_default();

    let mut hits: Vec<Artist> = Vec::new();
    for s in suggestions {
        if hits.len() >= limit {
            break;
        }
        let mut canonical = s.name.clone();
        if let (Some(mb), Some(mbid)) = (mb, s.mbid.as_deref()) {
            if let Some(lookup) = mb.artist_by_mbid(mbid).await {
                canonical = lookup.name;
            }
        }
        if let Some(local) = find_local_artist(pool, &canonical).await {
            if !hits.iter().any(|a| a.id == local.id) {
                hits.push(local);
            }
        }
    }
    SimilarResult {
        artists: hits,
        ..Default::default()
    }
}

async fn find_local_artist(pool: &Pool<Sqlite>, name: &str) -> Option<Artist> {
    // Exact-name lookup first (cheap), then case-insensitive.
    let exact = repo::artist::filter(
        pool.clone(),
        ("name = ?1".to_string(), vec![name.to_string()]),
    )
    .await
    .ok()?;
    if let Some(a) = exact.into_iter().next() {
        return Some(a);
    }
    let fuzzy = repo::artist::filter(
        pool.clone(),
        (
            "LOWER(name) = LOWER(?1)".to_string(),
            vec![name.to_string()],
        ),
    )
    .await
    .ok()?;
    fuzzy.into_iter().next()
}

// ── Albums ──────────────────────────────────────────────────────────────────

/// Album similarity has no direct Last.fm endpoint. We expand via the
/// album's primary artist: fetch similar artists, then for each of them
/// pull their local albums. `limit` bounds the final unique album list.
async fn similar_albums(
    pool: &Pool<Sqlite>,
    lastfm: &LastFm,
    mb: Option<&MusicBrainz>,
    native_id: &str,
    limit: usize,
) -> SimilarResult {
    let Ok(Some(seed_album)) = repo::album::find(pool.clone(), native_id).await else {
        return SimilarResult::default();
    };
    let Ok(Some(seed_artist)) = repo::artist::find(pool.clone(), &seed_album.artist_id).await
    else {
        return SimilarResult::default();
    };
    let suggestions = lastfm
        .similar_artists(Some(&seed_artist.name), None, limit)
        .await
        .unwrap_or_default();

    let mut albums: Vec<Album> = Vec::new();
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    seen.insert(seed_album.id.clone());
    for s in suggestions {
        if albums.len() >= limit {
            break;
        }
        let mut canonical = s.name.clone();
        if let (Some(mb), Some(mbid)) = (mb, s.mbid.as_deref()) {
            if let Some(lookup) = mb.artist_by_mbid(mbid).await {
                canonical = lookup.name;
            }
        }
        let Some(local_artist) = find_local_artist(pool, &canonical).await else {
            continue;
        };
        let by_artist = repo::album::find_by_artist(pool.clone(), &local_artist.id)
            .await
            .unwrap_or_default();
        for a in by_artist {
            if seen.insert(a.id.clone()) {
                albums.push(a);
                if albums.len() >= limit {
                    break;
                }
            }
        }
    }
    SimilarResult {
        albums,
        ..Default::default()
    }
}

// ── Tracks ──────────────────────────────────────────────────────────────────

async fn similar_tracks(
    pool: &Pool<Sqlite>,
    lastfm: &LastFm,
    mb: Option<&MusicBrainz>,
    native_id: &str,
    limit: usize,
) -> SimilarResult {
    let Ok(Some(seed)) = repo::track::find(pool.clone(), native_id).await else {
        return SimilarResult::default();
    };
    let suggestions = lastfm
        .similar_tracks(Some(&seed.title), Some(&seed.artist), None, limit * 2)
        .await
        .unwrap_or_default();

    let mut hits: Vec<Track> = Vec::new();
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    seen.insert(seed.id.clone());
    for s in suggestions {
        if hits.len() >= limit {
            break;
        }
        let mut canonical_artist = s.artist.clone();
        if let (Some(mb), Some(mbid)) = (mb, s.artist_mbid.as_deref()) {
            if let Some(lookup) = mb.artist_by_mbid(mbid).await {
                canonical_artist = lookup.name;
            }
        }
        if let Some(local) = find_local_track(pool, &s.title, &canonical_artist).await {
            if seen.insert(local.id.clone()) {
                hits.push(local);
            }
        }
    }
    SimilarResult {
        tracks: hits,
        ..Default::default()
    }
}

async fn find_local_track(pool: &Pool<Sqlite>, title: &str, artist: &str) -> Option<Track> {
    let matches = repo::track::filter(
        pool.clone(),
        (
            "LOWER(title) = LOWER(?1) AND LOWER(artist) = LOWER(?2)".to_string(),
            vec![title.to_string(), artist.to_string()],
        ),
    )
    .await
    .ok()?;
    matches.into_iter().next()
}
