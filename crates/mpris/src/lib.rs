use std::{env, future, sync::Arc, thread};

use anyhow::Error;
use mpris_server::Player;
use rockbox_rpc::api::rockbox::v1alpha1::{
    playback_service_client::PlaybackServiceClient, AdjustVolumeRequest, HardStopRequest,
    NextRequest, PauseRequest, PlayRequest, PreviousRequest, ResumeRequest, SaveSettingsRequest,
};
use tokio::sync::Mutex;

pub mod macros;

const PLAYER_NAME: &str = "rockbox";

pub struct MprisServer {
    player: Arc<Player>,
}

impl MprisServer {
    pub async fn start() -> Result<(), Error> {
        let host = env::var("ROCKBOX_HOST").unwrap_or_else(|_| "localhost".to_string());
        let port = env::var("ROCKBOX_PORT").unwrap_or_else(|_| "6061".to_string());
        let url = format!("tcp://{}:{}", host, port);

        let rt = tokio::runtime::Runtime::new()?;
        let client = Arc::new(Mutex::new(
            rt.block_on(PlaybackServiceClient::connect(url))?,
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
        connect_player_seek_action!(player, client);
        connect_player_action!(player, client, connect_stop, hard_stop, HardStopRequest {});

        player.connect_set_loop_status(|_player, _status| {
            println!("Repeat");
        });

        player.connect_set_shuffle(|_player, _shuffle| {
            println!("Shuffle");
        });

        SaveSettingsRequest {
            playlist_shuffle: Some(false),
            ..Default::default()
        };

        AdjustVolumeRequest { steps: 0 };

        player.connect_set_volume(|_player, _volume| {
            println!("Volume");
        });

        let server = Self {
            player: Arc::new(player),
        };

        async_std::task::spawn_local(server.player.run());

        thread::spawn(move || {});

        future::pending::<()>().await;

        Ok(())
    }
}
