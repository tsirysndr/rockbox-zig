use std::net::SocketAddr;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};

use crate::api::rockbox::v1alpha1::browse_service_server::BrowseServiceServer;
use crate::api::rockbox::v1alpha1::library_service_server::LibraryServiceServer;
use crate::api::rockbox::v1alpha1::playback_service_server::PlaybackServiceServer;
use crate::api::rockbox::v1alpha1::playlist_service_server::PlaylistServiceServer;
use crate::api::rockbox::v1alpha1::settings_service_server::SettingsServiceServer;
use crate::api::rockbox::v1alpha1::sound_service_server::SoundServiceServer;
use crate::api::rockbox::FILE_DESCRIPTOR_SET;
use crate::browse::Browse;
use crate::library::Library;
use crate::playback::Playback;
use crate::playlist::Playlist;
use crate::settings::Settings;
use crate::sound::Sound;
use crate::system::System;
use owo_colors::OwoColorize;
use rockbox_library::create_connection_pool;
use rockbox_sys::events::RockboxCommand;
use tonic::transport::Server;

pub async fn start(
    cmd_tx: Arc<Mutex<Sender<RockboxCommand>>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let rockbox_port: u16 = std::env::var("ROCKBOX_PORT")
        .unwrap_or_else(|_| "6061".to_string())
        .parse()
        .expect("ROCKBOX_PORT must be a number");

    let addr: SocketAddr = format!("0.0.0.0:{}", rockbox_port).parse()?;

    let host_and_port = format!("0.0.0.0:{}", rockbox_port);

    println!(
        "{} server is running on {}",
        "Rockbox gRPC".bright_purple(),
        host_and_port.bright_green()
    );

    let client = reqwest::Client::new();
    let pool = create_connection_pool().await?;

    Server::builder()
        .accept_http1(true)
        .add_service(
            tonic_reflection::server::Builder::configure()
                .register_encoded_file_descriptor_set(FILE_DESCRIPTOR_SET)
                .build_v1alpha()?,
        )
        .add_service(tonic_web::enable(LibraryServiceServer::new(
            Library::new(pool),
        )))
        .add_service(tonic_web::enable(PlaylistServiceServer::new(
            Playlist::new(cmd_tx.clone(), client.clone()),
        )))
        .add_service(tonic_web::enable(PlaybackServiceServer::new(
            Playback::new(cmd_tx.clone(), client.clone()),
        )))
        .add_service(tonic_web::enable(BrowseServiceServer::new(Browse::new(
            client.clone(),
        ))))
        .add_service(tonic_web::enable(SoundServiceServer::new(Sound::default())))
        .add_service(tonic_web::enable(SettingsServiceServer::new(
            Settings::new(client.clone()),
        )))
        .add_service(tonic_web::enable(
            crate::api::rockbox::v1alpha1::system_service_server::SystemServiceServer::new(
                System::new(client.clone()),
            ),
        ))
        .serve(addr)
        .await?;
    Ok(())
}
