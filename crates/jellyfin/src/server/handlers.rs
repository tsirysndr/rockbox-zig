//! Jellyfin endpoint handlers. Audio-only — this server is built on top of
//! rockbox-library's track/album/artist tables.

use actix_web::{web, HttpRequest, HttpResponse};
use chrono::Utc;
use rockbox_library::entity::{album::Album, artist::Artist, track::Track};
use rockbox_library::repo;
use serde::Deserialize;
use serde_json::{json, Value};
use std::path::PathBuf;

use super::auth::{self, AuthedUser, EmbyAuth};
use super::dto::{
    AuthenticationResult, BaseItemDto, ImageBlurHashes, ImageTags, ItemsResult, MediaSource,
    MediaStream, NameGuidPair, PlaybackInfoResponse, PublicSystemInfo, SessionInfoDto, SystemInfo,
    UserConfiguration, UserDto, UserItemDataDto, UserPolicy, ViewsResult, JELLYFIN_API_VERSION,
};
use super::mapping;
use super::JellyfinState;

const TICKS_PER_MS: i64 = 10_000;
const TICKS_PER_SEC: i64 = TICKS_PER_MS * 1000;

fn now_iso() -> String {
    let now = Utc::now();
    let ticks = now.timestamp_subsec_nanos() / 100;
    format!("{}.{ticks:07}", now.format("%Y-%m-%dT%H:%M:%S"))
}

fn parse_auth(req: &HttpRequest) -> EmbyAuth {
    for name in ["x-emby-authorization", "authorization"] {
        if let Some(v) = req.headers().get(name) {
            if let Ok(s) = v.to_str() {
                return auth::parse_emby_auth_header(s);
            }
        }
    }
    EmbyAuth::default()
}

fn server_base(state: &JellyfinState, req: &HttpRequest) -> String {
    if let Some(h) = req.headers().get("host").and_then(|v| v.to_str().ok()) {
        let scheme = req.connection_info().scheme().to_string();
        return format!("{scheme}://{h}");
    }
    format!("http://0.0.0.0:{}", state.port)
}

// ── Root index ────────────────────────────────────────────────────────────────

pub async fn index(state: web::Data<JellyfinState>) -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/plain; charset=utf-8")
        .body(format!(
            "rockbox-jellyfin — Jellyfin-compatible API\nserver: {} ({})\n",
            state.server_name, state.server_id
        ))
}

// ── System ───────────────────────────────────────────────────────────────────

fn jellyfin_os_name() -> &'static str {
    match std::env::consts::OS {
        "macos" => "Darwin",
        "linux" => "Linux",
        "windows" => "Windows",
        "freebsd" => "FreeBSD",
        "openbsd" => "OpenBSD",
        "netbsd" => "NetBSD",
        "android" => "Android",
        "ios" => "iOS",
        other => other,
    }
}

fn public_info(state: &JellyfinState, req: &HttpRequest) -> PublicSystemInfo {
    PublicSystemInfo {
        local_address: Some(server_base(state, req)),
        server_name: Some(state.server_name.clone()),
        version: Some(JELLYFIN_API_VERSION.to_string()),
        product_name: Some("Jellyfin Server".to_string()),
        operating_system: Some(jellyfin_os_name().to_string()),
        id: Some(state.server_id.clone()),
        startup_wizard_completed: Some(true),
    }
}

pub async fn system_info_public(state: web::Data<JellyfinState>, req: HttpRequest) -> HttpResponse {
    HttpResponse::Ok().json(public_info(&state, &req))
}

pub async fn system_info(
    _user: AuthedUser,
    state: web::Data<JellyfinState>,
    req: HttpRequest,
) -> HttpResponse {
    let pub_info = public_info(&state, &req);
    HttpResponse::Ok().json(SystemInfo {
        local_address: pub_info.local_address,
        server_name: pub_info.server_name,
        version: pub_info.version,
        product_name: pub_info.product_name,
        operating_system: pub_info.operating_system,
        id: pub_info.id,
        startup_wizard_completed: pub_info.startup_wizard_completed,
        operating_system_display_name: Some(jellyfin_os_name().to_string()),
        package_name: Some("rockbox".to_string()),
        has_pending_restart: false,
        is_shutting_down: false,
        supports_library_monitor: true,
        web_socket_port_number: state.port as i32,
        completed_installations: Some(vec![]),
        can_self_restart: false,
        can_launch_web_browser: false,
        program_data_path: Some(String::new()),
        web_path: Some(String::new()),
        items_by_name_path: Some(String::new()),
        cache_path: Some(String::new()),
        log_path: Some(String::new()),
        internal_metadata_path: Some(String::new()),
        transcoding_temp_path: Some(String::new()),
        cast_receiver_applications: Some(vec![]),
        has_update_available: false,
        encoder_location: Some("Default".to_string()),
        system_architecture: Some(std::env::consts::ARCH.to_string()),
    })
}

pub async fn system_endpoint(_user: AuthedUser, req: HttpRequest) -> HttpResponse {
    let info = req.connection_info();
    let addr = info.realip_remote_addr().unwrap_or("");
    HttpResponse::Ok().json(json!({
        "IsLocal": true,
        "IsInNetwork": true,
        "RemoteAddress": addr,
    }))
}

// ── Users / Auth ─────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct AuthenticateBody {
    pub username: Option<String>,
    pub pw: Option<String>,
    pub password: Option<String>,
}

fn build_user(state: &JellyfinState) -> UserDto {
    UserDto {
        name: Some(state.username.as_str().to_string()),
        server_id: Some(state.server_id.clone()),
        server_name: Some(state.server_name.clone()),
        id: state.user_id.as_str().to_string(),
        primary_image_tag: None,
        primary_image_aspect_ratio: None,
        has_password: Some(true),
        has_configured_password: Some(true),
        has_configured_easy_password: Some(false),
        enable_auto_login: Some(false),
        last_login_date: Some(now_iso()),
        last_activity_date: Some(now_iso()),
        configuration: Some(UserConfiguration::default()),
        policy: Some(UserPolicy::admin()),
    }
}

