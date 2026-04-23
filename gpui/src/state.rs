#[derive(Clone, Debug, Default)]
pub struct Track {
    pub id: String,
    pub path: String,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub album_id: String,
    pub artist_id: String,
    pub genre: String,
    pub duration: u64,
    pub track_number: u32,
    pub album_art: Option<String>,
}

pub type ArtistImages = std::collections::HashMap<String, String>;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PlaybackStatus {
    Stopped,
    Playing,
    Paused,
}

pub struct AppState {
    pub tracks: Vec<Track>,
    pub queue: Vec<Track>,
    pub current_idx: Option<usize>,
    pub status: PlaybackStatus,
    pub position: u64,
    pub volume: f32,
    pub shuffling: bool,
    pub repeat: bool,
    pub artist_images: ArtistImages,
}

impl AppState {
    pub fn new() -> Self {
        AppState {
            tracks: vec![],
            queue: vec![],
            current_idx: None,
            status: PlaybackStatus::Stopped,
            position: 0,
            volume: 0.8,
            shuffling: false,
            repeat: false,
            artist_images: Default::default(),
        }
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
}

pub fn format_duration(secs: u64) -> String {
    format!("{}:{:02}", secs / 60, secs % 60)
}
