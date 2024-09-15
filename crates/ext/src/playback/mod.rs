use std::path::PathBuf;

use deno_core::{extension, op2};

extension!(
    rb_playback,
    ops = [
        op_play,
        op_pause,
        op_resume,
        op_next,
        op_previous,
        op_fast_forward_rewind,
        op_status,
        op_current_track,
        op_flush_and_reload,
        op_get_file_position,
        op_hard_stop,
    ],
    esm = ["src/playback/playback.js"],
);

pub fn get_declaration() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/playback/lib.rb_playback.d.ts")
}

#[op2(async)]
pub async fn op_play() {}

#[op2(async)]
pub async fn op_pause() {}

#[op2(async)]
pub async fn op_resume() {}

#[op2(async)]
pub async fn op_next() {}

#[op2(async)]
pub async fn op_previous() {}

#[op2(async)]
pub async fn op_fast_forward_rewind() {}

#[op2(async)]
pub async fn op_status() {}

#[op2(async)]
pub async fn op_current_track() {}

#[op2(async)]
pub async fn op_flush_and_reload() {}

#[op2(async)]
pub async fn op_get_file_position() {}

#[op2(async)]
pub async fn op_hard_stop() {}