pub async fn authenticate_by_name(
    state: web::Data<JellyfinState>,
    req: HttpRequest,
    body: web::Json<AuthenticateBody>,
) -> HttpResponse {
    let body = body.into_inner();
    let username = body.username.unwrap_or_default();
    let password = body.pw.or(body.password).unwrap_or_default();

    if username != *state.username || password != *state.password {
        return HttpResponse::Unauthorized()
            .json(json!({"Message": "Invalid username or password"}));
    }

    let parsed = parse_auth(&req);
    let token = auth::random_hex(16);
    let now = now_iso();
    if let Err(e) =
        auth::store_token(&state.pool, &token, state.user_id.as_str(), &parsed, &now).await
    {
        tracing::error!("jellyfin: store_token: {e}");
        return HttpResponse::InternalServerError().finish();
    }

    let user = build_user(&state);
    let session = SessionInfoDto {
        play_state: None,
        additional_users: Some(vec![]),
        capabilities: None,
        remote_end_point: Some(
            req.connection_info()
                .realip_remote_addr()
                .unwrap_or("")
                .to_string(),
        ),
        playable_media_types: vec!["Audio".into()],
        id: Some(auth::random_hex(16)),
        user_id: user.id.clone(),
        user_name: user.name.clone(),
        client: parsed.client.clone(),
        last_activity_date: now.clone(),
        last_playback_check_in: now.clone(),
        last_paused_date: None,
        device_name: parsed.device.clone(),
        device_type: None,
        now_playing_item: None,
        now_viewing_item: None,
        device_id: parsed.device_id.clone(),
        application_version: parsed.version.clone(),
        transcoding_info: None,
        is_active: true,
        supports_media_control: false,
        supports_remote_control: false,
        now_playing_queue: Some(vec![]),
        has_custom_device_name: false,
        playlist_item_id: None,
        server_id: Some(state.server_id.clone()),
        user_primary_image_tag: None,
        supported_commands: vec![],
    };

    HttpResponse::Ok().json(AuthenticationResult {
        user: Some(user),
        session_info: Some(session),
        access_token: Some(token),
        server_id: Some(state.server_id.clone()),
    })
}

pub async fn users_public(state: web::Data<JellyfinState>) -> HttpResponse {
    HttpResponse::Ok().json(vec![build_user(&state)])
}

pub async fn users_list(_user: AuthedUser, state: web::Data<JellyfinState>) -> HttpResponse {
    HttpResponse::Ok().json(vec![build_user(&state)])
}

pub async fn users_me(_user: AuthedUser, state: web::Data<JellyfinState>) -> HttpResponse {
    HttpResponse::Ok().json(build_user(&state))
}

pub async fn user_by_id(
    _user: AuthedUser,
    state: web::Data<JellyfinState>,
    _path: web::Path<String>,
) -> HttpResponse {
    HttpResponse::Ok().json(build_user(&state))
}

// ── Views (top-level library list) ───────────────────────────────────────────

fn music_library_view(state: &JellyfinState) -> BaseItemDto {
    BaseItemDto {
        id: mapping::library_guid(),
        server_id: Some(state.server_id.clone()),
        name: Some("Music".to_string()),
        item_type: "CollectionFolder",
        media_type: "Unknown",
        is_folder: Some(true),
        collection_type: Some("music"),
        location_type: Some("FileSystem"),
        ..Default::default()
    }
}

fn all_library_views(state: &JellyfinState) -> Vec<BaseItemDto> {
    vec![music_library_view(state)]
}

pub async fn user_views(
    _user: AuthedUser,
    state: web::Data<JellyfinState>,
    _path: web::Path<String>,
) -> HttpResponse {
    let views = all_library_views(&state);
    let total = views.len() as i32;
    HttpResponse::Ok().json(ViewsResult {
        items: views,
        total_record_count: total,
        start_index: 0,
    })
}

pub async fn user_views_query(_user: AuthedUser, state: web::Data<JellyfinState>) -> HttpResponse {
    let views = all_library_views(&state);
    let total = views.len() as i32;
    HttpResponse::Ok().json(ViewsResult {
        items: views,
        total_record_count: total,
        start_index: 0,
    })
}

pub async fn media_folders(_user: AuthedUser, state: web::Data<JellyfinState>) -> HttpResponse {
    let views = all_library_views(&state);
    let total = views.len() as i32;
    HttpResponse::Ok().json(ViewsResult {
        items: views,
        total_record_count: total,
        start_index: 0,
    })
}

pub async fn library_virtual_folders(
    _user: AuthedUser,
    state: web::Data<JellyfinState>,
) -> HttpResponse {
    HttpResponse::Ok().json(vec![json!({
        "Name": "Music",
        "Locations": [state.music_dir.to_string_lossy()],
        "CollectionType": "music",
        "ItemId": mapping::library_guid(),
    })])
}

// ── Items query (custom parser — handles repeated keys + Pascal/camelCase) ──

#[derive(Debug, Default)]
#[allow(dead_code)]
pub struct ItemsQuery {
    pub parent_id: Option<String>,
    pub include_item_types: Option<String>,
    pub recursive: Option<bool>,
    pub search_term: Option<String>,
    pub ids: Option<String>,
    pub album_artist_ids: Option<String>,
    pub artist_ids: Option<String>,
    pub album_ids: Option<String>,
    pub start_index: Option<i64>,
    pub limit: Option<i64>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
    pub user_id: Option<String>,
    pub enable_user_data: Option<bool>,
    pub enable_total_record_count: Option<bool>,
    pub enable_images: Option<bool>,
    pub fields: Option<String>,
}

fn collect_query(req: &HttpRequest) -> std::collections::HashMap<String, Vec<String>> {
    let mut out: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
    for pair in req.query_string().split('&').filter(|s| !s.is_empty()) {
        let mut it = pair.splitn(2, '=');
        let key = it.next().unwrap_or("");
        let val = it.next().unwrap_or("");
        let decoded = urlencoding::decode(val)
            .map(|s| s.into_owned())
            .unwrap_or_else(|_| val.to_string());
        out.entry(key.to_string()).or_default().push(decoded);
    }
    out
}

