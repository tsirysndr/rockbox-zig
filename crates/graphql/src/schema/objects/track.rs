use async_graphql::*;
use rockbox_sys::types::mp3_entry::Mp3Entry;
use serde::{Deserialize, Serialize};
use tantivy::schema::Schema;
use tantivy::schema::SchemaBuilder;
use tantivy::schema::Value;
use tantivy::schema::*;
use tantivy::TantivyDocument;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Track {
    pub id: Option<String>,
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
    pub album_id: Option<String>,
    pub artist_id: Option<String>,
    pub genre_id: Option<String>,
    pub album_art: Option<String>,
}

#[Object]
impl Track {
    async fn id(&self) -> Option<String> {
        self.id.clone()
    }

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

    async fn album_id(&self) -> Option<&str> {
        self.album_id.as_deref()
    }

    async fn artist_id(&self) -> Option<&str> {
        self.artist_id.as_deref()
    }

    async fn genre_id(&self) -> Option<&str> {
        self.genre_id.as_deref()
    }

    async fn album_art(&self) -> Option<&str> {
        self.album_art.as_deref()
    }
}

impl From<Mp3Entry> for Track {
    fn from(mp3entry: Mp3Entry) -> Self {
        let id = mp3entry.id;
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
        let album_id = mp3entry.album_id;
        let artist_id = mp3entry.artist_id;
        let genre_id = mp3entry.genre_id;
        let album_art = mp3entry.album_art;

        Track {
            id,
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
            album_id,
            artist_id,
            genre_id,
            album_art,
            ..Default::default()
        }
    }
}

impl From<rockbox_library::entity::track::Track> for Track {
    fn from(track: rockbox_library::entity::track::Track) -> Self {
        Self {
            id: Some(track.id),
            title: track.title,
            artist: track.artist,
            album: track.album,
            genre: track.genre.unwrap_or_default(),
            year_string: track.year_string.unwrap_or_default(),
            composer: track.composer,
            album_artist: track.album_artist,
            discnum: track.disc_number as i32,
            tracknum: track.track_number.unwrap_or_default() as i32,
            year: track.year.unwrap_or_default() as i32,
            bitrate: track.bitrate,
            frequency: track.frequency as u64,
            filesize: track.filesize as u64,
            length: track.length as u64,
            artist_id: Some(track.artist_id),
            album_id: Some(track.album_id),
            genre_id: Some(track.genre_id),
            path: track.path,
            album_art: track.album_art,
            ..Default::default()
        }
    }
}

impl From<rockbox_search::track::Track> for Track {
    fn from(track: rockbox_search::track::Track) -> Self {
        Self {
            id: Some(track.id),
            title: track.title,
            artist: track.artist,
            album: track.album,
            genre: track.genre,
            year_string: track.year_string,
            composer: track.composer,
            album_artist: track.album_artist,
            discnum: track.disc_number as i32,
            tracknum: track.track_number as i32,
            year: track.year as i32,
            bitrate: track.bitrate as u32,
            frequency: track.frequency as u64,
            filesize: track.filesize as u64,
            length: track.length as u64,
            artist_id: track.artist_id,
            album_id: track.album_id,
            genre_id: track.genre_id,
            path: track.path,
            album_art: track.album_art,
            ..Default::default()
        }
    }
}

impl From<rockbox_search::liked_track::LikedTrack> for Track {
    fn from(track: rockbox_search::liked_track::LikedTrack) -> Self {
        Self {
            id: Some(track.id),
            title: track.title,
            artist: track.artist,
            album: track.album,
            genre: track.genre,
            year_string: track.year_string,
            composer: track.composer,
            album_artist: track.album_artist,
            discnum: track.disc_number as i32,
            tracknum: track.track_number as i32,
            year: track.year as i32,
            bitrate: track.bitrate as u32,
            frequency: track.frequency as u64,
            filesize: track.filesize as u64,
            length: track.length as u64,
            artist_id: track.artist_id,
            album_id: track.album_id,
            genre_id: track.genre_id,
            path: track.path,
            album_art: track.album_art,
            ..Default::default()
        }
    }
}

