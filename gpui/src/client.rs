use crate::api::v1alpha1::{
    browse_service_client::BrowseServiceClient, library_service_client::LibraryServiceClient,
    playback_service_client::PlaybackServiceClient, playlist_service_client::PlaylistServiceClient,
    settings_service_client::SettingsServiceClient, sound_service_client::SoundServiceClient,
    system_service_client::SystemServiceClient, AdjustVolumeRequest, FastForwardRewindRequest,
    GetArtistsRequest, GetCurrentRequest, GetGlobalSettingsRequest, GetGlobalStatusRequest,
    GetLikedTracksRequest, GetTracksRequest, InsertDirectoryRequest, InsertTracksRequest,
    LikeTrackRequest, NextRequest, PauseRequest, PlayAlbumRequest, PlayAllTracksRequest,
    PlayArtistTracksRequest, PlayDirectoryRequest, PlayTrackRequest, PlaylistResumeRequest,
    PreviousRequest, RemoveTracksRequest, ResumeRequest, ResumeTrackRequest, SaveSettingsRequest,
    SearchRequest, ShufflePlaylistRequest, StartRequest, StatusRequest, StreamCurrentTrackRequest,
    StreamLibraryRequest, StreamPlaylistRequest, StreamStatusRequest, TreeGetEntriesRequest,
    UnlikeTrackRequest,
};
use crate::state::{SearchAlbum, SearchArtist, SearchResults};

// Matches apps/playlist.h PLAYLIST_INSERT_* constants
pub const INSERT_FIRST: i32 = -4; // play next (after current)
pub const INSERT_LAST: i32 = -3; // play last
pub const INSERT_SHUFFLED: i32 = -5; // add shuffled (next)
pub const INSERT_LAST_SHUFFLED: i32 = -7; // play last shuffled
use crate::state::{ArtistImages, PlaybackStatus, StateUpdate, Track};
use anyhow::Result;
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
        disc_number: t.disc_number,
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

// Resume from saved state after a daemon restart (playlist_resume + resume_track).
pub async fn resume_track() -> Result<()> {
    let mut c = PlaylistServiceClient::connect(URL).await?;
    c.resume_track(ResumeTrackRequest {
        start_index: 0,
        crc: 0,
        elapsed: 0,
        offset: 0,
    })
    .await?;
    Ok(())
}

pub async fn pause() -> Result<()> {
    let mut c = PlaybackServiceClient::connect(URL).await?;
    c.pause(PauseRequest {}).await?;
    Ok(())
}

/// Seek to `new_time_ms` milliseconds from the start of the current track.
pub async fn seek(new_time_ms: i32) -> Result<()> {
    let mut c = PlaybackServiceClient::connect(URL).await?;
    c.fast_forward_rewind(FastForwardRewindRequest {
        new_time: new_time_ms,
    })
    .await?;
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

pub async fn search(term: String) -> Result<SearchResults> {
    let mut c = LibraryServiceClient::connect(URL).await?;
    let resp = c.search(SearchRequest { term }).await?;
    let resp = resp.into_inner();
    let tracks = resp.tracks.into_iter().map(track_from_proto).collect();
    let albums = resp
        .albums
        .into_iter()
        .map(|a| SearchAlbum {
            id: a.id,
            title: a.title,
            artist: a.artist,
            year: a.year,
            album_art: a.album_art,
            artist_id: a.artist_id,
        })
        .collect();
    let artists = resp
        .artists
        .into_iter()
        .map(|a| SearchArtist {
            id: a.id,
            name: a.name,
            image: a.image,
        })
        .collect();
    Ok(SearchResults {
        tracks,
        albums,
        artists,
    })
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
                        disc_number: 0,
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
        shuffle: Some(false),
    })
    .await?;
    if shuffle {
        c.shuffle_playlist(ShufflePlaylistRequest { start_index: 0 })
            .await?;
    }
    c.start(StartRequest {
        start_index: Some(0),
        elapsed: Some(0),
        offset: Some(0),
    })
    .await?;
    Ok(())
}

