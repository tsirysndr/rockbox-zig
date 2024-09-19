use crate::types::mp3_entry::Mp3Entry;
use serde::{Deserialize, Serialize};

use crate::cast_ptr;

#[derive(Serialize, Deserialize, Default)]
pub struct PlaylistInfo {
    pub utf8: bool,               // bool utf8
    pub control_created: bool,    // bool control_created
    pub flags: u32,               // unsigned int flags
    pub fd: i32,                  // int fd
    pub control_fd: i32,          // int control_fd
    pub max_playlist_size: i32,   // int max_playlist_size
    pub indices: Vec<u64>,        // unsigned long* indices
    pub index: i32,               // int index
    pub first_index: i32,         // int first_index
    pub amount: i32,              // int amount
    pub last_insert_pos: i32,     // int last_insert_pos
    pub started: bool,            // bool started
    pub last_shuffled_start: i32, // int last_shuffled_start
    pub seed: i32,                // int seed
    pub dirlen: i32,              // int dirlen
    pub filename: String,         // char filename[MAX_PATH]
    pub control_filename: String, // char control_filename[sizeof(PLAYLIST_CONTROL_FILE) + 8]
    pub dcfrefs_handle: i32,      // int dcfrefs_handle
    pub entries: Vec<Mp3Entry>,
}

impl From<crate::PlaylistInfo> for PlaylistInfo {
    fn from(info: crate::PlaylistInfo) -> Self {
        Self {
            utf8: info.utf8,
            control_created: info.control_created,
            flags: info.flags,
            fd: info.fd,
            control_fd: info.control_fd,
            max_playlist_size: info.max_playlist_size,
            indices: vec![],
            index: info.index,
            first_index: info.first_index,
            amount: info.amount,
            last_insert_pos: info.last_insert_pos,
            started: info.started,
            last_shuffled_start: info.last_shuffled_start,
            seed: info.seed,
            dirlen: info.dirlen,
            filename: unsafe {
                std::ffi::CStr::from_ptr(cast_ptr!(info.filename.as_ptr()))
                    .to_string_lossy()
                    .into_owned()
            },
            control_filename: unsafe {
                std::ffi::CStr::from_ptr(cast_ptr!(info.control_filename.as_ptr()))
                    .to_string_lossy()
                    .into_owned()
            },
            dcfrefs_handle: info.dcfrefs_handle,
            entries: vec![],
        }
    }
}
