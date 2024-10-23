use crate::{Indexable, Searchable};
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
            genre => self.genre.to_owned(),
            md5 => self.md5.to_owned(),
            created_at => self.created_at.to_owned(),
            updated_at => self.updated_at.to_owned(),
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
            "composer".to_string(),
            "album_artist".to_string(),
        ]
    }
}