// ── Queue mutation ────────────────────────────────────────────────────────────

pub async fn remove_from_queue(position: i32) -> Result<()> {
    let mut c = PlaylistServiceClient::connect(URL).await?;
    c.remove_tracks(RemoveTracksRequest {
        positions: vec![position],
    })
    .await?;
    Ok(())
}

// ── Sound / Volume ────────────────────────────────────────────────────────────

pub async fn adjust_volume(steps: i32) -> Result<()> {
    let mut c = SoundServiceClient::connect(URL).await?;
    c.adjust_volume(AdjustVolumeRequest { steps }).await?;
    Ok(())
}

// ── Settings (shuffle, repeat) ────────────────────────────────────────────────

pub async fn save_shuffle(enabled: bool) -> Result<()> {
    let mut c = SettingsServiceClient::connect(URL).await?;
    c.save_settings(SaveSettingsRequest {
        playlist_shuffle: Some(enabled),
        ..Default::default()
    })
    .await?;
    Ok(())
}

pub async fn save_repeat(repeat_mode: i32) -> Result<()> {
    let mut c = SettingsServiceClient::connect(URL).await?;
    c.save_settings(SaveSettingsRequest {
        repeat_mode: Some(repeat_mode),
        ..Default::default()
    })
    .await?;
    Ok(())
}

// Fetch the saved resume position on startup so the progress bar shows the
// right value before the user presses play.
pub async fn run_resume_info_sync(tx: Sender<StateUpdate>) {
    match SystemServiceClient::connect(URL).await {
        Ok(mut c) => match c.get_global_status(GetGlobalStatusRequest {}).await {
            Ok(resp) => {
                let s = resp.into_inner();
                if s.resume_elapsed > 0 {
                    let _ = tx
                        .send(StateUpdate::Position(s.resume_elapsed as u64 / 1000))
                        .await;
                }
            }
            Err(e) => log::warn!("resume info sync: {e}"),
        },
        Err(e) => log::warn!("resume info sync connect: {e}"),
    }

    // The status stream only fires on changes; fetch the initial status once so
    // the Now Playing widget shows correctly when the app opens with a paused track.
    match PlaybackServiceClient::connect(URL).await {
        Ok(mut c) => match c.status(StatusRequest {}).await {
            Ok(resp) => {
                let s = resp.into_inner();
                let init_status = if s.status & 0x02 != 0 {
                    PlaybackStatus::Paused
                } else if s.status & 0x01 != 0 {
                    PlaybackStatus::Playing
                } else {
                    PlaybackStatus::Stopped
                };
                let _ = tx.send(StateUpdate::Status(init_status)).await;
            }
            Err(e) => log::warn!("initial status fetch: {e}"),
        },
        Err(e) => log::warn!("initial status connect: {e}"),
    }
}

pub async fn run_settings_sync(tx: Sender<StateUpdate>) {
    match SettingsServiceClient::connect(URL).await {
        Ok(mut c) => match c.get_global_settings(GetGlobalSettingsRequest {}).await {
            Ok(resp) => {
                let s = resp.into_inner();
                let _ = tx
                    .send(StateUpdate::Settings {
                        volume: s.volume,
                        shuffling: s.playlist_shuffle,
                        repeat_mode: s.repeat_mode,
                    })
                    .await;
            }
            Err(e) => log::warn!("settings sync: {e}"),
        },
        Err(e) => log::warn!("settings sync connect: {e}"),
    }
}

// ── Likes ─────────────────────────────────────────────────────────────────────

