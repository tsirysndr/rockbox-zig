use once_cell::sync::Lazy;
use std::collections::HashSet;
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tracing::{debug, info, warn};

// ─── Config ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Default)]
pub struct CacheConfig {
    /// Whether HTTP file caching is enabled (default: true).
    pub enabled: bool,
    /// Directory where cached files are stored.
    /// Default: ~/.config/rockbox.org/cache
    pub dir: PathBuf,
    /// Maximum total cache size in bytes (default: 512 MB).
    pub max_size_bytes: u64,
    /// Minimum free disk space that must remain before a new file is cached.
    /// Default: 100 MB
    pub min_free_space_bytes: u64,
    /// Number of parallel HTTP range-request parts used for large files.
    /// Set to 1 to disable parallel downloading.  Default: 4.
    pub parallel_parts: usize,
    /// URL substrings whose presence causes caching to be skipped entirely.
    /// Useful for live radio streams, HLS manifests, etc.
    /// Example entries: `"icecast"`, `".m3u8"`, `"live"`, `"stream"`.
    pub no_cache_patterns: Vec<String>,
}

impl CacheConfig {
    pub fn with_defaults() -> Self {
        let config_base = std::env::var("ROCKBOX_CONFIG_DIR").unwrap_or_else(|_| {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
            format!("{}/.config/rockbox.org", home)
        });
        let dir = PathBuf::from(format!("{}/cache", config_base));
        if let Err(e) = fs::create_dir_all(&dir) {
            eprintln!(
                "rockbox-cache: could not create default cache dir {:?}: {}",
                dir, e
            );
        }
        Self {
            enabled: true,
            dir,
            max_size_bytes: 512 * 1024 * 1024,
            min_free_space_bytes: 100 * 1024 * 1024,
            parallel_parts: 4,
            no_cache_patterns: Vec::new(),
        }
    }
}

// ─── Global state ────────────────────────────────────────────────────────────

struct CacheManager {
    config: CacheConfig,
    /// URLs whose background download is currently in-flight.
    in_progress: HashSet<String>,
}

static CACHE: Lazy<Mutex<CacheManager>> = Lazy::new(|| {
    Mutex::new(CacheManager {
        config: CacheConfig::with_defaults(),
        in_progress: HashSet::new(),
    })
});

static CLIENT: Lazy<reqwest::blocking::Client> = Lazy::new(|| {
    reqwest::blocking::Client::builder()
        .use_rustls_tls()
        .build()
        .expect("failed to build cache HTTP client")
});

// ─── Public API ──────────────────────────────────────────────────────────────

/// Replace the active cache configuration.  Called once from `load_settings`.
pub fn configure(config: CacheConfig) {
    if let Err(e) = fs::create_dir_all(&config.dir) {
        warn!(
            "http cache: could not create cache dir {:?}: {}",
            config.dir, e
        );
    } else {
        info!("http cache: dir {:?} ready", config.dir);
    }
    let mut mgr = CACHE.lock().unwrap();
    mgr.config = config;
    mgr.in_progress.clear();
}

/// Check whether `url` is already cached on disk.
///
/// On a hit: touches the file (updates mtime for LRU eviction ordering) and
/// returns the local path.  On a miss: returns `None`.
pub fn lookup(url: &str) -> Option<PathBuf> {
    let config = CACHE.lock().unwrap().config.clone();
    if !config.enabled {
        return None;
    }
    let key = url_to_key(url);
    let path = config.dir.join(format!("{key}.cache"));
    if path.exists() {
        touch_file(&path);
        debug!("cache: HIT  {}", url);
        Some(path)
    } else {
        None
    }
}

