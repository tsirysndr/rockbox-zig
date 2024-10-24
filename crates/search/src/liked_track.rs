use crate::{Indexable, Searchable};
use rockbox_library::entity;
use serde::{Deserialize, Serialize};
use tantivy::{doc, schema::*, TantivyDocument};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct LikedTrack {
    pub id: String,
    pub path: String,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub album_artist: String,
    pub bitrate: i64,
    pub composer: String,
    pub disc_number: i64,
    pub filesize: i64,
    pub frequency: i64,
    pub length: i64,
    pub track_number: i64,
    pub year: i64,
    pub year_string: String,
    pub genre: String,
    pub md5: String,
    pub album_art: Option<String>,
    pub artist_id: Option<String>,
    pub album_id: Option<String>,
    pub genre_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl Indexable for LikedTrack {
    fn to_document(&self) -> TantivyDocument {
        let schema: Schema = self.build_schema();

        let id = schema.get_field("id").unwrap();
        let path = schema.get_field("path").unwrap();
        let title = schema.get_field("title").unwrap();
        let artist = schema.get_field("artist").unwrap();
        let album = schema.get_field("album").unwrap();
        let album_artist = schema.get_field("album_artist").unwrap();
        let bitrate = schema.get_field("bitrate").unwrap();
        let composer = schema.get_field("composer").unwrap();
        let disc_number = schema.get_field("disc_number").unwrap();
        let filesize = schema.get_field("filesize").unwrap();
        let frequency = schema.get_field("frequency").unwrap();
        let length = schema.get_field("length").unwrap();
        let track_number = schema.get_field("track_number").unwrap();
        let year = schema.get_field("year").unwrap();
        let year_string = schema.get_field("year_string").unwrap();
        let genre = schema.get_field("genre").unwrap();
        let md5 = schema.get_field("md5").unwrap();
        let album_art = schema.get_field("album_art").unwrap();
        let artist_id = schema.get_field("artist_id").unwrap();
        let album_id = schema.get_field("album_id").unwrap();
        let genre_id = schema.get_field("genre_id").unwrap();
        let created_at = schema.get_field("created_at").unwrap();
        let updated_at = schema.get_field("updated_at").unwrap();

        let mut document = doc!(
            id => self.id.to_owned(),
            path => self.path.to_owned(),
            title => self.title.to_owned(),
            artist => self.artist.to_owned(),
            album => self.album.to_owned(),
            album_artist => self.album_artist.to_owned(),
            bitrate => self.bitrate,
            composer => self.composer.to_owned(),
            disc_number => self.disc_number,
            filesize => self.filesize,
            frequency => self.frequency,
            length => self.length,
            track_number => self.track_number,
            year => self.year,
            year_string => self.year_string.to_owned(),
            genre => self.genre.to_owned(),
            md5 => self.md5.to_owned(),
        );

        if let Some(value) = &self.album_art {
            document.add_text(album_art, value);
        }

        if let Some(value) = &self.artist_id {
            document.add_text(artist_id, value);
        }

        if let Some(value) = &self.album_id {
            document.add_text(album_id, value);
        }

        if let Some(value) = &self.genre_id {
            document.add_text(genre_id, value);
        }

        document.add_text(created_at, &self.created_at);
        document.add_text(updated_at, &self.updated_at);

        document
    }

    fn build_schema(&self) -> Schema {
        let mut schema_builder: SchemaBuilder = Schema::builder();

        schema_builder.add_text_field("id", STRING | STORED);
        schema_builder.add_text_field("path", TEXT | STORED);
        schema_builder.add_text_field("title", TEXT | STORED);
        schema_builder.add_text_field("artist", TEXT | STORED);
        schema_builder.add_text_field("album", TEXT | STORED);
        schema_builder.add_text_field("album_artist", TEXT | STORED);
        schema_builder.add_i64_field("bitrate", STORED);
        schema_builder.add_text_field("composer", TEXT | STORED);
        schema_builder.add_i64_field("disc_number", STORED);
        schema_builder.add_i64_field("filesize", STORED);
        schema_builder.add_i64_field("frequency", STORED);
        schema_builder.add_i64_field("length", STORED);
        schema_builder.add_i64_field("track_number", STORED);
        schema_builder.add_i64_field("year", STORED);
        schema_builder.add_text_field("year_string", STRING | STORED);
        schema_builder.add_text_field("genre", TEXT | STORED);
        schema_builder.add_text_field("md5", STRING | STORED);
        schema_builder.add_text_field("album_art", STRING | STORED);
        schema_builder.add_text_field("artist_id", STRING | STORED);
        schema_builder.add_text_field("album_id", STRING | STORED);
        schema_builder.add_text_field("genre_id", STRING | STORED);
        schema_builder.add_text_field("created_at", STRING | STORED);
        schema_builder.add_text_field("updated_at", STRING | STORED);

        schema_builder.build()
    }
}

impl Searchable for LikedTrack {
    fn schema(&self) -> Schema {
        self.build_schema()
    }

    fn default_fields(&self) -> Vec<String> {
        vec![
            "title".to_string(),
            "artist".to_string(),
            "album".to_string(),
            "album_artist".to_string(),
            "composer".to_string(),
        ]
    }
}

impl From<entity::track::Track> for LikedTrack {
    fn from(track: entity::track::Track) -> Self {
        Self {
            id: track.id,
            path: track.path,
            title: track.title,
            artist: track.artist,
            album: track.album,
            album_artist: track.album_artist,
            bitrate: track.bitrate as i64,
            composer: track.composer,
            disc_number: track.disc_number as i64,
            filesize: track.filesize as i64,
            frequency: track.frequency as i64,
            length: track.length as i64,
            track_number: track.track_number.unwrap_or_default() as i64,
            year: track.year.unwrap_or_default() as i64,
            year_string: track.year_string.unwrap_or_default(),
            genre: track.genre.unwrap_or_default(),
            md5: track.md5,
            album_art: track.album_art,
            artist_id: Some(track.artist_id),
            album_id: Some(track.album_id),
            genre_id: Some(track.genre_id),
            created_at: track.created_at.to_rfc3339(),
            updated_at: track.updated_at.to_rfc3339(),
        }
    }
}

impl From<TantivyDocument> for LikedTrack {
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
        let bitrate = document.get_first(bitrate_field).unwrap().as_i64().unwrap();
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
            .unwrap();
        let filesize = document
            .get_first(filesize_field)
            .unwrap()
            .as_i64()
            .unwrap();
        let frequency = document
            .get_first(frequency_field)
            .unwrap()
            .as_i64()
            .unwrap();
        let length = document.get_first(length_field).unwrap().as_i64().unwrap();
        let track_number = document
            .get_first(track_number_field)
            .unwrap()
            .as_i64()
            .unwrap();
        let year = document.get_first(year_field).unwrap().as_i64().unwrap();
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
        let md5 = document
            .get_first(md5_field)
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
        let created_at = document
            .get_first(created_at_field)
            .unwrap()
            .as_str()
            .unwrap()
            .to_string();
        let updated_at = match document.get_first(updated_at_field) {
            Some(updated_at) => updated_at.as_str().unwrap().to_string(),
            None => "".to_string(),
        };

        Self {
            id,
            path,
            title,
            artist,
            album,
            album_artist,
            bitrate,
            composer,
            disc_number,
            filesize,
            frequency,
            length,
            track_number,
            year,
            year_string,
            genre,
            md5,
            album_art,
            artist_id,
            album_id,
            genre_id,
            created_at,
            updated_at,
        }
    }
}
