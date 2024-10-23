use serde::{Deserialize, Serialize};

#[derive(sqlx::FromRow, Default, Serialize, Deserialize, Clone)]
pub struct Artist {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bio: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,
}

impl Into<rockbox_search::rockbox::search::v1alpha1::Artist> for Artist {
    fn into(self) -> rockbox_search::rockbox::search::v1alpha1::Artist {
        rockbox_search::rockbox::search::v1alpha1::Artist {
            id: self.id,
            name: self.name,
            bio: self.bio,
            image: self.image,
        }
    }
}
