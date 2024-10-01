use std::path::PathBuf;

use deno_core::{
    error::AnyError,
    extension, op2,
    serde::{Deserialize, Serialize},
};
use rockbox_sys::types::{playlist_amount::PlaylistAmount, playlist_info::PlaylistInfo};

use crate::rockbox_url;

#[derive(Serialize, Deserialize)]
pub struct NewPlaylist {
    pub name: String,
    pub tracks: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct InsertTracks {
    pub position: i32,
    pub tracks: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct RemoveTracks {
    pub positions: Vec<i32>,
}

#[derive(Serialize, Deserialize)]
pub struct InsertDirectory {
    pub position: i32,
    pub directory: String,
}

extension!(
    rb_playlist,
    ops = [
        op_playlist_get_current,
        op_playlist_get_resume_info,
        op_playlist_get_track_info,
        op_playlist_get_first_index,
        op_playlist_get_display_index,
        op_playlist_amount,
        op_playlist_resume,
        op_playlist_resume_track,
        op_playlist_set_modified,
        op_playlist_start,
        op_playlist_sync,
        op_playlist_remove_all_tracks,
        op_playlist_remove_tracks,
        op_create_playlist,
        op_playlist_insert_tracks,
        op_playlist_insert_directory,
        op_insert_playlist,
        op_shuffle_playlist,
    ],
    esm = ["src/playlist/playlist.js"],
);

pub fn get_declaration() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/playlist/lib.rb_playlist.d.ts")
}

#[op2(async)]
#[serde]
pub async fn op_playlist_get_current() -> Result<PlaylistInfo, AnyError> {
    let client = reqwest::Client::new();
    let url = format!("{}/playlists/current", rockbox_url());
    let res = client.get(&url).send().await?;
    let info = res.json::<PlaylistInfo>().await?;
    Ok(info)
}

#[op2(async)]
pub async fn op_playlist_get_resume_info() {}

#[op2(async)]
pub async fn op_playlist_get_track_info() {}

#[op2(async)]
pub async fn op_playlist_get_first_index() {}

#[op2(async)]
pub async fn op_playlist_get_display_index() {}

#[op2(async)]
pub async fn op_playlist_amount() -> Result<i32, AnyError> {
    let client = reqwest::Client::new();
    let url = format!("{}/playlists/amount", rockbox_url());
    let res = client.get(&url).send().await?;
    let data = res.json::<PlaylistAmount>().await?;
    Ok(data.amount)
}

#[op2(async)]
pub async fn op_playlist_resume() -> Result<(), AnyError> {
    let client = reqwest::Client::new();
    let url = format!("{}/playlists/resume", rockbox_url());
    client.put(&url).send().await?;
    Ok(())
}

#[op2(async)]
pub async fn op_playlist_resume_track() -> Result<(), AnyError> {
    let client = reqwest::Client::new();
    let url = format!("{}/playlists/resume-track", rockbox_url());
    client.put(&url).send().await?;
    Ok(())
}

#[op2(async)]
pub async fn op_playlist_set_modified() {}

#[op2(async)]
pub async fn op_playlist_start() -> Result<(), AnyError> {
    let client = reqwest::Client::new();
    let url = format!("{}/playlists/start", rockbox_url());
    client.put(&url).send().await?;
    Ok(())
}

#[op2(async)]
pub async fn op_playlist_sync() {}

#[op2(async)]
pub async fn op_playlist_remove_all_tracks() -> Result<i32, AnyError> {
    let client = reqwest::Client::new();
    let url = format!("{}/playlists/current/tracks", rockbox_url());
    let body = serde_json::json!({ "positions": [] });
    let response = client.delete(&url).json(&body).send().await?;
    let start_index = response.text().await?.parse()?;
    Ok(start_index)
}

#[op2(async)]
pub async fn op_playlist_remove_tracks(#[serde] params: RemoveTracks) -> Result<i32, AnyError> {
    let client = reqwest::Client::new();
    let url = format!("{}/playlists/current/tracks", rockbox_url());
    let body = serde_json::json!({ "positions": params.positions });
    let response = client.delete(&url).json(&body).send().await?;
    let start_index = response.text().await?.parse()?;
    Ok(start_index)
}

#[op2(async)]
pub async fn op_create_playlist(#[serde] params: NewPlaylist) -> Result<i32, AnyError> {
    let client = reqwest::Client::new();
    let url = format!("{}/playlists", rockbox_url());
    let response = client.post(&url).json(&params).send().await?;
    let start_index = response.text().await?.parse()?;
    Ok(start_index)
}

#[op2(async)]
pub async fn op_playlist_insert_tracks(#[serde] params: InsertTracks) -> Result<i32, AnyError> {
    let client = reqwest::Client::new();
    let url = format!("{}/playlists/current/tracks", rockbox_url());
    let response = client.post(&url).json(&params).send().await?;
    let start_index = response.text().await?.parse()?;
    Ok(start_index)
}

#[op2(async)]
pub async fn op_playlist_insert_directory(
    #[serde] params: InsertDirectory,
) -> Result<i32, AnyError> {
    let client = reqwest::Client::new();
    let url = format!("{}/playlists/current/tracks", rockbox_url());
    let response = client.post(&url).json(&params).send().await?;
    let start_index = response.text().await?.parse()?;
    Ok(start_index)
}

#[op2(async)]
pub async fn op_insert_playlist() {}

#[op2(async)]
pub async fn op_shuffle_playlist(start_index: i32) -> Result<(), AnyError> {
    let client = reqwest::Client::new();
    let url = format!(
        "{}/playlists/shuffle?start_index={}",
        rockbox_url(),
        start_index
    );
    client.put(&url).send().await?;
    Ok(())
}
