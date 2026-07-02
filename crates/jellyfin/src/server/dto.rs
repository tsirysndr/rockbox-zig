//! Jellyfin response DTOs aligned to the official OpenAPI spec.
//!
//! Conventions:
//! - **Required non-nullable** fields are plain types — MUST be emitted.
//!   Missing them causes SDK-generated clients (jellyfin-sdk-kotlin,
//!   kotlinx.serialization) to silently drop the whole object.
//! - **Nullable** fields are `Option<T>` and serialize as `null` when None.
//! - **Optional non-nullable** uses `#[serde(skip_serializing_if =
//!   "Option::is_none")]`.

use serde::Serialize;
use serde_json::Value;

/// Version string we report to Jellyfin clients. The official Android app
/// and several SDK-generated clients refuse anything older than recent
/// Jellyfin releases — we pose as a modern server.
pub const JELLYFIN_API_VERSION: &str = "10.11.11";

// ── System info ─────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct PublicSystemInfo {
    pub local_address: Option<String>,
    pub server_name: Option<String>,
    pub version: Option<String>,
    pub product_name: Option<String>,
    pub operating_system: Option<String>,
    pub id: Option<String>,
    pub startup_wizard_completed: Option<bool>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct SystemInfo {
    pub local_address: Option<String>,
    pub server_name: Option<String>,
    pub version: Option<String>,
    pub product_name: Option<String>,
    pub operating_system: Option<String>,
    pub id: Option<String>,
    pub startup_wizard_completed: Option<bool>,
    pub operating_system_display_name: Option<String>,
    pub package_name: Option<String>,
    pub has_pending_restart: bool,
    pub is_shutting_down: bool,
    pub supports_library_monitor: bool,
    pub web_socket_port_number: i32,
    pub completed_installations: Option<Vec<Value>>,
    pub can_self_restart: bool,
    pub can_launch_web_browser: bool,
    pub program_data_path: Option<String>,
    pub web_path: Option<String>,
    pub items_by_name_path: Option<String>,
    pub cache_path: Option<String>,
    pub log_path: Option<String>,
    pub internal_metadata_path: Option<String>,
    pub transcoding_temp_path: Option<String>,
    pub cast_receiver_applications: Option<Vec<Value>>,
    pub has_update_available: bool,
    pub encoder_location: Option<String>,
    pub system_architecture: Option<String>,
}

// ── User ────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct UserConfiguration {
    pub audio_language_preference: Option<String>,
    pub play_default_audio_track: bool,
    pub subtitle_language_preference: Option<String>,
    pub display_missing_episodes: bool,
    pub grouped_folders: Vec<String>,
    pub subtitle_mode: &'static str,
    pub display_collections_view: bool,
    pub enable_local_password: bool,
    pub ordered_views: Vec<String>,
    pub latest_items_excludes: Vec<String>,
    pub my_media_excludes: Vec<String>,
    pub hide_played_in_latest: bool,
    pub remember_audio_selections: bool,
    pub remember_subtitle_selections: bool,
    pub enable_next_episode_auto_play: bool,
    pub cast_receiver_id: Option<String>,
}

