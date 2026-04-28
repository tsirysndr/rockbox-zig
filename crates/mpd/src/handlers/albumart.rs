use anyhow::Error;
use rockbox_library::{album_art, repo};
use rockbox_settings::get_music_dir;
use tokio::sync::mpsc::Sender;
use tracing::debug;

use crate::Context;

const CHUNK_SIZE: usize = 8192;

pub async fn handle_albumart(
    ctx: &mut Context,
    request: &str,
    tx: Sender<Vec<u8>>,
) -> Result<String, Error> {
    // TEMPORARILY DISABLED
    if !ctx.batch {
        tx.send(b"ACK [50@0] {albumart} No file exists\n".to_vec())
            .await?;
    }
    return Ok("ACK [50@0] {albumart} No file exists\n".to_string());

    #[allow(unreachable_code)]
    let (uri, offset) = parse_args(request);
    debug!("albumart command: uri={:?} offset={}", uri, offset);

    if uri.is_empty() {
        if !ctx.batch {
            tx.send(b"ACK [2@0] {albumart} missing argument\n".to_vec())
                .await?;
        }
        return Ok("ACK [2@0] {albumart} missing argument\n".to_string());
    }

    let full_path = resolve_path(&uri)?;
    let data = get_art_bytes(ctx, &full_path).await;

    match data {
        None => {
            if !ctx.batch {
                tx.send(b"ACK [50@0] {albumart} No file exists\n".to_vec())
                    .await?;
            }
            Ok("ACK [50@0] {albumart} No file exists\n".to_string())
        }
        Some(data) => send_binary_chunk("albumart", &data, offset, None, &tx, ctx.batch).await,
    }
}

pub async fn handle_readpicture(
    ctx: &mut Context,
    request: &str,
    tx: Sender<Vec<u8>>,
) -> Result<String, Error> {
    // TEMPORARILY DISABLED
    if !ctx.batch {
        tx.send(b"OK\n".to_vec()).await?;
    }
    return Ok("OK\n".to_string());

    #[allow(unreachable_code)]
    let (uri, offset) = parse_args(request);
    debug!("readpicture command: uri={:?} offset={}", uri, offset);

    if uri.is_empty() {
        if !ctx.batch {
            tx.send(b"ACK [2@0] {readpicture} missing argument\n".to_vec())
                .await?;
        }
        return Ok("ACK [2@0] {readpicture} missing argument\n".to_string());
    }

    let full_path = resolve_path(&uri)?;
    let art = get_art_bytes_with_type(ctx, &full_path).await;

    match art {
        None => {
            // readpicture returns empty OK when no art found
            if !ctx.batch {
                tx.send(b"OK\n".to_vec()).await?;
            }
            Ok("OK\n".to_string())
        }
        Some((data, mime_type)) => {
            send_binary_chunk(
                "readpicture",
                &data,
                offset,
                Some(mime_type),
                &tx,
                ctx.batch,
            )
            .await
        }
    }
}

fn parse_args(request: &str) -> (String, usize) {
    // Skip the command name, then parse a possibly-quoted path that may contain spaces.
    let rest = request
        .trim_start()
        .splitn(2, char::is_whitespace)
        .nth(1)
        .unwrap_or("")
        .trim_start();

    let (uri, remainder) = if rest.starts_with('"') {
        // Quoted path: find the closing quote
        if let Some(end) = rest[1..].find('"') {
            let path = &rest[1..end + 1];
            let rem = rest[end + 2..].trim_start();
            (path.to_string(), rem)
        } else {
            (rest.trim_matches('"').to_string(), "")
        }
    } else {
        // Unquoted: split on first whitespace
        let mut it = rest.splitn(2, char::is_whitespace);
        let p = it.next().unwrap_or("").to_string();
        let rem = it.next().unwrap_or("").trim_start();
        (p, rem)
    };

    let offset = remainder.trim_matches('"').parse::<usize>().unwrap_or(0);

    (uri, offset)
}

fn resolve_path(uri: &str) -> Result<String, Error> {
    if uri.starts_with('/') {
        Ok(uri.to_string())
    } else {
        let music_dir = get_music_dir()?;
        Ok(format!("{}/{}", music_dir, uri))
    }
}

fn covers_dir() -> Option<String> {
    let home = std::env::var("HOME").ok()?;
    Some(format!("{}/.config/rockbox.org/covers", home))
}

