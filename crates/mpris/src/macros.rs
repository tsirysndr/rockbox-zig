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
        $player.connect_seek(move |_player, _time| {
            let client = Arc::clone(&client);
            match rt.block_on(async move {
                let mut client = client.lock().await;
                client
                    .play(PlayRequest {
                        elapsed: 0,
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
macro_rules! connect_player_volume_action {
    ($player:expr, $client:expr) => {
        let client = Arc::clone(&$client);
        let rt = tokio::runtime::Runtime::new()?;
        $player.connect_set_volume(move |_player, new_volume| {
            let client = Arc::clone(&client);
            match rt.block_on(async move {
                let mut client = client.lock().await;
                client
                    .adjust_volume(AdjustVolumeRequest { steps: 1 })
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
