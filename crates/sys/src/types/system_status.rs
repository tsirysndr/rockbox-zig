use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
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

impl From<crate::SystemStatus> for SystemStatus {
    fn from(status: crate::SystemStatus) -> Self {
        Self {
            resume_index: status.resume_index,
            resume_crc32: status.resume_crc32,
            resume_elapsed: status.resume_elapsed,
            resume_offset: status.resume_offset,
            runtime: status.runtime,
            topruntime: status.topruntime,
            dircache_size: status.dircache_size,
            last_screen: status.last_screen,
            viewer_icon_count: status.viewer_icon_count,
            last_volume_change: status.last_volume_change,
        }
    }
}
