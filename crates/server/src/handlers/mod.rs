pub mod albums;
pub mod artists;
pub mod browse;
pub mod devices;
pub mod docs;
pub mod player;
pub mod playlists;
pub mod saved_playlists;
pub mod search;
pub mod settings;
pub mod smart_playlists;
pub mod system;
pub mod tracks;

use crate::http::{Context, Request, Response};
use anyhow::Error;

macro_rules! async_handler {
    ($module:ident, $handler:ident) => {
        pub fn $handler(
            context: &Context,
            request: &Request,
            response: &mut Response,
        ) -> Result<(), Error> {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on($module::$handler(context, request, response))?;
            Ok(())
        }
    };
}

async_handler!(albums, get_albums);
async_handler!(albums, get_album);
async_handler!(albums, get_album_tracks);
async_handler!(artists, get_artists);
async_handler!(artists, get_artist);
async_handler!(artists, get_artist_albums);
async_handler!(artists, get_artist_tracks);
async_handler!(browse, get_tree_entries);
async_handler!(player, load);
async_handler!(player, play);
async_handler!(player, pause);
async_handler!(player, resume);
async_handler!(player, ff_rewind);
async_handler!(player, status);
async_handler!(player, current_track);
async_handler!(player, next_track);
async_handler!(player, flush_and_reload_tracks);
async_handler!(player, next);
async_handler!(player, previous);
async_handler!(player, stop);
async_handler!(player, get_file_position);
async_handler!(player, get_volume);
async_handler!(player, adjust_volume);
async_handler!(player, get_current_player);
async_handler!(playlists, create_playlist);
async_handler!(playlists, start_playlist);
async_handler!(playlists, shuffle_playlist);
async_handler!(playlists, get_playlist_amount);
async_handler!(playlists, resume_playlist);
async_handler!(playlists, resume_track);
async_handler!(playlists, get_playlist_tracks);
async_handler!(playlists, insert_tracks);
async_handler!(playlists, remove_tracks);
async_handler!(playlists, get_playlist);
async_handler!(saved_playlists, list_saved_playlists);
async_handler!(saved_playlists, get_saved_playlist);
async_handler!(saved_playlists, create_saved_playlist);
async_handler!(saved_playlists, update_saved_playlist);
async_handler!(saved_playlists, delete_saved_playlist);
async_handler!(saved_playlists, get_saved_playlist_tracks);
async_handler!(saved_playlists, get_saved_playlist_track_ids);
async_handler!(saved_playlists, add_tracks_to_saved_playlist);
async_handler!(saved_playlists, remove_track_from_saved_playlist);
async_handler!(saved_playlists, play_saved_playlist);
async_handler!(saved_playlists, list_playlist_folders);
async_handler!(saved_playlists, create_playlist_folder);
async_handler!(saved_playlists, delete_playlist_folder);
async_handler!(tracks, get_tracks);
async_handler!(tracks, get_track);
async_handler!(tracks, save_stream_track_metadata);
async_handler!(system, get_rockbox_version);
async_handler!(system, get_status);
async_handler!(system, scan_library);
async_handler!(settings, get_global_settings);
async_handler!(settings, update_global_settings);
async_handler!(docs, get_openapi);
async_handler!(docs, index);
async_handler!(search, search);
async_handler!(devices, connect);
async_handler!(devices, disconnect);
async_handler!(devices, get_devices);
async_handler!(devices, get_device);
async_handler!(smart_playlists, list_smart_playlists);
async_handler!(smart_playlists, get_smart_playlist);
async_handler!(smart_playlists, create_smart_playlist);
async_handler!(smart_playlists, update_smart_playlist);
async_handler!(smart_playlists, delete_smart_playlist);
async_handler!(smart_playlists, get_smart_playlist_tracks);
async_handler!(smart_playlists, play_smart_playlist);
async_handler!(smart_playlists, record_track_played);
async_handler!(smart_playlists, record_track_skipped);
async_handler!(smart_playlists, get_track_stats);
