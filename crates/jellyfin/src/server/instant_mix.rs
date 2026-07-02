//! Instant-mix seed algorithm.
//!
//! Given a seed (track / album / artist / playlist / genre) we produce an
//! audio-only track list that "goes with" the seed. Real Jellyfin runs
//! this against a recommendation engine; we approximate with a
//! deterministic-but-shuffled expansion:
//!
//! 1. The seed's own tracks (or the seed track itself) come first.
//! 2. Fill from the seed's artist (same `artist_id`).
//! 3. Fill from the seed's genre (same `genre_id`).
//! 4. If still short, pad with a random tail from the whole library.
//! 5. Shuffle the whole result and truncate to the requested `limit`.
//!
//! Dedup is by native id — the seed track is anchored at position 0 to
//! keep "start playing this song" behaviour intact.
use std::collections::HashSet;

use rand::seq::SliceRandom;
use rand::thread_rng;
use rockbox_library::entity::track::Track;
use rockbox_library::repo;
use rockbox_playlists::PlaylistStore;
use sqlx::{Pool, Sqlite};

use super::mapping::{KIND_ALBUM, KIND_ARTIST, KIND_PLAYLIST, KIND_TRACK};

pub const DEFAULT_LIMIT: usize = 50;

/// Anchor the seed track at position 0 if it's a track seed, then
/// shuffle the rest. Callers get a stable "play me first" ordering
/// even under repeated hits.
fn shuffle_but_anchor_head(mut tracks: Vec<Track>, anchor_head: bool) -> Vec<Track> {
    let mut rng = thread_rng();
    if anchor_head && !tracks.is_empty() {
        let head = tracks.remove(0);
        tracks.shuffle(&mut rng);
        tracks.insert(0, head);
    } else {
        tracks.shuffle(&mut rng);
    }
    tracks
}

fn dedup_by_id(tracks: Vec<Track>) -> Vec<Track> {
    let mut seen: HashSet<String> = HashSet::new();
    tracks
        .into_iter()
        .filter(|t| seen.insert(t.id.clone()))
        .collect()
}

/// Fill helper: append tracks matching the same artist / genre until
/// `desired` tracks are collected, always skipping ids already in `seen`.
async fn fill_from(
    pool: &Pool<Sqlite>,
    accumulator: &mut Vec<Track>,
    seen: &mut HashSet<String>,
    desired: usize,
    where_clause: (String, Vec<String>),
) {
    if accumulator.len() >= desired {
        return;
    }
    if let Ok(rows) = repo::track::filter(pool.clone(), where_clause).await {
        for t in rows {
            if seen.insert(t.id.clone()) {
                accumulator.push(t);
                if accumulator.len() >= desired {
                    return;
                }
            }
        }
    }
}

/// Fetch tracks for `native_id` and pad the result to `limit` items
/// with same-artist / same-genre / random picks. Non-track seeds return
/// the seed's own tracks first, then padding.
pub async fn generate(
    pool: &Pool<Sqlite>,
    playlist_store: &PlaylistStore,
    kind: &str,
    native_id: &str,
    limit: usize,
) -> Vec<Track> {
    let limit = limit.max(1);
    let mut acc: Vec<Track> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    let mut anchor_head = false;
    let mut hint_artist: Option<String> = None;
    let mut hint_genre: Option<String> = None;

    match kind {
        KIND_TRACK => {
            if let Ok(Some(t)) = repo::track::find(pool.clone(), native_id).await {
                hint_artist = Some(t.artist_id.clone());
                hint_genre = if t.genre_id.is_empty() {
                    None
                } else {
                    Some(t.genre_id.clone())
                };
                seen.insert(t.id.clone());
                acc.push(t);
                anchor_head = true;
            }
        }
        KIND_ALBUM => {
            if let Ok(rows) = repo::album_tracks::find_by_album(pool.clone(), native_id).await {
                for t in rows {
                    if hint_artist.is_none() {
                        hint_artist = Some(t.artist_id.clone());
                    }
                    if hint_genre.is_none() && !t.genre_id.is_empty() {
                        hint_genre = Some(t.genre_id.clone());
                    }
                    if seen.insert(t.id.clone()) {
                        acc.push(t);
                    }
                }
            }
        }
        KIND_ARTIST => {
            if let Ok(rows) = repo::artist_tracks::find_by_artist(pool.clone(), native_id).await {
                for t in rows {
                    if hint_genre.is_none() && !t.genre_id.is_empty() {
                        hint_genre = Some(t.genre_id.clone());
                    }
                    if seen.insert(t.id.clone()) {
                        acc.push(t);
                    }
                }
            }
            hint_artist = Some(native_id.to_string());
        }
        KIND_PLAYLIST => {
            let track_ids = playlist_store
                .get_track_ids(native_id)
                .await
                .unwrap_or_default();
            for tid in track_ids {
                if let Ok(Some(t)) = repo::track::find(pool.clone(), &tid).await {
                    if hint_artist.is_none() {
                        hint_artist = Some(t.artist_id.clone());
                    }
                    if hint_genre.is_none() && !t.genre_id.is_empty() {
                        hint_genre = Some(t.genre_id.clone());
                    }
                    if seen.insert(t.id.clone()) {
                        acc.push(t);
                    }
                }
            }
        }
        _ => return Vec::new(),
    }

    // Fill from same-artist matches (if we know an artist).
    if let Some(artist_id) = hint_artist.as_ref() {
        fill_from(
            pool,
            &mut acc,
            &mut seen,
            limit,
            ("artist_id = ?1".to_string(), vec![artist_id.clone()]),
        )
        .await;
    }

    // Fill from same-genre matches (if we know a genre).
    if let Some(genre_id) = hint_genre.as_ref() {
        fill_from(
            pool,
            &mut acc,
            &mut seen,
            limit,
            ("genre_id = ?1".to_string(), vec![genre_id.clone()]),
        )
        .await;
    }

    // Random tail — no filter, capped by SQLite's own ORDER BY RANDOM().
    if acc.len() < limit {
        if let Ok(rows) =
            repo::track::filtered(pool.clone(), None, None, None, limit as i64, 0).await
        {
            for t in rows {
                if seen.insert(t.id.clone()) {
                    acc.push(t);
                    if acc.len() >= limit {
                        break;
                    }
                }
            }
        }
    }

    let mixed = shuffle_but_anchor_head(dedup_by_id(acc), anchor_head);
    mixed.into_iter().take(limit).collect()
}

/// Genre-name seed — the `/MusicGenres/{name}/InstantMix` variant.
/// Resolves the name to a genre_id and returns all tracks in that
/// genre, padded with random picks if the pool is small.
pub async fn generate_from_genre_name(pool: &Pool<Sqlite>, name: &str, limit: usize) -> Vec<Track> {
    let limit = limit.max(1);
    let Ok(Some(genre)) = repo::genre::find_by_name(pool.clone(), name).await else {
        return Vec::new();
    };
    let mut acc = repo::genre::find_tracks(pool.clone(), &genre.id)
        .await
        .unwrap_or_default();
    let mut seen: HashSet<String> = acc.iter().map(|t| t.id.clone()).collect();
    if acc.len() < limit {
        if let Ok(rows) =
            repo::track::filtered(pool.clone(), None, None, None, limit as i64, 0).await
        {
            for t in rows {
                if seen.insert(t.id.clone()) {
                    acc.push(t);
                    if acc.len() >= limit {
                        break;
                    }
                }
            }
        }
    }
    let mixed = shuffle_but_anchor_head(dedup_by_id(acc), false);
    mixed.into_iter().take(limit).collect()
}
