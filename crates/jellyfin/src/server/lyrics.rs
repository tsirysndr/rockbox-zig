//! Lyric sidecar discovery, parsing, and I/O.
//!
//! Reads a `.lrc` (synced) or `.txt` (plain) file sitting next to the
//! audio file — mirrors what most players (Finamp, Symfonium, iSyncr,
//! Rockbox itself) expect for offline lyric storage.
//!
//! LRC parsing follows the informal spec: header tags like `[ar:artist]`
//! populate `LyricMetadata`; timed lines like `[mm:ss.xx]lyric` become
//! `LyricLine`s with `Start` in 100-ns ticks (Jellyfin's unit).
//! Multiple timestamps per line are supported — each yields its own
//! `LyricLine` pointing at the same text.

use std::path::{Path, PathBuf};

use super::dto::{LyricDto, LyricLine, LyricMetadata};

/// 100-ns ticks per millisecond — Jellyfin's timing unit.
const TICKS_PER_MS: i64 = 10_000;

/// Locate a lyric sidecar next to `track_path`. Prefers `.lrc` (synced)
/// over `.txt` (plain) and matches case-insensitively so `Song.MP3`
/// and `song.lrc` still pair up.
pub fn find_sidecar(track_path: &Path) -> Option<PathBuf> {
    let parent = track_path.parent()?;
    let stem = track_path.file_stem()?.to_str()?;
    let dir = std::fs::read_dir(parent).ok()?;
    let mut txt_hit: Option<PathBuf> = None;
    for entry in dir.flatten() {
        let path = entry.path();
        let Some(fname) = path.file_stem().and_then(|s| s.to_str()) else {
            continue;
        };
        if !fname.eq_ignore_ascii_case(stem) {
            continue;
        }
        let Some(ext) = path.extension().and_then(|s| s.to_str()) else {
            continue;
        };
        match ext.to_ascii_lowercase().as_str() {
            "lrc" => return Some(path),
            "txt" if txt_hit.is_none() => txt_hit = Some(path),
            _ => {}
        }
    }
    txt_hit
}

/// Path we write to on upload. Always `.lrc` — even when the caller
/// only sent plain text, since `.lrc` is a strict superset.
pub fn sidecar_write_path(track_path: &Path) -> Option<PathBuf> {
    let parent = track_path.parent()?;
    let stem = track_path.file_stem()?.to_str()?;
    Some(parent.join(format!("{stem}.lrc")))
}

/// Parse the sidecar file at `path`. Returns `None` if the file is
/// unreadable or empty. Chooses parser by extension — plain-text `.txt`
/// files skip the LRC header logic.
pub fn parse_sidecar(path: &Path) -> Option<LyricDto> {
    let text = std::fs::read_to_string(path).ok()?;
    if text.trim().is_empty() {
        return None;
    }
    let is_lrc = path
        .extension()
        .and_then(|s| s.to_str())
        .map(|e| e.eq_ignore_ascii_case("lrc"))
        .unwrap_or(false);
    Some(if is_lrc {
        parse_lrc(&text)
    } else {
        parse_plain(&text)
    })
}

/// Parse LRC content. Handles header tags and per-line timestamps.
/// A line with N timestamps yields N `LyricLine`s all pointing at the
/// same text — matches how synced players expand karaoke-style lines.
pub fn parse_lrc(text: &str) -> LyricDto {
    let mut meta = LyricMetadata::default();
    let mut lines: Vec<LyricLine> = Vec::new();
    let mut has_synced = false;

    for raw_line in text.lines() {
        let line = raw_line.trim_end_matches('\r').trim();
        if line.is_empty() {
            continue;
        }
        // Extract every `[…]` prefix; the remainder is the lyric text.
        let mut rest = line;
        let mut tags: Vec<&str> = Vec::new();
        while let Some(inner) = rest.strip_prefix('[') {
            let Some(end) = inner.find(']') else {
                break;
            };
            tags.push(&inner[..end]);
            rest = &inner[end + 1..];
        }
        if tags.is_empty() {
            // No brackets → plain text line (rare in LRC, treat as unsynced).
            lines.push(LyricLine {
                text: rest.trim().to_string(),
                start: None,
            });
            continue;
        }

        let lyric_text = rest.trim().to_string();
        for tag in tags {
            if let Some(ticks) = parse_timestamp_tag(tag) {
                has_synced = true;
                lines.push(LyricLine {
                    text: lyric_text.clone(),
                    start: Some(ticks),
                });
            } else if let Some((key, value)) = parse_metadata_tag(tag) {
                apply_metadata(&mut meta, key, value);
            }
        }
    }

    // Apply LRC `offset:` field (in milliseconds; positive = later).
    if let Some(offset_ticks) = meta.offset {
        for line in &mut lines {
            if let Some(start) = line.start.as_mut() {
                *start = (*start + offset_ticks).max(0);
            }
        }
    }
    meta.is_synced = Some(has_synced);

    LyricDto {
        metadata: Some(meta),
        lyrics: lines,
    }
}

