use crate::schema::objects::track::Track;
use async_graphql::*;
use serde::Serialize;

#[derive(Default, Clone, Serialize)]
pub struct Playlist {
    pub amount: i32,
    pub index: i32,
    pub max_playlist_size: i32,
    pub first_index: i32,
    pub last_insert_pos: i32,
    pub seed: i32,
    pub last_shuffled_start: i32,
    pub tracks: Vec<Track>,
    pub folder_id: Option<String>,
    pub name: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub description: Option<String>,
    pub image: Option<String>,
    pub id: Option<String>,
}

#[Object]
impl Playlist {
    async fn amount(&self) -> i32 {
        self.amount
    }

    async fn index(&self) -> i32 {
        self.index
    }

    async fn max_playlist_size(&self) -> i32 {
        self.max_playlist_size
    }

    async fn first_index(&self) -> i32 {
        self.first_index
    }

    async fn last_insert_pos(&self) -> i32 {
        self.last_insert_pos
    }

    async fn seed(&self) -> i32 {
        self.seed
    }

    async fn last_shuffled_start(&self) -> i32 {
        self.last_shuffled_start
    }

    async fn tracks(&self) -> &Vec<Track> {
        &self.tracks
    }

    async fn folder_id(&self) -> Option<String> {
        self.folder_id.clone()
    }

    async fn name(&self) -> Option<String> {
        self.name.clone()
    }

    async fn created_at(&self) -> Option<String> {
        self.created_at.clone()
    }

    async fn updated_at(&self) -> Option<String> {
        self.updated_at.clone()
    }

    async fn description(&self) -> Option<String> {
        self.description.clone()
    }

    async fn image(&self) -> Option<String> {
        self.image.clone()
    }

    async fn id(&self) -> Option<String> {
        self.id.clone()
    }
}