async fn get_art_bytes_with_type(
    ctx: &Context,
    track_path: &str,
) -> Option<(Vec<u8>, &'static str)> {
    debug!("albumart lookup for: {}", track_path);

    // 1. Try DB-cached cover file
    match repo::track::find_by_path(ctx.pool.clone(), track_path).await {
        Ok(Some(track)) => {
            debug!("albumart DB hit, album_art field = {:?}", track.album_art);
            if let Some(filename) = track.album_art {
                if let Some(dir) = covers_dir() {
                    let full_path = format!("{}/{}", dir, filename);
                    match tokio::fs::read(&full_path).await {
                        Ok(data) => {
                            debug!(
                                "albumart loaded from cache: {} ({} bytes)",
                                full_path,
                                data.len()
                            );
                            let mime = if filename.ends_with(".png") {
                                "image/png"
                            } else {
                                "image/jpeg"
                            };
                            return Some((data, mime));
                        }
                        Err(e) => debug!("albumart cache file unreadable {}: {}", full_path, e),
                    }
                }
            }
        }
        Ok(None) => debug!("albumart: track not found in DB for path: {}", track_path),
        Err(e) => debug!("albumart DB error: {}", e),
    }

    // 2. Read embedded art directly from the audio file tags
    let path = track_path.to_string();
    match tokio::task::spawn_blocking(move || album_art::read_album_art_bytes(&path)).await {
        Ok(Some((data, mime))) => {
            debug!(
                "albumart loaded from embedded tags ({} bytes, {})",
                data.len(),
                mime
            );
            return Some((data, mime));
        }
        Ok(None) => debug!("albumart: no embedded art in file tags"),
        Err(e) => debug!("albumart spawn_blocking error: {}", e),
    }

    // 3. Look for cover image files in the same directory
    match find_cover_file_with_type(track_path) {
        Some(art) => {
            debug!(
                "albumart loaded from directory cover file ({} bytes)",
                art.0.len()
            );
            Some(art)
        }
        None => {
            debug!("albumart: no art found for {}", track_path);
            None
        }
    }
}

async fn get_art_bytes(ctx: &Context, track_path: &str) -> Option<Vec<u8>> {
    get_art_bytes_with_type(ctx, track_path)
        .await
        .map(|(data, _)| data)
}

fn find_cover_file(track_path: &str) -> Option<Vec<u8>> {
    find_cover_file_with_type(track_path).map(|(data, _)| data)
}

fn find_cover_file_with_type(track_path: &str) -> Option<(Vec<u8>, &'static str)> {
    let dir = std::path::Path::new(track_path).parent()?;
    const CANDIDATES: &[(&str, &'static str)] = &[
        ("cover.jpg", "image/jpeg"),
        ("cover.jpeg", "image/jpeg"),
        ("cover.png", "image/png"),
        ("cover.webp", "image/webp"),
        ("folder.jpg", "image/jpeg"),
        ("folder.jpeg", "image/jpeg"),
        ("folder.png", "image/png"),
        ("album.jpg", "image/jpeg"),
        ("album.png", "image/png"),
        ("front.jpg", "image/jpeg"),
        ("front.jpeg", "image/jpeg"),
        ("front.png", "image/png"),
        ("artwork.jpg", "image/jpeg"),
        ("artwork.png", "image/png"),
        ("AlbumArt.jpg", "image/jpeg"),
        ("AlbumArt.jpeg", "image/jpeg"),
        ("AlbumArt.png", "image/png"),
    ];
    for (name, mime) in CANDIDATES {
        let p = dir.join(name);
        if let Ok(data) = std::fs::read(&p) {
            return Some((data, mime));
        }
    }
    None
}

async fn send_binary_chunk(
    _cmd: &str,
    data: &[u8],
    offset: usize,
    mime_type: Option<&str>,
    tx: &Sender<Vec<u8>>,
    batch: bool,
) -> Result<String, Error> {
    let total = data.len();
    let start = offset.min(total);
    let end = (start + CHUNK_SIZE).min(total);
    let chunk = &data[start..end];
    let chunk_len = chunk.len();

    let mut header = format!("size: {}\n", total);
    if let Some(mime) = mime_type {
        header.push_str(&format!("type: {}\n", mime));
    }
    header.push_str(&format!("binary: {}\n", chunk_len));

    let mut response = header.into_bytes();
    response.extend_from_slice(chunk);
    // In a command list each binary response ends with list_OK; standalone ends with OK.
    response.extend_from_slice(if batch { b"list_OK\n" } else { b"OK\n" });

    // Always send immediately — binary data cannot be accumulated in a String buffer.
    tx.send(response).await?;
    Ok("".to_string())
}
