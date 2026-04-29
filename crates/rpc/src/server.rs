use std::net::SocketAddr;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};

use crate::api::rockbox::v1alpha1::bluetooth_service_server::BluetoothServiceServer;
use crate::api::rockbox::v1alpha1::browse_service_server::BrowseServiceServer;
use crate::api::rockbox::v1alpha1::device_service_server::DeviceServiceServer;
use crate::api::rockbox::v1alpha1::library_service_server::LibraryServiceServer;
use crate::api::rockbox::v1alpha1::playback_service_server::PlaybackServiceServer;
use crate::api::rockbox::v1alpha1::playlist_service_server::PlaylistServiceServer;
use crate::api::rockbox::v1alpha1::saved_playlist_service_server::SavedPlaylistServiceServer;
use crate::api::rockbox::v1alpha1::settings_service_server::SettingsServiceServer;
use crate::api::rockbox::v1alpha1::smart_playlist_service_server::SmartPlaylistServiceServer;
use crate::api::rockbox::v1alpha1::sound_service_server::SoundServiceServer;
use crate::api::rockbox::FILE_DESCRIPTOR_SET;
use crate::bluetooth::Bluetooth;
use crate::browse::Browse;
use crate::device::Device;
use crate::library::Library;
use crate::playback::Playback;
use crate::playlist::Playlist;
use crate::saved_playlist::SavedPlaylist;
use crate::settings::Settings;
use crate::smart_playlist::SmartPlaylistRpc;
use crate::sound::Sound;
use crate::system::System;
use rockbox_library::create_connection_pool;
use rockbox_playlists::PlaylistStore;
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

    let client = reqwest::Client::new();
    let pool = create_connection_pool().await?;
    let playlist_store = PlaylistStore::new(pool.clone());

    Server::builder()
        .accept_http1(true)
        .add_service(
            tonic_reflection::server::Builder::configure()
                .register_encoded_file_descriptor_set(FILE_DESCRIPTOR_SET)
                .build_v1alpha()?,
        )
        .add_service(tonic_web::enable(DeviceServiceServer::new(Device::new(
            client.clone(),
        ))))
        .add_service(tonic_web::enable(LibraryServiceServer::new(Library::new(
            pool.clone(),
            client.clone(),
        ))))
        .add_service(tonic_web::enable(PlaylistServiceServer::new(
            Playlist::new(cmd_tx.clone(), client.clone(), pool.clone()),
        )))
        .add_service(tonic_web::enable(PlaybackServiceServer::new(
            Playback::new(cmd_tx.clone(), client.clone(), pool.clone()),
        )))
        .add_service(tonic_web::enable(BrowseServiceServer::new(
            Browse::default(),
        )))
        .add_service(tonic_web::enable(SoundServiceServer::new(Sound::new(
            client.clone(),
        ))))
        .add_service(tonic_web::enable(SettingsServiceServer::new(
            Settings::new(client.clone()),
        )))
        .add_service(tonic_web::enable(
            crate::api::rockbox::v1alpha1::system_service_server::SystemServiceServer::new(
                System::new(client.clone()),
            ),
        ))
        .add_service(tonic_web::enable(SavedPlaylistServiceServer::new(
            SavedPlaylist::new(playlist_store.clone(), client.clone()),
        )))
        .add_service(tonic_web::enable(SmartPlaylistServiceServer::new(
            SmartPlaylistRpc::new(playlist_store.clone(), pool.clone(), client.clone()),
        )))
        .add_service(tonic_web::enable(BluetoothServiceServer::new(
            Bluetooth::new(client.clone()),
        )))
        .serve(addr)
        .await?;
    Ok(())
}
