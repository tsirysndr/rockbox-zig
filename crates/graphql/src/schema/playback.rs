use std::sync::{mpsc::Sender, Arc, Mutex};

use crate::schema::objects;
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
    async fn play(&self, ctx: &Context<'_>, elapsed: i64, offset: i64) -> Result<String, Error> {
        let cmd = ctx.data::<Arc<Mutex<Sender<RockboxCommand>>>>().unwrap();
        cmd.lock()
            .unwrap()
            .send(RockboxCommand::Play(elapsed, offset))?;
        Ok("play".to_string())
    }

    async fn pause(&self, ctx: &Context<'_>) -> Result<String, Error> {
        ctx.data::<Arc<Mutex<Sender<RockboxCommand>>>>()
            .unwrap()
            .lock()
            .unwrap()
            .send(RockboxCommand::Pause)?;
        Ok("pause".to_string())
    }

    async fn resume(&self, ctx: &Context<'_>) -> Result<String, Error> {
        ctx.data::<Arc<Mutex<Sender<RockboxCommand>>>>()
            .unwrap()
            .lock()
            .unwrap()
            .send(RockboxCommand::Resume)?;
        Ok("resume".to_string())
    }

    async fn next(&self, ctx: &Context<'_>) -> Result<String, Error> {
        ctx.data::<Arc<Mutex<Sender<RockboxCommand>>>>()
            .unwrap()
            .lock()
            .unwrap()
            .send(RockboxCommand::Next)?;
        Ok("next".to_string())
    }

    async fn previous(&self, ctx: &Context<'_>) -> Result<String, Error> {
        ctx.data::<Arc<Mutex<Sender<RockboxCommand>>>>()
            .unwrap()
            .lock()
            .unwrap()
            .send(RockboxCommand::Prev)?;
        Ok("previous".to_string())
    }

    async fn fast_forward_rewind(&self, ctx: &Context<'_>, new_time: i32) -> Result<String, Error> {
        ctx.data::<Arc<Mutex<Sender<RockboxCommand>>>>()
            .unwrap()
            .lock()
            .unwrap()
            .send(RockboxCommand::FfRewind(new_time))?;
        Ok("fast_forward_rewind".to_string())
    }

    async fn flush_and_reload_tracks(&self, ctx: &Context<'_>) -> Result<String, Error> {
        ctx.data::<Arc<Mutex<Sender<RockboxCommand>>>>()
            .unwrap()
            .lock()
            .unwrap()
            .send(RockboxCommand::FlushAndReloadTracks)?;
        Ok("flush_and_reload_tracks".to_string())
    }

    async fn hard_stop(&self, ctx: &Context<'_>) -> Result<String, Error> {
        ctx.data::<Arc<Mutex<Sender<RockboxCommand>>>>()
            .unwrap()
            .lock()
            .unwrap()
            .send(RockboxCommand::Stop)?;
        Ok("hard_stop".to_string())
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
