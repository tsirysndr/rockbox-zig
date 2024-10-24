use super::{album::Album, track::Track};
use async_graphql::*;
use serde::{Deserialize, Serialize};
use tantivy::schema::Schema;
use tantivy::schema::SchemaBuilder;
use tantivy::schema::Value;
use tantivy::schema::*;
use tantivy::TantivyDocument;

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct Artist {
    pub id: String,
    pub name: String,
    pub bio: Option<String>,
    pub image: Option<String>,
    pub tracks: Vec<Track>,
    pub albums: Vec<Album>,
}

#[Object]
impl Artist {
    async fn id(&self) -> &str {
        &self.id
    }

    async fn name(&self) -> &str {
        &self.name
    }

    async fn bio(&self) -> Option<&str> {
        self.bio.as_deref()
    }

    async fn image(&self) -> Option<&str> {
        self.image.as_deref()
    }

    async fn tracks(&self) -> Vec<Track> {
        self.tracks.clone()
    }

    async fn albums(&self) -> Vec<Album> {
        self.albums.clone()
    }
}

impl From<rockbox_library::entity::artist::Artist> for Artist {
    fn from(artist: rockbox_library::entity::artist::Artist) -> Self {
        Self {
            id: artist.id,
            name: artist.name,
            bio: artist.bio,
            image: artist.image,
            tracks: vec![],
            albums: vec![],
        }
    }
}

impl From<rockbox_search::artist::Artist> for Artist {
    fn from(artist: rockbox_search::artist::Artist) -> Self {
        Self {
            id: artist.id,
            name: artist.name,
            bio: artist.bio,
            image: artist.image,
            tracks: vec![],
            albums: vec![],
        }
    }
}

impl From<TantivyDocument> for Artist {
    fn from(document: TantivyDocument) -> Self {
        let mut schema_builder: SchemaBuilder = Schema::builder();

        let id_field = schema_builder.add_text_field("id", STRING | STORED);
        let name_field = schema_builder.add_text_field("name", TEXT | STORED);
        let bio_field = schema_builder.add_text_field("bio", TEXT | STORED);
        let image_field = schema_builder.add_text_field("image", STRING | STORED);

        let id = document
            .get_first(id_field)
            .unwrap()
            .as_str()
            .unwrap()
            .to_string();
        let name = document
            .get_first(name_field)
            .unwrap()
            .as_str()
            .unwrap()
            .to_string();
        let bio = document
            .get_first(bio_field)
            .map(|value| value.as_str().unwrap().to_string());
        let image = document
            .get_first(image_field)
            .map(|value| value.as_str().unwrap().to_string());

        Self {
            id,
            name,
            bio,
            image,
            ..Default::default()
        }
    }
}