/// Kick off a parallel background download of `url` if it is not already
/// cached or in-flight.
///
/// Skips silently when:
/// - caching is disabled
/// - the URL matches a `no_cache_patterns` entry (e.g. live radio streams)
/// - the server reports no Content-Length (infinite/chunked streams)
/// - disk space is low
///
/// Returns immediately; the caller's live HTTP stream continues uninterrupted.
pub fn start_background_fetch(url: &str) {
    info!("cache: start_background_fetch called for {}", url);
    {
        let mut mgr = CACHE.lock().unwrap();
        if !mgr.config.enabled {
            info!("cache: caching disabled — skipping {}", url);
            return;
        }
        // Skip URLs that match a no-cache pattern (e.g. live streams, HLS).
        if mgr
            .config
            .no_cache_patterns
            .iter()
            .any(|p| url.contains(p.as_str()))
        {
            info!("cache: no-cache pattern match, skipping: {}", url);
            return;
        }
        if mgr.in_progress.contains(url) {
            info!("cache: already in-flight, skipping: {}", url);
            return;
        }
        let key = url_to_key(url);
        if mgr.config.dir.join(format!("{key}.cache")).exists() {
            info!("cache: already cached, skipping: {}", url);
            return;
        }
        let avail = available_disk_space(&mgr.config.dir).unwrap_or(u64::MAX);
        if avail < mgr.config.min_free_space_bytes {
            info!(
                "cache: low disk space ({} bytes free) — skipping background fetch for {}",
                avail, url
            );
            return;
        }
        mgr.in_progress.insert(url.to_string());
    }

    info!("cache: spawning download thread for {}", url);
    let url_owned = url.to_string();
    std::thread::Builder::new()
        .name("cache-fetch".into())
        .spawn(move || {
            let config = CACHE.lock().unwrap().config.clone();
            perform_download(&url_owned, &config);
            CACHE.lock().unwrap().in_progress.remove(&url_owned);
        })
        .ok();
}

// ─── Key derivation ──────────────────────────────────────────────────────────

/// Convert a URL into a fixed-length, filesystem-safe cache key (SHA-256 hex).
pub fn url_to_key(url: &str) -> String {
    use sha2::{Digest, Sha256};
    let hash = Sha256::digest(url.as_bytes());
    hash.iter().map(|b| format!("{:02x}", b)).collect()
}

// ─── Disk helpers ────────────────────────────────────────────────────────────

#[cfg(unix)]
fn available_disk_space(path: &Path) -> Option<u64> {
    use std::ffi::CString;
    // Walk up to the nearest existing ancestor so this works even when the
    // cache directory hasn't been created yet.
    let mut candidate = path.to_path_buf();
    loop {
        if candidate.exists() {
            break;
        }
        match candidate.parent().map(|p| p.to_path_buf()) {
            Some(p) => candidate = p,
            None => return None,
        }
    }
    let candidate = candidate.as_path();
    let p = CString::new(candidate.to_string_lossy().as_bytes()).ok()?;
    let mut stat: libc::statvfs = unsafe { std::mem::zeroed() };
    if unsafe { libc::statvfs(p.as_ptr(), &mut stat) } != 0 {
        return None;
    }
    Some(stat.f_bavail as u64 * stat.f_frsize as u64)
}

#[cfg(not(unix))]
fn available_disk_space(_path: &Path) -> Option<u64> {
    Some(u64::MAX)
}

fn current_cache_size(dir: &Path) -> u64 {
    let Ok(rd) = fs::read_dir(dir) else {
        return 0;
    };
    rd.flatten()
        .filter_map(|e| {
            let p = e.path();
            if p.extension().and_then(|x| x.to_str()) == Some("cache") {
                e.metadata().ok().map(|m| m.len())
            } else {
                None
            }
        })
        .sum()
}

fn evict_lru_until(dir: &Path, needed: u64) {
    let Ok(rd) = fs::read_dir(dir) else {
        return;
    };
    let mut files: Vec<(PathBuf, u64, std::time::SystemTime)> = rd
        .flatten()
        .filter_map(|e| {
            let p = e.path();
            if p.extension().and_then(|x| x.to_str()) != Some("cache") {
                return None;
            }
            let meta = e.metadata().ok()?;
            let mtime = meta.modified().ok()?;
            Some((p, meta.len(), mtime))
        })
        .collect();
    files.sort_by_key(|(_, _, mtime)| *mtime); // oldest first

    let mut freed = 0u64;
    for (path, size, _) in files {
        if freed >= needed {
            break;
        }
        if fs::remove_file(&path).is_ok() {
            freed += size;
            debug!("cache: evicted {}", path.display());
        }
    }
}

#[cfg(unix)]
fn touch_file(path: &Path) {
    use std::ffi::CString;
    if let Ok(p) = CString::new(path.to_string_lossy().as_bytes()) {
        unsafe { libc::utimes(p.as_ptr(), std::ptr::null()) };
    }
}

#[cfg(not(unix))]
fn touch_file(_path: &Path) {}

// ─── Parallel-write helper ───────────────────────────────────────────────────