/// Parse plain-text lyrics — one line per `LyricLine`, all unsynced.
pub fn parse_plain(text: &str) -> LyricDto {
    let lines = text
        .lines()
        .map(|l| LyricLine {
            text: l.trim_end_matches('\r').trim_end().to_string(),
            start: None,
        })
        .collect();
    LyricDto {
        metadata: Some(LyricMetadata {
            is_synced: Some(false),
            ..Default::default()
        }),
        lyrics: lines,
    }
}

/// Try to parse `mm:ss.xx` / `mm:ss` / `mm:ss.xxx` inside a `[…]` tag.
/// Returns 100-ns ticks so we can plug straight into `LyricLine.Start`.
fn parse_timestamp_tag(tag: &str) -> Option<i64> {
    // First char must be a digit; header tags like `ar:foo` won't match.
    if !tag.chars().next()?.is_ascii_digit() {
        return None;
    }
    let (mm_str, rest) = tag.split_once(':')?;
    let minutes: i64 = mm_str.parse().ok()?;
    let (ss_str, frac_str) = match rest.split_once('.') {
        Some((s, f)) => (s, f),
        None => (rest, "0"),
    };
    let seconds: i64 = ss_str.parse().ok()?;
    // LRC fractions can be 2 or 3 digits — normalize to milliseconds.
    let mut frac: i64 = frac_str.parse().ok()?;
    match frac_str.len() {
        1 => frac *= 100,
        2 => frac *= 10,
        3 => {} // already ms
        _ => return None,
    }
    let total_ms = minutes * 60 * 1000 + seconds * 1000 + frac;
    Some(total_ms * TICKS_PER_MS)
}

/// Header-style tags: `ar:artist`, `ti:title`, `offset:200`, etc.
fn parse_metadata_tag(tag: &str) -> Option<(&str, &str)> {
    let (k, v) = tag.split_once(':')?;
    Some((k.trim(), v.trim()))
}

fn apply_metadata(meta: &mut LyricMetadata, key: &str, value: &str) {
    match key.to_ascii_lowercase().as_str() {
        "ar" => meta.artist = Some(value.to_string()),
        "al" => meta.album = Some(value.to_string()),
        "ti" => meta.title = Some(value.to_string()),
        "au" => meta.author = Some(value.to_string()),
        "by" => meta.by = Some(value.to_string()),
        "length" | "len" => {
            // LRC length is `mm:ss` — store in ticks.
            if let Some((m, s)) = value.split_once(':') {
                let mm: i64 = m.parse().unwrap_or(0);
                let ss: i64 = s.parse().unwrap_or(0);
                meta.length = Some((mm * 60 + ss) * 1000 * TICKS_PER_MS);
            }
        }
        "offset" => {
            let ms: i64 = value.parse().unwrap_or(0);
            meta.offset = Some(ms * TICKS_PER_MS);
        }
        "re" => meta.creator = Some(value.to_string()),
        "ve" => meta.version = Some(value.to_string()),
        _ => {}
    }
}

/// Write a lyric payload back to disk. If `content_type` looks like
/// JSON, serialize `LyricDto` as LRC; otherwise write the bytes
/// verbatim (client-supplied `.lrc` / `.txt` text).
pub fn write_sidecar(
    track_path: &Path,
    body: &[u8],
    content_type: &str,
) -> anyhow::Result<PathBuf> {
    let out = sidecar_write_path(track_path)
        .ok_or_else(|| anyhow::anyhow!("cannot derive sidecar path from {track_path:?}"))?;

    let lrc_bytes: Vec<u8> = if content_type.contains("json") {
        // Client sent a LyricDto — re-serialize as LRC so external
        // players can read it without knowing our JSON schema.
        let dto: LyricDto = serde_json::from_slice(body)?;
        serialize_lrc(&dto).into_bytes()
    } else {
        body.to_vec()
    };

    std::fs::write(&out, lrc_bytes)?;
    Ok(out)
}

