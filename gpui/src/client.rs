use crate::api::v1alpha1::{
    library_service_client::LibraryServiceClient, playback_service_client::PlaybackServiceClient,
    playlist_service_client::PlaylistServiceClient, GetArtistsRequest, GetCurrentRequest,
    GetLikedTracksRequest, GetTracksRequest, InsertTracksRequest, LikeTrackRequest, NextRequest,
    PauseRequest, PlayAlbumRequest, PlayAllTracksRequest, PlayArtistTracksRequest, PlayTrackRequest,
    PreviousRequest, ResumeRequest, StartRequest, StreamCurrentTrackRequest, StreamPlaylistRequest,
    StreamStatusRequest, UnlikeTrackRequest,
};

// Matches apps/playlist.h PLAYLIST_INSERT_* constants
pub const INSERT_FIRST: i32 = -4; // play next (after current)
pub const INSERT_LAST: i32 = -3; // play last
pub const INSERT_SHUFFLED: i32 = -5; // add shuffled (next)
pub const INSERT_LAST_SHUFFLED: i32 = -7; // play last shuffled
use crate::state::{ArtistImages, PlaybackStatus, StateUpdate, Track};
use anyhow::Result;
use std::collections::HashSet;
use tokio::sync::mpsc::Sender;

const URL: &str = "http://127.0.0.1:6061";

// ── Library ───────────────────────────────────────────────────────────────────

pub async fn fetch_tracks() -> Result<Vec<Track>> {
    let mut c = LibraryServiceClient::connect(URL).await?;
    let resp = c.get_tracks(GetTracksRequest {}).await?;
    Ok(resp
        .into_inner()
        .tracks
        .into_iter()
        .map(track_from_proto)
        .collect())
}

fn track_from_proto(t: crate::api::v1alpha1::Track) -> Track {
    Track {
        id: t.id,
        path: t.path,
        title: t.title,
        artist: t.artist,
        album_artist: t.album_artist,
        album: t.album,
        album_id: t.album_id.unwrap_or_default(),
        artist_id: t.artist_id.unwrap_or_default(),
        genre: t.genre,
        duration: t.length as u64 / 1000,
        track_number: t.track_number,
        year: t.year,
        album_art: t.album_art.filter(|s| !s.is_empty()),
    }
}

// ── Playback control ──────────────────────────────────────────────────────────

pub async fn resume() -> Result<()> {
    let mut c = PlaybackServiceClient::connect(URL).await?;
    c.resume(ResumeRequest {}).await?;
    Ok(())
}

pub async fn pause() -> Result<()> {
    let mut c = PlaybackServiceClient::connect(URL).await?;
    c.pause(PauseRequest {}).await?;
    Ok(())
}

pub async fn next() -> Result<()> {
    let mut c = PlaybackServiceClient::connect(URL).await?;
    c.next(NextRequest {}).await?;
    Ok(())
}

pub async fn prev() -> Result<()> {
    let mut c = PlaybackServiceClient::connect(URL).await?;
    c.previous(PreviousRequest {}).await?;
    Ok(())
}

pub async fn play_track(path: String) -> Result<()> {
    let mut c = PlaybackServiceClient::connect(URL).await?;
    c.play_track(PlayTrackRequest { path }).await?;
    Ok(())
}

pub async fn play_album(album_id: String, shuffle: bool) -> Result<()> {
    let mut c = PlaybackServiceClient::connect(URL).await?;
    c.play_album(PlayAlbumRequest {
        album_id,
        shuffle: Some(shuffle),
        position: Some(0),
    })
    .await?;
    Ok(())
}

pub async fn play_artist_tracks(artist_id: String, shuffle: bool) -> Result<()> {
    let mut c = PlaybackServiceClient::connect(URL).await?;
    c.play_artist_tracks(PlayArtistTracksRequest {
        artist_id,
        shuffle: Some(shuffle),
        position: Some(0),
    })
    .await?;
    Ok(())
}

pub async fn play_all_tracks() -> Result<()> {
    let mut c = PlaybackServiceClient::connect(URL).await?;
    c.play_all_tracks(PlayAllTracksRequest {
        shuffle: Some(false),
        position: Some(0),
    })
    .await?;
    Ok(())
}

// ── Queue / Playlist ──────────────────────────────────────────────────────────

