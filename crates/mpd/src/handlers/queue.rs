use std::fs;

use crate::{consts::PLAYLIST_INSERT_LAST, handlers::Subsystem, Context};
use anyhow::Error;
use regex::Regex;
use rockbox_rpc::api::rockbox::v1alpha1::{
    GetGlobalSettingsRequest, InsertDirectoryRequest, InsertTracksRequest, RemoveAllTracksRequest,
    RemoveTracksRequest, ShufflePlaylistRequest, StartRequest,
};
use tokio::sync::mpsc::Sender;

pub async fn handle_shuffle(
    ctx: &mut Context,
    _request: &str,
    tx: Sender<Vec<u8>>,
) -> Result<String, Error> {
    ctx.playlist
        .shuffle_playlist(ShufflePlaylistRequest { start_index: 0 })
        .await?;
    if !ctx.batch {
        tx.send(b"OK\n".to_vec()).await?;
    }

    match ctx.event_sender.send(Subsystem::Playlist) {
        Ok(_) => {}
        Err(_) => {}
    }

    Ok("OK\n".to_string())
}

pub async fn handle_add(
    ctx: &mut Context,
    request: &str,
    tx: Sender<Vec<u8>>,
) -> Result<String, Error> {
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
            tx.send(b"ACK [2@0] {add} missing argument\n".to_vec())
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
            tx.send(b"ACK [2@0] {add} missing argument\n".to_vec())
                .await?;
        }
        return Ok("ACK [2@0] {add} missing argument\n".to_string());
    }

    let path = match path.starts_with('/') {
        true => path,
        false => format!("{}/{}", music_dir, path),
    };

    if fs::metadata(&path).is_err() {
        if !ctx.batch {
            tx.send(b"ACK [50@0] {add} No such file or directory\n".to_vec())
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
        drop(current_track);
        ctx.playlist.start(StartRequest::default()).await?;
    }

    if !ctx.batch {
        tx.send(b"OK\n".to_vec()).await?;
    }

    match ctx.event_sender.send(Subsystem::Playlist) {
        Ok(_) => {}
        Err(_) => {}
    }

    Ok("OK\n".to_string())
}

pub async fn handle_addid(
    ctx: &mut Context,
    request: &str,
    tx: Sender<Vec<u8>>,
) -> Result<String, Error> {
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
            tx.send(b"ACK [2@0] {addid} missing argument\n".to_vec())
                .await?;
        }
        return Ok("ACK [2@0] {addid} missing argument\n".to_string());
    }
    let captures = captures.unwrap();

    let path = captures.get(2).unwrap().as_str().to_string();
    let position = captures
        .get(3)
        .map(|x| x.as_str().parse::<i32>().unwrap_or(PLAYLIST_INSERT_LAST))
        .unwrap_or(PLAYLIST_INSERT_LAST);

    if path.is_empty() {
        if !ctx.batch {
            tx.send(b"ACK [2@0] {addid} missing argument\n".to_vec())
                .await?;
        }
        return Ok("ACK [2@0] {addid} missing argument\n".to_string());
    }

    let path = match path.starts_with('/') {
        true => path,
        false => format!("{}/{}", music_dir, path),
    };

    if fs::metadata(&path).is_err() {
        if !ctx.batch {
            tx.send(b"ACK [50@0] {addid} No such file or directory\n".to_vec())
                .await?;
        }
        return Ok("ACK [50@0] {addid} No such file or directory\n".to_string());
    }

    if fs::metadata(&path)?.is_dir() {
        if !ctx.batch {
            tx.send(b"ACK [2@0] {addid} cannot add directory; use add instead\n".to_vec())
                .await?;
        }
        return Ok("ACK [2@0] {addid} cannot add directory; use add instead\n".to_string());
    }

    let current_len = {
        let current_playlist = ctx.current_playlist.lock().await;
        current_playlist
            .as_ref()
            .map(|p| p.tracks.len())
            .unwrap_or(0)
    };

    ctx.playlist
        .insert_tracks(InsertTracksRequest {
            tracks: vec![path.clone()],
            position,
            ..Default::default()
        })
        .await?;

    let current_track = ctx.current_track.lock().await;
    if current_track.is_none() {
        drop(current_track);
        ctx.playlist.start(StartRequest::default()).await?;
    }

    let new_id = if position == PLAYLIST_INSERT_LAST {
        (current_len + 1) as i32
    } else {
        position + 1
    };

    let response = format!("Id: {}\nOK\n", new_id);

    if !ctx.batch {
        tx.send(response.clone().into_bytes()).await?;
    }

    match ctx.event_sender.send(Subsystem::Playlist) {
        Ok(_) => {}
        Err(_) => {}
    }

    Ok(response)
}