fn parse_items_query(req: &HttpRequest) -> ItemsQuery {
    let q = collect_query(req);
    let one = |k: &str| q.get(k).and_then(|v| v.first()).cloned();
    let csv = |k: &str| q.get(k).map(|v| v.join(","));
    let parse_bool = |k: &str| one(k).and_then(|s| s.parse::<bool>().ok());
    let parse_i64 = |k: &str| one(k).and_then(|s| s.parse::<i64>().ok());
    ItemsQuery {
        parent_id: one("parentId").or_else(|| one("ParentId")),
        include_item_types: csv("includeItemTypes").or_else(|| csv("IncludeItemTypes")),
        recursive: parse_bool("recursive"),
        search_term: one("searchTerm").or_else(|| one("SearchTerm")),
        ids: csv("ids").or_else(|| csv("Ids")),
        album_artist_ids: csv("albumArtistIds").or_else(|| csv("AlbumArtistIds")),
        artist_ids: csv("artistIds").or_else(|| csv("ArtistIds")),
        album_ids: csv("albumIds").or_else(|| csv("AlbumIds")),
        start_index: parse_i64("startIndex").or_else(|| parse_i64("StartIndex")),
        limit: parse_i64("limit").or_else(|| parse_i64("Limit")),
        sort_by: one("sortBy").or_else(|| one("SortBy")),
        sort_order: one("sortOrder").or_else(|| one("SortOrder")),
        user_id: one("userId").or_else(|| one("UserId")),
        enable_user_data: parse_bool("enableUserData"),
        enable_total_record_count: parse_bool("enableTotalRecordCount"),
        enable_images: parse_bool("enableImages"),
        fields: csv("fields").or_else(|| csv("Fields")),
    }
}

// ── Track/album/artist → BaseItemDto ────────────────────────────────────────

async fn artist_to_dto(state: &JellyfinState, a: &Artist) -> BaseItemDto {
    let id = mapping::remember_artist(&state.pool, a)
        .await
        .unwrap_or_else(|_| mapping::guid(mapping::KIND_ARTIST, &a.id));
    BaseItemDto {
        id,
        server_id: Some(state.server_id.clone()),
        name: Some(a.name.clone()),
        item_type: "MusicArtist",
        media_type: "Unknown",
        is_folder: Some(true),
        sort_name: Some(a.name.clone()),
        location_type: Some("FileSystem"),
        image_tags: Some(ImageTags {
            primary: a.image.clone().map(|_| a.id.clone()),
        }),
        ..Default::default()
    }
}

async fn album_to_dto(state: &JellyfinState, al: &Album) -> BaseItemDto {
    let id = mapping::remember_album(&state.pool, al)
        .await
        .unwrap_or_else(|_| mapping::guid(mapping::KIND_ALBUM, &al.id));
    let artist_guid = mapping::guid(mapping::KIND_ARTIST, &al.artist_id);
    let tracks = repo::album_tracks::find_by_album(state.pool.clone(), &al.id)
        .await
        .unwrap_or_default();
    let song_count = tracks.len() as i32;
    let duration_ms: i64 = tracks.iter().map(|t| t.length as i64).sum();
    BaseItemDto {
        id: id.clone(),
        server_id: Some(state.server_id.clone()),
        name: Some(al.title.clone()),
        item_type: "MusicAlbum",
        media_type: "Unknown",
        is_folder: Some(true),
        production_year: if al.year > 0 {
            Some(al.year as i32)
        } else {
            None
        },
        premiere_date: if al.year > 0 {
            Some(format!("{:04}-01-01T00:00:00.0000000", al.year))
        } else {
            None
        },
        album: Some(al.title.clone()),
        album_id: Some(id),
        album_artist: Some(al.artist.clone()),
        album_artists: Some(vec![NameGuidPair {
            name: Some(al.artist.clone()),
            id: artist_guid.clone(),
        }]),
        artist_items: Some(vec![NameGuidPair {
            name: Some(al.artist.clone()),
            id: artist_guid.clone(),
        }]),
        artists: Some(vec![al.artist.clone()]),
        parent_id: Some(artist_guid),
        sort_name: Some(al.title.clone()),
        run_time_ticks: Some(duration_ms * TICKS_PER_MS),
        song_count: Some(song_count),
        child_count: Some(song_count),
        location_type: Some("FileSystem"),
        image_tags: Some(ImageTags {
            primary: al.album_art.clone().map(|_| al.id.clone()),
        }),
        ..Default::default()
    }
}

async fn track_to_dto(state: &JellyfinState, t: &Track) -> BaseItemDto {
    let id = mapping::remember_track(&state.pool, t)
        .await
        .unwrap_or_else(|_| mapping::guid(mapping::KIND_TRACK, &t.id));
    let album_guid = mapping::guid(mapping::KIND_ALBUM, &t.album_id);
    let artist_guid = mapping::guid(mapping::KIND_ARTIST, &t.artist_id);
    // rockbox-library stores length in milliseconds.
    let run_time_ticks = (t.length as i64) * TICKS_PER_MS;
    let container = std::path::Path::new(&t.path)
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_ascii_lowercase())
        .unwrap_or_else(|| "mp3".to_string());

    let audio_stream = MediaStream {
        codec: Some(container.clone()),
        stream_type: "Audio",
        index: 0,
        is_default: true,
        channels: Some(2),
        sample_rate: Some(t.frequency as i32),
        bit_rate: Some((t.bitrate as i32) * 1000),
        video_range: "Unknown",
        video_range_type: "Unknown",
        audio_spatial_format: "None",
        is_interlaced: false,
        is_forced: false,
        is_hearing_impaired: false,
        is_original: false,
        is_external: false,
        is_text_subtitle_stream: false,
        supports_external_stream: false,
        ..Default::default()
    };

    let media_source = MediaSource {
        protocol: "File",
        id: Some(id.clone()),
        path: Some(t.path.clone()),
        source_type: "Default",
        container: Some(container.clone()),
        size: Some(t.filesize as i64),
        name: Some(t.title.clone()),
        is_remote: false,
        run_time_ticks: Some(run_time_ticks),
        read_at_native_framerate: false,
        ignore_dts: false,
        ignore_index: false,
        gen_pts_input: false,
        supports_transcoding: false,
        supports_direct_stream: true,
        supports_direct_play: true,
        is_infinite_stream: false,
        use_most_compatible_transcoding_profile: false,
        requires_opening: false,
        requires_closing: false,
        requires_looping: false,
        supports_probing: false,
        media_streams: Some(vec![audio_stream.clone()]),
        formats: None,
        bitrate: Some((t.bitrate as i32) * 1000),
        transcoding_sub_protocol: "http",
        default_audio_stream_index: Some(0),
        default_subtitle_stream_index: None,
        has_segments: false,
    };

    BaseItemDto {
        id: id.clone(),
        server_id: Some(state.server_id.clone()),
        name: Some(t.title.clone()),
        item_type: "Audio",
        media_type: "Audio",
        is_folder: Some(false),
        production_year: t.year.map(|y| y as i32),
        index_number: t.track_number.map(|n| n as i32),
        parent_index_number: Some(t.disc_number as i32),
        run_time_ticks: Some(run_time_ticks),
        container: Some(container),
        path: Some(t.path.clone()),
        album: Some(t.album.clone()),
        album_id: Some(album_guid.clone()),
        album_artist: Some(t.album_artist.clone()),
        album_artists: Some(vec![NameGuidPair {
            name: Some(t.album_artist.clone()),
            id: artist_guid.clone(),
        }]),
        artist_items: Some(vec![NameGuidPair {
            name: Some(t.artist.clone()),
            id: artist_guid.clone(),
        }]),
        artists: Some(vec![t.artist.clone()]),
        parent_id: Some(album_guid),
        genres: t.genre.as_ref().map(|g| vec![g.clone()]),
        location_type: Some("FileSystem"),
        media_sources: Some(vec![media_source.clone()]),
        media_source_count: Some(1),
        media_streams: Some(vec![audio_stream]),
        image_tags: Some(ImageTags {
            primary: t.album_art.clone().map(|_| t.album_id.clone()),
        }),
        image_blur_hashes: Some(ImageBlurHashes::default()),
        user_data: Some(UserItemDataDto {
            rating: None,
            played_percentage: None,
            unplayed_item_count: None,
            playback_position_ticks: 0,
            play_count: 0,
            is_favorite: false,
            likes: None,
            last_played_date: None,
            played: false,
            key: id.clone(),
            item_id: id,
        }),
        ..Default::default()
    }
}

