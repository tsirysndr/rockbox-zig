#[macro_export]
macro_rules! connect_player_action {
    ($player:expr, $client:expr, $connect:ident, $action:ident, $request:expr) => {{
        let client = Arc::clone(&$client);
        let rt = tokio::runtime::Runtime::new()?;
        $player.$connect(move |_player| {
            let client = Arc::clone(&client);
            match rt.block_on(async move {
                let mut client = client.lock().await;
                client.$action($request).await?;
                Ok::<(), Error>(())
            }) {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("Error: {}", e);
                }
            }
        });
    }};
}

#[macro_export]
macro_rules! connect_player_seek_action {
    ($player:expr, $client:expr) => {{
        let client = Arc::clone(&$client);
        let rt = tokio::runtime::Runtime::new()?;
        $player.connect_seek(move |_player, time| {
            let client = Arc::clone(&client);
            match rt.block_on(async move {
                let mut client = client.lock().await;
                client
                    .play(PlayRequest {
                        elapsed: time.as_millis(),
                        offset: 0,
                    })
                    .await?;
                Ok::<(), Error>(())
            }) {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("Error: {}", e);
                }
            }
        });
    }};
}

#[macro_export]
macro_rules! connect_player_set_position_action {
    ($player:expr, $client:expr) => {{
        let client = Arc::clone(&$client);
        let rt = tokio::runtime::Runtime::new()?;
        $player.connect_set_position(move |_player, _track_id, time| {
            let client = Arc::clone(&client);
            match rt.block_on(async move {
                let mut client = client.lock().await;
                client
                    .play(PlayRequest {
                        elapsed: time.as_millis(),
                        offset: 0,
                    })
                    .await?;
                Ok::<(), Error>(())
            }) {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("Error: {}", e);
                }
            }
        });
    }};
}

#[macro_export]
macro_rules! connect_player_open_uri_action {
    ($player:expr, $client:expr) => {{
        let client = Arc::clone(&$client);
        let rt = tokio::runtime::Runtime::new()?;
        $player.connect_open_uri(move |_player, uri| {
            let client = Arc::clone(&client);
            match rt.block_on(async move {
                let mut client = client.lock().await;
                let path = uri.to_string();
                let path = match path.starts_with("file://") {
                    true => urlencoding::decode(&path.replace("file://", ""))
                        .unwrap()
                        .to_string(),
                    false => path,
                };
                client.play_track(PlayTrackRequest { path }).await?;
                Ok::<(), Error>(())
            }) {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("Error: {}", e);
                }
            }
        });
    }};
}

#[macro_export]
macro_rules! connect_player_volume_action {
    ($player:expr, $client:expr, $settings_client:expr) => {
        let client = Arc::clone(&$client);
        let settings_client = Arc::clone(&$settings_client);
        let rt = tokio::runtime::Runtime::new()?;
        $player.connect_set_volume(move |_player, new_volume| {
            let client = Arc::clone(&client);
            let settings_client = Arc::clone(&settings_client);
            let mut settings_client = rt.block_on(settings_client.lock());
            let volume = match rt
                .block_on(settings_client.get_global_settings(GetGlobalSettingsRequest {}))
            {
                Ok(response) => response.into_inner().volume,
                Err(e) => {
                    eprintln!("Error: {}", e);
                    0
                }
            };
            match rt.block_on(async move {
                let mut client = client.lock().await;
                // new_volume is a float between 0.0 and 1.0
                // we need to convert it to an i32 between -80 db and 0 db
                // 0.0 -> -80 db
                // 1.0 -> 0 db
                // volume = -80 + 80 * new_volume
                let new_volume = (-80.0 + 80.0 * new_volume) as i32;
                let steps = (new_volume as i32 - volume) as i32;
                client.adjust_volume(AdjustVolumeRequest { steps }).await?;
                Ok::<(), Error>(())
            }) {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("Error: {}", e);
                }
            }
        });
    };
}

#[macro_export]
macro_rules! connect_player_shuffle_action {
    ($player:expr, $client:expr) => {
        let client = Arc::clone(&$client);
        let rt = tokio::runtime::Runtime::new()?;
        $player.connect_set_shuffle(move |_player, new_shuffle| {
            let client = Arc::clone(&client);
            match rt.block_on(async move {
                let mut client = client.lock().await;
                client
                    .save_settings(SaveSettingsRequest {
                        playlist_shuffle: Some(new_shuffle),
                        ..Default::default()
                    })
                    .await?;
                Ok::<(), Error>(())
            }) {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("Error: {}", e);
                }
            }
        });
    };
}

#[macro_export]
macro_rules! connect_player_loop_status_action {
    ($player:expr, $client:expr) => {
        let client = Arc::clone(&$client);
        let rt = tokio::runtime::Runtime::new()?;
        $player.connect_set_loop_status(move |_player, new_loop_status| {
            let client = Arc::clone(&client);
            match rt.block_on(async move {
                let mut client = client.lock().await;
                let repeat_mode = match new_loop_status {
                    LoopStatus::None => Some(0),
                    LoopStatus::Playlist => Some(1),
                    LoopStatus::Track => Some(2),
                };
                client
                    .save_settings(SaveSettingsRequest {
                        repeat_mode,
                        ..Default::default()
                    })
                    .await?;
                Ok::<(), Error>(())
            }) {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("Error: {}", e);
                }
            }
        });
    };
}
