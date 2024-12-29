use async_graphql::*;
use serde::{Deserialize, Serialize};

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct Folder {
    pub id: String,
    pub name: String,
    pub parent_id: Option<String>,
    pub created_at: u64,
    pub updated_at: u64,
}

#[Object]
impl Folder {
    async fn id(&self) -> &str {
        &self.id
    }

    async fn name(&self) -> &str {
        &self.name
    }

    async fn parent_id(&self) -> Option<String> {
        self.parent_id.clone()
    }

    async fn created_at(&self) -> u64 {
        self.created_at
    }

    async fn updated_at(&self) -> u64 {
        self.updated_at
    }
}