fn includes(types_csv: &Option<String>, want: &str) -> bool {
    match types_csv {
        None => false,
        Some(s) => s.split(',').any(|p| p.trim().eq_ignore_ascii_case(want)),
    }
}

async fn resolve_native(state: &JellyfinState, guid: &str) -> Option<(String, String)> {
    mapping::lookup(&state.pool, guid).await.ok().flatten()
}

// ── Items endpoints ─────────────────────────────────────────────────────────

pub async fn items(
    _user: AuthedUser,
    state: web::Data<JellyfinState>,
    req: HttpRequest,
) -> HttpResponse {
    items_impl(state, parse_items_query(&req)).await
}

pub async fn user_items(
    _user: AuthedUser,
    state: web::Data<JellyfinState>,
    _path: web::Path<String>,
    req: HttpRequest,
) -> HttpResponse {
    items_impl(state, parse_items_query(&req)).await
}

async fn items_impl(state: web::Data<JellyfinState>, q: ItemsQuery) -> HttpResponse {
    let library_id = mapping::library_guid();

    // `/Items?userId=X` with no filters = list libraries.
    if q.parent_id.is_none()
        && q.ids.is_none()
        && q.search_term.is_none()
        && q.include_item_types.is_none()
        && q.album_artist_ids.is_none()
        && q.artist_ids.is_none()
        && q.album_ids.is_none()
    {
        let views = all_library_views(&state);
        let total = views.len() as i32;
        return HttpResponse::Ok().json(ItemsResult {
            items: views,
            total_record_count: total,
            start_index: 0,
        });
    }

    // Free-text search.
    if let Some(term) = q.search_term.as_deref().filter(|s| !s.is_empty()) {
        let limit = q.limit.unwrap_or(50).max(1);
        let no_filter = q.include_item_types.is_none();
        let pattern = like_pattern(term);
        let mut dtos: Vec<BaseItemDto> = Vec::new();
        if no_filter || includes(&q.include_item_types, "MusicArtist") {
            let where_clause = ("name LIKE ?1".to_string(), vec![pattern.clone()]);
            if let Ok(rows) = repo::artist::filter(state.pool.clone(), where_clause).await {
                for a in rows.into_iter().take(limit as usize) {
                    dtos.push(artist_to_dto(&state, &a).await);
                }
            }
        }
        if no_filter || includes(&q.include_item_types, "MusicAlbum") {
            let where_clause = ("title LIKE ?1".to_string(), vec![pattern.clone()]);
            if let Ok(rows) = repo::album::filter(state.pool.clone(), where_clause).await {
                for a in rows.into_iter().take(limit as usize) {
                    dtos.push(album_to_dto(&state, &a).await);
                }
            }
        }
        if no_filter || includes(&q.include_item_types, "Audio") {
            let where_clause = ("title LIKE ?1".to_string(), vec![pattern]);
            if let Ok(rows) = repo::track::filter(state.pool.clone(), where_clause).await {
                for t in rows.into_iter().take(limit as usize) {
                    dtos.push(track_to_dto(&state, &t).await);
                }
            }
        }
        let total = dtos.len() as i32;
        return HttpResponse::Ok().json(ItemsResult {
            items: dtos,
            total_record_count: total,
            start_index: 0,
        });
    }

    // Explicit Ids= lookup.
    if let Some(ids) = q.ids.as_ref() {
        let mut out: Vec<BaseItemDto> = Vec::new();
        for raw in ids.split(',') {
            let g = mapping::normalize_guid(raw.trim());
            if let Some((kind, native)) = resolve_native(&state, &g).await {
                match kind.as_str() {
                    "track" => {
                        if let Ok(Some(t)) = repo::track::find(state.pool.clone(), &native).await {
                            out.push(track_to_dto(&state, &t).await);
                        }
                    }
                    "album" => {
                        if let Ok(Some(a)) = repo::album::find(state.pool.clone(), &native).await {
                            out.push(album_to_dto(&state, &a).await);
                        }
                    }
                    "artist" => {
                        if let Ok(Some(a)) = repo::artist::find(state.pool.clone(), &native).await {
                            out.push(artist_to_dto(&state, &a).await);
                        }
                    }
                    _ => {}
                }
            }
        }
        let total = out.len() as i32;
        return HttpResponse::Ok().json(ItemsResult {
            items: out,
            total_record_count: total,
            start_index: 0,
        });
    }

    // Filter by parent — artist parent → albums; album parent → tracks.
    if let Some(parent) = q.parent_id.as_ref() {
        let g = mapping::normalize_guid(parent);
        if g == library_id {
            return list_artists_or_albums_or_tracks(&state, &q).await;
        }
        if let Some((kind, native)) = resolve_native(&state, &g).await {
            match kind.as_str() {
                "artist" => {
                    let albums = repo::album::find_by_artist(state.pool.clone(), &native)
                        .await
                        .unwrap_or_default();
                    let mut dtos = Vec::with_capacity(albums.len());
                    for a in &albums {
                        dtos.push(album_to_dto(&state, a).await);
                    }
                    let total = dtos.len() as i32;
                    return HttpResponse::Ok().json(ItemsResult {
                        items: dtos,
                        total_record_count: total,
                        start_index: 0,
                    });
                }
                "album" => {
                    let tracks = repo::album_tracks::find_by_album(state.pool.clone(), &native)
                        .await
                        .unwrap_or_default();
                    let mut dtos = Vec::with_capacity(tracks.len());
                    for t in &tracks {
                        dtos.push(track_to_dto(&state, t).await);
                    }
                    let total = dtos.len() as i32;
                    return HttpResponse::Ok().json(ItemsResult {
                        items: dtos,
                        total_record_count: total,
                        start_index: 0,
                    });
                }
                _ => {}
            }
        }
        return HttpResponse::Ok().json(ItemsResult {
            items: vec![],
            total_record_count: 0,
            start_index: 0,
        });
    }

    // No parent: filter by artistIds / albumArtistIds.
    let artist_filter = q.album_artist_ids.clone().or_else(|| q.artist_ids.clone());

    if includes(&q.include_item_types, "Audio") {
        if let Some(filter) = artist_filter.clone() {
            if let Some(first) = filter.split(',').next() {
                let g = mapping::normalize_guid(first);
                if let Some((kind, native)) = resolve_native(&state, &g).await {
                    if kind == "artist" {
                        let tracks =
                            repo::artist_tracks::find_by_artist(state.pool.clone(), &native)
                                .await
                                .unwrap_or_default();
                        let mut dtos = Vec::with_capacity(tracks.len());
                        for t in &tracks {
                            dtos.push(track_to_dto(&state, t).await);
                        }
                        let total = dtos.len() as i32;
                        return HttpResponse::Ok().json(ItemsResult {
                            items: dtos,
                            total_record_count: total,
                            start_index: 0,
                        });
                    }
                }
            }
        }
        // Plain all-songs.
        return list_artists_or_albums_or_tracks(&state, &q).await;
    }

    if includes(&q.include_item_types, "MusicAlbum") {
        if let Some(filter) = artist_filter {
            if let Some(first) = filter.split(',').next() {
                let g = mapping::normalize_guid(first);
                if let Some((kind, native)) = resolve_native(&state, &g).await {
                    if kind == "artist" {
                        let albums = repo::album::find_by_artist(state.pool.clone(), &native)
                            .await
                            .unwrap_or_default();
                        let mut dtos = Vec::with_capacity(albums.len());
                        for a in &albums {
                            dtos.push(album_to_dto(&state, a).await);
                        }
                        let total = dtos.len() as i32;
                        return HttpResponse::Ok().json(ItemsResult {
                            items: dtos,
                            total_record_count: total,
                            start_index: 0,
                        });
                    }
                }
            }
        }
        return list_artists_or_albums_or_tracks(&state, &q).await;
    }

    list_artists_or_albums_or_tracks(&state, &q).await
}

