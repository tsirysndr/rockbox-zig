use crate::rules::{resolve, Candidate, RuleCriteria};
use crate::PlaylistStore;
use anyhow::Result;
use rockbox_library::repo;
use sqlx::{Pool, Sqlite};
use std::collections::{HashMap, HashSet};

/// Build the candidate vector from the library's local tracks, joined with
/// playback stats and likes. Reused by smart playlists and the artist/album
/// advanced filter endpoints.
pub async fn build_candidates(
    store: &PlaylistStore,
    pool: &Pool<Sqlite>,
) -> Result<(Vec<Candidate>, Vec<rockbox_library::entity::track::Track>)> {
    let all_tracks = repo::track::all(pool.clone()).await?;

    let stats_map: HashMap<String, crate::TrackStats> = store
        .get_all_track_stats()
        .await?
        .into_iter()
        .map(|s| (s.track_id.clone(), s))
        .collect();

    let liked_ids: HashSet<String> = repo::favourites::all_tracks(pool.clone())
        .await?
        .into_iter()
        .map(|t| t.id)
        .collect();

    let candidates: Vec<Candidate> = all_tracks
        .iter()
        .map(|t| {
            let stats = stats_map.get(&t.id);
            Candidate {
                id: t.id.clone(),
                title: t.title.clone(),
                artist: t.artist.clone(),
                album: t.album.clone(),
                year: t.year.map(|y| y as i64),
                genre: t.genre.clone(),
                duration_ms: t.length as i64 * 1000,
                bitrate: t.bitrate as i64,
                date_added_ts: t.created_at.timestamp(),
                play_count: stats.map(|s| s.play_count).unwrap_or(0),
                skip_count: stats.map(|s| s.skip_count).unwrap_or(0),
                last_played: stats.and_then(|s| s.last_played),
                last_skipped: stats.and_then(|s| s.last_skipped),
                is_liked: liked_ids.contains(&t.id),
            }
        })
        .collect();

    Ok((candidates, all_tracks))
}

/// Resolve a rule criteria over the library and return matching tracks
/// (in the order produced by the rule resolver — i.e. honouring sort/limit).
pub async fn resolve_tracks(
    store: &PlaylistStore,
    pool: &Pool<Sqlite>,
    criteria: &RuleCriteria,
) -> Result<Vec<rockbox_library::entity::track::Track>> {
    let (candidates, all_tracks) = build_candidates(store, pool).await?;
    let resolved = resolve(criteria, candidates);
    let track_map: HashMap<&str, &rockbox_library::entity::track::Track> =
        all_tracks.iter().map(|t| (t.id.as_str(), t)).collect();
    Ok(resolved
        .iter()
        .filter_map(|c| track_map.get(c.id.as_str()).map(|t| (*t).clone()))
        .collect())
}

/// Count how many tracks would be included in the smart playlist
/// described by `criteria`.
pub async fn count_tracks(
    store: &PlaylistStore,
    pool: &Pool<Sqlite>,
    criteria: &RuleCriteria,
) -> Result<i64> {
    let (candidates, _) = build_candidates(store, pool).await?;
    let resolved = resolve(criteria, candidates);
    Ok(resolved.len() as i64)
}

/// Resolve and return the unique album ids from the matching tracks,
/// preserving the order in which the resolver returned them (i.e. the
/// album of the first matching track first, etc.).
pub async fn resolve_album_ids(
    store: &PlaylistStore,
    pool: &Pool<Sqlite>,
    criteria: &RuleCriteria,
) -> Result<Vec<String>> {
    let tracks = resolve_tracks(store, pool, criteria).await?;
    let mut seen: HashSet<String> = HashSet::new();
    let mut ordered: Vec<String> = Vec::new();
    for t in tracks {
        if seen.insert(t.album_id.clone()) {
            ordered.push(t.album_id);
        }
    }
    Ok(ordered)
}

/// Resolve and return the unique artist ids from the matching tracks,
/// preserving order.
pub async fn resolve_artist_ids(
    store: &PlaylistStore,
    pool: &Pool<Sqlite>,
    criteria: &RuleCriteria,
) -> Result<Vec<String>> {
    let tracks = resolve_tracks(store, pool, criteria).await?;
    let mut seen: HashSet<String> = HashSet::new();
    let mut ordered: Vec<String> = Vec::new();
    for t in tracks {
        if seen.insert(t.artist_id.clone()) {
            ordered.push(t.artist_id);
        }
    }
    Ok(ordered)
}

/// Resolve a `RuleCriteria` and return the matching `Album`s. Albums are
/// returned in the order their first matching track appears in the resolver
/// output.
pub async fn filter_albums(
    store: &PlaylistStore,
    pool: &Pool<Sqlite>,
    criteria: &RuleCriteria,
) -> Result<Vec<rockbox_library::entity::album::Album>> {
    let ordered_ids = resolve_album_ids(store, pool, criteria).await?;
    let mut albums = Vec::with_capacity(ordered_ids.len());
    for id in ordered_ids {
        if let Some(album) = repo::album::find(pool.clone(), &id).await? {
            albums.push(album);
        }
    }
    Ok(albums)
}

/// Resolve a `RuleCriteria` and return the matching `Artist`s.
pub async fn filter_artists(
    store: &PlaylistStore,
    pool: &Pool<Sqlite>,
    criteria: &RuleCriteria,
) -> Result<Vec<rockbox_library::entity::artist::Artist>> {
    let ordered_ids = resolve_artist_ids(store, pool, criteria).await?;
    let mut artists = Vec::with_capacity(ordered_ids.len());
    for id in ordered_ids {
        if let Some(artist) = repo::artist::find(pool.clone(), &id).await? {
            artists.push(artist);
        }
    }
    Ok(artists)
}
