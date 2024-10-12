pub mod albums;
pub mod artists;
pub mod browse;
pub mod docs;
pub mod player;
pub mod playlists;
pub mod settings;
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
async_handler!(playlists, create_playlist);
async_handler!(playlists, start_playlist);
async_handler!(playlists, shuffle_playlist);
async_handler!(playlists, get_playlist_amount);
async_handler!(playlists, resume_playlist);
async_handler!(playlists, resume_track);
async_handler!(playlists, get_playlist_tracks);
async_handler!(playlists, insert_tracks);
async_handler!(playlists, remove_tracks);
async_handler!(tracks, get_tracks);
async_handler!(tracks, get_track);
async_handler!(system, get_rockbox_version);
async_handler!(system, get_status);
async_handler!(settings, get_global_settings);
async_handler!(docs, get_openapi);
async_handler!(docs, index);
