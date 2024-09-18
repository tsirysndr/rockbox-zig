use async_graphql::*;
use rockbox_sys::types::mp3_entry::Mp3Entry;
use serde::Serialize;

#[derive(Default, Clone, Serialize)]
pub struct Track {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub genre: String,
    pub disc: String,
    pub track_string: String,
    pub year_string: String,
    pub composer: String,
    pub comment: String,
    pub album_artist: String,
    pub grouping: String,
    pub discnum: i32,
    pub tracknum: i32,
    pub layer: i32,
    pub year: i32,
    pub bitrate: u32,
    pub frequency: u64,
    pub filesize: u64,
    pub length: u64,
    pub elapsed: u64,
    pub path: String,
}

#[Object]
impl Track {
    async fn title(&self) -> &str {
        &self.title
    }

    async fn artist(&self) -> &str {
        &self.artist
    }

    async fn album(&self) -> &str {
        &self.album
    }

    async fn genre(&self) -> &str {
        &self.genre
    }

    async fn disc(&self) -> &str {
        &self.disc
    }

    async fn track_string(&self) -> &str {
        &self.track_string
    }

    async fn year_string(&self) -> &str {
        &self.year_string
    }

    async fn composer(&self) -> &str {
        &self.composer
    }

    async fn comment(&self) -> &str {
        &self.comment
    }

    async fn album_artist(&self) -> &str {
        &self.album_artist
    }

    async fn grouping(&self) -> &str {
        &self.grouping
    }

    async fn discnum(&self) -> i32 {
        self.discnum
    }

    async fn tracknum(&self) -> i32 {
        self.tracknum
    }

    async fn layer(&self) -> i32 {
        self.layer
    }

    async fn year(&self) -> i32 {
        self.year
    }

    async fn bitrate(&self) -> u32 {
        self.bitrate
    }

    async fn frequency(&self) -> u64 {
        self.frequency
    }

    async fn filesize(&self) -> u64 {
        self.filesize
    }

    async fn length(&self) -> u64 {
        self.length
    }

    async fn elapsed(&self) -> u64 {
        self.elapsed
    }

    async fn path(&self) -> &str {
        &self.path
    }
}

impl From<Mp3Entry> for Track {
    fn from(mp3entry: Mp3Entry) -> Self {
        let title = mp3entry.title;
        let artist = mp3entry.artist;
        let album = mp3entry.album;
        let genre = mp3entry.genre_string;
        let disc = mp3entry.disc_string;
        let track_string = mp3entry.track_string;
        let year_string = mp3entry.year_string;
        let composer = mp3entry.composer;
        let comment = mp3entry.comment;
        let album_artist = mp3entry.albumartist;
        let grouping = mp3entry.grouping;
        let discnum = mp3entry.discnum;
        let tracknum = mp3entry.tracknum;
        let layer = mp3entry.layer;
        let year = mp3entry.year;
        let bitrate = mp3entry.bitrate;
        let frequency = mp3entry.frequency;
        let filesize = mp3entry.filesize;
        let length = mp3entry.length;
        let elapsed = mp3entry.elapsed;
        let path = mp3entry.path;

        Track {
            title,
            artist,
            album,
            genre,
            disc,
            track_string,
            year_string,
            composer,
            comment,
            album_artist,
            grouping,
            discnum,
            tracknum,
            layer,
            year,
            bitrate,
            frequency,
            filesize,
            length,
            elapsed,
            path,
        }
    }
}
