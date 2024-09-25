use async_graphql::*;
use serde::{Deserialize, Serialize};

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct Artist {
    pub id: String,
    pub name: String,
    pub bio: Option<String>,
    pub image: Option<String>,
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
}

impl From<rockbox_library::entity::artist::Artist> for Artist {
    fn from(artist: rockbox_library::entity::artist::Artist) -> Self {
        Self {
            id: artist.id,
            name: artist.name,
            bio: artist.bio,
            image: artist.image,
        }
    }
}