async fn list_artists_or_albums_or_tracks(state: &JellyfinState, q: &ItemsQuery) -> HttpResponse {
    let limit = q.limit.unwrap_or(100).max(1) as usize;
    let offset = q.start_index.unwrap_or(0).max(0) as usize;

    if includes(&q.include_item_types, "Audio") {
        let mut tracks = repo::track::all(state.pool.clone())
            .await
            .unwrap_or_default();
        let total = tracks.len() as i32;
        if offset < tracks.len() {
            tracks = tracks.split_off(offset);
        } else {
            tracks.clear();
        }
        tracks.truncate(limit);
        let mut dtos = Vec::with_capacity(tracks.len());
        for t in &tracks {
            dtos.push(track_to_dto(state, t).await);
        }
        return HttpResponse::Ok().json(ItemsResult {
            items: dtos,
            total_record_count: total,
            start_index: offset as i32,
        });
    }
    if includes(&q.include_item_types, "MusicAlbum") {
        let mut albums = repo::album::all(state.pool.clone())
            .await
            .unwrap_or_default();
        let total = albums.len() as i32;
        if offset < albums.len() {
            albums = albums.split_off(offset);
        } else {
            albums.clear();
        }
        albums.truncate(limit);
        let mut dtos = Vec::with_capacity(albums.len());
        for a in &albums {
            dtos.push(album_to_dto(state, a).await);
        }
        return HttpResponse::Ok().json(ItemsResult {
            items: dtos,
            total_record_count: total,
            start_index: offset as i32,
        });
    }
    let artists = repo::artist::all(state.pool.clone())
        .await
        .unwrap_or_default();
    let mut dtos = Vec::with_capacity(artists.len());
    for a in &artists {
        dtos.push(artist_to_dto(state, a).await);
    }
    let total = dtos.len() as i32;
    HttpResponse::Ok().json(ItemsResult {
        items: dtos,
        total_record_count: total,
        start_index: 0,
    })
}

fn like_pattern(term: &str) -> String {
    let escaped = term.replace('%', "\\%").replace('_', "\\_");
    format!("%{}%", escaped)
}

pub async fn item_by_id(
    _user: AuthedUser,
    state: web::Data<JellyfinState>,
    path: web::Path<String>,
) -> HttpResponse {
    let g = mapping::normalize_guid(&path.into_inner());
    let Some((kind, native)) = resolve_native(&state, &g).await else {
        return HttpResponse::NotFound().finish();
    };
    match kind.as_str() {
        "track" => match repo::track::find(state.pool.clone(), &native).await {
            Ok(Some(t)) => HttpResponse::Ok().json(track_to_dto(&state, &t).await),
            _ => HttpResponse::NotFound().finish(),
        },
        "album" => match repo::album::find(state.pool.clone(), &native).await {
            Ok(Some(a)) => HttpResponse::Ok().json(album_to_dto(&state, &a).await),
            _ => HttpResponse::NotFound().finish(),
        },
        "artist" => match repo::artist::find(state.pool.clone(), &native).await {
            Ok(Some(a)) => HttpResponse::Ok().json(artist_to_dto(&state, &a).await),
            _ => HttpResponse::NotFound().finish(),
        },
        _ => HttpResponse::NotFound().finish(),
    }
}

