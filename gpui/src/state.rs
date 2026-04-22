#[derive(Clone, Debug)]
pub struct Track {
    pub id: usize,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub genre: String,
    pub duration: u64,
    pub track_number: u32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PlaybackStatus {
    Stopped,
    Playing,
    Paused,
}

pub struct AppState {
    pub tracks: Vec<Track>,
    pub queue: Vec<usize>,
    pub current_idx: Option<usize>,
    pub status: PlaybackStatus,
    pub position: u64,
    pub volume: f32,
    pub shuffling: bool,
    pub repeat: bool,
}

impl AppState {
    pub fn new() -> Self {
        let tracks = mock_tracks();
        let n = tracks.len();
        AppState {
            tracks,
            queue: (0..n).collect(),
            current_idx: Some(0),
            status: PlaybackStatus::Paused,
            position: 42,
            volume: 0.8,
            shuffling: false,
            repeat: false,
        }
    }

    pub fn current_track(&self) -> Option<&Track> {
        self.current_idx.map(|i| &self.tracks[i])
    }

    pub fn play(&mut self) {
        self.status = PlaybackStatus::Playing;
    }

    pub fn pause(&mut self) {
        self.status = PlaybackStatus::Paused;
    }

    pub fn next(&mut self) {
        if let Some(idx) = self.current_idx {
            self.current_idx = Some((idx + 1) % self.tracks.len());
            self.position = 0;
        }
    }

    pub fn prev(&mut self) {
        if let Some(idx) = self.current_idx {
            self.current_idx = Some(if idx == 0 {
                self.tracks.len() - 1
            } else {
                idx - 1
            });
            self.position = 0;
        }
    }

    pub fn toggle_shuffle(&mut self) {
        self.shuffling = !self.shuffling;
    }

    pub fn toggle_repeat(&mut self) {
        self.repeat = !self.repeat;
    }

    pub fn play_track(&mut self, idx: usize) {
        self.current_idx = Some(idx);
        self.position = 0;
        self.status = PlaybackStatus::Playing;
    }
}

fn mock_tracks() -> Vec<Track> {
    vec![
        Track {
            id: 1,
            title: "Bohemian Rhapsody".into(),
            artist: "Queen".into(),
            album: "A Night at the Opera".into(),
            genre: "Rock".into(),
            duration: 354,
            track_number: 11,
        },
        Track {
            id: 2,
            title: "Hotel California".into(),
            artist: "Eagles".into(),
            album: "Hotel California".into(),
            genre: "Rock".into(),
            duration: 391,
            track_number: 1,
        },
        Track {
            id: 3,
            title: "Stairway to Heaven".into(),
            artist: "Led Zeppelin".into(),
            album: "Led Zeppelin IV".into(),
            genre: "Rock".into(),
            duration: 482,
            track_number: 4,
        },
        Track {
            id: 4,
            title: "Blinding Lights".into(),
            artist: "The Weeknd".into(),
            album: "After Hours".into(),
            genre: "Pop".into(),
            duration: 200,
            track_number: 2,
        },
        Track {
            id: 5,
            title: "Shape of You".into(),
            artist: "Ed Sheeran".into(),
            album: "Divide".into(),
            genre: "Pop".into(),
            duration: 234,
            track_number: 1,
        },
        Track {
            id: 6,
            title: "Lose Yourself".into(),
            artist: "Eminem".into(),
            album: "8 Mile".into(),
            genre: "Hip-Hop".into(),
            duration: 326,
            track_number: 1,
        },
        Track {
            id: 7,
            title: "One More Time".into(),
            artist: "Daft Punk".into(),
            album: "Discovery".into(),
            genre: "Electronic".into(),
            duration: 320,
            track_number: 1,
        },
        Track {
            id: 8,
            title: "Get Lucky".into(),
            artist: "Daft Punk".into(),
            album: "Random Access Memories".into(),
            genre: "Electronic".into(),
            duration: 248,
            track_number: 8,
        },
        Track {
            id: 9,
            title: "Comfortably Numb".into(),
            artist: "Pink Floyd".into(),
            album: "The Wall".into(),
            genre: "Rock".into(),
            duration: 382,
            track_number: 27,
        },
        Track {
            id: 10,
            title: "Wish You Were Here".into(),
            artist: "Pink Floyd".into(),
            album: "Wish You Were Here".into(),
            genre: "Rock".into(),
            duration: 310,
            track_number: 1,
        },
        Track {
            id: 11,
            title: "Superstition".into(),
            artist: "Stevie Wonder".into(),
            album: "Talking Book".into(),
            genre: "Soul".into(),
            duration: 245,
            track_number: 1,
        },
        Track {
            id: 12,
            title: "Billie Jean".into(),
            artist: "Michael Jackson".into(),
            album: "Thriller".into(),
            genre: "Pop".into(),
            duration: 294,
            track_number: 6,
        },
    ]
}

pub fn format_duration(secs: u64) -> String {
    format!("{}:{:02}", secs / 60, secs % 60)
}
