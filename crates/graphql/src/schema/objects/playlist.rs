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
}
