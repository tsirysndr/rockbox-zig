use async_graphql::*;
use crate::schema::objects::track::Track;
use serde::Serialize;

#[derive(Default, Clone, Serialize)]
pub struct Playlist {
    pub tracks: Vec<Track>,
}

#[Object]
impl Playlist {
    async fn tracks(&self) -> &Vec<Track> {
        &self.tracks
    }
}