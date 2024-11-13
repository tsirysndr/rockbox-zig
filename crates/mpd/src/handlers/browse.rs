use std::fs;

use crate::{dir::read_dir, Context};
use anyhow::Error;
use regex::Regex;
use rockbox_library::repo;
use rockbox_settings::get_music_dir;
use tokio::{
    io::{AsyncWriteExt, BufReader},
    net::TcpStream,
};

pub async fn handle_lsinfo(
    ctx: &mut Context,
    request: &str,
    stream: &mut BufReader<TcpStream>,
) -> Result<String, Error> {
    repo::track::all(ctx.pool.clone()).await?;

    let request = request.trim();
    let re = Regex::new(r#"^([\w-]+)(?:\s+"([^"]*)")?$"#).unwrap();
    let music_dir = get_music_dir()?;
    let path = match re.captures(request).unwrap().get(2) {
        Some(x) => x.as_str(),
        None => &music_dir,
    };

    let path = match path.starts_with(&music_dir) {
        true => path,
        false => &format!("{}/{}", music_dir, path),
    };

    // verify if path is a file or directory or doesn't exist
    if fs::metadata(path).is_err() {
        if !ctx.batch {
            stream
                .write_all(b"ACK [50@0] {lsinfo} No such file or directory\n")
                .await?;
        }
        return Ok("ACK [50@0] {lsinfo} No such file or directory\n".to_string());
    }

    let mut response: String = "".to_string();

    if !fs::metadata(path)?.is_dir() {
        build_file_metadata(ctx.clone(), path, &mut response, true).await?;
    }

    if fs::metadata(path)?.is_dir() {
        let files = read_dir(ctx.clone(), path.to_string(), false, true)?;
        response.push_str(&files);
        response.push_str("OK\n");
    }

    if !ctx.batch {
        stream.write_all(response.as_bytes()).await?;
    }

    Ok(response)
}

pub async fn handle_listall(
    ctx: &mut Context,
    _request: &str,
    stream: &mut BufReader<TcpStream>,
) -> Result<String, Error> {
    let mut response: String = "".to_string();
    let music_dir = get_music_dir()?;

    if fs::metadata(&music_dir)?.is_dir() {
        let files = read_dir(ctx.clone(), music_dir, true, false)?;
        response.push_str(&files);
        response.push_str("OK\n");
    }

    if !ctx.batch {
        stream.write_all(response.as_bytes()).await?;
    }

    Ok(response)
}

pub async fn handle_listallinfo(
    ctx: &mut Context,
    _request: &str,
    stream: &mut BufReader<TcpStream>,
) -> Result<String, Error> {
    repo::track::all(ctx.pool.clone()).await?;
    let music_dir = get_music_dir()?;
    let path = music_dir.clone();

    // verify if path is a file or directory or doesn't exist
    if fs::metadata(&path).is_err() {
        if !ctx.batch {
            stream
                .write_all(b"ACK [50@0] {lsinfo} No such file or directory\n")
                .await?;
        }
        return Ok("ACK [50@0] {lsinfo} No such file or directory\n".to_string());
    }

    let mut response: String = "".to_string();

    let files = read_dir(ctx.clone(), path, true, true)?;
    response.push_str(&files);
    response.push_str("OK\n");

    if !ctx.batch {
        stream.write_all(response.as_bytes()).await?;
    }

    Ok(response)
}

pub async fn handle_listfiles(
    ctx: &mut Context,
    request: &str,
    stream: &mut BufReader<TcpStream>,
) -> Result<String, Error> {
    let request = request.trim();
    let re = Regex::new(r#"^([\w-]+)(?:\s+"([^"]*)")?$"#).unwrap();
    let music_dir = get_music_dir()?;
    let path = match re.captures(request).unwrap().get(2) {
        Some(x) => x.as_str(),
        None => &music_dir,
    };

    let path = match path.starts_with(&music_dir) {
        true => path,
        false => &format!("{}/{}", music_dir, path),
    };

    // verify if path is a file or directory or doesn't exist
    if fs::metadata(&path).is_err() {
        if !ctx.batch {
            stream
                .write_all(b"ACK [50@0] {lsinfo} No such file or directory\n")
                .await?;
        }
        return Ok("ACK [50@0] {lsinfo} No such file or directory\n".to_string());
    }

    let mut response: String = "".to_string();

    if !fs::metadata(path)?.is_dir() {
        build_file_metadata(ctx.clone(), path, &mut response, false).await?;
    }

    if fs::metadata(path)?.is_dir() {
        let files = read_dir(ctx.clone(), path.to_string(), false, false)?;
        response.push_str(&files);
        response.push_str("OK\n");
    }

    if !ctx.batch {
        stream.write_all(response.as_bytes()).await?;
    }

    Ok(response)
}

async fn build_file_metadata(
    ctx: Context,
    path: &str,
    response: &mut String,
    with_metadata: bool,
) -> Result<(), Error> {
    let kv = ctx.kv.lock().await;
    let track = kv.get(path);
    let music_dir = get_music_dir()?;
    let file = path.replace(&music_dir, "");
    let last_modified = fs::metadata(path)?.modified().unwrap();
    let last_modified = chrono::DateTime::from_timestamp(
        last_modified
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64,
        0,
    )
    .unwrap();
    let last_modified = last_modified.format("%Y-%m-%dT%H:%M:%SZ").to_string();
    response.push_str(&format!(
        "file: {}\nLast-Modified: {}\n",
        file, last_modified
    ));

    if !with_metadata {
        return Ok(());
    }

    if let Some(track) = track {
        response.push_str(&format!(
            "Title: {}\nArtist: {}\nAlbum: {}\nTime: {}\nDuration: {}\nAlbumArtist: {}\n",
            track.title,
            track.artist,
            track.album,
            (track.length / 1000) as u32,
            track.length / 1000,
            track.album_artist,
        ));
        if let Some(track_number) = track.track_number {
            response.push_str(&format!("Track: {}\n", track_number));
        }

        if let Some(year) = track.year {
            response.push_str(&format!("Date: {}\n", year));
        }
    }

    response.push_str("OK\n");
    Ok(())
}
