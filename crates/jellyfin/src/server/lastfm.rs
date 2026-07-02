//! Last.fm HTTP client for the Similar endpoints.
//!
//! Gated on a `lastfm_api_key` in `settings.toml` — [`LastFm::from_key`]
//! returns `None` when the key is absent, and all callers must
//! short-circuit through that check before hitting the network.
//!
//! We only use two calls from the Last.fm API:
//!
//! * `artist.getsimilar` — seed artist → similar artists (name + MBID).
//! * `track.getsimilar` — seed track → similar tracks (title, artist,
//!   MBID). Album similarity has no official endpoint; the orchestrator
//!   in [`super::similar`] approximates it via `artist.getsimilar` +
//!   their top albums.

use reqwest::Client;
use serde::Deserialize;
use std::time::Duration;

const LASTFM_ROOT: &str = "https://ws.audioscrobbler.com/2.0/";

/// A similar-artist / similar-track suggestion coming back from Last.fm.
#[derive(Debug, Clone)]
pub struct SimilarArtist {
    pub name: String,
    pub mbid: Option<String>,
    /// Similarity score in the range `0.0..1.0`.
    pub score: f64,
}

#[derive(Debug, Clone)]
pub struct SimilarTrack {
    pub title: String,
    pub artist: String,
    pub artist_mbid: Option<String>,
    pub mbid: Option<String>,
    pub score: f64,
}

/// Handle to the Last.fm API. Cheap to clone — the underlying
/// `reqwest::Client` uses an `Arc` for connection reuse.
#[derive(Clone)]
pub struct LastFm {
    api_key: String,
    http: Client,
}

impl LastFm {
    /// Build a `LastFm` client when the config carries an API key. The
    /// `Option<String>` argument comes straight from
    /// `settings.lastfm_api_key`; passing `None` (or `Some("")`)
    /// returns `None` and every caller silently skips the network.
    pub fn from_key(api_key: Option<String>) -> Option<Self> {
        let key = api_key?;
        if key.trim().is_empty() {
            return None;
        }
        let http = Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .ok()?;
        Some(Self { api_key: key, http })
    }

    async fn call<T: for<'de> Deserialize<'de>>(
        &self,
        method: &str,
        extra: &[(&str, &str)],
    ) -> anyhow::Result<T> {
        let mut params: Vec<(&str, &str)> = vec![
            ("method", method),
            ("api_key", &self.api_key),
            ("format", "json"),
        ];
        params.extend_from_slice(extra);
        let resp = self.http.get(LASTFM_ROOT).query(&params).send().await?;
        let status = resp.status();
        let body = resp.bytes().await?;
        if !status.is_success() {
            return Err(anyhow::anyhow!(
                "lastfm {method} → {status}: {}",
                String::from_utf8_lossy(&body)
            ));
        }
        Ok(serde_json::from_slice::<T>(&body)?)
    }

    /// `artist.getsimilar` — seeds with either an MBID (preferred, more
    /// stable) or a plain artist name.
    pub async fn similar_artists(
        &self,
        name: Option<&str>,
        mbid: Option<&str>,
        limit: usize,
    ) -> anyhow::Result<Vec<SimilarArtist>> {
        let limit_str = limit.to_string();
        let mut extra: Vec<(&str, &str)> = vec![("limit", &limit_str), ("autocorrect", "1")];
        if let Some(m) = mbid {
            extra.push(("mbid", m));
        } else if let Some(n) = name {
            extra.push(("artist", n));
        } else {
            return Ok(Vec::new());
        }
        let body: SimilarArtistsBody = self.call("artist.getsimilar", &extra).await?;
        Ok(body
            .similarartists
            .artist
            .into_iter()
            .map(|a| SimilarArtist {
                name: a.name,
                mbid: a.mbid.filter(|s| !s.is_empty()),
                score: a.r#match.parse().unwrap_or(0.0),
            })
            .collect())
    }

    /// `track.getsimilar` — seeds with either (artist_name, track_name) or
    /// an MBID.
    pub async fn similar_tracks(
        &self,
        title: Option<&str>,
        artist: Option<&str>,
        mbid: Option<&str>,
        limit: usize,
    ) -> anyhow::Result<Vec<SimilarTrack>> {
        let limit_str = limit.to_string();
        let mut extra: Vec<(&str, &str)> = vec![("limit", &limit_str), ("autocorrect", "1")];
        if let Some(m) = mbid {
            extra.push(("mbid", m));
        } else if let (Some(t), Some(a)) = (title, artist) {
            extra.push(("track", t));
            extra.push(("artist", a));
        } else {
            return Ok(Vec::new());
        }
        let body: SimilarTracksBody = self.call("track.getsimilar", &extra).await?;
        Ok(body
            .similartracks
            .track
            .into_iter()
            .map(|t| {
                let artist_mbid = t
                    .artist
                    .as_ref()
                    .and_then(|a| a.mbid.clone())
                    .filter(|s| !s.is_empty());
                let artist_name = t
                    .artist
                    .as_ref()
                    .map(|a| a.name.clone())
                    .unwrap_or_default();
                SimilarTrack {
                    title: t.name,
                    artist: artist_name,
                    artist_mbid,
                    mbid: t.mbid.filter(|s| !s.is_empty()),
                    score: t.r#match.unwrap_or(0.0),
                }
            })
            .collect())
    }
}

// ── Response shapes ─────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct SimilarArtistsBody {
    #[serde(default)]
    similarartists: SimilarArtistsList,
}

#[derive(Deserialize, Default)]
struct SimilarArtistsList {
    #[serde(default)]
    artist: Vec<ArtistRow>,
}

#[derive(Deserialize)]
struct ArtistRow {
    name: String,
    #[serde(default)]
    mbid: Option<String>,
    /// Last.fm returns similarity as a string, not a float.
    r#match: String,
}

#[derive(Deserialize)]
struct SimilarTracksBody {
    #[serde(default)]
    similartracks: SimilarTracksList,
}

#[derive(Deserialize, Default)]
struct SimilarTracksList {
    #[serde(default)]
    track: Vec<TrackRow>,
}

#[derive(Deserialize)]
struct TrackRow {
    name: String,
    #[serde(default)]
    mbid: Option<String>,
    /// Track endpoint returns a float directly (unlike artist).
    #[serde(default)]
    r#match: Option<f64>,
    #[serde(default)]
    artist: Option<TrackArtist>,
}

#[derive(Deserialize)]
struct TrackArtist {
    name: String,
    #[serde(default)]
    mbid: Option<String>,
}
