use async_graphql::*;
use serde::{Deserialize, Serialize};

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct Entry {
    pub name: String,
    pub attr: i32,
    pub time_write: u32,
    pub customaction: i32,
}

#[Object]
impl Entry {
    async fn name(&self) -> &str {
        &self.name
    }

    async fn attr(&self) -> i32 {
        self.attr
    }

    async fn time_write(&self) -> u32 {
        self.time_write
    }

    async fn customaction(&self) -> i32 {
        self.customaction
    }
}

impl From<rockbox_sys::types::tree::Entry> for Entry {
    fn from(entry: rockbox_sys::types::tree::Entry) -> Self {
        Self {
            name: entry.name,
            attr: entry.attr,
            time_write: entry.time_write,
            customaction: entry.customaction,
        }
    }
}
