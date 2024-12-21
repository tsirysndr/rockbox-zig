use std::fs;

use crate::{consts::PLAYLIST_INSERT_LAST, Context};
use anyhow::Error;
use regex::Regex;
use rockbox_rpc::api::rockbox::v1alpha1::{
    GetGlobalSettingsRequest, InsertDirectoryRequest, InsertTracksRequest, RemoveAllTracksRequest,
    RemoveTracksRequest, ShufflePlaylistRequest, StartRequest,
};
use tokio::{
    io::{AsyncWriteExt, BufReader},
    net::TcpStream,
};

pub async fn handle_shuffle(
    ctx: &mut Context,
    _request: &str,
    stream: &mut BufReader<TcpStream>,
) -> Result<String, Error> {
    let mut idle = ctx.idle.lock().await;
    *idle = true;

    ctx.playlist
        .shuffle_playlist(ShufflePlaylistRequest { start_index: 0 })
        .await?;
    if !ctx.batch {
        stream.write_all(b"OK\n").await?;
    }

    match ctx.event_sender.send("playlist".to_string()) {
        Ok(_) => {}
        Err(_) => {}
    }

    Ok("OK\n".to_string())
}

pub async fn handle_add(
    ctx: &mut Context,
    request: &str,
    stream: &mut BufReader<TcpStream>,
) -> Result<String, Error> {
    let mut idle = ctx.idle.lock().await;
    *idle = true;

    let response = ctx
        .settings
        .get_global_settings(GetGlobalSettingsRequest {})
        .await?;
    let response = response.into_inner();
    let music_dir = response.music_dir;

    let request = request.trim();
    let re = Regex::new(r#"^(\w+)\s+"([^"]+)"(?:\s+"?(-?\d+)"?)?$"#).unwrap();
    let captures = re.captures(request);
    if captures.is_none() {
        if !ctx.batch {
            stream
                .write_all(b"ACK [2@0] {add} missing argument\n")
                .await?;
        }
        return Ok("ACK [2@0] {add} missing argument\n".to_string());
    }
    let captures = captures.unwrap();

    let path = captures.get(2).unwrap().as_str().to_string();
    let position = captures
        .get(3)
        .map(|x| x.as_str().parse::<i32>().unwrap_or(PLAYLIST_INSERT_LAST))
        .unwrap_or(PLAYLIST_INSERT_LAST);

    if path.is_empty() {
        if !ctx.batch {
            stream
                .write_all(b"ACK [2@0] {add} missing argument\n")
                .await?;
        }
        return Ok("ACK [2@0] {add} missing argument\n".to_string());
    }

    let path = match path.starts_with('/') {
        true => path,
        false => format!("{}/{}", music_dir, path),
    };

    // verify if path is a file or directory or doesn't exist
    if fs::metadata(&path).is_err() {
        if !ctx.batch {
            stream
                .write_all(b"ACK [50@0] {add} No such file or directory\n")
                .await?;
        }
        return Ok("ACK [50@0] {add} No such file or directory\n".to_string());
    }

    if fs::metadata(&path)?.is_file() {
        ctx.playlist
            .insert_tracks(InsertTracksRequest {
                tracks: vec![path.clone()],
                position,
                ..Default::default()
            })
            .await?;
    }

    if fs::metadata(&path)?.is_dir() {
        ctx.playlist
            .insert_directory(InsertDirectoryRequest {
                directory: path,
                position,
                ..Default::default()
            })
            .await?;
    }

    let current_track = ctx.current_track.lock().await;

    if current_track.is_none() {
        ctx.playlist.start(StartRequest::default()).await?;
    }

    if !ctx.batch {
        stream.write_all(b"OK\n").await?;
    }

    match ctx.event_sender.send("playlist".to_string()) {
        Ok(_) => {}
        Err(_) => {}
    }

    Ok("OK\n".to_string())
}

pub async fn handle_addid(
    ctx: &mut Context,
    request: &str,
    stream: &mut BufReader<TcpStream>,
) -> Result<String, Error> {
    let mut idle = ctx.idle.lock().await;
    *idle = true;

    let response = ctx
        .settings
        .get_global_settings(GetGlobalSettingsRequest {})
        .await?;
    let response = response.into_inner();
    let music_dir = response.music_dir;

    let request = request.trim();
    let re = Regex::new(r#"^(\w+)\s+"([^"]+)"(?:\s+"?(-?\d+)"?)?$"#).unwrap();
    let captures = re.captures(request);

    println!("captures: {:?}", captures);

    if captures.is_none() {
        if !ctx.batch {
            stream
                .write_all(b"ACK [2@0] {add} missing argument\n")
                .await?;
        }
        return Ok("ACK [2@0] {add} missing argument\n".to_string());
    }
    let captures = captures.unwrap();

    let path = captures.get(2).unwrap().as_str().to_string();
    let position = captures
        .get(3)
        .map(|x| x.as_str().parse::<i32>().unwrap_or(PLAYLIST_INSERT_LAST))
        .unwrap_or(PLAYLIST_INSERT_LAST);

    if path.is_empty() {
        if !ctx.batch {
            stream
                .write_all(b"ACK [2@0] {add} missing argument\n")
                .await?;
        }
        return Ok("ACK [2@0] {add} missing argument\n".to_string());
    }

    let path = match path.starts_with('/') {
        true => path,
        false => format!("{}/{}", music_dir, path),
    };

    // verify if path is a file or directory or doesn't exist
    if fs::metadata(&path).is_err() {
        if !ctx.batch {
            stream
                .write_all(b"ACK [50@0] {add} No such file or directory\n")
                .await?;
        }
        return Ok("ACK [50@0] {add} No such file or directory\n".to_string());
    }

    if fs::metadata(&path)?.is_file() {
        ctx.playlist
            .insert_tracks(InsertTracksRequest {
                tracks: vec![path.clone()],
                position,
                ..Default::default()
            })
            .await?;
    }

    if fs::metadata(&path)?.is_dir() {
        // return error if directory, invalid for addid
        if !ctx.batch {
            stream
                .write_all(b"ACK [2@0] {addid} invalid argument\n")
                .await?;
        }
        return Ok("ACK [2@0] {addid} invalid argument\n".to_string());
    }

    let current_track = ctx.current_track.lock().await;
    if current_track.is_none() {
        ctx.playlist.start(StartRequest::default()).await?;
    }

    if !ctx.batch {
        stream.write_all(b"OK\n").await?;
    }

    match ctx.event_sender.send("playlist".to_string()) {
        Ok(_) => {}
        Err(_) => {}
    }

    Ok("OK\n".to_string())
}

pub async fn handle_playlistinfo(
    ctx: &mut Context,
    _request: &str,
    stream: &mut BufReader<TcpStream>,
) -> Result<String, Error> {
    let current_playlist = ctx.current_playlist.lock().await;

    if current_playlist.is_none() {
        if !ctx.batch {
            stream.write_all(b"OK\n").await?;
        }
        return Ok("OK\n".to_string());
    }

    let mut index = -1;
    let current_playlist = current_playlist.as_ref().unwrap();
    let response = current_playlist
        .tracks
        .iter()
        .map(|x| {
            index += 1;
            format!(
                "file: {}\nTitle: {}\nArtist: {}\nAlbum: {}\nTime: {}\nPos: {}\nDisc: {}\nDate: {}\nAlbumArtist: {}\nTrack: {}\nId: {}\n",
                x.path,
                x.title,
                x.artist,
                x.album,
                (x.length / 1000) as u32,
                index,
                x.discnum,
                x.year_string,
                x.album_artist,
                x.tracknum,
                index + 1
            )
        })
        .collect::<String>();
    let response = format!("{}OK\n", response);

    if !ctx.batch {
        stream.write_all(response.as_bytes()).await?;
    }

    Ok(response)
}

pub async fn handle_delete(
    ctx: &mut Context,
    request: &str,
    stream: &mut BufReader<TcpStream>,
) -> Result<String, Error> {
    let mut idle = ctx.idle.lock().await;
    *idle = true;

    let arg = request.split_whitespace().last();
    if arg.is_none() {
        if !ctx.batch {
            stream
                .write_all(b"ACK [2@0] {delete} missing argument\n")
                .await?;
        }
        return Ok("ACK [2@0] {delete} missing argument\n".to_string());
    }
    let arg = arg.unwrap();
    let arg = arg.trim();
    let arg = arg.trim_matches('"');
    if arg.contains(':') {
        // get the range
        let range: Vec<i32> = arg.split(':').map(|x| x.parse::<i32>().unwrap()).collect();
        let positions: Vec<i32> = (range[0]..=range[1]).collect();
        ctx.playlist
            .remove_tracks(RemoveTracksRequest { positions })
            .await?;
        if !ctx.batch {
            stream.write_all(b"OK\n").await?;
        }
        return Ok("OK\n".to_string());
    }
    let positions = match arg.parse::<i32>() {
        Ok(x) => vec![x],
        Err(_) => {
            if !ctx.batch {
                stream
                    .write_all(b"ACK [2@0] {delete} invalid argument\n")
                    .await?;
            }
            return Ok("ACK [2@0] {delete} invalid argument\n".to_string());
        }
    };
    ctx.playlist
        .remove_tracks(RemoveTracksRequest { positions })
        .await?;
    if !ctx.batch {
        stream.write_all(b"OK\n").await?;
    }

    match ctx.event_sender.send("playlist".to_string()) {
        Ok(_) => {}
        Err(_) => {}
    }

    Ok("OK\n".to_string())
}

pub async fn handle_clear(
    ctx: &mut Context,
    _request: &str,
    stream: &mut BufReader<TcpStream>,
) -> Result<String, Error> {
    let mut idle = ctx.idle.lock().await;
    *idle = true;

    ctx.playlist
        .remove_all_tracks(RemoveAllTracksRequest {})
        .await?;
    if !ctx.batch {
        stream.write_all(b"OK\n").await?;
    }

    match ctx.event_sender.send("playlist".to_string()) {
        Ok(_) => {}
        Err(_) => {}
    }

    Ok("OK\n".to_string())
}

pub async fn handle_move(
    ctx: &mut Context,
    request: &str,
    stream: &mut BufReader<TcpStream>,
) -> Result<String, Error> {
    println!("{}", request);
    if !ctx.batch {
        stream.write_all(b"OK\n").await?;
    }
    Ok("OK\n".to_string())
}
