use async_graphql::*;

#[derive(Default)]
pub struct PlaybackQuery;

#[Object]
impl PlaybackQuery {
    async fn status(&self) -> String {
        "status".to_string()
    }

    async fn current_track(&self) -> String {
        "current_track".to_string()
    }

    async fn get_file_position(&self) -> String {
        "get_file_position".to_string()
    }
}

#[derive(Default)]
pub struct PlaybackMutation;

#[Object]
impl PlaybackMutation {
    async fn play(&self) -> String {
        "play".to_string()
    }

    async fn pause(&self) -> String {
        "pause".to_string()
    }

    async fn resume(&self) -> String {
        "resume".to_string()
    }

    async fn next(&self) -> String {
        "next".to_string()
    }

    async fn previous(&self) -> String {
        "previous".to_string()
    }

    async fn fast_forward_rewind(&self) -> String {
        "fast_forward_rewind".to_string()
    }

    async fn flush_and_reload_tracks(&self) -> String {
        "flush_and_reload_tracks".to_string()
    }

    async fn hard_stop(&self) -> String {
        "hard_stop".to_string()
    }
}
