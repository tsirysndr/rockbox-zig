//! Cover Art Archive client for the RemoteImage endpoints.
//!
//! CAA is a MusicBrainz sister-service that ships release / release-group
//! cover art with permissive licensing. We gate it on the same
//! `musicbrainz_user_agent` because CAA's terms require the standard MB
//! User-Agent header. If MB isn't configured, this plugin is disabled
//! and RemoteImages returns an empty set.

use reqwest::Client;
use serde::Deserialize;
use std::time::Duration;

pub const PROVIDER_NAME: &str = "CoverArtArchive";
const CAA_ROOT: &str = "https://coverartarchive.org";

#[derive(Debug, Clone)]
pub struct CaaImage {
    pub url: String,
    pub thumbnail_url: Option<String>,
    pub image_types: Vec<String>,
    pub is_front: bool,
}

#[derive(Clone)]
pub struct CoverArtArchive {
    http: Client,
}

impl CoverArtArchive {
    /// Same gating as [`super::musicbrainz::MusicBrainz`] — a UA in the
    /// settings turns both plugins on together.
    pub fn from_ua(user_agent: Option<String>) -> Option<Self> {
        let ua = user_agent?;
        if ua.trim().is_empty() {
            return None;
        }
        let http = Client::builder()
            .user_agent(ua)
            .timeout(Duration::from_secs(6))
            .build()
            .ok()?;
        Some(Self { http })
    }

    /// `GET /release-group/{mbid}` — returns every candidate cover art
    /// image. CAA follows redirects on 307 → the front image; on 404
    /// we return an empty vec so the caller can degrade gracefully.
    pub async fn release_group_images(&self, mbid: &str) -> Vec<CaaImage> {
        if mbid.is_empty() {
            return Vec::new();
        }
        let url = format!("{CAA_ROOT}/release-group/{mbid}");
        let Ok(resp) = self.http.get(&url).send().await else {
            return Vec::new();
        };
        if !resp.status().is_success() {
            return Vec::new();
        }
        let Ok(body): Result<CaaBody, _> = resp.json().await else {
            return Vec::new();
        };
        body.images
            .into_iter()
            .filter(|img| !img.image.is_empty())
            .map(|img| {
                let thumb = img
                    .thumbnails
                    .large
                    .clone()
                    .or_else(|| img.thumbnails.n500.clone())
                    .or_else(|| img.thumbnails.small.clone())
                    .or_else(|| img.thumbnails.n250.clone());
                CaaImage {
                    url: img.image,
                    thumbnail_url: thumb,
                    image_types: img.types,
                    is_front: img.front,
                }
            })
            .collect()
    }
}

// ── Response shapes ─────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct CaaBody {
    #[serde(default)]
    images: Vec<CaaImageBody>,
}

#[derive(Deserialize)]
struct CaaImageBody {
    image: String,
    #[serde(default)]
    thumbnails: CaaThumbnails,
    #[serde(default)]
    types: Vec<String>,
    #[serde(default)]
    front: bool,
}

/// CAA returns thumbnails under multiple keys — the historical
/// small/large plus size-suffixed 250/500/1200. Handle both.
#[derive(Deserialize, Default)]
struct CaaThumbnails {
    #[serde(default)]
    small: Option<String>,
    #[serde(default)]
    large: Option<String>,
    #[serde(default, rename = "250")]
    n250: Option<String>,
    #[serde(default, rename = "500")]
    n500: Option<String>,
}
