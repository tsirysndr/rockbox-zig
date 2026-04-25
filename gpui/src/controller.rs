use crate::now_playing::{MediaCommand, NowPlayingManager};
use crate::state::{AppState, PlaybackStatus, StateUpdate};
use crate::ui::components::{LikedOrder, LikedSongs};
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
        rt.spawn(crate::client::run_liked_tracks_sync(tx.clone()));
        rt.spawn(crate::client::run_artist_images_sync(tx.clone()));
        rt.spawn(crate::client::run_settings_sync(tx.clone()));
        rt.spawn(crate::client::run_resume_info_sync(tx.clone()));
        rt.spawn(crate::client::run_status_stream(tx.clone()));
        rt.spawn(crate::client::run_current_track_stream(tx.clone()));
        rt.spawn(crate::client::run_playlist_stream(tx.clone()));

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
                            StateUpdate::Settings { volume, shuffling, repeat_mode } => {
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
                        let _ = cx.update(|app| {
                            let s = state_for_poll.read(app);
                            np.update(s.current_track(), s.status, s.position);
                        });
                    }
                }

                cx.background_executor()
                    .timer(Duration::from_millis(100))
                    .await;
            }
        })
        .detach();

        Controller { state, rt, tx, search_gen: Arc::new(AtomicU64::new(0)), now_playing }
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
        if duration_secs == 0 { return; }
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
