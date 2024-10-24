use crate::{Indexable, Searchable};
use rockbox_library::entity;
use serde::{Deserialize, Serialize};
use tantivy::{doc, schema::*, TantivyDocument};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Album {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub year: i64,
    pub year_string: String,
    pub album_art: Option<String>,
    pub md5: String,
    pub artist_id: String,
}

impl Indexable for Album {
    fn to_document(&self) -> TantivyDocument {
        let schema: Schema = self.build_schema();

        let id = schema.get_field("id").unwrap();
        let title = schema.get_field("title").unwrap();
        let artist = schema.get_field("artist").unwrap();
        let year = schema.get_field("year").unwrap();
        let year_string = schema.get_field("year_string").unwrap();
        let album_art = schema.get_field("album_art").unwrap();
        let md5 = schema.get_field("md5").unwrap();
        let artist_id = schema.get_field("artist_id").unwrap();

        let mut document = doc!(
            id => self.id.to_owned(),
            title => self.title.to_owned(),
            artist => self.artist.to_owned(),
            year => self.year,
            year_string => self.year_string.to_owned(),
        );

        if let Some(value) = &self.album_art {
            document.add_text(album_art, value);
        }

        document.add_text(md5, &self.md5);
        document.add_text(artist_id, &self.artist_id);

        document
    }

    fn build_schema(&self) -> Schema {
        let mut schema_builder: SchemaBuilder = Schema::builder();

        schema_builder.add_text_field("id", STRING | STORED);
        schema_builder.add_text_field("title", TEXT | STORED);
        schema_builder.add_text_field("artist", TEXT | STORED);
        schema_builder.add_i64_field("year", STORED);
        schema_builder.add_text_field("year_string", STRING | STORED);
        schema_builder.add_text_field("album_art", STRING | STORED);
        schema_builder.add_text_field("md5", STRING | STORED);
        schema_builder.add_text_field("artist_id", STRING | STORED);

        schema_builder.build()
    }
}

impl Searchable for Album {
    fn schema(&self) -> Schema {
        self.build_schema()
    }

    fn default_fields(&self) -> Vec<String> {
        vec!["title".to_string(), "artist".to_string()]
    }
}

impl From<entity::album::Album> for Album {
    fn from(album: entity::album::Album) -> Self {
        Self {
            id: album.id,
            title: album.title,
            artist: album.artist,
            year: album.year as i64,
            year_string: album.year_string,
            album_art: album.album_art,
            md5: album.md5,
            artist_id: album.artist_id,
        }
    }
}

impl From<TantivyDocument> for Album {
    fn from(document: TantivyDocument) -> Self {
        let mut schema_builder: SchemaBuilder = Schema::builder();

        let id_field = schema_builder.add_text_field("id", STRING | STORED);
        let title_field = schema_builder.add_text_field("title", TEXT | STORED);
        let artist_field = schema_builder.add_text_field("artist", TEXT | STORED);
        let year_field = schema_builder.add_i64_field("year", STORED);
        let year_string_field = schema_builder.add_text_field("year_string", STRING | STORED);
        let album_art_field = schema_builder.add_text_field("album_art", STRING | STORED);
        let md5_field = schema_builder.add_text_field("md5", STRING | STORED);
        let artist_id_field = schema_builder.add_text_field("artist_id", STRING | STORED);

        let id = document
            .get_first(id_field)
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
        let year = document.get_first(year_field).unwrap().as_i64().unwrap();
        let year_string = document
            .get_first(year_string_field)
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
        let md5 = document
            .get_first(md5_field)
            .unwrap()
            .as_str()
            .unwrap()
            .to_string();
        let artist_id = match document.get_first(artist_id_field) {
            Some(artist_id) => artist_id.as_str().unwrap().to_string(),
            None => "".to_string(),
        };

        Self {
            id,
            title,
            artist,
            year,
            year_string,
            album_art,
            md5,
            artist_id,
        }
    }
}