/// Write `buf` starting at byte `offset` in `file` without advancing the file
/// cursor (uses `pwrite` on Unix, `seek_write` on Windows).  Loops until all
/// bytes are written so callers never see short writes.
#[cfg(unix)]
fn write_at_offset(file: &File, buf: &[u8], mut offset: u64) -> io::Result<()> {
    use std::os::unix::fs::FileExt;
    let mut pos = 0;
    while pos < buf.len() {
        let n = file.write_at(&buf[pos..], offset)?;
        if n == 0 {
            return Err(io::Error::new(
                io::ErrorKind::WriteZero,
                "write_at returned 0",
            ));
        }
        pos += n;
        offset += n as u64;
    }
    Ok(())
}

#[cfg(windows)]
fn write_at_offset(file: &File, buf: &[u8], mut offset: u64) -> io::Result<()> {
    use std::os::windows::fs::FileExt;
    let mut pos = 0;
    while pos < buf.len() {
        let n = file.seek_write(&buf[pos..], offset)?;
        if n == 0 {
            return Err(io::Error::new(
                io::ErrorKind::WriteZero,
                "seek_write returned 0",
            ));
        }
        pos += n;
        offset += n as u64;
    }
    Ok(())
}

#[cfg(not(any(unix, windows)))]
fn write_at_offset(_file: &File, _buf: &[u8], _offset: u64) -> io::Result<()> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "write_at_offset unavailable",
    ))
}

// ─── Download strategies ─────────────────────────────────────────────────────

/// Files at or above this size use parallel range-request downloading.
const MIN_PARALLEL_SIZE: u64 = 2 * 1024 * 1024; // 2 MB

/// Download `url` using `num_parts` simultaneous HTTP Range requests.
///
/// Each part writes directly to its allocated region in a pre-allocated temp
/// file via `pwrite` / `seek_write`, so threads never contend on a lock.
///
/// Returns `Ok(())` on success or `Err` if the server rejects range requests —
/// the caller falls back to `sequential_download` in that case.
fn parallel_download(url: &str, temp: &Path, total_size: u64, num_parts: usize) -> io::Result<()> {
    // Pre-allocate the complete file so every thread can write at any offset.
    {
        let f = File::create(temp)?;
        f.set_len(total_size)?;
    }

    let chunk = (total_size + num_parts as u64 - 1) / num_parts as u64; // ⌈total/parts⌉
    let mut handles: Vec<std::thread::JoinHandle<io::Result<()>>> = Vec::with_capacity(num_parts);

    for i in 0..num_parts {
        let start = i as u64 * chunk;
        if start >= total_size {
            break;
        }
        let end = (start + chunk - 1).min(total_size - 1);
        let url = url.to_string();
        let temp = temp.to_path_buf();

        let handle = std::thread::Builder::new()
            .name(format!("cache-part-{i}"))
            .spawn(move || -> io::Result<()> {
                let range = format!("bytes={start}-{end}");
                let mut resp = CLIENT
                    .get(&url)
                    .header("Range", range)
                    .send()
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

                if resp.status().as_u16() != 206 {
                    return Err(io::Error::new(
                        io::ErrorKind::Other,
                        format!("expected 206, got {}", resp.status()),
                    ));
                }

                let file = OpenOptions::new().write(true).open(&temp)?;
                let mut offset = start;
                let mut buf = [0u8; 65536];

                loop {
                    let n = resp
                        .read(&mut buf)
                        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
                    if n == 0 {
                        break;
                    }
                    write_at_offset(&file, &buf[..n], offset)?;
                    offset += n as u64;
                }
                Ok(())
            })
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        handles.push(handle);
    }

    // Wait for all threads; surface the first error.
    let mut first_err: Option<io::Error> = None;
    for handle in handles {
        match handle.join() {
            Ok(Ok(())) => {}
            Ok(Err(e)) if first_err.is_none() => first_err = Some(e),
            Err(_) if first_err.is_none() => {
                first_err = Some(io::Error::new(
                    io::ErrorKind::Other,
                    "download thread panicked",
                ))
            }
            _ => {}
        }
    }

    if let Some(e) = first_err {
        return Err(e);
    }
    Ok(())
}

/// Sequential (single-connection) download.  Used for small files and as a
/// fallback when the server does not support range requests.
fn sequential_download(url: &str, temp: &Path) -> io::Result<u64> {
    let mut resp = CLIENT
        .get(url)
        .send()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    if !resp.status().is_success() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("HTTP {}", resp.status()),
        ));
    }

    let file = File::create(temp)?;
    let mut writer = BufWriter::with_capacity(64 * 1024, file);
    let mut buf = [0u8; 65536];
    let mut total = 0u64;

    loop {
        let n = resp
            .read(&mut buf)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        if n == 0 {
            break;
        }
        writer.write_all(&buf[..n])?;
        total += n as u64;
    }
    writer.flush()?;
    Ok(total)
}

