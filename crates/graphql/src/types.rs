use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct StatusCode {
    pub code: i32,
}

#[derive(Clone, Debug)]
pub struct ScanCompleted;
