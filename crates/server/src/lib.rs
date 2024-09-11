use owo_colors::OwoColorize;
use rockbox_sys::{self as rb, events::RockboxCommand};
use std::{
    sync::{Arc, Mutex},
    thread,
};

#[no_mangle]
pub extern "C" fn start_server() {
    const BANNER: &str = r#"
              __________               __   ___.
    Open      \______   \ ____   ____ |  | _\_ |__   _______  ___
    Source     |       _//  _ \_/ ___\|  |/ /| __ \ /  _ \  \/  /
    Jukebox    |    |   (  <_> )  \___|    < | \_\ (  <_> > <  <
    Firmware   |____|_  /\____/ \___  >__|_ \|___  /\____/__/\_ \
                      \/            \/     \/    \/            \/
    "#;

    // Start the server
    println!("{}", BANNER.yellow());

    let (cmd_tx, cmd_rx) = std::sync::mpsc::channel::<RockboxCommand>();
    let cmd_tx = Arc::new(Mutex::new(cmd_tx));

    thread::spawn(move || {
        while let Ok(event) = cmd_rx.recv() {
            match event {
                RockboxCommand::Play(elapsed, offset) => {
                    rb::playback::play(elapsed, offset);
                }
                RockboxCommand::Pause => {
                    rb::playback::pause();
                }
                RockboxCommand::Resume => {
                    rb::playback::resume();
                }
                RockboxCommand::Next => {
                    rb::playback::next();
                }
                RockboxCommand::Prev => {
                    rb::playback::prev();
                }
                RockboxCommand::FfRewind(newtime) => {
                    rb::playback::ff_rewind(newtime);
                }
                RockboxCommand::FlushAndReloadTracks => {
                    rb::playback::flush_and_reload_tracks();
                }
                RockboxCommand::Stop => {
                    rb::playback::hard_stop();
                }
            }
        }
    });

    let cloned_cmd_tx = cmd_tx.clone();

    thread::spawn(move || {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        match runtime.block_on(rockbox_rpc::server::start(cmd_tx.clone())) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("Error starting server: {}", e);
            }
        }
    });

    thread::spawn(move || {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        match runtime.block_on(rockbox_graphql::server::start(cloned_cmd_tx.clone())) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("Error starting server: {}", e);
            }
        }
    });
}
