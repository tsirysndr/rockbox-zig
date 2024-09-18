use std::path::PathBuf;

use deno_ast::swc::codegen::Result;
use deno_core::{error::AnyError, extension, op2};
use rockbox_sys::types::{
    audio_status::AudioStatus, file_position::FilePosition, mp3_entry::Mp3Entry,
};

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
pub async fn op_play(elapsed: i32, offset: i32) -> Result<(), AnyError> {
    let client = reqwest::Client::new();
    let url = format!(
        "{}/play?elapsed={}&offset={}",
        rockbox_url(),
        elapsed,
        offset
    );
    client.get(&url).send().await?;
    Ok(())
}

#[op2(async)]
pub async fn op_pause() -> Result<(), AnyError> {
    let client = reqwest::Client::new();
    let url = format!("{}/pause", rockbox_url());
    client.get(&url).send().await?;
    Ok(())
}

#[op2(async)]
pub async fn op_resume() -> Result<(), AnyError> {
    let client = reqwest::Client::new();
    let url = format!("{}/resume", rockbox_url());
    client.get(&url).send().await?;
    Ok(())
}

#[op2(async)]
pub async fn op_next() -> Result<(), AnyError> {
    let client = reqwest::Client::new();
    let url = format!("{}/next", rockbox_url());
    client.get(&url).send().await?;
    Ok(())
}

#[op2(async)]
pub async fn op_previous() -> Result<(), AnyError> {
    let client = reqwest::Client::new();
    let url = format!("{}/prev", rockbox_url());
    client.get(&url).send().await?;
    Ok(())
}

#[op2(async)]
pub async fn op_fast_forward_rewind() -> Result<(), AnyError> {
    let client = reqwest::Client::new();
    let url = format!("{}/ff_rewind", rockbox_url());
    client.get(&url).send().await?;
    Ok(())
}

#[op2(async)]
pub async fn op_status() -> Result<i32, AnyError> {
    let client = reqwest::Client::new();
    let url = format!("{}/audio_status", rockbox_url());
    let response = client.get(&url).send().await?;
    let response = response.json::<AudioStatus>().await?;
    Ok(response.status)
}

#[op2(async)]
#[serde]
pub async fn op_current_track() -> Result<Option<Mp3Entry>, AnyError> {
    let client = reqwest::Client::new();
    let url = format!("{}/current_track", rockbox_url());
    let response = client.get(&url).send().await?;
    let track = response.json::<Option<Mp3Entry>>().await?;
    Ok(track)
}

#[op2(async)]
#[serde]
pub async fn op_next_track() -> Result<Option<Mp3Entry>, AnyError> {
    let client = reqwest::Client::new();
    let url = format!("{}/next_track", rockbox_url());
    let response = client.get(&url).send().await?;
    let track = response.json::<Option<Mp3Entry>>().await?;
    Ok(track)
}

#[op2(async)]
pub async fn op_flush_and_reload_tracks() -> Result<(), AnyError> {
    let client = reqwest::Client::new();
    let url = format!("{}/flush_and_reload_tracks", rockbox_url());
    client.get(&url).send().await?;
}

#[op2(async)]
pub async fn op_get_file_position() -> Result<i32, AnyError> {
    let client = reqwest::Client::new();
    let url = format!("{}/file_position", rockbox_url());
    let response = client.get(&url).send().await?;
    let response = response.json::<FilePosition>().await?;
    Ok(response.position)
}

#[op2(async)]
pub async fn op_hard_stop() -> Result<(), AnyError> {
    let client = reqwest::Client::new();
    let url = format!("{}/stop", rockbox_url());
    client.get(&url).send().await?;
    Ok(())
}
