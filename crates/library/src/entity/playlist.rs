use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(sqlx::FromRow, Default, Serialize, Deserialize)]
pub struct Playlist {
    pub id: String,
    pub name: String,
    pub image: Option<String>,
    pub description: Option<String>,
    pub folder_id: Option<String>,
    #[serde(with = "chrono::serde::ts_seconds")] 
    pub created_at: DateTime<Utc>,
    #[serde(with = "chrono::serde::ts_seconds")] 
    pub updated_at: DateTime<Utc>,
}