impl From<TantivyDocument> for Track {
    fn from(document: TantivyDocument) -> Self {
        let mut schema_builder: SchemaBuilder = Schema::builder();

        let id_field = schema_builder.add_text_field("id", STRING | STORED);
        let path_field = schema_builder.add_text_field("path", TEXT | STORED);
        let title_field = schema_builder.add_text_field("title", TEXT | STORED);
        let artist_field = schema_builder.add_text_field("artist", TEXT | STORED);
        let album_field = schema_builder.add_text_field("album", TEXT | STORED);
        let album_artist_field = schema_builder.add_text_field("album_artist", TEXT | STORED);
        let bitrate_field = schema_builder.add_i64_field("bitrate", STORED);
        let composer_field = schema_builder.add_text_field("composer", TEXT | STORED);
        let disc_number_field = schema_builder.add_i64_field("disc_number", STORED);
        let filesize_field = schema_builder.add_i64_field("filesize", STORED);
        let frequency_field = schema_builder.add_i64_field("frequency", STORED);
        let length_field = schema_builder.add_i64_field("length", STORED);
        let track_number_field = schema_builder.add_i64_field("track_number", STORED);
        let year_field = schema_builder.add_i64_field("year", STORED);
        let year_string_field = schema_builder.add_text_field("year_string", STRING | STORED);
        let genre_field = schema_builder.add_text_field("genre", TEXT | STORED);
        let md5_field = schema_builder.add_text_field("md5", STRING | STORED);
        let album_art_field = schema_builder.add_text_field("album_art", STRING | STORED);
        let artist_id_field = schema_builder.add_text_field("artist_id", STRING | STORED);
        let album_id_field = schema_builder.add_text_field("album_id", STRING | STORED);
        let genre_id_field = schema_builder.add_text_field("genre_id", STRING | STORED);
        let created_at_field = schema_builder.add_text_field("created_at", STRING | STORED);
        let updated_at_field = schema_builder.add_text_field("updated_at", STRING | STORED);

        let id = document
            .get_first(id_field)
            .unwrap()
            .as_str()
            .unwrap()
            .to_string();
        let path = document
            .get_first(path_field)
            .unwrap()
            .as_str()
            .unwrap()
            .to_string();
        let title = document
            .get_first(title_field)
            .unwrap()
            .as_str()
            .unwrap()
            .to_string();
        let artist = document
            .get_first(artist_field)
            .unwrap()
            .as_str()
            .unwrap()
            .to_string();
        let album = document
            .get_first(album_field)
            .unwrap()
            .as_str()
            .unwrap()
            .to_string();
        let album_artist = document
            .get_first(album_artist_field)
            .unwrap()
            .as_str()
            .unwrap()
            .to_string();
        let bitrate = document.get_first(bitrate_field).unwrap().as_i64().unwrap() as u32;
        let composer = document
            .get_first(composer_field)
            .unwrap()
            .as_str()
            .unwrap()
            .to_string();
        let disc_number = document
            .get_first(disc_number_field)
            .unwrap()
            .as_i64()
            .unwrap() as u64;
        let filesize = document
            .get_first(filesize_field)
            .unwrap()
            .as_i64()
            .unwrap() as u64;
        let frequency = document
            .get_first(frequency_field)
            .unwrap()
            .as_i64()
            .unwrap() as u64;
        let length = document.get_first(length_field).unwrap().as_i64().unwrap() as u64;
        let track_number = document
            .get_first(track_number_field)
            .unwrap()
            .as_i64()
            .unwrap() as u64;
        let year = document.get_first(year_field).unwrap().as_i64().unwrap() as i32;
        let year_string = document
            .get_first(year_string_field)
            .unwrap()
            .as_str()
            .unwrap()
            .to_string();
        let genre = document
            .get_first(genre_field)
            .unwrap()
            .as_str()
            .unwrap()
            .to_string();
        let album_art = match document.get_first(album_art_field) {
            Some(album_art) => album_art.as_str(),
            None => None,
        };
        let album_art = match album_art {
            Some("") => None,
            Some(album_art) => Some(album_art.to_string()),
            None => None,
        };
        let artist_id = match document.get_first(artist_id_field) {
            Some(artist_id) => Some(artist_id.as_str().unwrap().to_string()),
            None => None,
        };
        let album_id = match document.get_first(album_id_field) {
            Some(album_id) => Some(album_id.as_str().unwrap().to_string()),
            None => None,
        };
        let album_id = match album_id {
            Some(album_id) => Some(album_id.to_string()),
            None => None,
        };
        let genre_id = match document.get_first(genre_id_field) {
            Some(genre_id) => Some(genre_id.as_str().unwrap().to_string()),
            None => None,
        };

        Self {
            id: Some(id),
            path,
            title,
            artist,
            album,
            album_artist,
            bitrate,
            composer,
            discnum: disc_number as i32,
            filesize,
            frequency,
            length,
            tracknum: track_number as i32,
            year,
            year_string,
            genre,
            album_art,
            artist_id,
            album_id,
            genre_id,
            ..Default::default()
        }
    }
}
