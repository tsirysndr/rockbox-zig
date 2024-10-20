use std::{
    fs,
    sync::{mpsc::Sender, Arc, Mutex},
};

use crate::{read_files, schema::objects, AUDIO_EXTENSIONS};
use async_graphql::*;
use futures_util::Stream;
use rockbox_library::repo;
use rockbox_sys::{
    events::RockboxCommand,
    types::{audio_status::AudioStatus, file_position::FilePosition, mp3_entry::Mp3Entry},
};
use sqlx::{Pool, Sqlite};

use crate::{rockbox_url, schema::objects::track::Track, simplebroker::SimpleBroker};

#[derive(Default)]
pub struct PlaybackQuery;

#[Object]
impl PlaybackQuery {
    async fn status(&self) -> Result<i32, Error> {
        let client = reqwest::Client::new();
        let url = format!("{}/player/status", rockbox_url());
        let response = client.get(&url).send().await?;
        let response = response.json::<AudioStatus>().await?;
        Ok(response.status)
    }

    async fn current_track(&self, ctx: &Context<'_>) -> Result<Option<Track>, Error> {
        let pool = ctx.data::<Pool<Sqlite>>()?;
        let client = ctx.data::<reqwest::Client>().unwrap();
        let url = format!("{}/player/current-track", rockbox_url());
        let response = client.get(&url).send().await?;
        let track = response.json::<Option<Mp3Entry>>().await?;
        let mut track: Option<Track> = track.map(|t| t.into());
        let path: Option<String> = track.as_ref().map(|t| t.path.clone());
        if let Some(path) = path {
            let hash = format!("{:x}", md5::compute(path.as_bytes()));
            if let Some(metadata) = repo::track::find_by_md5(pool.clone(), &hash).await? {
                track.as_mut().unwrap().id = Some(metadata.id);
                track.as_mut().unwrap().album_art = metadata.album_art;
                track.as_mut().unwrap().album_id = Some(metadata.album_id);
                track.as_mut().unwrap().artist_id = Some(metadata.artist_id);
            }
        }
        Ok(track)
    }

    async fn next_track(&self, ctx: &Context<'_>) -> Result<Option<Track>, Error> {
        let pool = ctx.data::<Pool<Sqlite>>()?;
        let client = ctx.data::<reqwest::Client>().unwrap();
        let url = format!("{}/player/next-track", rockbox_url());
        let response = client.get(&url).send().await?;
        let track = response.json::<Option<Mp3Entry>>().await?;
        let mut track: Option<Track> = track.map(|t| t.into());
        let path: Option<String> = track.as_ref().map(|t| t.path.clone());
        if let Some(path) = path {
            let hash = format!("{:x}", md5::compute(path.as_bytes()));
            if let Some(metadata) = repo::track::find_by_md5(pool.clone(), &hash).await? {
                track.as_mut().unwrap().id = Some(metadata.id);
                track.as_mut().unwrap().album_art = metadata.album_art;
                track.as_mut().unwrap().album_id = Some(metadata.album_id);
                track.as_mut().unwrap().artist_id = Some(metadata.artist_id);
            }
        }
        Ok(track)
    }

    async fn get_file_position(&self, ctx: &Context<'_>) -> Result<i32, Error> {
        let client = ctx.data::<reqwest::Client>().unwrap();
        let url = format!("{}/player/file-position", rockbox_url());
        let response = client.get(&url).send().await?;
        let response = response.json::<FilePosition>().await?;
        Ok(response.position)
    }
}

#[derive(Default)]
pub struct PlaybackMutation;

#[Object]
impl PlaybackMutation {
    async fn play(&self, ctx: &Context<'_>, elapsed: i64, offset: i64) -> Result<i32, Error> {
        let cmd = ctx.data::<Arc<Mutex<Sender<RockboxCommand>>>>().unwrap();
        cmd.lock()
            .unwrap()
            .send(RockboxCommand::Play(elapsed, offset))?;
        Ok(0)
    }

    async fn pause(&self, ctx: &Context<'_>) -> Result<i32, Error> {
        ctx.data::<Arc<Mutex<Sender<RockboxCommand>>>>()
            .unwrap()
            .lock()
            .unwrap()
            .send(RockboxCommand::Pause)?;
        Ok(0)
    }

    async fn resume(&self, ctx: &Context<'_>) -> Result<i32, Error> {
        ctx.data::<Arc<Mutex<Sender<RockboxCommand>>>>()
            .unwrap()
            .lock()
            .unwrap()
            .send(RockboxCommand::Resume)?;
        Ok(0)
    }

    async fn next(&self, ctx: &Context<'_>) -> Result<i32, Error> {
        ctx.data::<Arc<Mutex<Sender<RockboxCommand>>>>()
            .unwrap()
            .lock()
            .unwrap()
            .send(RockboxCommand::Next)?;
        Ok(0)
    }

    async fn previous(&self, ctx: &Context<'_>) -> Result<i32, Error> {
        ctx.data::<Arc<Mutex<Sender<RockboxCommand>>>>()
            .unwrap()
            .lock()
            .unwrap()
            .send(RockboxCommand::Prev)?;
        Ok(0)
    }

