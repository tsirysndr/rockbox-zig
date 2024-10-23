use crate::{Indexable, Searchable};
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
        let album_art = schema.get_field("album_art").unwrap();
        let md5 = schema.get_field("md5").unwrap();
        let artist_id = schema.get_field("artist_id").unwrap();

        let mut document = doc!(
            id => self.id.to_owned(),
            title => self.title.to_owned(),
            artist => self.artist.to_owned(),
            year => self.year,
            md5 => self.md5.to_owned(),
            artist_id => self.artist_id.to_owned(),
        );

        if let Some(value) = &self.album_art {
            document.add_text(album_art, value);
        }

        document
    }

    fn build_schema(&self) -> Schema {
        let mut schema_builder: SchemaBuilder = Schema::builder();

        schema_builder.add_text_field("id", STRING | STORED);
        schema_builder.add_text_field("title", TEXT | STORED);
        schema_builder.add_text_field("artist", TEXT | STORED);
        schema_builder.add_i64_field("year", STORED);
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
