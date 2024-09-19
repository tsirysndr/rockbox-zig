use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct PlaylistAmount {
    pub amount: i32,
}
