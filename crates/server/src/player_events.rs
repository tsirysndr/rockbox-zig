use std::{
    env,
    sync::{Arc, Mutex},
    thread,
};

use rockbox_graphql::{
    schema::objects::{audio_status::AudioStatus, playlist::Playlist, track::Track},
    simplebroker::SimpleBroker,
};
use rockbox_library::repo;
use rockbox_sys::types::mp3_entry::Mp3Entry;
use rockbox_traits::Player;
use sqlx::{Pool, Sqlite};
use url::Url;

pub fn listen_for_playback_changes(
    player: Arc<Mutex<Option<Box<dyn Player + Send>>>>,
    pool: Pool<Sqlite>,
) {
    let cloned_player = player.clone();
    thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let client = reqwest::blocking::Client::new();
        loop {
            let mut player = cloned_player.lock().unwrap();

            if let Some(player) = player.as_deref_mut() {
                if let Ok(current_playback) = rt.block_on(player.get_current_playback()) {
                    if let Some(current_track) = current_playback.current_track {
                        if let Ok(Some(metadata)) =
                            rt.block_on(repo::track::find(pool.clone(), &current_track.id))
                        {
                            let album_art = match current_track.album_cover {
                                Some(cover) => {
                                    let url = Url::parse(&cover).unwrap();
                                    let path = url.path();
                                    match client
                                        .get(&format!(
                                            "{}:{}{}",
                                            "http://localhost",
                                            env::var("ROCKBOX_GRAPHQL_PORT")
                                                .unwrap_or("6062".to_string()),
                                            path
                                        ))
                                        .send()
                                    {
                                        Ok(response) => match response.status() {
                                            reqwest::StatusCode::OK => Some(format!(
                                                "http://localhost:{}{}",
                                                env::var("ROCKBOX_GRAPHQL_PORT")
                                                    .unwrap_or("6062".to_string()),
                                                path
                                            )),
                                            _ => Some(cover),
                                        },
                                        Err(_) => Some(cover),
                                    }
                                }
                                None => None,
                            };

                            let mut track: Track = Default::default();
                            track.id = Some(metadata.id);
                            track.title = metadata.title;
                            track.artist = metadata.artist;
                            track.album = metadata.album;
                            track.length = metadata.length as u64;
                            track.album_art = album_art;
                            track.album_id = Some(metadata.album_id);
                            track.artist_id = Some(metadata.artist_id);
                            track.elapsed = current_playback.position_ms as u64;
                            track.path = metadata.path;
                            track.tracknum =
                                metadata.track_number.map(|n| n as i32).unwrap_or_default();
                            track.discnum = metadata.disc_number as i32;
                            SimpleBroker::publish(track);
                            SimpleBroker::publish(AudioStatus {
                                status: match current_playback.is_playing {
                                    true => 1,
                                    false => 3,
                                },
                            });

                            let tracks = current_playback.items;
                            let index = match tracks.len() >= 2 {
                                true => tracks.len() - 2,
                                false => 0,
                            } as i32;

                            let tracks: Vec<Mp3Entry> =
                                tracks.into_iter().map(|(t, _)| t.into()).collect();
                            SimpleBroker::publish(Playlist {
                                amount: tracks.len() as i32,
                                index,
                                tracks: tracks.into_iter().map(|t| t.into()).collect(),
                                ..Default::default()
                            });
                        }
                    }
                }
            }

            drop(player);

            thread::sleep(std::time::Duration::from_millis(500));
        }
    });
}
