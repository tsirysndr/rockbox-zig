//! SQLite FTS5 backed search, exposing the same `search_*` API surface as
//! `rockbox_typesense::client` so callers can swap implementations behind a
//! single cargo feature.
//!
//! Each function returns `Option<*Result>` from `rockbox_typesense::types` so
//! the result shape stays identical to the Typesense path. Numeric fields
//! that don't exist in FTS5 (`text_match`, `search_time_ms`, etc.) are filled
//! with sensible defaults — callers only read `hits[].document` today.

use anyhow::Error;
use rockbox_library::entity;
use rockbox_typesense::types::{
    Album, AlbumHit, AlbumResult, Artist, ArtistHit, ArtistResult, Playlist, PlaylistHit,
    PlaylistResult, Track, TrackHit, TrackResult,
};
use sqlx::{Pool, Sqlite};
use tracing::warn;

const RESULT_LIMIT: i64 = 50;

/// Sanitize free-form user input into a safe FTS5 MATCH expression.
///
/// FTS5 MATCH syntax interprets characters like `"`, `*`, `(`, `)`, `:`, `^`,
/// `-`, `+`, `AND`, `OR`, `NOT` and `NEAR` specially; passing a raw user
/// string would either error or match the wrong rows. We strip the dangerous
/// punctuation, drop empty tokens, then quote each remaining token and append
/// a `*` for prefix matching so partial typing (`"beyo"` → `"beyonce"`) works.
fn build_match_expression(query: &str) -> Option<String> {
    let cleaned: String = query
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c.is_whitespace() {
                c
            } else {
                ' '
            }
        })
        .collect();
    let tokens: Vec<String> = cleaned
        .split_whitespace()
        .filter(|t| !t.is_empty())
        .map(|t| format!("\"{}\"*", t))
        .collect();
    if tokens.is_empty() {
        None
    } else {
        Some(tokens.join(" "))
    }
}

pub async fn search_tracks(pool: Pool<Sqlite>, query: &str) -> Result<Option<TrackResult>, Error> {
    let expr = match build_match_expression(query) {
        Some(e) => e,
        None => return Ok(Some(TrackResult::default())),
    };

    let rows: Vec<entity::track::Track> = match sqlx::query_as(
        r#"
        SELECT t.*
        FROM track t
        JOIN track_fts f ON f.id = t.id
        WHERE track_fts MATCH ?
          AND t.is_remote = 0
        ORDER BY rank
        LIMIT ?
        "#,
    )
    .bind(&expr)
    .bind(RESULT_LIMIT)
    .fetch_all(&pool)
    .await
    {
        Ok(rows) => rows,
        Err(e) => {
            warn!("fts5 search_tracks failed: {}", e);
            return Ok(Some(TrackResult::default()));
        }
    };

    let hits: Vec<TrackHit> = rows
        .into_iter()
        .map(|t| TrackHit {
            document: Track::from(t),
            ..Default::default()
        })
        .collect();
    let found = hits.len() as i64;

    Ok(Some(TrackResult {
        found,
        out_of: found,
        hits,
        ..Default::default()
    }))
}

pub async fn search_albums(pool: Pool<Sqlite>, query: &str) -> Result<Option<AlbumResult>, Error> {
    let expr = match build_match_expression(query) {
        Some(e) => e,
        None => return Ok(Some(AlbumResult::default())),
    };

    let rows: Vec<entity::album::Album> = match sqlx::query_as(
        r#"
        SELECT a.*
        FROM album a
        JOIN album_fts f ON f.id = a.id
        WHERE album_fts MATCH ?
        ORDER BY rank
        LIMIT ?
        "#,
    )
    .bind(&expr)
    .bind(RESULT_LIMIT)
    .fetch_all(&pool)
    .await
    {
        Ok(rows) => rows,
        Err(e) => {
            warn!("fts5 search_albums failed: {}", e);
            return Ok(Some(AlbumResult::default()));
        }
    };

    let hits: Vec<AlbumHit> = rows
        .into_iter()
        .map(|a| AlbumHit {
            document: Album::from(a),
            ..Default::default()
        })
        .collect();
    let found = hits.len() as i64;

    Ok(Some(AlbumResult {
        found,
        out_of: found,
        hits,
        ..Default::default()
    }))
}