pub async fn handle_playlistinfo(
    ctx: &mut Context,
    _request: &str,
    tx: Sender<Vec<u8>>,
) -> Result<String, Error> {
    let current_playlist = ctx.current_playlist.lock().await;

    if current_playlist.is_none() {
        if !ctx.batch {
            tx.send(b"OK\n".to_vec()).await?;
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
                "file: {}\nTitle: {}\nArtist: {}\nAlbum: {}\nTime: {}\nDuration: {}\nPos: {}\nDisc: {}\nDate: {}\nAlbumArtist: {}\nTrack: {}\nId: {}\n",
                x.path,
                x.title,
                x.artist,
                x.album,
                (x.length / 1000) as u32,
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
        tx.send(response.clone().into_bytes()).await?;
    }

    Ok(response)
}

pub async fn handle_playlistid(
    ctx: &mut Context,
    request: &str,
    tx: Sender<Vec<u8>>,
) -> Result<String, Error> {
    let arg = request.split_whitespace().nth(1);
    let id = arg.and_then(|x| x.trim_matches('"').parse::<usize>().ok());

    let current_playlist = ctx.current_playlist.lock().await;

    if current_playlist.is_none() {
        if !ctx.batch {
            tx.send(b"OK\n".to_vec()).await?;
        }
        return Ok("OK\n".to_string());
    }

    let current_playlist = current_playlist.as_ref().unwrap();

    let response = match id {
        Some(id) => {
            let idx = id.saturating_sub(1);
            if let Some(x) = current_playlist.tracks.get(idx) {
                format!(
                    "file: {}\nTitle: {}\nArtist: {}\nAlbum: {}\nTime: {}\nDuration: {}\nPos: {}\nDisc: {}\nDate: {}\nAlbumArtist: {}\nTrack: {}\nId: {}\n",
                    x.path, x.title, x.artist, x.album,
                    (x.length / 1000) as u32, (x.length / 1000) as u32,
                    idx, x.discnum, x.year_string, x.album_artist, x.tracknum, id
                )
            } else {
                return {
                    let msg = format!("ACK [50@0] {{playlistid}} No such song\n");
                    if !ctx.batch {
                        tx.send(msg.clone().into_bytes()).await?;
                    }
                    Ok(msg)
                };
            }
        }
        None => current_playlist
            .tracks
            .iter()
            .enumerate()
            .map(|(idx, x)| {
                format!(
                    "file: {}\nTitle: {}\nArtist: {}\nAlbum: {}\nTime: {}\nDuration: {}\nPos: {}\nDisc: {}\nDate: {}\nAlbumArtist: {}\nTrack: {}\nId: {}\n",
                    x.path, x.title, x.artist, x.album,
                    (x.length / 1000) as u32, (x.length / 1000) as u32,
                    idx, x.discnum, x.year_string, x.album_artist, x.tracknum, idx + 1
                )
            })
            .collect::<String>(),
    };

    let response = format!("{}OK\n", response);

    if !ctx.batch {
        tx.send(response.clone().into_bytes()).await?;
    }

    Ok(response)
}

pub async fn handle_deleteid(
    ctx: &mut Context,
    request: &str,
    tx: Sender<Vec<u8>>,
) -> Result<String, Error> {
    let arg = request.split_whitespace().last();
    if arg.is_none() {
        if !ctx.batch {
            tx.send(b"ACK [2@0] {deleteid} missing argument\n".to_vec())
                .await?;
        }
        return Ok("ACK [2@0] {deleteid} missing argument\n".to_string());
    }
    let arg = arg.unwrap();
    let arg = arg.trim();
    let arg = arg.trim_matches('"');
    let positions = match arg.parse::<i32>() {
        Ok(x) => vec![x - 1],
        Err(_) => {
            if !ctx.batch {
                tx.send(b"ACK [2@0] {deleteid} invalid argument\n".to_vec())
                    .await?;
            }
            return Ok("ACK [2@0] {deleteid} invalid argument\n".to_string());
        }
    };
    ctx.playlist
        .remove_tracks(RemoveTracksRequest { positions })
        .await?;
    if !ctx.batch {
        tx.send(b"OK\n".to_vec()).await?;
    }

    match ctx.event_sender.send(Subsystem::Playlist) {
        Ok(_) => {}
        Err(_) => {}
    }
    Ok("OK\n".to_string())
}

pub async fn handle_delete(
    ctx: &mut Context,
    request: &str,
    tx: Sender<Vec<u8>>,
) -> Result<String, Error> {
    let arg = request.split_whitespace().last();
    if arg.is_none() {
        if !ctx.batch {
            tx.send(b"ACK [2@0] {delete} missing argument\n".to_vec())
                .await?;
        }
        return Ok("ACK [2@0] {delete} missing argument\n".to_string());
    }
    let arg = arg.unwrap();
    let arg = arg.trim();
    let arg = arg.trim_matches('"');
    if arg.contains(':') {
        let range: Vec<i32> = arg.split(':').map(|x| x.parse::<i32>().unwrap()).collect();
        let positions: Vec<i32> = (range[0]..range[1]).collect();
        ctx.playlist
            .remove_tracks(RemoveTracksRequest { positions })
            .await?;
        if !ctx.batch {
            tx.send(b"OK\n".to_vec()).await?;
        }
        match ctx.event_sender.send(Subsystem::Playlist) {
            Ok(_) => {}
            Err(_) => {}
        }
        return Ok("OK\n".to_string());
    }
    let positions = match arg.parse::<i32>() {
        Ok(x) => vec![x],
        Err(_) => {
            if !ctx.batch {
                tx.send(b"ACK [2@0] {delete} invalid argument\n".to_vec())
                    .await?;
            }
            return Ok("ACK [2@0] {delete} invalid argument\n".to_string());
        }
    };
    ctx.playlist
        .remove_tracks(RemoveTracksRequest { positions })
        .await?;
    if !ctx.batch {
        tx.send(b"OK\n".to_vec()).await?;
    }

    match ctx.event_sender.send(Subsystem::Playlist) {
        Ok(_) => {}
        Err(_) => {}
    }

    Ok("OK\n".to_string())
}

pub async fn handle_clear(
    ctx: &mut Context,
    _request: &str,
    tx: Sender<Vec<u8>>,
) -> Result<String, Error> {
    ctx.playlist
        .remove_all_tracks(RemoveAllTracksRequest {})
        .await?;
    if !ctx.batch {
        tx.send(b"OK\n".to_vec()).await?;
    }

    match ctx.event_sender.send(Subsystem::Playlist) {
        Ok(_) => {}
        Err(_) => {}
    }

    Ok("OK\n".to_string())
}

pub async fn handle_move(
    ctx: &mut Context,
    request: &str,
    tx: Sender<Vec<u8>>,
) -> Result<String, Error> {
    let mut parts = request.split_whitespace().skip(1);
    let from_str = parts.next();
    let to_str = parts.next();

    if from_str.is_none() || to_str.is_none() {
        if !ctx.batch {
            tx.send(b"ACK [2@0] {move} incorrect arguments\n".to_vec())
                .await?;
        }
        return Ok("ACK [2@0] {move} incorrect arguments\n".to_string());
    }

    let from = from_str.unwrap().trim_matches('"').parse::<i32>();
    let to = to_str.unwrap().trim_matches('"').parse::<i32>();

    if from.is_err() || to.is_err() {
        if !ctx.batch {
            tx.send(b"ACK [2@0] {move} invalid argument\n".to_vec())
                .await?;
        }
        return Ok("ACK [2@0] {move} invalid argument\n".to_string());
    }

    let from = from.unwrap();
    let to = to.unwrap();

    let track_path = {
        let current_playlist = ctx.current_playlist.lock().await;
        current_playlist
            .as_ref()
            .and_then(|p| p.tracks.get(from as usize))
            .map(|t| t.path.clone())
    };

    if let Some(path) = track_path {
        ctx.playlist
            .remove_tracks(RemoveTracksRequest {
                positions: vec![from],
            })
            .await?;
        let insert_pos = if to > from { to - 1 } else { to };
        ctx.playlist
            .insert_tracks(InsertTracksRequest {
                tracks: vec![path],
                position: insert_pos,
                ..Default::default()
            })
            .await?;

        match ctx.event_sender.send(Subsystem::Playlist) {
            Ok(_) => {}
            Err(_) => {}
        }
    }

    if !ctx.batch {
        tx.send(b"OK\n".to_vec()).await?;
    }
    Ok("OK\n".to_string())
}

pub async fn handle_moveid(
    ctx: &mut Context,
    request: &str,
    tx: Sender<Vec<u8>>,
) -> Result<String, Error> {
    let mut parts = request.split_whitespace().skip(1);
    let id_str = parts.next();
    let to_str = parts.next();

    if id_str.is_none() || to_str.is_none() {
        if !ctx.batch {
            tx.send(b"ACK [2@0] {moveid} incorrect arguments\n".to_vec())
                .await?;
        }
        return Ok("ACK [2@0] {moveid} incorrect arguments\n".to_string());
    }

    let id = id_str.unwrap().trim_matches('"').parse::<i32>();
    let to = to_str.unwrap().trim_matches('"').parse::<i32>();

    if id.is_err() || to.is_err() {
        if !ctx.batch {
            tx.send(b"ACK [2@0] {moveid} invalid argument\n".to_vec())
                .await?;
        }
        return Ok("ACK [2@0] {moveid} invalid argument\n".to_string());
    }

    let from = id.unwrap() - 1;
    let to = to.unwrap();

    let track_path = {
        let current_playlist = ctx.current_playlist.lock().await;
        current_playlist
            .as_ref()
            .and_then(|p| p.tracks.get(from as usize))
            .map(|t| t.path.clone())
    };

    if let Some(path) = track_path {
        ctx.playlist
            .remove_tracks(RemoveTracksRequest {
                positions: vec![from],
            })
            .await?;
        let insert_pos = if to > from { to - 1 } else { to };
        ctx.playlist
            .insert_tracks(InsertTracksRequest {
                tracks: vec![path],
                position: insert_pos,
                ..Default::default()
            })
            .await?;

        match ctx.event_sender.send(Subsystem::Playlist) {
            Ok(_) => {}
            Err(_) => {}
        }
    }

    if !ctx.batch {
        tx.send(b"OK\n".to_vec()).await?;
    }
    Ok("OK\n".to_string())
}

pub async fn handle_swap(
    ctx: &mut Context,
    request: &str,
    tx: Sender<Vec<u8>>,
) -> Result<String, Error> {
    let mut parts = request.split_whitespace().skip(1);
    let pos1 = parts
        .next()
        .and_then(|x| x.trim_matches('"').parse::<i32>().ok());
    let pos2 = parts
        .next()
        .and_then(|x| x.trim_matches('"').parse::<i32>().ok());

    if pos1.is_none() || pos2.is_none() {
        if !ctx.batch {
            tx.send(b"ACK [2@0] {swap} incorrect arguments\n".to_vec())
                .await?;
        }
        return Ok("ACK [2@0] {swap} incorrect arguments\n".to_string());
    }

    let pos1 = pos1.unwrap();
    let pos2 = pos2.unwrap();

    let (path1, path2) = {
        let current_playlist = ctx.current_playlist.lock().await;
        let p1 = current_playlist
            .as_ref()
            .and_then(|p| p.tracks.get(pos1 as usize))
            .map(|t| t.path.clone());
        let p2 = current_playlist
            .as_ref()
            .and_then(|p| p.tracks.get(pos2 as usize))
            .map(|t| t.path.clone());
        (p1, p2)
    };

    if let (Some(path1), Some(path2)) = (path1, path2) {
        let (lo, hi, lo_path, hi_path) = if pos1 < pos2 {
            (pos1, pos2, path1, path2)
        } else {
            (pos2, pos1, path2, path1)
        };
        // Remove higher index first to avoid position shifts
        ctx.playlist
            .remove_tracks(RemoveTracksRequest {
                positions: vec![hi],
            })
            .await?;
        ctx.playlist
            .insert_tracks(InsertTracksRequest {
                tracks: vec![lo_path],
                position: hi,
                ..Default::default()
            })
            .await?;
        ctx.playlist
            .remove_tracks(RemoveTracksRequest {
                positions: vec![lo],
            })
            .await?;
        ctx.playlist
            .insert_tracks(InsertTracksRequest {
                tracks: vec![hi_path],
                position: lo,
                ..Default::default()
            })
            .await?;

        match ctx.event_sender.send(Subsystem::Playlist) {
            Ok(_) => {}
            Err(_) => {}
        }
    }

    if !ctx.batch {
        tx.send(b"OK\n".to_vec()).await?;
    }
    Ok("OK\n".to_string())
}

pub async fn handle_swapid(
    ctx: &mut Context,
    request: &str,
    tx: Sender<Vec<u8>>,
) -> Result<String, Error> {
    let mut parts = request.split_whitespace().skip(1);
    let id1 = parts
        .next()
        .and_then(|x| x.trim_matches('"').parse::<i32>().ok());
    let id2 = parts
        .next()
        .and_then(|x| x.trim_matches('"').parse::<i32>().ok());

    match (id1, id2) {
        (Some(id1), Some(id2)) => {
            let modified = format!("swap {} {}", id1 - 1, id2 - 1);
            handle_swap(ctx, &modified, tx).await
        }
        _ => {
            if !ctx.batch {
                tx.send(b"ACK [2@0] {swapid} incorrect arguments\n".to_vec())
                    .await?;
            }
            Ok("ACK [2@0] {swapid} incorrect arguments\n".to_string())
        }
    }
}
