use async_graphql::*;
use serde::Serialize;

#[derive(Default, Clone, Serialize)]
pub struct SystemStatus {
    pub resume_index: i32,
    pub resume_crc32: u32,
    pub resume_elapsed: u32,
    pub resume_offset: u32,
    pub runtime: i32,
    pub topruntime: i32,
    pub dircache_size: i32,
    pub last_screen: i8,
    pub viewer_icon_count: i32,
    pub last_volume_change: i32,
}

#[Object]
impl SystemStatus {
    async fn resume_index(&self) -> i32 {
        self.resume_index
    }

    async fn resume_crc32(&self) -> u32 {
        self.resume_crc32
    }

    async fn resume_elapsed(&self) -> u32 {
        self.resume_elapsed
    }

    async fn resume_offset(&self) -> u32 {
        self.resume_offset
    }

    async fn runtime(&self) -> i32 {
        self.runtime
    }

    async fn topruntime(&self) -> i32 {
        self.topruntime
    }

    async fn dircache_size(&self) -> i32 {
        self.dircache_size
    }

    async fn last_screen(&self) -> i8 {
        self.last_screen
    }

    async fn viewer_icon_count(&self) -> i32 {
        self.viewer_icon_count
    }

    async fn last_volume_change(&self) -> i32 {
        self.last_volume_change
    }
}