pub async fn jump_to_queue_position(pos: i32) -> Result<()> {
    let mut c = PlaylistServiceClient::connect(URL).await?;
    c.start(StartRequest {
        start_index: Some(pos),
        elapsed: Some(0),
        offset: Some(0),
    })
    .await?;
    Ok(())
}

pub async fn insert_track_next(path: String) -> Result<()> {
    let mut c = PlaylistServiceClient::connect(URL).await?;
    c.insert_tracks(InsertTracksRequest {
        playlist_id: None,
        position: INSERT_FIRST,
        tracks: vec![path],
        shuffle: Some(false),
    })
    .await?;
    Ok(())
}

pub async fn insert_track_last(path: String) -> Result<()> {
    let mut c = PlaylistServiceClient::connect(URL).await?;
    c.insert_tracks(InsertTracksRequest {
        playlist_id: None,
        position: INSERT_LAST,
        tracks: vec![path],
        shuffle: Some(false),
    })
    .await?;
    Ok(())
}

pub async fn insert_tracks(paths: Vec<String>, position: i32, shuffle: bool) -> Result<()> {
    let mut c = PlaylistServiceClient::connect(URL).await?;
    c.insert_tracks(InsertTracksRequest {
        playlist_id: None,
        position,
        tracks: paths,
        shuffle: Some(shuffle),
    })
    .await?;
    Ok(())
}

pub async fn fetch_queue(tx: Sender<StateUpdate>) {
    match PlaylistServiceClient::connect(URL).await {
        Ok(mut c) => match c.get_current(GetCurrentRequest {}).await {
            Ok(resp) => {
                let resp = resp.into_inner();
                let current_idx = if resp.index >= 0 {
                    Some(resp.index as usize)
                } else {
                    None
                };
                let queue: Vec<Track> = resp
                    .tracks
                    .into_iter()
                    .map(|t| Track {
                        id: t.id,
                        path: t.path,
                        title: t.title,
                        artist: t.artist,
                        album_artist: t.album_artist,
                        album: t.album,
                        album_id: t.album_id,
                        artist_id: t.artist_id,
                        genre: t.genre,
                        duration: t.length / 1000,
                        track_number: t.tracknum as u32,
                        year: t.year as u32,
                        album_art: t.album_art.filter(|s| !s.is_empty()),
                    })
                    .collect();
                let _ = tx.send(StateUpdate::Playlist { queue, current_idx }).await;
            }
            Err(e) => log::warn!("fetch_queue: {e}"),
        },
        Err(e) => log::warn!("fetch_queue connect: {e}"),
    }
}

pub async fn play_liked_tracks(paths: Vec<String>, shuffle: bool) -> Result<()> {
    let mut c = PlaylistServiceClient::connect(URL).await?;
    c.insert_tracks(InsertTracksRequest {
        playlist_id: None,
        position: 0,
        tracks: paths,
        shuffle: Some(shuffle),
    })
    .await?;
    c.start(StartRequest {
        start_index: Some(0),
        elapsed: Some(0),
        offset: Some(0),
    })
    .await?;
    Ok(())
}

// ── Likes ─────────────────────────────────────────────────────────────────────

pub async fn fetch_liked_tracks() -> Result<HashSet<String>> {
    let mut c = LibraryServiceClient::connect(URL).await?;
    let resp = c.get_liked_tracks(GetLikedTracksRequest {}).await?;
    Ok(resp.into_inner().tracks.into_iter().map(|t| t.id).collect())
}

pub async fn like_track(id: String) -> Result<()> {
    let mut c = LibraryServiceClient::connect(URL).await?;
    c.like_track(LikeTrackRequest { id }).await?;
    Ok(())
}

pub async fn unlike_track(id: String) -> Result<()> {
    let mut c = LibraryServiceClient::connect(URL).await?;
    c.unlike_track(UnlikeTrackRequest { id }).await?;
    Ok(())
}

// ── Streaming loops (tokio tasks — communicate via Sender<StateUpdate>) ───────

pub async fn run_library_sync(tx: Sender<StateUpdate>) {
    match fetch_tracks().await {
        Ok(mut tracks) => {
            tracks.sort_by(|a, b| a.title.to_lowercase().cmp(&b.title.to_lowercase()));
            let _ = tx.send(StateUpdate::Tracks(tracks)).await;
        }
        Err(e) => log::warn!("library sync: {e}"),
    }
}

