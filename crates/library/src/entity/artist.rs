use serde::{Deserialize, Serialize};

#[derive(sqlx::FromRow, Default, Serialize, Deserialize)]
pub struct Artist {
    pub id: String,
    pub name: String,
    pub bio: Option<String>,
    pub image: Option<String>,
}
