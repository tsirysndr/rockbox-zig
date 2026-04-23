use crate::state::{AppState, StateUpdate};
use gpui::{App, Entity, Global};
use std::time::Duration;
use tokio::sync::mpsc;

pub struct Controller {
    pub state: Entity<AppState>,
    rt: tokio::runtime::Runtime,
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
        rt.spawn(crate::client::run_artist_images_sync(tx.clone()));
        rt.spawn(crate::client::run_status_stream(tx.clone()));
        rt.spawn(crate::client::run_current_track_stream(tx.clone()));
        rt.spawn(crate::client::run_playlist_stream(tx.clone()));

        // GPUI foreground poll task — not required to be Send
        let state_for_poll = state.clone();
        cx.spawn(async move |cx| {
            let mut rx = rx;
            loop {
                let mut did_update = false;
                while let Ok(update) = rx.try_recv() {
                    let _ = cx.update(|app| {
                        state_for_poll.update(app, |s, _| match update {
                            StateUpdate::Status(status) => s.status = status,
                            StateUpdate::Position(pos) => s.position = pos,
                            StateUpdate::Playlist { queue, current_idx } => {
                                s.queue = queue;
                                s.current_idx = current_idx;
                            }
                            StateUpdate::Tracks(tracks) => s.tracks = tracks,
                            StateUpdate::ArtistImages(images) => s.artist_images = images,
                        });
                    });
                    did_update = true;
                }
                if did_update {
                    let _ = cx.update(|app| {
                        state_for_poll.update(app, |_, cx| cx.notify());
                    });
                }
                cx.background_executor()
                    .timer(Duration::from_millis(100))
                    .await;
            }
        })
        .detach();

        Controller { state, rt }
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

    pub fn play_track_at_idx(&self, idx: usize, cx: &App) {
        let path = self.state.read(cx).tracks.get(idx).map(|t| t.path.clone());
        if let Some(path) = path {
            self.rt().spawn(crate::client::play_track(path));
        }
    }

    pub fn play_album(&self, album_id: String) {
        self.rt().spawn(crate::client::play_album(album_id));
    }

    pub fn play_artist_tracks(&self, artist_id: String) {
        self.rt()
            .spawn(crate::client::play_artist_tracks(artist_id));
    }

    pub fn play_all_tracks(&self) {
        self.rt().spawn(crate::client::play_all_tracks());
    }

    pub fn jump_to_queue_position(&self, pos: i32) {
        self.rt().spawn(crate::client::jump_to_queue_position(pos));
    }
}

impl Global for Controller {}