pub async fn user_item_by_id(
    _user: AuthedUser,
    state: web::Data<JellyfinState>,
    path: web::Path<(String, String)>,
) -> HttpResponse {
    let (_user_id, item_id) = path.into_inner();
    let g = mapping::normalize_guid(&item_id);
    let Some((kind, native)) = resolve_native(&state, &g).await else {
        return HttpResponse::NotFound().finish();
    };
    match kind.as_str() {
        "track" => match repo::track::find(state.pool.clone(), &native).await {
            Ok(Some(t)) => HttpResponse::Ok().json(track_to_dto(&state, &t).await),
            _ => HttpResponse::NotFound().finish(),
        },
        "album" => match repo::album::find(state.pool.clone(), &native).await {
            Ok(Some(a)) => HttpResponse::Ok().json(album_to_dto(&state, &a).await),
            _ => HttpResponse::NotFound().finish(),
        },
        "artist" => match repo::artist::find(state.pool.clone(), &native).await {
            Ok(Some(a)) => HttpResponse::Ok().json(artist_to_dto(&state, &a).await),
            _ => HttpResponse::NotFound().finish(),
        },
        _ => HttpResponse::NotFound().finish(),
    }
}

// ── Artists endpoints ───────────────────────────────────────────────────────

pub async fn artists(
    _user: AuthedUser,
    state: web::Data<JellyfinState>,
    _req: HttpRequest,
) -> HttpResponse {
    let artists = repo::artist::all(state.pool.clone())
        .await
        .unwrap_or_default();
    let mut dtos = Vec::with_capacity(artists.len());
    for a in &artists {
        dtos.push(artist_to_dto(&state, a).await);
    }
    let total = dtos.len() as i32;
    HttpResponse::Ok().json(ItemsResult {
        items: dtos,
        total_record_count: total,
        start_index: 0,
    })
}

pub async fn artist_by_name(
    _user: AuthedUser,
    state: web::Data<JellyfinState>,
    path: web::Path<String>,
) -> HttpResponse {
    let name = path.into_inner();
    if let Ok(Some(a)) = repo::artist::find_by_name(state.pool.clone(), &name).await {
        HttpResponse::Ok().json(artist_to_dto(&state, &a).await)
    } else {
        HttpResponse::NotFound().finish()
    }
}

// ── Images ───────────────────────────────────────────────────────────────────

/// Resolve a Jellyfin item GUID to a stored `album_art` / `image` value.
/// The returned string is one of:
///   - a bare filename like `"abc123.jpg"` → lives under `~/.config/rockbox.org/covers/`
///   - an absolute filesystem path starting with `/`
///   - an `http(s)://` URL (artist images sourced from Rocksky)
async fn resolve_image_value(state: &JellyfinState, guid: &str) -> Option<String> {
    let g = mapping::normalize_guid(guid);
    let (kind, native) = mapping::lookup(&state.pool, &g).await.ok().flatten()?;
    match kind.as_str() {
        "album" => album_art_value(state, &native).await,
        "track" => {
            let t = repo::track::find(state.pool.clone(), &native)
                .await
                .ok()
                .flatten()?;
            if let Some(art) = t.album_art.clone() {
                return Some(art);
            }
            if !t.album_id.is_empty() {
                if let Some(a) = album_art_value(state, &t.album_id).await {
                    return Some(a);
                }
            }
            None
        }
        "artist" => repo::artist::find(state.pool.clone(), &native)
            .await
            .ok()
            .flatten()
            .and_then(|a| a.image),
        _ => None,
    }
}

/// Find album art either on the album row directly, or on any of its tracks.
async fn album_art_value(state: &JellyfinState, album_id: &str) -> Option<String> {
    if let Ok(Some(album)) = repo::album::find(state.pool.clone(), album_id).await {
        if let Some(art) = album.album_art {
            return Some(art);
        }
    }
    repo::album_tracks::find_by_album(state.pool.clone(), album_id)
        .await
        .ok()?
        .into_iter()
        .find_map(|t| t.album_art)
}

pub async fn item_image(
    state: web::Data<JellyfinState>,
    path: web::Path<(String, String)>,
) -> HttpResponse {
    let (item_guid, _kind) = path.into_inner();
    serve_image(&state, &item_guid).await
}

pub async fn item_image_by_index(
    state: web::Data<JellyfinState>,
    path: web::Path<(String, String, i32)>,
) -> HttpResponse {
    let (item_guid, _kind, _idx) = path.into_inner();
    serve_image(&state, &item_guid).await
}

async fn serve_image(state: &JellyfinState, item_guid: &str) -> HttpResponse {
    let Some(art) = resolve_image_value(state, item_guid).await else {
        return HttpResponse::NotFound().finish();
    };
    serve_art_value(&art).await
}

async fn serve_art_value(art: &str) -> HttpResponse {
    // Remote artist images (Rocksky) — proxy via reqwest.
    if art.starts_with("http://") || art.starts_with("https://") {
        match reqwest::get(art).await {
            Ok(resp) => {
                let mime = resp
                    .headers()
                    .get("content-type")
                    .and_then(|v| v.to_str().ok())
                    .unwrap_or("image/jpeg")
                    .to_string();
                match resp.bytes().await {
                    Ok(data) => HttpResponse::Ok().content_type(mime).body(data.to_vec()),
                    Err(_) => HttpResponse::NotFound().finish(),
                }
            }
            Err(e) => {
                tracing::warn!("jellyfin image proxy {art}: {e}");
                HttpResponse::NotFound().finish()
            }
        }
    } else {
        // Bare filename → resolve under ~/.config/rockbox.org/covers/.
        // Absolute path → use as-is.
        let full = if art.starts_with('/') {
            art.to_string()
        } else {
            let home = std::env::var("HOME").unwrap_or_default();
            format!("{}/.config/rockbox.org/covers/{}", home, art)
        };
        match std::fs::read(&full) {
            Ok(data) => {
                let mime = mime_guess::from_path(&full)
                    .first_or_octet_stream()
                    .to_string();
                HttpResponse::Ok().content_type(mime).body(data)
            }
            Err(e) => {
                tracing::warn!("jellyfin image read {full}: {e}");
                HttpResponse::NotFound().finish()
            }
        }
    }
}