/// Delete any sidecar `.lrc` / `.txt` next to `track_path`. Returns
/// `true` if at least one file was removed. Missing file = success (idempotent).
pub fn delete_sidecar(track_path: &Path) -> bool {
    let mut removed = false;
    if let Some(sc) = find_sidecar(track_path) {
        if std::fs::remove_file(&sc).is_ok() {
            removed = true;
        }
    }
    // Also try the canonical write path, since find_sidecar might have
    // returned the .txt while an obsolete .lrc still lingers.
    if let Some(canonical) = sidecar_write_path(track_path) {
        if canonical.exists() && std::fs::remove_file(&canonical).is_ok() {
            removed = true;
        }
    }
    removed
}

/// Serialize a `LyricDto` back to LRC text — enough round-trip for
/// external players. Metadata that we don't have LRC tags for (Author,
/// Creator, Version) is dropped; timings round to milliseconds.
fn serialize_lrc(dto: &LyricDto) -> String {
    let mut out = String::new();
    if let Some(m) = dto.metadata.as_ref() {
        if let Some(v) = m.artist.as_deref() {
            out.push_str(&format!("[ar:{v}]\n"));
        }
        if let Some(v) = m.album.as_deref() {
            out.push_str(&format!("[al:{v}]\n"));
        }
        if let Some(v) = m.title.as_deref() {
            out.push_str(&format!("[ti:{v}]\n"));
        }
        if let Some(v) = m.by.as_deref() {
            out.push_str(&format!("[by:{v}]\n"));
        }
        if let Some(ticks) = m.offset {
            out.push_str(&format!("[offset:{}]\n", ticks / TICKS_PER_MS));
        }
    }
    for line in &dto.lyrics {
        match line.start {
            Some(ticks) => {
                let ms = ticks / TICKS_PER_MS;
                let mm = ms / 60_000;
                let ss = (ms / 1_000) % 60;
                let hh = (ms % 1_000) / 10;
                out.push_str(&format!("[{mm:02}:{ss:02}.{hh:02}]{}\n", line.text));
            }
            None => {
                out.push_str(&line.text);
                out.push('\n');
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_synced_lrc_with_header() {
        let src = "[ar:The Artist]\n[ti:Song]\n[00:12.34]Hello world\n[01:00.00]Bye\n";
        let dto = parse_lrc(src);
        let meta = dto.metadata.unwrap();
        assert_eq!(meta.artist.as_deref(), Some("The Artist"));
        assert_eq!(meta.title.as_deref(), Some("Song"));
        assert_eq!(meta.is_synced, Some(true));
        assert_eq!(dto.lyrics.len(), 2);
        assert_eq!(dto.lyrics[0].text, "Hello world");
        assert_eq!(dto.lyrics[0].start, Some(12_340 * TICKS_PER_MS));
        assert_eq!(dto.lyrics[1].start, Some(60_000 * TICKS_PER_MS));
    }

    #[test]
    fn multiple_timestamps_yield_multiple_lines() {
        let src = "[00:10.00][00:20.00]Chorus\n";
        let dto = parse_lrc(src);
        assert_eq!(dto.lyrics.len(), 2);
        assert_eq!(dto.lyrics[0].start, Some(10_000 * TICKS_PER_MS));
        assert_eq!(dto.lyrics[1].start, Some(20_000 * TICKS_PER_MS));
        assert_eq!(dto.lyrics[0].text, "Chorus");
        assert_eq!(dto.lyrics[1].text, "Chorus");
    }

    #[test]
    fn parses_plain_text() {
        let src = "line one\nline two\n";
        let dto = parse_plain(src);
        assert_eq!(dto.lyrics.len(), 2);
        assert!(dto.lyrics.iter().all(|l| l.start.is_none()));
        assert_eq!(dto.metadata.as_ref().and_then(|m| m.is_synced), Some(false));
    }
}
