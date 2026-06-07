//! Segment fetcher with a small concurrent prefetch window.

use anyhow::{Context, Result};
use bytes::Bytes;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::manifest::SegmentRef;

/// Cache of recently-fetched segment payloads. Sized to comfortably hold
/// the prefetch window plus the segment currently being played out so a
/// pause-then-resume doesn't re-fetch.
const CACHE_CAP: usize = 12;

#[derive(Default)]
pub struct SegmentCache {
    inner: Mutex<SegmentCacheInner>,
}

#[derive(Default)]
struct SegmentCacheInner {
    map: HashMap<u64, Bytes>,
    // LRU-ish: insertion order. Oldest evicted when over CACHE_CAP.
    order: Vec<u64>,
}

impl SegmentCache {
    pub async fn put(&self, seq: u64, bytes: Bytes) {
        let mut g = self.inner.lock().await;
        if g.map.insert(seq, bytes).is_none() {
            g.order.push(seq);
        }
        while g.order.len() > CACHE_CAP {
            let old = g.order.remove(0);
            g.map.remove(&old);
        }
    }

    pub async fn get(&self, seq: u64) -> Option<Bytes> {
        self.inner.lock().await.map.get(&seq).cloned()
    }
}

pub async fn fetch_bytes(client: &reqwest::Client, url: &str) -> Result<Bytes> {
    let resp = client
        .get(url)
        .send()
        .await
        .with_context(|| format!("GET {url}"))?
        .error_for_status()
        .with_context(|| format!("HTTP error on {url}"))?;
    let bytes = resp
        .bytes()
        .await
        .with_context(|| format!("read body {url}"))?;
    Ok(bytes)
}

/// Fetch a window of segments concurrently and stuff them in `cache`. Caller
/// awaits this to make sure the next N segments are ready.
pub async fn prefetch(
    client: Arc<reqwest::Client>,
    cache: Arc<SegmentCache>,
    refs: Vec<SegmentRef>,
) {
    let mut handles = Vec::with_capacity(refs.len());
    for s in refs {
        let client = client.clone();
        let cache = cache.clone();
        handles.push(tokio::spawn(async move {
            if cache.get(s.seq).await.is_some() {
                return;
            }
            match fetch_bytes(&client, &s.url).await {
                Ok(b) => cache.put(s.seq, b).await,
                Err(e) => tracing::warn!("hls/fetcher: seg {} fetch failed: {e}", s.seq),
            }
        }));
    }
    for h in handles {
        let _ = h.await;
    }
}