// ── PlaybackInfo + stream ───────────────────────────────────────────────────

pub async fn playback_info(
    _user: AuthedUser,
    state: web::Data<JellyfinState>,
    path: web::Path<String>,
) -> HttpResponse {
    let g = mapping::normalize_guid(&path.into_inner());
    let Some((kind, native)) = resolve_native(&state, &g).await else {
        return HttpResponse::NotFound().finish();
    };
    if kind != "track" {
        return HttpResponse::BadRequest().finish();
    }
    let Ok(Some(t)) = repo::track::find(state.pool.clone(), &native).await else {
        return HttpResponse::NotFound().finish();
    };
    let dto = track_to_dto(&state, &t).await;
    HttpResponse::Ok().json(PlaybackInfoResponse {
        media_sources: dto.media_sources.unwrap_or_default(),
        play_session_id: Some(auth::random_hex(8)),
    })
}

fn content_type_for(container: &str) -> &'static str {
    match container {
        "mp3" => "audio/mpeg",
        "flac" => "audio/flac",
        "ogg" | "oga" => "audio/ogg",
        "opus" => "audio/opus",
        "m4a" | "aac" | "mp4" => "audio/mp4",
        "wav" => "audio/wav",
        _ => "application/octet-stream",
    }
}

async fn audio_by_guid(state: &JellyfinState, guid: &str, req: &HttpRequest) -> HttpResponse {
    let token = auth::extract_token(req);
    let authorized = match token {
        Some(t) => auth::token_valid(&state.pool, &t).await,
        None => false,
    };
    if !authorized {
        return HttpResponse::Unauthorized().finish();
    }
    let g = mapping::normalize_guid(guid);
    let Some((kind, native)) = resolve_native(state, &g).await else {
        return HttpResponse::NotFound().finish();
    };
    if kind != "track" {
        return HttpResponse::BadRequest().finish();
    }
    let Ok(Some(t)) = repo::track::find(state.pool.clone(), &native).await else {
        return HttpResponse::NotFound().finish();
    };
    let container = std::path::Path::new(&t.path)
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_ascii_lowercase())
        .unwrap_or_else(|| "mp3".to_string());
    serve_file(&t.path, content_type_for(&container), req)
}

pub async fn audio_stream(
    state: web::Data<JellyfinState>,
    path: web::Path<String>,
    req: HttpRequest,
) -> HttpResponse {
    audio_by_guid(&state, &path.into_inner(), &req).await
}

pub async fn audio_stream_ext(
    state: web::Data<JellyfinState>,
    path: web::Path<(String, String)>,
    req: HttpRequest,
) -> HttpResponse {
    let (id, _ext) = path.into_inner();
    audio_by_guid(&state, &id, &req).await
}

pub async fn audio_universal(
    state: web::Data<JellyfinState>,
    path: web::Path<String>,
    req: HttpRequest,
) -> HttpResponse {
    audio_by_guid(&state, &path.into_inner(), &req).await
}

pub async fn item_file_stream(
    state: web::Data<JellyfinState>,
    path: web::Path<String>,
    req: HttpRequest,
) -> HttpResponse {
    audio_by_guid(&state, &path.into_inner(), &req).await
}

fn serve_file(path_str: &str, content_type: &str, req: &HttpRequest) -> HttpResponse {
    let path = PathBuf::from(path_str);
    let file_size = match std::fs::metadata(&path) {
        Ok(m) => m.len(),
        Err(e) => {
            tracing::error!("jellyfin stream stat {path_str}: {e}");
            return HttpResponse::InternalServerError().finish();
        }
    };

    if let Some(range_hdr) = req.headers().get(actix_web::http::header::RANGE) {
        if let Ok(range_str) = range_hdr.to_str() {
            if let Some(range) = range_str.strip_prefix("bytes=") {
                let mut parts = range.splitn(2, '-');
                let start: u64 = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0);
                let end: u64 = parts
                    .next()
                    .filter(|s| !s.is_empty())
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(file_size.saturating_sub(1))
                    .min(file_size.saturating_sub(1));
                if start <= end && file_size > 0 {
                    use std::io::{Read, Seek, SeekFrom};
                    return match std::fs::File::open(&path) {
                        Ok(mut file) => {
                            let _ = file.seek(SeekFrom::Start(start));
                            let length = (end - start + 1) as usize;
                            let mut buf = vec![0u8; length];
                            let n = file.read(&mut buf).unwrap_or(0);
                            buf.truncate(n);
                            let actual_end = start + n as u64 - 1;
                            HttpResponse::PartialContent()
                                .content_type(content_type.to_string())
                                .insert_header(("Accept-Ranges", "bytes"))
                                .insert_header(("Content-Length", n.to_string()))
                                .insert_header((
                                    "Content-Range",
                                    format!("bytes {start}-{actual_end}/{file_size}"),
                                ))
                                .body(buf)
                        }
                        Err(e) => {
                            tracing::error!("jellyfin stream open {path_str}: {e}");
                            HttpResponse::InternalServerError().finish()
                        }
                    };
                }
            }
        }
    }

    match std::fs::read(&path) {
        Ok(data) => HttpResponse::Ok()
            .content_type(content_type.to_string())
            .insert_header(("Accept-Ranges", "bytes"))
            .insert_header(("Content-Length", file_size.to_string()))
            .body(data),
        Err(e) => {
            tracing::error!("jellyfin stream read {path_str}: {e}");
            HttpResponse::InternalServerError().finish()
        }
    }
}

// ── Sessions / scrobble ──────────────────────────────────────────────────────

pub async fn sessions_capabilities(_user: AuthedUser) -> HttpResponse {
    HttpResponse::NoContent().finish()
}

pub async fn sessions_list(_user: AuthedUser) -> HttpResponse {
    HttpResponse::Ok().json(Vec::<Value>::new())
}

pub async fn sessions_playing(_user: AuthedUser, _body: web::Json<Value>) -> HttpResponse {
    HttpResponse::NoContent().finish()
}

pub async fn sessions_playing_progress(_user: AuthedUser, _body: web::Json<Value>) -> HttpResponse {
    HttpResponse::NoContent().finish()
}