    async fn fast_forward_rewind(&self, ctx: &Context<'_>, new_time: i32) -> Result<i32, Error> {
        ctx.data::<Arc<Mutex<Sender<RockboxCommand>>>>()
            .unwrap()
            .lock()
            .unwrap()
            .send(RockboxCommand::FfRewind(new_time))?;
        Ok(0)
    }

    async fn flush_and_reload_tracks(&self, ctx: &Context<'_>) -> Result<i32, Error> {
        ctx.data::<Arc<Mutex<Sender<RockboxCommand>>>>()
            .unwrap()
            .lock()
            .unwrap()
            .send(RockboxCommand::FlushAndReloadTracks)?;
        Ok(0)
    }

    async fn hard_stop(&self, ctx: &Context<'_>) -> Result<i32, Error> {
        ctx.data::<Arc<Mutex<Sender<RockboxCommand>>>>()
            .unwrap()
            .lock()
            .unwrap()
            .send(RockboxCommand::Stop)?;
        Ok(0)
    }

    async fn play_album(
        &self,
        ctx: &Context<'_>,
        album_id: String,
        shuffle: Option<bool>,
    ) -> Result<i32, Error> {
        let pool = ctx.data::<Pool<Sqlite>>()?;
        let tracks = repo::album_tracks::find_by_album(pool.clone(), &album_id).await?;
        let client = ctx.data::<reqwest::Client>().unwrap();
        let body = serde_json::json!({
            "tracks": tracks.into_iter().map(|t| t.path).collect::<Vec<String>>(),
        });

        let url = format!("{}/playlists", rockbox_url());
        client.post(&url).json(&body).send().await?;

        if let Some(true) = shuffle {
            let url = format!("{}/playlists/shuffle", rockbox_url());
            client.put(&url).send().await?;
        }

        let url = format!("{}/playlists/start", rockbox_url());
        client.put(&url).send().await?;

        Ok(0)
    }

    async fn play_artist_tracks(
        &self,
        ctx: &Context<'_>,
        artist_id: String,
        shuffle: Option<bool>,
    ) -> Result<i32, Error> {
        let pool = ctx.data::<Pool<Sqlite>>()?;
        let client = ctx.data::<reqwest::Client>().unwrap();
        let tracks = repo::artist_tracks::find_by_artist(pool.clone(), &artist_id).await?;
        let body = serde_json::json!({
            "tracks": tracks.into_iter().map(|t| t.path).collect::<Vec<String>>(),
        });

        let url = format!("{}/playlists", rockbox_url());
        client.post(&url).json(&body).send().await?;

        if let Some(true) = shuffle {
            let url = format!("{}/playlists/shuffle", rockbox_url());
            client.put(&url).send().await?;
        }

        let url = format!("{}/playlists/start", rockbox_url());
        client.put(&url).send().await?;

        Ok(0)
    }

    async fn play_playlist(
        &self,
        _ctx: &Context<'_>,
        _playlist_id: String,
        _shuffle: Option<bool>,
    ) -> Result<i32, Error> {
        todo!()
    }

    async fn play_directory(
        &self,
        ctx: &Context<'_>,
        path: String,
        recurse: Option<bool>,
        shuffle: Option<bool>,
    ) -> Result<i32, Error> {
        let client = ctx.data::<reqwest::Client>().unwrap();
        let mut tracks: Vec<String> = vec![];

        if !std::path::Path::new(&path).is_dir() {
            return Err(Error::new("Invalid path"));
        }

        match recurse {
            Some(true) => tracks = read_files(path).await?,
            _ => {
                for file in fs::read_dir(&path)? {
                    let file = file?;

                    if file.metadata()?.is_file() {
                        if !AUDIO_EXTENSIONS.iter().any(|ext| {
                            file.path()
                                .to_string_lossy()
                                .ends_with(&format!(".{}", ext))
                        }) {
                            continue;
                        }

                        tracks.push(file.path().to_string_lossy().to_string());
                    }
                }
            }
        }

        let body = serde_json::json!({
            "tracks": tracks,
        });

        let url = format!("{}/playlists", rockbox_url());
        client.post(&url).json(&body).send().await?;

        if let Some(true) = shuffle {
            let url = format!("{}/playlists/shuffle", rockbox_url());
            client.put(&url).send().await?;
        }

        let url = format!("{}/playlists/start", rockbox_url());
        client.put(&url).send().await?;

        Ok(0)
    }

    async fn play_track(&self, ctx: &Context<'_>, path: String) -> Result<i32, Error> {
        let client = ctx.data::<reqwest::Client>().unwrap();

        if !std::path::Path::new(&path).is_file() {
            return Err(Error::new("Invalid path"));
        }

        let body = serde_json::json!({
            "tracks": vec![path],
        });

        let url = format!("{}/playlists", rockbox_url());
        client.post(&url).json(&body).send().await?;

        let url = format!("{}/playlists/start", rockbox_url());
        client.put(&url).send().await?;

        Ok(0)
    }
}

#[derive(Default)]
pub struct PlaybackSubscription;

#[Subscription]
impl PlaybackSubscription {
    async fn currently_playing_song(&self) -> impl Stream<Item = Track> {
        SimpleBroker::<Track>::subscribe()
    }

    async fn playback_status(&self) -> impl Stream<Item = objects::audio_status::AudioStatus> {
        SimpleBroker::<objects::audio_status::AudioStatus>::subscribe()
    }
}