pub async fn search_artists(
    pool: Pool<Sqlite>,
    query: &str,
) -> Result<Option<ArtistResult>, Error> {
    let expr = match build_match_expression(query) {
        Some(e) => e,
        None => return Ok(Some(ArtistResult::default())),
    };

    let rows: Vec<entity::artist::Artist> = match sqlx::query_as(
        r#"
        SELECT a.*
        FROM artist a
        JOIN artist_fts f ON f.id = a.id
        WHERE artist_fts MATCH ?
        ORDER BY rank
        LIMIT ?
        "#,
    )
    .bind(&expr)
    .bind(RESULT_LIMIT)
    .fetch_all(&pool)
    .await
    {
        Ok(rows) => rows,
        Err(e) => {
            warn!("fts5 search_artists failed: {}", e);
            return Ok(Some(ArtistResult::default()));
        }
    };

    let hits: Vec<ArtistHit> = rows
        .into_iter()
        .map(|a| ArtistHit {
            document: Artist::from(a),
            ..Default::default()
        })
        .collect();
    let found = hits.len() as i64;

    Ok(Some(ArtistResult {
        found,
        out_of: found,
        hits,
        ..Default::default()
    }))
}

#[derive(sqlx::FromRow)]
struct PlaylistFtsHit {
    id: String,
    is_smart: i64,
    name: String,
    description: Option<String>,
    image: Option<String>,
    track_count: i64,
}

pub async fn search_playlists(
    pool: Pool<Sqlite>,
    query: &str,
) -> Result<Option<PlaylistResult>, Error> {
    let expr = match build_match_expression(query) {
        Some(e) => e,
        None => return Ok(Some(PlaylistResult::default())),
    };

    // playlist_fts is the union index over saved_playlists + smart_playlists;
    // we re-join each side back to its source table to recover image / track
    // count, then UNION the two halves and re-sort by FTS rank.
    let rows: Vec<PlaylistFtsHit> = match sqlx::query_as(
        r#"
        SELECT p.id AS id,
               0 AS is_smart,
               p.name AS name,
               p.description AS description,
               p.image AS image,
               COALESCE((SELECT COUNT(*) FROM saved_playlist_tracks t WHERE t.playlist_id = p.id), 0) AS track_count,
               f.rank AS rank
        FROM saved_playlists p
        JOIN playlist_fts f ON f.id = p.id AND f.is_smart = 0
        WHERE playlist_fts MATCH ?1
        UNION ALL
        SELECT p.id AS id,
               1 AS is_smart,
               p.name AS name,
               p.description AS description,
               p.image AS image,
               0 AS track_count,
               f.rank AS rank
        FROM smart_playlists p
        JOIN playlist_fts f ON f.id = p.id AND f.is_smart = 1
        WHERE playlist_fts MATCH ?1
        ORDER BY rank
        LIMIT ?2
        "#,
    )
    .bind(&expr)
    .bind(RESULT_LIMIT)
    .fetch_all(&pool)
    .await
    {
        Ok(rows) => rows,
        Err(e) => {
            warn!("fts5 search_playlists failed: {}", e);
            return Ok(Some(PlaylistResult::default()));
        }
    };

    let hits: Vec<PlaylistHit> = rows
        .into_iter()
        .map(|r| PlaylistHit {
            document: Playlist {
                id: r.id,
                name: r.name,
                description: r.description,
                image: r.image,
                is_smart: r.is_smart != 0,
                track_count: r.track_count,
            },
            ..Default::default()
        })
        .collect();
    let found = hits.len() as i64;

    Ok(Some(PlaylistResult {
        found,
        out_of: found,
        hits,
        ..Default::default()
    }))
}