pub async fn run_liked_tracks_sync(tx: Sender<StateUpdate>) {
    match fetch_liked_tracks().await {
        Ok(ids) => {
            let _ = tx.send(StateUpdate::LikedTracks(ids)).await;
        }
        Err(e) => log::warn!("liked tracks sync: {e}"),
    }
}

pub async fn run_artist_images_sync(tx: Sender<StateUpdate>) {
    match LibraryServiceClient::connect(URL).await {
        Ok(mut c) => match c.get_artists(GetArtistsRequest {}).await {
            Ok(resp) => {
                let images: ArtistImages = resp
                    .into_inner()
                    .artists
                    .into_iter()
                    .filter_map(|a| a.image.filter(|s| !s.is_empty()).map(|img| (a.name, img)))
                    .collect();
                let _ = tx.send(StateUpdate::ArtistImages(images)).await;
            }
            Err(e) => log::warn!("artist images sync: {e}"),
        },
        Err(e) => log::warn!("artist images connect: {e}"),
    }
}

pub async fn run_status_stream(tx: Sender<StateUpdate>) {
    loop {
        if let Err(e) = status_stream_inner(&tx).await {
            log::warn!("status stream: {e}");
        }
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    }
}

async fn status_stream_inner(tx: &Sender<StateUpdate>) -> Result<()> {
    let mut c = PlaybackServiceClient::connect(URL).await?;
    let resp = c.stream_status(StreamStatusRequest {}).await?;
    let mut stream = resp.into_inner();
    loop {
        match stream.message().await {
            Ok(Some(msg)) => {
                let new_status = match msg.status {
                    1 => PlaybackStatus::Playing,
                    2 => PlaybackStatus::Paused,
                    _ => PlaybackStatus::Stopped,
                };
                let _ = tx.send(StateUpdate::Status(new_status)).await;
            }
            Ok(None) => break,
            Err(e) => {
                log::warn!("status stream message: {e}");
                break;
            }
        }
    }
    Ok(())
}

pub async fn run_current_track_stream(tx: Sender<StateUpdate>) {
    loop {
        if let Err(e) = current_track_stream_inner(&tx).await {
            log::warn!("current track stream: {e}");
        }
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    }
}

async fn current_track_stream_inner(tx: &Sender<StateUpdate>) -> Result<()> {
    let mut c = PlaybackServiceClient::connect(URL).await?;
    let resp = c.stream_current_track(StreamCurrentTrackRequest {}).await?;
    let mut stream = resp.into_inner();
    loop {
        match stream.message().await {
            Ok(Some(msg)) => {
                let _ = tx.send(StateUpdate::Position(msg.elapsed / 1000)).await;
            }
            Ok(None) => break,
            Err(e) => {
                log::warn!("current track stream message: {e}");
                break;
            }
        }
    }
    Ok(())
}

pub async fn run_playlist_stream(tx: Sender<StateUpdate>) {
    loop {
        if let Err(e) = playlist_stream_inner(&tx).await {
            log::warn!("playlist stream: {e}");
        }
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    }
}

async fn playlist_stream_inner(tx: &Sender<StateUpdate>) -> Result<()> {
    let mut c = PlaybackServiceClient::connect(URL).await?;
    let resp = c.stream_playlist(StreamPlaylistRequest {}).await?;
    let mut stream = resp.into_inner();
    loop {
        match stream.message().await {
            Ok(Some(msg)) => {
                let current_idx = if msg.index >= 0 {
                    Some(msg.index as usize)
                } else {
                    None
                };
                let queue: Vec<Track> = msg
                    .tracks
                    .into_iter()
                    .map(|t| Track {
                        id: t.id,
                        path: t.path,
                        title: t.title,
                        artist: t.artist,
                        album_artist: t.album_artist,
                        album: t.album,
                        album_id: t.album_id,
                        artist_id: t.artist_id,
                        genre: t.genre,
                        duration: t.length / 1000,
                        track_number: t.tracknum as u32,
                        year: t.year as u32,
                        album_art: t.album_art.filter(|s| !s.is_empty()),
                    })
                    .collect();
                let _ = tx.send(StateUpdate::Playlist { queue, current_idx }).await;
            }
            Ok(None) => break,
            Err(e) => {
                log::warn!("playlist stream message: {e}");
                break;
            }
        }
    }
    Ok(())
}
