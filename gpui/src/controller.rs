use crate::now_playing::{MediaCommand, NowPlayingManager};
use crate::state::{AppState, PlaybackStatus, StateUpdate};
use crate::ui::components::{
    LikedOrder, LikedSongs, NavidromeServerState, NdCoverFetchState, NdCurrentCoverArt,
    NdScrobbleState,
};
use gpui::{App, Entity, Global};
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc, Mutex,
};
use std::time::Duration;
use tokio::sync::mpsc;


pub struct Controller {
    pub state: Entity<AppState>,
    rt: tokio::runtime::Runtime,
    tx: mpsc::Sender<StateUpdate>,
    search_gen: Arc<AtomicU64>,
    #[allow(dead_code)]
    now_playing: Option<Arc<Mutex<NowPlayingManager>>>,
}

impl Controller {
    pub fn new(state: Entity<AppState>, cx: &mut App) -> Self {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("tokio runtime");

        let (tx, rx) = mpsc::channel::<StateUpdate>(256);

        // Spawn tokio background tasks — these are Send because Sender<StateUpdate> is Send
        rt.spawn(crate::client::run_library_sync(tx.clone()));
        rt.spawn(crate::client::run_library_stream(tx.clone()));
        rt.spawn(crate::client::run_liked_tracks_sync(tx.clone()));
        rt.spawn(crate::client::run_artist_images_sync(tx.clone()));
        rt.spawn(crate::client::run_settings_sync(tx.clone()));
        rt.spawn(crate::client::run_resume_info_sync(tx.clone()));
        rt.spawn(crate::client::run_status_stream(tx.clone()));
        rt.spawn(crate::client::run_current_track_stream(tx.clone()));
        rt.spawn(crate::client::run_playlist_stream(tx.clone()));

        // Re-run one-shot syncs whenever the user switches the active server.
        let tx_for_switch = tx.clone();
        let notify_for_switch = crate::server::server_notify();
        rt.spawn(async move {
            loop {
                notify_for_switch.notified().await;
                // Small delay to let the new server's gRPC port come up.
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                crate::client::run_library_sync(tx_for_switch.clone()).await;
                crate::client::run_liked_tracks_sync(tx_for_switch.clone()).await;
                crate::client::run_artist_images_sync(tx_for_switch.clone()).await;
                crate::client::run_settings_sync(tx_for_switch.clone()).await;
                crate::client::run_resume_info_sync(tx_for_switch.clone()).await;
            }
        });

        // Initialise OS media controls on the main thread (required by macOS).
        let now_playing = NowPlayingManager::new().map(|m| Arc::new(Mutex::new(m)));

        // GPUI foreground poll task — not required to be Send
        let state_for_poll = state.clone();
        let np_for_poll = now_playing.clone();
        let rt_handle = rt.handle().clone();
        cx.spawn(async move |cx| {
            let mut rx = rx;
            loop {
                let mut did_update = false;
                while let Ok(update) = rx.try_recv() {
                    let _ = cx.update(|app| {
                        state_for_poll.update(app, |s, cx| match update {
                            StateUpdate::Status(status) => s.apply_status_from_stream(status),
                            StateUpdate::Position(pos) => s.position = pos,
                            StateUpdate::Playlist { queue, current_idx } => {
                                s.queue = queue;
                                s.current_idx = current_idx;
                            }
                            StateUpdate::Tracks(tracks) => s.tracks = tracks,
                            StateUpdate::ArtistImages(images) => s.artist_images = images,
                            StateUpdate::LikedTracks(ids) => {
                                let set = ids.iter().cloned().collect();
                                cx.set_global(LikedSongs(set));
                                cx.set_global(LikedOrder(ids));
                            }
                            StateUpdate::SearchResults(results) => {
                                s.search_results = results;
                            }
                            StateUpdate::Settings {
                                volume,
                                shuffling,
                                repeat_mode,
                            } => {
                                s.volume = volume;
                                s.shuffling = shuffling;
                                s.repeat = repeat_mode != 0;
                            }
                        });
                    });
                    did_update = true;
                }
                if did_update {
                    let _ = cx.update(|app| {
                        state_for_poll.update(app, |_, cx| cx.notify());
                    });
                }

                // Navidrome cover art: derive the getCoverArt URL directly from the
                // stream URL's embedded credentials (u=, t=, s=, id=).  This works
                // even when the user is not actively connected to a Navidrome server —
                // the stream URL is self-contained.  No getSong round-trip needed.
                let cover_url = cx.update(|app| {
                    let s = state_for_poll.read(app);
                    let track = s.current_track()?;
                    let path = &track.path;
                    if !path.starts_with("http") { return None; }

                    // Already have cover art for this track.
                    if app.global::<NdCurrentCoverArt>().0.is_some() { return None; }

                    let query = path.split('?').nth(1)?;
                    let param = |name: &str| -> Option<String> {
                        let prefix = format!("{name}=");
                        query.split('&')
                            .find(|p| p.starts_with(prefix.as_str()))
                            .map(|p| p[prefix.len()..].to_string())
                    };

                    let song_id = param("id")?;

                    // Already derived for this song.
                    if app.global::<NdCoverFetchState>().fetched_id.as_deref() == Some(&song_id) {
                        return None;
                    }

                    // Extract base_url as everything before /rest/.
                    let base_url = path.find("/rest/").map(|i| path[..i].to_string())?;
                    let user  = param("u")?;
                    let token = param("t")?;
                    let salt  = param("s")?;

                    let url = crate::navidrome::cover_art_url(&base_url, &user, &token, &salt, &song_id, Some(300));
                    Some((song_id, url))
                }).ok().flatten();

                if let Some((song_id, url)) = cover_url {
                    let _ = cx.update(|app| {
                        app.global_mut::<NdCoverFetchState>().fetched_id = Some(song_id);
                        app.global_mut::<NdCurrentCoverArt>().0 = Some(url);
                    });
                }

                // Navidrome scrobble: submit when position > 30s AND > 50% of duration.
                let scrobble_task = cx.update(|app| {
                    let s = state_for_poll.read(app);
                    let track = s.current_track()?;
                    let path = &track.path;
                    if !path.starts_with("http") {
                        return None; // local track — nothing to scrobble
                    }
                    let duration = track.duration;
                    let position = s.position;
                    if s.status != PlaybackStatus::Playing { return None; }
                    if duration == 0 { return None; }
                    let threshold = (duration / 2).max(30);
                    if position < threshold { return None; }

                    // Extract song ID from stream URL query param.
                    let song_id = path.split('?').nth(1)?
                        .split('&')
                        .find(|p| p.starts_with("id="))
                        .map(|p| p[3..].to_string())?;

                    // Check whether we already scrobbled this song ID.
                    let already = app.global::<NdScrobbleState>().scrobbled_id.as_deref() == Some(&song_id);
                    if already { return None; }

                    // Fetch ND credentials.
                    let nd = app.global::<NavidromeServerState>();
                    let creds = nd.active_server()?.clone();

                    Some((song_id, creds.base_url, creds.user, creds.token, creds.salt))
                }).ok().flatten();

                if let Some((song_id, base_url, user, token, salt)) = scrobble_task {
                    // Mark scrobbled before firing so rapid ticks don't double-submit.
                    let _ = cx.update(|app| {
                        app.global_mut::<NdScrobbleState>().scrobbled_id = Some(song_id.clone());
                    });
                    rt_handle.spawn(async move {
                        crate::navidrome::scrobble_song(&base_url, &user, &token, &salt, &song_id).await;
                    });
                }

                // Reset per-track state when the track changes.
                let _ = cx.update(|app| {
                    let s = state_for_poll.read(app);
                    let current_id = s.current_track()
                        .and_then(|t| {
                            t.path.split('?').nth(1)?
                                .split('&')
                                .find(|p| p.starts_with("id="))
                                .map(|p| p[3..].to_string())
                        });
                    let scrobbled = app.global::<NdScrobbleState>().scrobbled_id.clone();
                    if scrobbled.is_some() && scrobbled != current_id {
                        app.global_mut::<NdScrobbleState>().scrobbled_id = None;
                    }
                    let fetched = app.global::<NdCoverFetchState>().fetched_id.clone();
                    if fetched.is_some() && fetched != current_id {
                        app.global_mut::<NdCoverFetchState>().fetched_id = None;
                        app.global_mut::<NdCurrentCoverArt>().0 = None;
                    }
                });

                // Media-controls tick: drain OS key events and push now-playing info.
                if let Some(np) = &np_for_poll {
                    if let Ok(mut np) = np.try_lock() {
                        // Execute any media-key commands.
                        for cmd in np.drain_commands() {
                            match cmd {
                                MediaCommand::Play => {
                                    rt_handle.spawn(crate::client::resume());
                                }
                                MediaCommand::Pause => {
                                    rt_handle.spawn(crate::client::pause());
                                }
                                MediaCommand::Toggle => {
                                    let status = cx
                                        .update(|app| state_for_poll.read(app).status)
                                        .unwrap_or(PlaybackStatus::Stopped);
                                    match status {
                                        PlaybackStatus::Playing => {
                                            rt_handle.spawn(crate::client::pause());
                                        }
                                        _ => {
                                            rt_handle.spawn(crate::client::resume());
                                        }
                                    }
                                }
                                MediaCommand::Next => {
                                    rt_handle.spawn(crate::client::next());
                                }
                                MediaCommand::Prev => {
                                    rt_handle.spawn(crate::client::prev());
                                }
                                MediaCommand::SeekTo(pos) => {
                                    let ms = pos.as_millis() as i32;
                                    rt_handle.spawn(crate::client::seek(ms));
                                }
                            }
                        }

                        // Push current playback state to the OS notification bar.
                        // Fall back to queue.first() when current_idx is not yet set
                        // (playlist loaded but audio engine still initialising on open).
                        let _ = cx.update(|app| {
                            let s = state_for_poll.read(app);
                            let track = s.current_track().or_else(|| s.queue.first());
                            np.update(track, s.status, s.position);
                        });
                    }
                }

                cx.background_executor()
                    .timer(Duration::from_millis(100))
                    .await;
            }
        })
        .detach();

        Controller {
            state,
            rt,
            tx,
            search_gen: Arc::new(AtomicU64::new(0)),
            now_playing,
        }
    }

