#[derive(Clone, Debug, Default)]
pub struct Track {
    pub id: String,
    pub path: String,
    pub title: String,
    pub artist: String,
    pub album_artist: String,
    pub album: String,
    pub album_id: String,
    pub artist_id: String,
    pub genre: String,
    pub duration: u64,
    pub track_number: u32,
    pub disc_number: u32,
    pub year: u32,
    pub year_string: String,
    pub album_art: Option<String>,
}

pub type ArtistImages = std::collections::HashMap<String, String>;

#[derive(Clone, Default)]
pub struct SearchAlbum {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub year: u32,
    pub album_art: Option<String>,
    pub artist_id: String,
}

#[derive(Clone, Default)]
pub struct SearchArtist {
    pub id: String,
    pub name: String,
    pub image: Option<String>,
}

#[derive(Clone, Default)]
pub struct SearchResults {
    pub tracks: Vec<Track>,
    pub albums: Vec<SearchAlbum>,
    pub artists: Vec<SearchArtist>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PlaybackStatus {
    Stopped,
    Playing,
    Paused,
}

// Rockbox volume is in dB; typical SDL target range -74..0
pub const VOLUME_MIN_DB: i32 = -74;
pub const VOLUME_MAX_DB: i32 = 0;

pub fn volume_fraction(db: i32) -> f32 {
    let range = (VOLUME_MAX_DB - VOLUME_MIN_DB) as f32;
    (db.clamp(VOLUME_MIN_DB, VOLUME_MAX_DB) - VOLUME_MIN_DB) as f32 / range
}

pub struct AppState {
    pub tracks: Vec<Track>,
    pub queue: Vec<Track>,
    pub current_idx: Option<usize>,
    pub status: PlaybackStatus,
    pub position: u64,
    pub volume: i32,
    pub shuffling: bool,
    pub repeat: bool,
    pub artist_images: ArtistImages,
    pub search_results: Option<SearchResults>,
    /// Holds a deadline until which status stream updates are ignored.
    /// Set on user-initiated play/pause to prevent the still-stale server
    /// stream from immediately reverting the optimistic UI update.
    status_locked_until: Option<std::time::Instant>,
}

impl AppState {
    pub fn new() -> Self {
        AppState {
            tracks: vec![],
            queue: vec![],
            current_idx: None,
            status: PlaybackStatus::Stopped,
            position: 0,
            volume: -15,
            shuffling: false,
            repeat: false,
            artist_images: Default::default(),
            search_results: None,
            status_locked_until: None,
        }
    }

    /// Optimistically set status and block stream updates for `duration`.
    pub fn set_status_local(&mut self, status: PlaybackStatus, duration: std::time::Duration) {
        self.status = status;
        self.status_locked_until = Some(std::time::Instant::now() + duration);
    }

    /// Apply a status update from the server stream, respecting the lock.
    pub fn apply_status_from_stream(&mut self, status: PlaybackStatus) {
        if let Some(deadline) = self.status_locked_until {
            if std::time::Instant::now() < deadline {
                return;
            }
            self.status_locked_until = None;
        }
        self.status = status;
    }

    pub fn current_track(&self) -> Option<&Track> {
        self.current_idx.and_then(|i| self.queue.get(i))
    }

    /// Index into `self.tracks` matching the current queue track by path.
    pub fn current_library_idx(&self) -> Option<usize> {
        let current = self.current_track()?;
        self.tracks.iter().position(|t| t.path == current.path)
    }

    pub fn toggle_shuffle(&mut self) {
        self.shuffling = !self.shuffling;
    }

    pub fn toggle_repeat(&mut self) {
        self.repeat = !self.repeat;
    }
}

pub enum StateUpdate {
    Status(PlaybackStatus),
    Position(u64),
    Playlist {
        queue: Vec<Track>,
        current_idx: Option<usize>,
    },
    Tracks(Vec<Track>),
    ArtistImages(ArtistImages),
    LikedTracks(Vec<String>),
    SearchResults(Option<SearchResults>),
    Settings {
        volume: i32,
        shuffling: bool,
        repeat_mode: i32,
    },
}

pub fn format_duration(secs: u64) -> String {
    format!("{}:{:02}", secs / 60, secs % 60)
}

/// Stores the Tokio runtime handle so GPUI code can run async tasks that require a Tokio reactor.
#[derive(Clone)]
pub struct TokioHandle(pub tokio::runtime::Handle);
impl gpui::Global for TokioHandle {}
