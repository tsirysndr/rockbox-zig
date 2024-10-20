use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(sqlx::FromRow, Default, Debug, Clone, Serialize, Deserialize)]
pub struct Favourites {
    pub id: String,
    pub track_id: Option<String>,
    pub album_id: Option<String>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
}