    /// Cloneable handle to the tokio runtime — use for fire-and-forget spawns.
    pub fn rt(&self) -> tokio::runtime::Handle {
        self.rt.handle().clone()
    }

    // ── Playback actions ──────────────────────────────────────────────────────

    pub fn next(&self) {
        self.rt().spawn(crate::client::next());
    }

    pub fn prev(&self) {
        self.rt().spawn(crate::client::prev());
    }

    /// Seek to `position_secs` seconds from the start of the current track.
    pub fn seek(&self, position_secs: u64, duration_secs: u64) {
        if duration_secs == 0 {
            return;
        }
        let ms = (position_secs as i32).saturating_mul(1000);
        self.rt().spawn(crate::client::seek(ms));
    }

    pub fn play_track_at_idx(&self, idx: usize, cx: &App) {
        let path = self.state.read(cx).tracks.get(idx).map(|t| t.path.clone());
        if let Some(path) = path {
            self.rt().spawn(crate::client::play_track(path));
        }
    }

    pub fn play_album(&self, album_id: String, shuffle: bool) {
        self.rt()
            .spawn(crate::client::play_album(album_id, shuffle));
    }

    pub fn play_artist_tracks(&self, artist_id: String, shuffle: bool) {
        self.rt()
            .spawn(crate::client::play_artist_tracks(artist_id, shuffle));
    }

