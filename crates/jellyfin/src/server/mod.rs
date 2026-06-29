//! Jellyfin-compatible HTTP server. Mirrors the routes Finamp / Findroid /
//! Streamyfin / Amcfy / Symfonium hit, audio-only.

pub mod auth;
pub mod discovery;
pub mod dto;
pub mod handlers;
pub mod mapping;

use actix_cors::Cors;
use actix_web::{web, App, HttpServer};
use rockbox_library::create_connection_pool;
use rockbox_settings::read_settings;
use sqlx::{Executor, Pool, Sqlite};
use std::path::PathBuf;
use std::sync::Arc;

pub struct JellyfinState {
    pub pool: Pool<Sqlite>,
    pub username: Arc<String>,
    pub password: Arc<String>,
    pub music_dir: PathBuf,
    pub server_id: String,
    pub server_name: String,
    pub user_id: Arc<String>,
    pub port: u16,
}

pub async fn start() -> anyhow::Result<()> {
    let settings = read_settings().unwrap_or_default();
    // Activation is gated on `jellyfin_port` being present in settings.toml.
    // Omit the key entirely to disable the server (the Subsonic gate is
    // separately controlled by `subsonic_password`).
    let Some(port) = settings.jellyfin_port else {
        tracing::info!("Jellyfin server: jellyfin_port not set in settings.toml, server disabled");
        return Ok(());
    };
    let username = settings
        .subsonic_username
        .clone()
        .unwrap_or_else(|| "admin".to_string());
    let password = settings.subsonic_password.clone().unwrap_or_default();
    let server_name = "rockbox".to_string();

    if password.is_empty() {
        tracing::info!(
            "Jellyfin server: subsonic_password not set in settings.toml, server disabled"
        );
        return Ok(());
    }

    let music_dir = std::env::var("ROCKBOX_LIBRARY")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
            PathBuf::from(format!("{home}/Music"))
        });

    let pool = create_connection_pool().await?;

    // Apply our migrations on top of the library schema.
    pool.execute(include_str!(
        "../../migrations/20260629000000_jellyfin_tables.sql"
    ))
    .await?;

    let server_id = auth::ensure_server_id(&pool).await?;
    let user_id = mapping::user_guid(&username);
    let addr = format!("0.0.0.0:{port}");

    let state = web::Data::new(JellyfinState {
        pool,
        username: Arc::new(username),
        password: Arc::new(password),
        music_dir,
        server_id: server_id.clone(),
        server_name: server_name.clone(),
        user_id: Arc::new(user_id),
        port,
    });

    tracing::info!("Jellyfin API server listening on {addr} (id={server_id})");

    // Background tasks: mDNS announcement + UDP discovery listener.
    discovery::register_mdns("rockbox", port, &server_id);
    let dname = server_name.clone();
    let did = server_id.clone();
    tokio::spawn(async move {
        if let Err(e) = discovery::run(dname, did, port).await {
            tracing::error!("jellyfin discovery stopped: {e}");
        }
    });

    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .wrap(Cors::permissive())
            .configure(configure_routes)
    })
    .bind(&addr)?
    .run()
    .await?;

    Ok(())
}

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.route("/", web::get().to(handlers::index))
        .route("/", web::head().to(handlers::index))
        // System
        .route(
            "/System/Info/Public",
            web::get().to(handlers::system_info_public),
        )
        .route("/System/Info", web::get().to(handlers::system_info))
        .route("/System/Endpoint", web::get().to(handlers::system_endpoint))
        .route(
            "/System/Configuration/branding",
            web::get().to(handlers::branding_config),
        )
        .route(
            "/Branding/Configuration",
            web::get().to(handlers::branding_config),
        )
        .route(
            "/Branding/Css",
            web::get().to(|| async {
                actix_web::HttpResponse::Ok()
                    .content_type("text/css")
                    .body("")
            }),
        )
        // Users / auth — register PascalCase + common lowercase variants.
        .route(
            "/Users/AuthenticateByName",
            web::post().to(handlers::authenticate_by_name),
        )
        .route(
            "/Users/authenticatebyname",
            web::post().to(handlers::authenticate_by_name),
        )
        .route(
            "/Users/authenticateByName",
            web::post().to(handlers::authenticate_by_name),
        )
        .route("/Users/Public", web::get().to(handlers::users_public))
        .route("/Users", web::get().to(handlers::users_list))
        .route("/Users/Me", web::get().to(handlers::users_me))
        .route("/Users/{id}", web::get().to(handlers::user_by_id))
        // Views / libraries
        .route("/Users/{id}/Views", web::get().to(handlers::user_views))
        .route("/UserViews", web::get().to(handlers::user_views_query))
        .route(
            "/Library/MediaFolders",
            web::get().to(handlers::media_folders),
        )
        .route(
            "/Library/VirtualFolders",
            web::get().to(handlers::library_virtual_folders),
        )
        .route("/Library/Refresh", web::post().to(handlers::no_content))
        // Items — specific paths before /Items/{id}.
        .route("/Items", web::get().to(handlers::items))
        .route("/Items/Suggestions", web::get().to(handlers::empty_items))
        .route("/Items/Resume", web::get().to(handlers::empty_items))
        .route("/Items/Latest", web::get().to(handlers::items_latest))
        .route(
            "/Items/{id}/File",
            web::get().to(handlers::item_file_stream),
        )
        .route(
            "/Items/{id}/File",
            web::head().to(handlers::item_file_stream),
        )
        .route(
            "/Items/{id}/Download",
            web::get().to(handlers::item_file_stream),
        )
        .route("/Items/{id}", web::get().to(handlers::item_by_id))
        .route("/Users/{id}/Items", web::get().to(handlers::user_items))
        .route(
            "/Users/{id}/Items/Resume",
            web::get().to(handlers::empty_items),
        )
        .route(
            "/Users/{id}/Items/Latest",
            web::get().to(handlers::items_latest),
        )
        .route(
            "/Users/{uid}/Items/{id}",
            web::get().to(handlers::user_item_by_id),
        )
        // Legacy /UserItems/* aliases.
        .route("/UserItems/Resume", web::get().to(handlers::empty_items))
        .route("/UserItems/Latest", web::get().to(handlers::empty_array))
        // Search
        .route("/Search/Hints", web::get().to(handlers::search_hints))
        // /Shows/* — no series concept here.
        .route("/Shows/NextUp", web::get().to(handlers::empty_items))
        .route("/Shows/Upcoming", web::get().to(handlers::empty_items))
        // Artists
        .route("/Artists", web::get().to(handlers::artists))
        .route("/Artists/AlbumArtists", web::get().to(handlers::artists))
        .route("/Artists/{name}", web::get().to(handlers::artist_by_name))
        // Images — register both Items and items (lowercase used by Findroid).
        .route(
            "/Items/{id}/Images/{kind}",
            web::get().to(handlers::item_image),
        )
        .route(
            "/Items/{id}/Images/{kind}/{idx}",
            web::get().to(handlers::item_image_by_index),
        )
        .route(
            "/items/{id}/Images/{kind}",
            web::get().to(handlers::item_image),
        )
        .route(
            "/items/{id}/Images/{kind}/{idx}",
            web::get().to(handlers::item_image_by_index),
        )
        .route(
            "/Items/{id}/Images/{kind}",
            web::head().to(handlers::item_image),
        )
        // Playback
        .route(
            "/Items/{id}/PlaybackInfo",
            web::get().to(handlers::playback_info),
        )
        .route(
            "/Items/{id}/PlaybackInfo",
            web::post().to(handlers::playback_info),
        )
        .route("/Audio/{id}/stream", web::get().to(handlers::audio_stream))
        .route("/Audio/{id}/stream", web::head().to(handlers::audio_stream))
        .route(
            "/Audio/{id}/stream.{ext}",
            web::get().to(handlers::audio_stream_ext),
        )
        .route(
            "/Audio/{id}/stream.{ext}",
            web::head().to(handlers::audio_stream_ext),
        )
        .route(
            "/Audio/{id}/universal",
            web::get().to(handlers::audio_universal),
        )
        .route(
            "/Audio/{id}/universal",
            web::head().to(handlers::audio_universal),
        )
        // Sessions / scrobble
        .route("/Sessions", web::get().to(handlers::sessions_list))
        .route(
            "/Sessions/Capabilities/Full",
            web::post().to(handlers::sessions_capabilities),
        )
        .route(
            "/Sessions/Playing",
            web::post().to(handlers::sessions_playing),
        )
        .route(
            "/Sessions/Playing/Progress",
            web::post().to(handlers::sessions_playing_progress),
        )
        .route(
            "/Sessions/Playing/Stopped",
            web::post().to(handlers::sessions_playing_stopped),
        )
        .route(
            "/Users/{uid}/PlayedItems/{id}",
            web::post().to(handlers::user_played_item),
        )
        .route(
            "/Users/{uid}/PlayedItems/{id}",
            web::delete().to(handlers::user_played_item),
        )
        // ScheduledTasks
        .route("/ScheduledTasks", web::get().to(handlers::empty_array))
        .route(
            "/ScheduledTasks/Running/{id}",
            web::post().to(handlers::no_content),
        )
        .route(
            "/ScheduledTasks/Running/{id}",
            web::delete().to(handlers::no_content),
        )
        // Common probes
        .route(
            "/DisplayPreferences/{id}",
            web::get().to(handlers::displaypreferences),
        )
        .route("/Playlists", web::get().to(handlers::empty_items))
        .route("/Genres", web::get().to(handlers::empty_items))
        .route("/MusicGenres", web::get().to(handlers::empty_items));
}
