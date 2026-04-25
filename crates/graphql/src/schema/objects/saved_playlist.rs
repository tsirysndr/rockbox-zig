use async_graphql::*;
use serde::Serialize;

#[derive(Default, Clone, Serialize)]
pub struct SavedPlaylistFolder {
    pub id: String,
    pub name: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[Object]
impl SavedPlaylistFolder {
    async fn id(&self) -> &str {
        &self.id
    }
    async fn name(&self) -> &str {
        &self.name
    }
    async fn created_at(&self) -> i64 {
        self.created_at
    }
    async fn updated_at(&self) -> i64 {
        self.updated_at
    }
}

#[derive(Default, Clone, Serialize)]
pub struct SavedPlaylist {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub image: Option<String>,
    pub folder_id: Option<String>,
    pub track_count: i64,
    pub created_at: i64,
    pub updated_at: i64,
}

#[Object]
impl SavedPlaylist {
    async fn id(&self) -> &str {
        &self.id
    }
    async fn name(&self) -> &str {
        &self.name
    }
    async fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }
    async fn image(&self) -> Option<&str> {
        self.image.as_deref()
    }
    async fn folder_id(&self) -> Option<&str> {
        self.folder_id.as_deref()
    }
    async fn track_count(&self) -> i64 {
        self.track_count
    }
    async fn created_at(&self) -> i64 {
        self.created_at
    }
    async fn updated_at(&self) -> i64 {
        self.updated_at
    }
}

impl From<rockbox_playlists::Playlist> for SavedPlaylist {
    fn from(p: rockbox_playlists::Playlist) -> Self {
        Self {
            id: p.id,
            name: p.name,
            description: p.description,
            image: p.image,
            folder_id: p.folder_id,
            track_count: p.track_count,
            created_at: p.created_at,
            updated_at: p.updated_at,
        }
    }
}

impl From<rockbox_playlists::PlaylistFolder> for SavedPlaylistFolder {
    fn from(f: rockbox_playlists::PlaylistFolder) -> Self {
        Self {
            id: f.id,
            name: f.name,
            created_at: f.created_at,
            updated_at: f.updated_at,
        }
    }
}