    pub fn play_all_tracks(&self) {
        self.rt().spawn(crate::client::play_all_tracks());
    }

    pub fn jump_to_queue_position(&self, pos: i32) {
        self.rt().spawn(crate::client::jump_to_queue_position(pos));
    }

    pub fn play_liked_tracks(&self, paths: Vec<String>, shuffle: bool) {
        self.rt()
            .spawn(crate::client::play_liked_tracks(paths, shuffle));
    }

    pub fn insert_track_next(&self, path: String) {
        let tx = self.tx.clone();
        self.rt().spawn(async move {
            let _ = crate::client::insert_track_next(path).await;
            crate::client::fetch_queue(tx).await;
        });
    }

    pub fn insert_track_last(&self, path: String) {
        let tx = self.tx.clone();
        self.rt().spawn(async move {
            let _ = crate::client::insert_track_last(path).await;
            crate::client::fetch_queue(tx).await;
        });
    }

    pub fn remove_from_queue(&self, pos: usize) {
        let tx = self.tx.clone();
        self.rt().spawn(async move {
            let _ = crate::client::remove_from_queue(pos as i32).await;
            crate::client::fetch_queue(tx).await;
        });
    }

    pub fn adjust_volume(&self, steps: i32) {
        self.rt().spawn(crate::client::adjust_volume(steps));
    }

    pub fn insert_tracks(&self, paths: Vec<String>, position: i32, shuffle: bool) {
        let tx = self.tx.clone();
        self.rt().spawn(async move {
            let _ = crate::client::insert_tracks(paths, position, shuffle).await;
            crate::client::fetch_queue(tx).await;
        });
    }

    pub fn search(&self, query: String) {
        let tx = self.tx.clone();
        let gen = self.search_gen.fetch_add(1, Ordering::SeqCst) + 1;
        let search_gen = self.search_gen.clone();
        self.rt().spawn(async move {
            if query.trim().is_empty() {
                let _ = tx.send(StateUpdate::SearchResults(None)).await;
                return;
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
            if search_gen.load(Ordering::SeqCst) != gen {
                return;
            }
            match crate::client::search(query).await {
                Ok(results) => {
                    if search_gen.load(Ordering::SeqCst) == gen {
                        let _ = tx.send(StateUpdate::SearchResults(Some(results))).await;
                    }
                }
                Err(e) => log::warn!("search: {e}"),
            }
        });
    }
}

impl Global for Controller {}
