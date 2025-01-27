use std::{env, future, sync::Arc};

use anyhow::Error;
use async_std::stream::StreamExt;
use mpris_server::{LoopStatus, Metadata, PlaybackStatus, Player, Time, TrackId};
use rockbox_graphql::{
    schema::objects::{audio_status::AudioStatus, track::Track},
    simplebroker::SimpleBroker,
};
use rockbox_rpc::api::rockbox::v1alpha1::{
    playback_service_client::PlaybackServiceClient, settings_service_client::SettingsServiceClient,
    sound_service_client::SoundServiceClient, AdjustVolumeRequest, GetGlobalSettingsRequest,
    HardStopRequest, NextRequest, PauseRequest, PlayRequest, PlayTrackRequest, PreviousRequest,
    ResumeRequest, SaveSettingsRequest,
    PlayOrPauseRequest,
};
use tokio::sync::Mutex;

pub mod macros;

const PLAYER_NAME: &str = "rockbox";

pub struct MprisServer {
    player: Arc<Player>,
}

impl MprisServer {
    pub async fn start() -> Result<Self, Error> {
        let host = env::var("ROCKBOX_HOST").unwrap_or_else(|_| "localhost".to_string());
        let port = env::var("ROCKBOX_PORT").unwrap_or_else(|_| "6061".to_string());
        let url = format!("tcp://{}:{}", host, port);

        let rt = tokio::runtime::Runtime::new()?;
        let client = Arc::new(Mutex::new(
            rt.block_on(PlaybackServiceClient::connect(url.clone()))?,
        ));
        let settings_service_client = Arc::new(Mutex::new(
            rt.block_on(SettingsServiceClient::connect(url.clone()))?,
        ));
        let sound_service_client = Arc::new(Mutex::new(
            rt.block_on(SoundServiceClient::connect(url.clone()))?,
        ));

        let player = Player::builder(PLAYER_NAME)
            .can_play(true)
            .can_pause(true)
            .can_seek(true)
            .can_go_next(true)
            .can_go_previous(true)
            .can_control(true)
            .build()
            .await?;

       player.set_identity("Rockbox").await?; 

        connect_player_action!(
            player,
            client,
            connect_previous,
            previous,
            PreviousRequest {}
        );
        connect_player_action!(player, client, connect_next, next, NextRequest {});
        connect_player_action!(player, client, connect_play, resume, ResumeRequest {});
        connect_player_action!(player, client, connect_pause, pause, PauseRequest {});
        connect_player_action!(player, client, connect_play_pause, play_or_pause, PlayOrPauseRequest {});
        connect_player_seek_action!(player, client);
        connect_player_set_position_action!(player, client);
        connect_player_action!(player, client, connect_stop, hard_stop, HardStopRequest {});
        connect_player_volume_action!(player, sound_service_client, settings_service_client);
        connect_player_shuffle_action!(player, settings_service_client);
        connect_player_loop_status_action!(player, settings_service_client);
        connect_player_open_uri_action!(player, client);

        let server = MprisServer {
            player: Arc::new(player),
        };

        async_std::task::spawn_local(server.player.run());

        let player_mutex = Arc::new(std::sync::Mutex::new(server.player.clone()));
        let player_mutex_clone = Arc::clone(&player_mutex);

        async_std::task::spawn_local(async move {
            let mut subscription = SimpleBroker::<AudioStatus>::subscribe();
            while let Some(response) = subscription.next().await {
                let player = player_mutex_clone.lock().unwrap();
                match response.status {
                    1 => match player.set_playback_status(PlaybackStatus::Playing).await {
                        Ok(_) => {}
                        Err(e) => {
                            eprintln!("Error: {}", e);
                        }
                    },
                    3 => match player.set_playback_status(PlaybackStatus::Paused).await {
                        Ok(_) => {}
                        Err(e) => {
                            eprintln!("Error: {}", e);
                        }
                    },
                    _ => match player.set_playback_status(PlaybackStatus::Stopped).await {
                        Ok(_) => {}
                        Err(e) => {
                            eprintln!("Error: {}", e);
                        }
                    },
                }
            }
        });

        async_std::task::spawn_local(async move {
            let port = std::env::var("ROCKBOX_GRAPHQL_PORT").unwrap_or("6062".to_string());
            let mut subscription = SimpleBroker::<Track>::subscribe();
            while let Some(track) = subscription.next().await {
                let player = player_mutex.lock().unwrap();
                let mut metadata = Metadata::builder()
                    .title(track.title)
                    .artist([track.artist])
                    .album(track.album)
                    .album_artist([track.album_artist])
                    .track_number(track.tracknum)
                    .disc_number(track.discnum)
                    .length(Time::from_millis(track.length as i64));

                if let Some(album_art) = track.album_art {
                    metadata = match album_art.starts_with("http") {
                        true => metadata.art_url(album_art),
                        false => metadata
                            .art_url(format!("http://localhost:{}/covers/{}", port, album_art)),
                    }
                }

                if let Some(trackid) = track.id {
                    metadata = metadata.trackid(
                        TrackId::try_from(format!("/rockbox/tracks/{}", trackid)).unwrap(),
                    );
                }

                let metadata = metadata.build();

                match player.set_metadata(metadata).await {
                    Ok(_) => {}
                    Err(e) => {
                        eprintln!("Error: {}", e);
                    }
                }

                player.set_position(Time::from_millis(track.elapsed as i64));
                match player.seeked(Time::from_millis(track.elapsed as i64)).await {
                    Ok(_) => {}
                    Err(e) => {
                        eprintln!("Error: {}", e);
                    }
                }
            }
        });

        future::pending::<()>().await;

        Ok(server)
    }
}
