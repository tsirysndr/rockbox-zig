use serde::{Deserialize, Serialize};

#[derive(sqlx::FromRow, Default, Serialize, Deserialize)]
pub struct Album {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub year: u32,
    pub year_string: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub album_art: Option<String>,
    pub md5: String,
    pub artist_id: String,
}
