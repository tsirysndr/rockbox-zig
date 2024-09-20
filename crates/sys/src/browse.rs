use std::ffi::CString;

use crate::{
    types::tree::{Entry, TreeContext},
    AddToPlCallback, Mp3Entry, PlaylistInsertCb, Tm,
};

pub fn rockbox_browse_at(path: &str) -> i32 {
    let path = CString::new(path).unwrap();
    unsafe { crate::rockbox_browse_at(path.as_ptr()) }
}

pub fn rockbox_browse() -> i32 {
    unsafe { crate::rb_rockbox_browse() }
}

pub fn tree_get_context() -> TreeContext {
    let tc = unsafe { crate::rb_tree_get_context() };
    tc.into()
}

pub fn tree_get_entries() -> Entry {
    let entry = unsafe { crate::rb_tree_get_entries() };
    entry.into()
}

pub fn tree_get_entry_at(index: i32) -> Entry {
    let entry = unsafe { crate::rb_tree_get_entry_at(index) };
    entry.into()
}

pub fn set_current_file(path: &str) {
    let path = CString::new(path).unwrap();
    unsafe { crate::set_current_file(path.as_ptr()) }
}

pub fn set_dirfilter(filter: i32) {
    unsafe { crate::set_dirfilter(filter) }
}

pub fn onplay_show_playlist_menu(path: &str, attr: i32, playlist_insert_cb: PlaylistInsertCb) {
    let path = CString::new(path).unwrap();
    unsafe { crate::onplay_show_playlist_menu(path.as_ptr(), attr, playlist_insert_cb) }
}

pub fn onplay_show_playlist_cat_menu(track_name: &str, attr: i32, add_to_pl_cb: AddToPlCallback) {
    let track_name = CString::new(track_name).unwrap();
    unsafe { crate::onplay_show_playlist_cat_menu(track_name.as_ptr(), attr, add_to_pl_cb) }
}

pub fn browse_id3(
    id3: *mut Mp3Entry,
    playlist_display_index: i32,
    playlist_amount: i32,
    modified: *mut Tm,
    track_ct: i32,
) -> bool {
    let ret = unsafe {
        crate::browse_id3(
            id3,
            playlist_display_index,
            playlist_amount,
            modified,
            track_ct,
        )
    };
    ret != 0
}