impl Default for UserConfiguration {
    fn default() -> Self {
        Self {
            audio_language_preference: None,
            play_default_audio_track: true,
            subtitle_language_preference: None,
            display_missing_episodes: false,
            grouped_folders: vec![],
            subtitle_mode: "Default",
            display_collections_view: false,
            enable_local_password: false,
            ordered_views: vec![],
            latest_items_excludes: vec![],
            my_media_excludes: vec![],
            hide_played_in_latest: true,
            remember_audio_selections: true,
            remember_subtitle_selections: true,
            enable_next_episode_auto_play: true,
            cast_receiver_id: None,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct UserPolicy {
    pub is_administrator: bool,
    pub is_hidden: bool,
    pub enable_collection_management: bool,
    pub enable_subtitle_management: bool,
    pub enable_lyric_management: bool,
    pub is_disabled: bool,
    pub max_parental_rating: Option<i32>,
    pub max_parental_sub_rating: Option<i32>,
    pub blocked_tags: Option<Vec<String>>,
    pub allowed_tags: Option<Vec<String>>,
    pub enable_user_preference_access: bool,
    pub access_schedules: Option<Vec<Value>>,
    pub block_unrated_items: Option<Vec<String>>,
    pub enable_remote_control_of_other_users: bool,
    pub enable_shared_device_control: bool,
    pub enable_remote_access: bool,
    pub enable_live_tv_management: bool,
    pub enable_live_tv_access: bool,
    pub enable_media_playback: bool,
    pub enable_audio_playback_transcoding: bool,
    pub enable_video_playback_transcoding: bool,
    pub enable_playback_remuxing: bool,
    pub force_remote_source_transcoding: bool,
    pub enable_content_deletion: bool,
    pub enable_content_deletion_from_folders: Option<Vec<String>>,
    pub enable_content_downloading: bool,
    pub enable_sync_transcoding: bool,
    pub enable_media_conversion: bool,
    pub enabled_devices: Option<Vec<String>>,
    pub enable_all_devices: bool,
    pub enabled_channels: Option<Vec<String>>,
    pub enable_all_channels: bool,
    pub enabled_folders: Option<Vec<String>>,
    pub enable_all_folders: bool,
    pub invalid_login_attempt_count: i32,
    pub login_attempts_before_lockout: i32,
    pub max_active_sessions: i32,
    pub enable_public_sharing: bool,
    pub blocked_media_folders: Option<Vec<String>>,
    pub blocked_channels: Option<Vec<String>>,
    pub remote_client_bitrate_limit: i32,
    pub authentication_provider_id: String,
    pub password_reset_provider_id: String,
    pub sync_play_access: &'static str,
}

impl UserPolicy {
    pub fn admin() -> Self {
        Self {
            is_administrator: true,
            is_hidden: false,
            enable_collection_management: true,
            enable_subtitle_management: true,
            enable_lyric_management: true,
            is_disabled: false,
            max_parental_rating: None,
            max_parental_sub_rating: None,
            blocked_tags: Some(vec![]),
            allowed_tags: Some(vec![]),
            enable_user_preference_access: true,
            access_schedules: Some(vec![]),
            block_unrated_items: Some(vec![]),
            enable_remote_control_of_other_users: false,
            enable_shared_device_control: false,
            enable_remote_access: true,
            enable_live_tv_management: false,
            enable_live_tv_access: false,
            enable_media_playback: true,
            enable_audio_playback_transcoding: false,
            enable_video_playback_transcoding: false,
            enable_playback_remuxing: false,
            force_remote_source_transcoding: false,
            enable_content_deletion: false,
            enable_content_deletion_from_folders: Some(vec![]),
            enable_content_downloading: true,
            enable_sync_transcoding: false,
            enable_media_conversion: false,
            enabled_devices: Some(vec![]),
            enable_all_devices: true,
            enabled_channels: Some(vec![]),
            enable_all_channels: true,
            enabled_folders: Some(vec![]),
            enable_all_folders: true,
            invalid_login_attempt_count: 0,
            login_attempts_before_lockout: -1,
            max_active_sessions: 0,
            enable_public_sharing: true,
            blocked_media_folders: Some(vec![]),
            blocked_channels: Some(vec![]),
            remote_client_bitrate_limit: 0,
            authentication_provider_id:
                "Jellyfin.Server.Implementations.Users.DefaultAuthenticationProvider".to_string(),
            password_reset_provider_id:
                "Jellyfin.Server.Implementations.Users.DefaultPasswordResetProvider".to_string(),
            sync_play_access: "CreateAndJoinGroups",
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct UserDto {
    pub name: Option<String>,
    pub server_id: Option<String>,
    pub server_name: Option<String>,
    pub id: String,
    pub primary_image_tag: Option<String>,
    pub has_password: Option<bool>,
    pub has_configured_password: Option<bool>,
    pub has_configured_easy_password: Option<bool>,
    pub enable_auto_login: Option<bool>,
    pub last_login_date: Option<String>,
    pub last_activity_date: Option<String>,
    pub configuration: Option<UserConfiguration>,
    pub policy: Option<UserPolicy>,
    pub primary_image_aspect_ratio: Option<f64>,
}

// ── Authentication / session ────────────────────────────────────────────────

#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct AuthenticationResult {
    pub user: Option<UserDto>,
    pub session_info: Option<SessionInfoDto>,
    pub access_token: Option<String>,
    pub server_id: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct SessionInfoDto {
    pub play_state: Option<Value>,
    pub additional_users: Option<Vec<Value>>,
    pub capabilities: Option<Value>,
    pub remote_end_point: Option<String>,
    pub playable_media_types: Vec<String>,
    pub id: Option<String>,
    pub user_id: String,
    pub user_name: Option<String>,
    pub client: Option<String>,
    pub last_activity_date: String,
    pub last_playback_check_in: String,
    pub last_paused_date: Option<String>,
    pub device_name: Option<String>,
    pub device_type: Option<String>,
    pub now_playing_item: Option<Value>,
    pub now_viewing_item: Option<Value>,
    pub device_id: Option<String>,
    pub application_version: Option<String>,
    pub transcoding_info: Option<Value>,
    pub is_active: bool,
    pub supports_media_control: bool,
    pub supports_remote_control: bool,
    pub now_playing_queue: Option<Vec<Value>>,
    pub has_custom_device_name: bool,
    pub playlist_item_id: Option<String>,
    pub server_id: Option<String>,
    pub user_primary_image_tag: Option<String>,
    pub supported_commands: Vec<String>,
}

// ── BaseItem + helpers ──────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct UserItemDataDto {
    pub rating: Option<f64>,
    pub played_percentage: Option<f64>,
    pub unplayed_item_count: Option<i32>,
    pub playback_position_ticks: i64,
    pub play_count: i32,
    pub is_favorite: bool,
    pub likes: Option<bool>,
    pub last_played_date: Option<String>,
    pub played: bool,
    pub key: String,
    pub item_id: String,
}

#[derive(Debug, Serialize, Default, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct ImageBlurHashes {}

#[derive(Debug, Serialize, Default, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct ImageTags {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub primary: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct NameGuidPair {
    pub name: Option<String>,
    pub id: String,
}

#[derive(Debug, Serialize, Default, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct MediaStream {
    pub codec: Option<String>,
    pub language: Option<String>,
    pub title: Option<String>,
    pub video_range: &'static str,
    pub video_range_type: &'static str,
    pub audio_spatial_format: &'static str,
    pub display_title: Option<String>,
    pub is_interlaced: bool,
    pub channel_layout: Option<String>,
    pub bit_rate: Option<i32>,
    pub channels: Option<i32>,
    pub sample_rate: Option<i32>,
    pub is_default: bool,
    pub is_forced: bool,
    pub is_hearing_impaired: bool,
    pub is_original: bool,
    pub height: Option<i32>,
    pub width: Option<i32>,
    pub profile: Option<String>,
    #[serde(rename = "Type")]
    pub stream_type: &'static str,
    pub index: i32,
    pub is_external: bool,
    pub is_text_subtitle_stream: bool,
    pub supports_external_stream: bool,
}

#[derive(Debug, Serialize, Default, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct MediaSource {
    pub protocol: &'static str,
    pub id: Option<String>,
    pub path: Option<String>,
    #[serde(rename = "Type")]
    pub source_type: &'static str,
    pub container: Option<String>,
    pub size: Option<i64>,
    pub name: Option<String>,
    pub is_remote: bool,
    pub run_time_ticks: Option<i64>,
    pub read_at_native_framerate: bool,
    pub ignore_dts: bool,
    pub ignore_index: bool,
    pub gen_pts_input: bool,
    pub supports_transcoding: bool,
    pub supports_direct_stream: bool,
    pub supports_direct_play: bool,
    pub is_infinite_stream: bool,
    pub use_most_compatible_transcoding_profile: bool,
    pub requires_opening: bool,
    pub requires_closing: bool,
    pub requires_looping: bool,
    pub supports_probing: bool,
    pub media_streams: Option<Vec<MediaStream>>,
    pub formats: Option<Vec<String>>,
    pub bitrate: Option<i32>,
    pub transcoding_sub_protocol: &'static str,
    pub default_audio_stream_index: Option<i32>,
    pub default_subtitle_stream_index: Option<i32>,
    pub has_segments: bool,
}

#[derive(Debug, Serialize, Default)]
#[serde(rename_all = "PascalCase")]
pub struct BaseItemDto {
    pub name: Option<String>,
    pub server_id: Option<String>,
    pub id: String,
    pub etag: Option<String>,
    pub date_created: Option<String>,
    pub container: Option<String>,
    pub sort_name: Option<String>,
    pub premiere_date: Option<String>,
    pub media_sources: Option<Vec<MediaSource>>,
    pub path: Option<String>,
    pub overview: Option<String>,
    pub genres: Option<Vec<String>>,
    pub run_time_ticks: Option<i64>,
    pub production_year: Option<i32>,
    pub index_number: Option<i32>,
    pub parent_index_number: Option<i32>,
    pub is_folder: Option<bool>,
    pub parent_id: Option<String>,
    #[serde(rename = "Type")]
    pub item_type: &'static str,
    pub artist_items: Option<Vec<NameGuidPair>>,
    pub user_data: Option<UserItemDataDto>,
    pub child_count: Option<i32>,
    pub artists: Option<Vec<String>>,
    pub album: Option<String>,
    pub collection_type: Option<&'static str>,
    pub album_id: Option<String>,
    pub album_primary_image_tag: Option<String>,
    pub album_artist: Option<String>,
    pub album_artists: Option<Vec<NameGuidPair>>,
    pub media_streams: Option<Vec<MediaStream>>,
    pub media_source_count: Option<i32>,
    pub image_tags: Option<ImageTags>,
    pub backdrop_image_tags: Option<Vec<String>>,
    pub image_blur_hashes: Option<ImageBlurHashes>,
    pub location_type: Option<&'static str>,
    pub media_type: &'static str,
    pub song_count: Option<i32>,
    pub album_count: Option<i32>,
    pub artist_count: Option<i32>,
    /// Stable per-entry id inside a playlist. Only meaningful for tracks
    /// returned via `GET /Playlists/{id}/Items` — clients pass these back
    /// in `EntryIds=` for remove/move calls.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub playlist_item_id: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct ItemsResult {
    pub items: Vec<BaseItemDto>,
    pub total_record_count: i32,
    pub start_index: i32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct ViewsResult {
    pub items: Vec<BaseItemDto>,
    pub total_record_count: i32,
    pub start_index: i32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct PlaybackInfoResponse {
    pub media_sources: Vec<MediaSource>,
    pub play_session_id: Option<String>,
}

/// `PlaylistCreationResult` — sole required field is `Id` (the new playlist's
/// GUID). Clients (Findroid, Streamyfin) use this to jump straight into the
/// playlist detail after creation.
#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct PlaylistCreationResult {
    pub id: String,
}