pub async fn fetch_liked_tracks() -> Result<Vec<String>> {
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

pub async fn run_library_stream(tx: Sender<StateUpdate>) {
    loop {
        if let Err(e) = library_stream_inner(&tx).await {
            log::warn!("library stream: {e}");
        }
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    }
}

async fn library_stream_inner(tx: &Sender<StateUpdate>) -> Result<()> {
    let mut c = LibraryServiceClient::connect(URL).await?;
    let resp = c.stream_library(StreamLibraryRequest {}).await?;
    let mut stream = resp.into_inner();
    loop {
        match stream.message().await {
            Ok(Some(_)) => {
                if let Ok(mut tracks) = fetch_tracks().await {
                    tracks.sort_by(|a, b| a.title.to_lowercase().cmp(&b.title.to_lowercase()));
                    let _ = tx.send(StateUpdate::Tracks(tracks)).await;
                }
                if let Ok(ids) = fetch_liked_tracks().await {
                    let _ = tx.send(StateUpdate::LikedTracks(ids)).await;
                }
            }
            Ok(None) => break,
            Err(e) => {
                log::warn!("library stream message: {e}");
                break;
            }
        }
    }
    Ok(())
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
                // audio_status() is a bitmask: PLAY=0x01, PAUSE=0x02.
                // Paused-while-playing reports 0x03 — check PAUSE bit first.
                let new_status = if msg.status & 0x02 != 0 {
                    PlaybackStatus::Paused
                } else if msg.status & 0x01 != 0 {
                    PlaybackStatus::Playing
                } else {
                    PlaybackStatus::Stopped
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
                // Skip elapsed=0 from the server's stopped-state ticker — it
                // would overwrite the resume position loaded from global status.
                // Once playback starts, elapsed becomes non-zero immediately.
                if msg.elapsed > 0 {
                    let _ = tx.send(StateUpdate::Position(msg.elapsed / 1000)).await;
                }
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
    // On every (re)connect: restore the playlist from saved state (no-op when
    // already playing) then snapshot the current queue before waiting for stream
    // events.  StreamPlaylist uses SimpleBroker which only delivers *future*
    // publishes — without this fetch the queue would stay empty if we connect
    // after the broker's initial publish.
    {
        if let Ok(mut c) = PlaylistServiceClient::connect(URL).await {
            let _ = c.playlist_resume(PlaylistResumeRequest {}).await;
        }
    }
    fetch_queue(tx.clone()).await;

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
                        disc_number: 0,
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

// ── File browser ──────────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub struct FileEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
}

pub async fn tree_get_entries(path: Option<String>) -> Result<Vec<FileEntry>> {
    let mut c = BrowseServiceClient::connect(URL).await?;
    let resp = c.tree_get_entries(TreeGetEntriesRequest { path }).await?;
    let mut entries: Vec<FileEntry> = resp
        .into_inner()
        .entries
        .into_iter()
        .map(|e| {
            let is_dir = e.attr == 0x10;
            let name = std::path::Path::new(&e.name)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(&e.name)
                .to_string();
            FileEntry {
                name,
                path: e.name,
                is_dir,
            }
        })
        .collect();
    entries.sort_by(|a, b| match (a.is_dir, b.is_dir) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
    });
    Ok(entries)
}

pub async fn play_directory(path: String, shuffle: bool) -> Result<()> {
    let mut c = PlaybackServiceClient::connect(URL).await?;
    c.play_directory(PlayDirectoryRequest {
        path,
        shuffle: Some(shuffle),
        recurse: Some(true),
        position: None,
    })
    .await?;
    Ok(())
}

pub async fn play_directory_at(path: String, position: i32) -> Result<()> {
    let mut c = PlaybackServiceClient::connect(URL).await?;
    c.play_directory(PlayDirectoryRequest {
        path,
        shuffle: Some(false),
        recurse: Some(true),
        position: Some(position),
    })
    .await?;
    Ok(())
}

pub async fn insert_directory(path: String, position: i32) -> Result<()> {
    let mut c = PlaylistServiceClient::connect(URL).await?;
    c.insert_directory(InsertDirectoryRequest {
        directory: path,
        position,
        recurse: Some(true),
        shuffle: None,
        playlist_id: None,
    })
    .await?;
    Ok(())
}
