//! MusicBrainz lookup helper for the Similar endpoints.
//!
//! Gated on `musicbrainz_user_agent` in `settings.toml` — MB requires a
//! descriptive User-Agent per their API policy ("if you don't send one
//! we may block you"). Absent config → [`MusicBrainz::from_ua`] returns
//! `None` and every caller skips the network.
//!
//! We use MB only to cross-reference MBIDs to canonical artist / release
//! names so name-only fuzzy matches against the local library are more
//! reliable when Last.fm returns aliases or transliterations.

use reqwest::Client;
use serde::Deserialize;
use std::time::Duration;

const MB_ROOT: &str = "https://musicbrainz.org/ws/2";

/// Handle to the MusicBrainz API. Cheap to clone.
#[derive(Clone)]
pub struct MusicBrainz {
    http: Client,
}

impl MusicBrainz {
    /// Build a `MusicBrainz` client when the config carries a
    /// User-Agent. Missing / empty UA → `None` and the plugin is
    /// disabled.
    pub fn from_ua(user_agent: Option<String>) -> Option<Self> {
        let ua = user_agent?;
        if ua.trim().is_empty() {
            return None;
        }
        let http = Client::builder()
            .user_agent(ua)
            .timeout(Duration::from_secs(5))
            .build()
            .ok()?;
        Some(Self { http })
    }

    /// `lookup /artist/{mbid}` — returns the canonical artist name and
    /// its sort name. `None` on any error (network, 404, malformed).
    pub async fn artist_by_mbid(&self, mbid: &str) -> Option<ArtistLookup> {
        if mbid.is_empty() {
            return None;
        }
        let url = format!("{MB_ROOT}/artist/{mbid}");
        let resp = self
            .http
            .get(&url)
            .query(&[("fmt", "json")])
            .send()
            .await
            .ok()?;
        if !resp.status().is_success() {
            return None;
        }
        let body: ArtistBody = resp.json().await.ok()?;
        Some(ArtistLookup {
            name: body.name,
            sort_name: body.sort_name,
        })
    }

    /// `search release-group?query=…` — resolve `(artist, album)` to a
    /// release-group MBID. The first match wins; MB's own scoring keeps
    /// this good enough for RemoteImage lookups where a mediocre match
    /// still surfaces reasonable candidates.
    pub async fn search_release_group(&self, artist: &str, album: &str) -> Option<String> {
        if artist.trim().is_empty() || album.trim().is_empty() {
            return None;
        }
        // Escape Lucene syntax the query language uses.
        let query = format!(
            "release:\"{}\" AND artist:\"{}\"",
            escape_lucene(album),
            escape_lucene(artist),
        );
        let url = format!("{MB_ROOT}/release-group");
        let resp = self
            .http
            .get(&url)
            .query(&[("query", query.as_str()), ("fmt", "json"), ("limit", "1")])
            .send()
            .await
            .ok()?;
        if !resp.status().is_success() {
            return None;
        }
        let body: ReleaseGroupSearchBody = resp.json().await.ok()?;
        body.release_groups.into_iter().next().map(|r| r.id)
    }

    /// `lookup /release-group/{mbid}` — canonical release / album name.
    /// Useful when Last.fm's track suggestion carries a release-group
    /// MBID we want to map to a local album.
    pub async fn release_group_by_mbid(&self, mbid: &str) -> Option<ReleaseLookup> {
        if mbid.is_empty() {
            return None;
        }
        let url = format!("{MB_ROOT}/release-group/{mbid}");
        let resp = self
            .http
            .get(&url)
            .query(&[("fmt", "json")])
            .send()
            .await
            .ok()?;
        if !resp.status().is_success() {
            return None;
        }
        let body: ReleaseBody = resp.json().await.ok()?;
        Some(ReleaseLookup { title: body.title })
    }
}

#[derive(Debug, Clone)]
pub struct ArtistLookup {
    pub name: String,
    pub sort_name: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ReleaseLookup {
    pub title: String,
}

// ── Response shapes ─────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct ArtistBody {
    name: String,
    #[serde(rename = "sort-name", default)]
    sort_name: Option<String>,
}

#[derive(Deserialize)]
struct ReleaseBody {
    title: String,
}

#[derive(Deserialize)]
struct ReleaseGroupSearchBody {
    #[serde(rename = "release-groups", default)]
    release_groups: Vec<ReleaseGroupRow>,
}

#[derive(Deserialize)]
struct ReleaseGroupRow {
    id: String,
}

/// Escape the Lucene reserved chars so a rogue quote / bracket in an
/// album name doesn't blow up the query.
fn escape_lucene(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for c in input.chars() {
        match c {
            '\\' | '+' | '-' | '&' | '|' | '!' | '(' | ')' | '{' | '}' | '[' | ']' | '^' | '"'
            | '~' | '*' | '?' | ':' | '/' => {
                out.push('\\');
                out.push(c);
            }
            _ => out.push(c),
        }
    }
    out
}
