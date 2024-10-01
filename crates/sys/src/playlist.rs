use crate::types::{playlist_info::PlaylistInfo, playlist_track_info::PlaylistTrackInfo};
use std::{
    ffi::{c_int, CString},
    ptr,
};

pub fn get_current() -> PlaylistInfo {
    let playlist = unsafe { crate::playlist_get_current() };
    playlist.into()
}

pub fn get_resume_info(mut resume_index: i32) -> i32 {
    unsafe { crate::playlist_get_resume_info(&mut resume_index as *mut i32 as *mut c_int) }
}

pub fn get_track_info(index: i32) -> PlaylistTrackInfo {
    let track_info = unsafe { crate::rb_get_track_info_from_current_playlist(index) };
    track_info.into()
}

pub fn get_first_index(info: *mut crate::PlaylistInfo) -> i32 {
    unsafe { crate::playlist_get_first_index(info) }
}

pub fn get_display_index() -> i32 {
    unsafe { crate::playlist_get_display_index() }
}

pub fn amount() -> i32 {
    unsafe { crate::playlist_amount() }
}

pub fn resume() -> i32 {
    unsafe { crate::playlist_resume() }
}

pub fn resume_track(start_index: i32, crc: u32, elapsed: u64, offset: u64) {
    unsafe { crate::playlist_resume_track(start_index, crc, elapsed, offset) }
}

pub fn set_modified(playlist: *mut crate::PlaylistInfo, modified: bool) {
    unsafe { crate::playlist_set_modified(playlist, modified as u8) }
}

pub fn start(start_index: i32, elapsed: u64, offset: u64) {
    unsafe { crate::playlist_start(start_index, elapsed, offset) }
}

pub fn sync(playlist: *mut crate::PlaylistInfo) {
    unsafe { crate::playlist_sync(playlist) }
}

pub fn remove_all_tracks() -> i32 {
    unsafe { crate::rb_playlist_remove_all_tracks() }
}

pub fn create(dir: &str, file: Option<&str>) -> i32 {
    let dir = CString::new(dir).unwrap();
    let file = file.map(|file| CString::new(file).unwrap());
    unsafe {
        crate::playlist_create(
            dir.as_ptr(),
            match file {
                Some(file) => file.as_ptr(),
                None => ptr::null(),
            },
        )
    }
}

pub fn insert_directory(dir: &str, position: i32, queue: bool, recurse: bool) -> i32 {
    let dir = CString::new(dir).unwrap();
    unsafe { crate::rb_playlist_insert_directory(dir.as_ptr(), position, queue, recurse) }
}

pub fn shuffle(random_seed: i32, start_index: i32) -> i32 {
    unsafe { crate::playlist_shuffle(random_seed, start_index) }
}

pub fn warn_on_pl_erase() -> bool {
    let ret = unsafe { crate::warn_on_pl_erase() };
    ret != 0
}

pub fn build_playlist(files: Vec<&str>, start_index: i32, size: i32) -> i32 {
    let mut c_strings: Vec<CString> = Vec::with_capacity(files.len());
    let mut pointers: Vec<*const u8> = Vec::with_capacity(files.len());

    for file in files {
        let c_string = CString::new(file).expect("CString::new failed");
        pointers.push(c_string.as_ptr() as *const u8);
        c_strings.push(c_string);
    }

    // Create a raw pointer to the vector of pointers
    let files = pointers.as_ptr();

    unsafe { crate::rb_build_playlist(files, start_index, size) }
}

pub fn insert_track(filename: &str, position: i32, queue: bool, sync: bool) -> i32 {
    let filename = CString::new(filename).unwrap();
    unsafe {
        crate::rb_playlist_insert_track(filename.as_ptr() as *const u8, position, queue, sync)
    }
}

pub fn delete_track(index: i32) -> i32 {
    unsafe { crate::rb_playlist_delete_track(index) }
}
