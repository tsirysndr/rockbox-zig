use std::path::PathBuf;

use deno_core::{extension, op2};

use crate::rockbox_url;

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
        op_flush_and_reload_tracks,
        op_get_file_position,
        op_hard_stop,
    ],
    esm = ["src/playback/playback.js"],
);

pub fn get_declaration() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/playback/lib.rb_playback.d.ts")
}

#[op2(async)]
pub async fn op_play(elapsed: i32, offset: i32) {
    let client = reqwest::Client::new();
    let url = format!(
        "{}/play?elapsed={}&offset={}",
        rockbox_url(),
        elapsed,
        offset
    );
    client.get(&url).send().await.unwrap();
}

#[op2(async)]
pub async fn op_pause() {
    let client = reqwest::Client::new();
    let url = format!("{}/pause", rockbox_url());
    client.get(&url).send().await.unwrap();
}

#[op2(async)]
pub async fn op_resume() {
    let client = reqwest::Client::new();
    let url = format!("{}/resume", rockbox_url());
    client.get(&url).send().await.unwrap();
}

#[op2(async)]
pub async fn op_next() {
    let client = reqwest::Client::new();
    let url = format!("{}/next", rockbox_url());
    client.get(&url).send().await.unwrap();
}

#[op2(async)]
pub async fn op_previous() {
    let client = reqwest::Client::new();
    let url = format!("{}/prev", rockbox_url());
    client.get(&url).send().await.unwrap();
}

#[op2(async)]
pub async fn op_fast_forward_rewind() {
    let client = reqwest::Client::new();
    let url = format!("{}/ff_rewind", rockbox_url());
    client.get(&url).send().await.unwrap();
}

#[op2(async)]
pub async fn op_status() {}

#[op2(async)]
pub async fn op_current_track() {}

#[op2(async)]
pub async fn op_flush_and_reload_tracks() {
    let client = reqwest::Client::new();
    let url = format!("{}/flush_and_reload_tracks", rockbox_url());
    client.get(&url).send().await.unwrap();
}

#[op2(async)]
pub async fn op_get_file_position() {}

#[op2(async)]
pub async fn op_hard_stop() {
    let client = reqwest::Client::new();
    let url = format!("{}/stop", rockbox_url());
    client.get(&url).send().await.unwrap();
}