// ─── Main download orchestrator ──────────────────────────────────────────────

fn perform_download(url: &str, config: &CacheConfig) {
    let key = url_to_key(url);
    let temp_path = config.dir.join(format!("{key}.tmp"));
    let final_path = config.dir.join(format!("{key}.cache"));

    if final_path.exists() {
        return; // another thread finished first
    }

    if let Err(e) = fs::create_dir_all(&config.dir) {
        warn!("cache: cannot create cache dir: {}", e);
        return;
    }

    // HEAD request: get content-length and check range support.
    // If the server returns no Content-Length the resource is likely a live /
    // infinite stream — never attempt to cache it.
    let (content_length, server_accepts_ranges) = match CLIENT.head(url).send() {
        Ok(r) if r.status().is_success() => {
            let cl = r.content_length();
            let ok = r
                .headers()
                .get("accept-ranges")
                .and_then(|v| v.to_str().ok())
                .map(|v| v.trim() != "none")
                .unwrap_or(true); // assume yes when header is absent
            (cl, ok)
        }
        _ => (None, false),
    };

    let content_length = match content_length {
        Some(cl) => cl,
        None => {
            // No Content-Length → infinite/chunked stream; never cache.
            info!(
                "cache: no Content-Length for {} — skipping (live stream or transcoded?)",
                url
            );
            return;
        }
    };

    // Enforce max cache size, evicting the oldest files first if needed.
    let current = current_cache_size(&config.dir);
    if current + content_length > config.max_size_bytes {
        let excess = (current + content_length).saturating_sub(config.max_size_bytes);
        evict_lru_until(&config.dir, excess);
    }
    let avail = available_disk_space(&config.dir).unwrap_or(u64::MAX);
    if avail < config.min_free_space_bytes.saturating_add(content_length) {
        debug!(
            "cache: not enough free space for {} ({} bytes needed)",
            url, content_length
        );
        return;
    }

    // Strategy: parallel range-requests for large files, sequential otherwise.
    let num_parts = config.parallel_parts.max(1);
    let use_parallel =
        server_accepts_ranges && num_parts > 1 && content_length >= MIN_PARALLEL_SIZE;

    let downloaded: u64 = if use_parallel {
        debug!(
            "cache: parallel download ({} parts) {} ({} bytes)",
            num_parts, url, content_length
        );
        match parallel_download(url, &temp_path, content_length, num_parts) {
            Ok(()) => content_length,
            Err(e) => {
                debug!(
                    "cache: parallel failed for {} ({}), retrying sequential",
                    url, e
                );
                let _ = fs::remove_file(&temp_path);
                match sequential_download(url, &temp_path) {
                    Ok(n) => n,
                    Err(e) => {
                        debug!("cache: sequential also failed for {}: {}", url, e);
                        let _ = fs::remove_file(&temp_path);
                        return;
                    }
                }
            }
        }
    } else {
        match sequential_download(url, &temp_path) {
            Ok(n) => n,
            Err(e) => {
                debug!("cache: download failed for {}: {}", url, e);
                let _ = fs::remove_file(&temp_path);
                return;
            }
        }
    };

    // Verify the download is complete before promoting to the final path.
    if downloaded != content_length {
        debug!(
            "cache: incomplete download for {} ({}/{} bytes) — discarding",
            url, downloaded, content_length
        );
        let _ = fs::remove_file(&temp_path);
        return;
    }

    match fs::rename(&temp_path, &final_path) {
        Ok(()) => info!(
            "cache: stored {} ({} bytes, {} parts)",
            url,
            downloaded,
            if use_parallel { num_parts } else { 1 }
        ),
        Err(e) => {
            warn!("cache: rename failed for {}: {}", url, e);
            let _ = fs::remove_file(&temp_path);
        }
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn url_to_key_is_deterministic() {
        let k1 = url_to_key("https://example.com/song.mp3");
        let k2 = url_to_key("https://example.com/song.mp3");
        assert_eq!(k1, k2);
        assert_eq!(k1.len(), 64, "SHA-256 hex is always 64 chars");
    }

    #[test]
    fn url_to_key_differs_for_distinct_urls() {
        let k1 = url_to_key("https://example.com/a.mp3");
        let k2 = url_to_key("https://example.com/b.mp3");
        assert_ne!(k1, k2);
    }

    #[test]
    fn lookup_returns_none_for_uncached_url() {
        assert!(lookup("https://example.com/definitely-not-cached-xyz.mp3").is_none());
    }
}
