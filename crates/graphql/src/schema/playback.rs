use async_graphql::*;
use rockbox_sys as rb;

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
    async fn play(&self, _ctx: &Context<'_>, elapsed: i64, offset: i64) -> String {
        rb::playback::play(elapsed, offset);
        "play".to_string()
    }

    async fn pause(&self) -> String {
        rb::playback::pause();
        "pause".to_string()
    }

    async fn resume(&self) -> String {
        rb::playback::resume();
        "resume".to_string()
    }

    async fn next(&self) -> String {
        rb::playback::next();
        "next".to_string()
    }

    async fn previous(&self) -> String {
        rb::playback::prev();
        "previous".to_string()
    }

    async fn fast_forward_rewind(&self, _ctx: &Context<'_>, new_time: i32) -> String {
        rb::playback::ff_rewind(new_time);
        "fast_forward_rewind".to_string()
    }

    async fn flush_and_reload_tracks(&self) -> String {
        rb::playback::flush_and_reload_tracks();
        "flush_and_reload_tracks".to_string()
    }

    async fn hard_stop(&self) -> String {
        rb::playback::hard_stop();
        "hard_stop".to_string()
    }
}
