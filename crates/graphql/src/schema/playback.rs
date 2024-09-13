use std::sync::{mpsc::Sender, Arc, Mutex};

use async_graphql::*;
use rockbox_sys::{self as rb, events::RockboxCommand};

use crate::schema::objects::track::Track;

#[derive(Default)]
pub struct PlaybackQuery;

#[Object]
impl PlaybackQuery {
    async fn status(&self) -> String {
        rb::playback::status();
        "status".to_string()
    }

    async fn current_track(&self) -> Option<Track> {
        let mp3entry = rb::playback::current_track();
        Some(mp3entry.into())
    }

    async fn get_file_position(&self) -> i32 {
        rb::playback::get_file_pos()
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
