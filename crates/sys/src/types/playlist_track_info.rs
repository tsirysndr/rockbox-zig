use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct PlaylistTrackInfo {
    pub filename: String,
    pub attr: i32,
    pub index: i32,
    pub display_index: i32,
}

impl From<crate::PlaylistTrackInfo> for PlaylistTrackInfo {
    fn from(info: crate::PlaylistTrackInfo) -> Self {
        Self {
            filename: unsafe {
                std::ffi::CStr::from_ptr(crate::cast_ptr!(info.filename.as_ptr()))
                    .to_string_lossy()
                    .into_owned()
            },
            attr: info.attr,
            index: info.index,
            display_index: info.display_index,
        }
    }
}
