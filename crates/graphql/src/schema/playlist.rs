use async_graphql::*;

#[derive(Default)]
pub struct PlaylistQuery;

#[Object]
impl PlaylistQuery {
    async fn playlist_get_current(&self) -> String {
        "playlist get current".to_string()
    }

    async fn get_resume_info(&self) -> String {
        "get resume info".to_string()
    }

    async fn get_track_info(&self) -> String {
        "get track info".to_string()
    }

    async fn get_first_index(&self) -> String {
        "get first index".to_string()
    }

    async fn get_display_index(&self) -> String {
        "get display index".to_string()
    }

    async fn playlist_amount(&self) -> String {
        "playlist amount".to_string()
    }
}

#[derive(Default)]
pub struct PlaylistMutation;

#[Object]
impl PlaylistMutation {
    async fn playlist_resume(&self) -> String {
        "playlist resume".to_string()
    }

    async fn resume_track(&self) -> String {
        "resume track".to_string()
    }

    async fn playlist_set_modified(&self) -> String {
        "set modified".to_string()
    }

    async fn playlist_start(&self) -> String {
        "playlist start".to_string()
    }

    async fn playlist_sync(&self) -> String {
        "playlist sync".to_string()
    }

    async fn playlist_remove_all_tracks(&self) -> String {
        "playlist remove all tracks".to_string()
    }

    async fn playlist_create(&self) -> String {
        "playlist create".to_string()
    }

    async fn playlist_insert_track(&self) -> String {
        "playlist insert track".to_string()
    }

    async fn playlist_insert_directory(&self) -> String {
        "playlist insert directory".to_string()
    }

    async fn shuffle_playlist(&self) -> String {
        "shuffle playlist".to_string()
    }

    async fn warn_on_playlist_erase(&self) -> String {
        "warn on playlist erase".to_string()
    }
}