pub async fn sessions_playing_stopped(_user: AuthedUser, _body: web::Json<Value>) -> HttpResponse {
    HttpResponse::NoContent().finish()
}

pub async fn user_played_item(
    _user: AuthedUser,
    _state: web::Data<JellyfinState>,
    _path: web::Path<(String, String)>,
) -> HttpResponse {
    HttpResponse::NoContent().finish()
}

// ── Misc stubs ──────────────────────────────────────────────────────────────

pub async fn empty_array() -> HttpResponse {
    HttpResponse::Ok().json(Vec::<Value>::new())
}

pub async fn empty_items() -> HttpResponse {
    HttpResponse::Ok().json(ItemsResult {
        items: vec![],
        total_record_count: 0,
        start_index: 0,
    })
}

pub async fn no_content() -> HttpResponse {
    HttpResponse::NoContent().finish()
}

pub async fn branding_config() -> HttpResponse {
    HttpResponse::Ok().json(json!({
        "LoginDisclaimer": "",
        "CustomCss": "",
        "SplashscreenEnabled": false,
    }))
}

pub async fn displaypreferences(_user: AuthedUser) -> HttpResponse {
    HttpResponse::Ok().json(json!({
        "Id": "",
        "ViewType": "",
        "SortBy": "SortName",
        "SortOrder": "Ascending",
        "RememberIndexing": false,
        "PrimaryImageHeight": 250,
        "PrimaryImageWidth": 250,
        "CustomPrefs": {},
        "ScrollDirection": "Vertical",
        "ShowBackdrop": true,
        "RememberSorting": false,
        "IndexBy": "",
        "ShowSidebar": false,
        "Client": ""
    }))
}

// ── /Items/Latest ────────────────────────────────────────────────────────────

pub async fn items_latest(
    _user: AuthedUser,
    state: web::Data<JellyfinState>,
    req: HttpRequest,
) -> HttpResponse {
    let q = parse_items_query(&req);
    let limit = q.limit.unwrap_or(16).max(1) as usize;
    let mut albums = repo::album::all(state.pool.clone())
        .await
        .unwrap_or_default();
    albums.truncate(limit);
    let mut dtos = Vec::with_capacity(albums.len());
    for a in &albums {
        dtos.push(album_to_dto(&state, a).await);
    }
    HttpResponse::Ok().json(dtos)
}

// ── /Search/Hints ────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct SearchHintsQuery {
    pub search_term: Option<String>,
    pub limit: Option<i64>,
    pub start_index: Option<i64>,
    pub include_item_types: Option<String>,
    pub user_id: Option<String>,
}

pub async fn search_hints(
    _user: AuthedUser,
    state: web::Data<JellyfinState>,
    query: web::Query<SearchHintsQuery>,
) -> HttpResponse {
    let q = query.into_inner();
    let Some(term) = q.search_term.as_deref().filter(|s| !s.is_empty()) else {
        return HttpResponse::Ok().json(json!({
            "SearchHints": [],
            "TotalRecordCount": 0,
        }));
    };
    let limit = q.limit.unwrap_or(20).max(1);
    let pattern = like_pattern(term);
    let mut hints: Vec<Value> = Vec::new();
    let want_artist = q
        .include_item_types
        .as_deref()
        .map(|s| {
            s.split(',')
                .any(|t| t.trim().eq_ignore_ascii_case("MusicArtist"))
        })
        .unwrap_or(true);
    let want_album = q
        .include_item_types
        .as_deref()
        .map(|s| {
            s.split(',')
                .any(|t| t.trim().eq_ignore_ascii_case("MusicAlbum"))
        })
        .unwrap_or(true);
    let want_audio = q
        .include_item_types
        .as_deref()
        .map(|s| s.split(',').any(|t| t.trim().eq_ignore_ascii_case("Audio")))
        .unwrap_or(true);

    if want_artist {
        let w = ("name LIKE ?1".to_string(), vec![pattern.clone()]);
        if let Ok(rows) = repo::artist::filter(state.pool.clone(), w).await {
            for a in rows.into_iter().take(limit as usize) {
                let id = mapping::remember_artist(&state.pool, &a)
                    .await
                    .unwrap_or_else(|_| mapping::guid(mapping::KIND_ARTIST, &a.id));
                hints.push(json!({
                    "ItemId": id.clone(),
                    "Id": id,
                    "Name": a.name,
                    "Type": "MusicArtist",
                    "MediaType": "Unknown",
                    "IsFolder": true,
                }));
            }
        }
    }
    if want_album {
        let w = ("title LIKE ?1".to_string(), vec![pattern.clone()]);
        if let Ok(rows) = repo::album::filter(state.pool.clone(), w).await {
            for al in rows.into_iter().take(limit as usize) {
                let id = mapping::remember_album(&state.pool, &al)
                    .await
                    .unwrap_or_else(|_| mapping::guid(mapping::KIND_ALBUM, &al.id));
                hints.push(json!({
                    "ItemId": id.clone(),
                    "Id": id,
                    "Name": al.title,
                    "Album": al.title,
                    "AlbumArtist": al.artist,
                    "Type": "MusicAlbum",
                    "MediaType": "Unknown",
                    "IsFolder": true,
                }));
            }
        }
    }
    if want_audio {
        let w = ("title LIKE ?1".to_string(), vec![pattern]);
        if let Ok(rows) = repo::track::filter(state.pool.clone(), w).await {
            for t in rows.into_iter().take(limit as usize) {
                let id = mapping::remember_track(&state.pool, &t)
                    .await
                    .unwrap_or_else(|_| mapping::guid(mapping::KIND_TRACK, &t.id));
                hints.push(json!({
                    "ItemId": id.clone(),
                    "Id": id,
                    "Name": t.title,
                    "Album": t.album,
                    "AlbumArtist": t.album_artist,
                    "Type": "Audio",
                    "MediaType": "Audio",
                    "IsFolder": false,
                    "RunTimeTicks": (t.length as i64) * TICKS_PER_MS,
                }));
            }
        }
    }
    let total = hints.len() as i32;
    HttpResponse::Ok().json(json!({
        "SearchHints": hints,
        "TotalRecordCount": total,
    }))
}

// keep the warning quiet — TICKS_PER_SEC is reserved for future use.
#[allow(dead_code)]
const _RESERVED: i64 = TICKS_PER_SEC;
