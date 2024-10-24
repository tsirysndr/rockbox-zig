use async_graphql::*;
use serde::{Deserialize, Serialize};
use tantivy::schema::Schema;
use tantivy::schema::SchemaBuilder;
use tantivy::schema::Value;
use tantivy::schema::*;
use tantivy::TantivyDocument;

use super::track::Track;

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct Album {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub year: u32,
    pub year_string: String,
    pub album_art: Option<String>,
    pub md5: String,
    pub artist_id: String,
    pub tracks: Vec<Track>,
}

#[Object]
impl Album {
    async fn id(&self) -> &str {
        &self.id
    }

    async fn title(&self) -> &str {
        &self.title
    }

    async fn artist(&self) -> &str {
        &self.artist
    }

    async fn year(&self) -> i32 {
        self.year as i32
    }

    async fn year_string(&self) -> &str {
        &self.year_string
    }

    async fn album_art(&self) -> Option<&str> {
        self.album_art.as_deref()
    }

    async fn md5(&self) -> &str {
        &self.md5
    }

    async fn artist_id(&self) -> &str {
        &self.artist_id
    }

    async fn tracks(&self) -> Vec<Track> {
        self.tracks.clone()
    }
}

impl From<rockbox_library::entity::album::Album> for Album {
    fn from(album: rockbox_library::entity::album::Album) -> Self {
        Self {
            id: album.id,
            title: album.title,
            artist: album.artist,
            year: album.year,
            year_string: album.year_string,
            album_art: album.album_art,
            md5: album.md5,
            artist_id: album.artist_id,
            tracks: vec![],
        }
    }
}

impl From<rockbox_search::album::Album> for Album {
    fn from(album: rockbox_search::album::Album) -> Self {
        Self {
            id: album.id,
            title: album.title,
            artist: album.artist,
            year: album.year as u32,
            year_string: album.year_string,
            album_art: album.album_art,
            md5: album.md5,
            artist_id: album.artist_id,
            tracks: vec![],
        }
    }
}

impl From<rockbox_search::liked_album::LikedAlbum> for Album {
    fn from(album: rockbox_search::liked_album::LikedAlbum) -> Self {
        Self {
            id: album.id,
            title: album.title,
            artist: album.artist,
            year: album.year as u32,
            year_string: album.year_string,
            album_art: album.album_art,
            md5: album.md5,
            artist_id: album.artist_id,
            tracks: vec![],
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
        let year = document.get_first(year_field).unwrap().as_i64().unwrap() as u32;
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
            ..Default::default()
        }
    }
}
