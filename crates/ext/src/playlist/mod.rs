use std::path::PathBuf;

use deno_core::{extension, op2};

extension!(
    rb_playlist,
    ops = [
        op_get_current,
        op_get_resume_info,
        op_get_track_info,
        op_get_first_index,
        op_get_display_index,
        op_amount,
        op_playlist_resume,
        op_resume_track,
        op_set_modified,
        op_start,
        op_sync,
        op_remove_all_tracks,
        op_create_playlist,
        op_insert_track,
        op_insert_directory,
        op_insert_playlist,
        op_shuffle_playlist,
        op_warn_on_playlist_erase,
    ],
    esm = ["src/playlist/playlist.js"],
);

pub fn get_declaration() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/playlist/lib.rb_playlist.d.ts")
}

#[op2(async)]
pub async fn op_get_current() {}

#[op2(async)]
pub async fn op_get_resume_info() {}

#[op2(async)]
pub async fn op_get_track_info() {}

#[op2(async)]
pub async fn op_get_first_index() {}

#[op2(async)]
pub async fn op_get_display_index() {}

#[op2(async)]
pub async fn op_amount() {}

#[op2(async)]
pub async fn op_playlist_resume() {}

#[op2(async)]
pub async fn op_resume_track() {}

#[op2(async)]
pub async fn op_set_modified() {}

#[op2(async)]
pub async fn op_start() {}

#[op2(async)]
pub async fn op_sync() {}

#[op2(async)]
pub async fn op_remove_all_tracks() {}

#[op2(async)]
pub async fn op_create_playlist() {}

#[op2(async)]
pub async fn op_insert_track() {}

#[op2(async)]
pub async fn op_insert_directory() {}

#[op2(async)]
pub async fn op_insert_playlist() {}

#[op2(async)]
pub async fn op_shuffle_playlist() {}

#[op2(async)]
pub async fn op_warn_on_playlist_erase() {}
