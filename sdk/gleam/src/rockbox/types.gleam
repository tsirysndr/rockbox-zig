//// Domain types and JSON decoders shared across the SDK.

import gleam/dynamic/decode
import gleam/option.{type Option, None}

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

pub type PlaybackStatus {
  Stopped
  Playing
  Paused
  UnknownStatus(Int)
}

pub fn playback_status_from_int(value: Int) -> PlaybackStatus {
  case value {
    0 -> Stopped
    1 -> Playing
    3 -> Paused
    other -> UnknownStatus(other)
  }
}

pub fn playback_status_to_int(status: PlaybackStatus) -> Int {
  case status {
    Stopped -> 0
    Playing -> 1
    Paused -> 3
    UnknownStatus(n) -> n
  }
}

pub type RepeatMode {
  RepeatOff
  RepeatAll
  RepeatOne
  RepeatShuffle
  RepeatAB
}

pub fn repeat_mode_to_int(mode: RepeatMode) -> Int {
  case mode {
    RepeatOff -> 0
    RepeatAll -> 1
    RepeatOne -> 2
    RepeatShuffle -> 3
    RepeatAB -> 4
  }
}

pub type ChannelConfig {
  Stereo
  StereoNarrow
  Mono
  LeftMix
  RightMix
  Karaoke
}

pub fn channel_config_to_int(config: ChannelConfig) -> Int {
  case config {
    Stereo -> 0
    StereoNarrow -> 1
    Mono -> 2
    LeftMix -> 3
    RightMix -> 4
    Karaoke -> 5
  }
}

pub type ReplaygainType {
  ReplaygainTrack
  ReplaygainAlbum
  ReplaygainShuffle
}

/// Where new tracks land in the queue (matches Mopidy/Kodi conventions).
pub type InsertPosition {
  /// Right after the currently playing track.
  Next
  /// After the last manually inserted track.
  AfterCurrent
  /// At the very end of the queue.
  Last
  /// Replace the entire playlist.
  First
}

pub fn insert_position_to_int(position: InsertPosition) -> Int {
  case position {
    Next -> 0
    AfterCurrent -> 1
    Last -> 2
    First -> 3
  }
}

// ---------------------------------------------------------------------------
// Core audio types
// ---------------------------------------------------------------------------

pub type Track {
  Track(
    id: Option(String),
    title: String,
    artist: String,
    album: String,
    genre: String,
    disc: String,
    track_string: String,
    year_string: String,
    composer: String,
    comment: String,
    album_artist: String,
    grouping: String,
    discnum: Int,
    tracknum: Int,
    layer: Int,
    year: Int,
    bitrate: Int,
    frequency: Int,
    filesize: Int,
    /// Duration in milliseconds.
    length: Int,
    /// Current playback position in milliseconds.
    elapsed: Int,
    path: String,
    album_id: Option(String),
    artist_id: Option(String),
    genre_id: Option(String),
    album_art: Option(String),
  )
}

pub type Album {
  Album(
    id: String,
    title: String,
    artist: String,
    year: Int,
    year_string: String,
    album_art: Option(String),
    md5: String,
    artist_id: String,
    copyright_message: Option(String),
    tracks: List(Track),
  )
}

pub type Artist {
  Artist(
    id: String,
    name: String,
    bio: Option(String),
    image: Option(String),
    tracks: List(Track),
    albums: List(Album),
  )
}

pub type SearchResults {
  SearchResults(
    artists: List(Artist),
    albums: List(Album),
    tracks: List(Track),
    liked_tracks: List(Track),
    liked_albums: List(Album),
  )
}

// ---------------------------------------------------------------------------
// Playlist types
// ---------------------------------------------------------------------------

pub type Playlist {
  Playlist(
    amount: Int,
    index: Int,
    max_playlist_size: Int,
    first_index: Int,
    last_insert_pos: Int,
    seed: Int,
    last_shuffled_start: Int,
    tracks: List(Track),
  )
}

pub type SavedPlaylist {
  SavedPlaylist(
    id: String,
    name: String,
    description: Option(String),
    image: Option(String),
    folder_id: Option(String),
    track_count: Int,
    created_at: Int,
    updated_at: Int,
  )
}

pub type SavedPlaylistFolder {
  SavedPlaylistFolder(
    id: String,
    name: String,
    created_at: Int,
    updated_at: Int,
  )
}

pub type SmartPlaylist {
  SmartPlaylist(
    id: String,
    name: String,
    description: Option(String),
    image: Option(String),
    folder_id: Option(String),
    is_system: Bool,
    /// JSON-encoded rules string.
    rules: String,
    created_at: Int,
    updated_at: Int,
  )
}

pub type TrackStats {
  TrackStats(
    track_id: String,
    play_count: Int,
    skip_count: Int,
    last_played: Option(Int),
    last_skipped: Option(Int),
    updated_at: Int,
  )
}

// ---------------------------------------------------------------------------
// Bluetooth / Sound / Devices / Browse / System / Settings
// ---------------------------------------------------------------------------

pub type BluetoothDevice {
  BluetoothDevice(
    address: String,
    name: String,
    paired: Bool,
    trusted: Bool,
    connected: Bool,
    rssi: Option(Int),
  )
}

pub type VolumeInfo {
  VolumeInfo(volume: Int, min: Int, max: Int)
}

pub type Device {
  Device(
    id: String,
    name: String,
    host: String,
    ip: String,
    port: Int,
    service: String,
    app: String,
    is_connected: Bool,
    base_url: Option(String),
    is_cast_device: Bool,
    is_source_device: Bool,
    is_current_device: Bool,
  )
}

pub type Entry {
  Entry(
    name: String,
    /// Bitmask: bit 4 (`0x10`) marks a directory.
    attr: Int,
    time_write: Int,
    customaction: Int,
    /// Human-readable display name (used for UPnP entries).
    display_name: Option(String),
  )
}

/// True when a directory bit (0x10) is set on the entry's attribute mask.
pub fn is_directory(entry: Entry) -> Bool {
  entry.attr / 16 % 2 == 1
}

pub type SystemStatus {
  SystemStatus(
    resume_index: Int,
    resume_crc32: Int,
    resume_elapsed: Int,
    resume_offset: Int,
    runtime: Int,
    topruntime: Int,
    dircache_size: Int,
    last_screen: Int,
    viewer_icon_count: Int,
    last_volume_change: Int,
  )
}

pub type EqBandSetting {
  EqBandSetting(cutoff: Int, q: Int, gain: Int)
}

pub type ReplaygainSettings {
  ReplaygainSettings(noclip: Bool, type_: Int, preamp: Int)
}

pub type CompressorSettings {
  CompressorSettings(
    threshold: Int,
    makeup_gain: Int,
    ratio: Int,
    knee: Int,
    release_time: Int,
    attack_time: Int,
  )
}

pub type UserSettings {
  UserSettings(
    music_dir: String,
    volume: Int,
    balance: Int,
    bass: Int,
    treble: Int,
    channel_config: Int,
    stereo_width: Int,
    eq_enabled: Bool,
    eq_precut: Int,
    eq_band_settings: List(EqBandSetting),
    replaygain_settings: ReplaygainSettings,
    compressor_settings: CompressorSettings,
    crossfade_enabled: Int,
    crossfade_fade_in_delay: Int,
    crossfade_fade_in_duration: Int,
    crossfade_fade_out_delay: Int,
    crossfade_fade_out_duration: Int,
    crossfade_fade_out_mixmode: Int,
    crossfeed_enabled: Bool,
    crossfeed_direct_gain: Int,
    crossfeed_cross_gain: Int,
    crossfeed_hf_attenuation: Int,
    crossfeed_hf_cutoff: Int,
    repeat_mode: Int,
    single_mode: Bool,
    party_mode: Bool,
    shuffle: Bool,
    player_name: String,
  )
}

// ---------------------------------------------------------------------------
// Decoders
// ---------------------------------------------------------------------------

fn opt_string() -> decode.Decoder(Option(String)) {
  decode.optional(decode.string)
}

fn opt_int() -> decode.Decoder(Option(Int)) {
  decode.optional(decode.int)
}

pub fn track_decoder() -> decode.Decoder(Track) {
  use id <- decode.optional_field("id", None, opt_string())
  use title <- decode.optional_field("title", "", decode.string)
  use artist <- decode.optional_field("artist", "", decode.string)
  use album <- decode.optional_field("album", "", decode.string)
  use genre <- decode.optional_field("genre", "", decode.string)
  use disc <- decode.optional_field("disc", "", decode.string)
  use track_string <- decode.optional_field("trackString", "", decode.string)
  use year_string <- decode.optional_field("yearString", "", decode.string)
  use composer <- decode.optional_field("composer", "", decode.string)
  use comment <- decode.optional_field("comment", "", decode.string)
  use album_artist <- decode.optional_field("albumArtist", "", decode.string)
  use grouping <- decode.optional_field("grouping", "", decode.string)
  use discnum <- decode.optional_field("discnum", 0, decode.int)
  use tracknum <- decode.optional_field("tracknum", 0, decode.int)
  use layer <- decode.optional_field("layer", 0, decode.int)
  use year <- decode.optional_field("year", 0, decode.int)
  use bitrate <- decode.optional_field("bitrate", 0, decode.int)
  use frequency <- decode.optional_field("frequency", 0, decode.int)
  use filesize <- decode.optional_field("filesize", 0, decode.int)
  use length <- decode.optional_field("length", 0, decode.int)
  use elapsed <- decode.optional_field("elapsed", 0, decode.int)
  use path <- decode.optional_field("path", "", decode.string)
  use album_id <- decode.optional_field("albumId", None, opt_string())
  use artist_id <- decode.optional_field("artistId", None, opt_string())
  use genre_id <- decode.optional_field("genreId", None, opt_string())
  use album_art <- decode.optional_field("albumArt", None, opt_string())
  decode.success(Track(
    id:,
    title:,
    artist:,
    album:,
    genre:,
    disc:,
    track_string:,
    year_string:,
    composer:,
    comment:,
    album_artist:,
    grouping:,
    discnum:,
    tracknum:,
    layer:,
    year:,
    bitrate:,
    frequency:,
    filesize:,
    length:,
    elapsed:,
    path:,
    album_id:,
    artist_id:,
    genre_id:,
    album_art:,
  ))
}

pub fn album_decoder() -> decode.Decoder(Album) {
  use id <- decode.field("id", decode.string)
  use title <- decode.optional_field("title", "", decode.string)
  use artist <- decode.optional_field("artist", "", decode.string)
  use year <- decode.optional_field("year", 0, decode.int)
  use year_string <- decode.optional_field("yearString", "", decode.string)
  use album_art <- decode.optional_field("albumArt", None, opt_string())
  use md5 <- decode.optional_field("md5", "", decode.string)
  use artist_id <- decode.optional_field("artistId", "", decode.string)
  use copyright_message <- decode.optional_field(
    "copyrightMessage",
    None,
    opt_string(),
  )
  use tracks <- decode.optional_field("tracks", [], decode.list(track_decoder()))
  decode.success(Album(
    id:,
    title:,
    artist:,
    year:,
    year_string:,
    album_art:,
    md5:,
    artist_id:,
    copyright_message:,
    tracks:,
  ))
}

pub fn artist_decoder() -> decode.Decoder(Artist) {
  use id <- decode.field("id", decode.string)
  use name <- decode.optional_field("name", "", decode.string)
  use bio <- decode.optional_field("bio", None, opt_string())
  use image <- decode.optional_field("image", None, opt_string())
  use tracks <- decode.optional_field("tracks", [], decode.list(track_decoder()))
  use albums <- decode.optional_field("albums", [], decode.list(album_decoder()))
  decode.success(Artist(id:, name:, bio:, image:, tracks:, albums:))
}

pub fn search_results_decoder() -> decode.Decoder(SearchResults) {
  use artists <- decode.optional_field(
    "artists",
    [],
    decode.list(artist_decoder()),
  )
  use albums <- decode.optional_field(
    "albums",
    [],
    decode.list(album_decoder()),
  )
  use tracks <- decode.optional_field("tracks", [], decode.list(track_decoder()))
  use liked_tracks <- decode.optional_field(
    "likedTracks",
    [],
    decode.list(track_decoder()),
  )
  use liked_albums <- decode.optional_field(
    "likedAlbums",
    [],
    decode.list(album_decoder()),
  )
  decode.success(SearchResults(
    artists:,
    albums:,
    tracks:,
    liked_tracks:,
    liked_albums:,
  ))
}

pub fn playlist_decoder() -> decode.Decoder(Playlist) {
  use amount <- decode.optional_field("amount", 0, decode.int)
  use index <- decode.optional_field("index", 0, decode.int)
  use max_playlist_size <- decode.optional_field(
    "maxPlaylistSize",
    0,
    decode.int,
  )
  use first_index <- decode.optional_field("firstIndex", 0, decode.int)
  use last_insert_pos <- decode.optional_field("lastInsertPos", 0, decode.int)
  use seed <- decode.optional_field("seed", 0, decode.int)
  use last_shuffled_start <- decode.optional_field(
    "lastShuffledStart",
    0,
    decode.int,
  )
  use tracks <- decode.optional_field("tracks", [], decode.list(track_decoder()))
  decode.success(Playlist(
    amount:,
    index:,
    max_playlist_size:,
    first_index:,
    last_insert_pos:,
    seed:,
    last_shuffled_start:,
    tracks:,
  ))
}

pub fn saved_playlist_decoder() -> decode.Decoder(SavedPlaylist) {
  use id <- decode.field("id", decode.string)
  use name <- decode.field("name", decode.string)
  use description <- decode.optional_field("description", None, opt_string())
  use image <- decode.optional_field("image", None, opt_string())
  use folder_id <- decode.optional_field("folderId", None, opt_string())
  use track_count <- decode.optional_field("trackCount", 0, decode.int)
  use created_at <- decode.optional_field("createdAt", 0, decode.int)
  use updated_at <- decode.optional_field("updatedAt", 0, decode.int)
  decode.success(SavedPlaylist(
    id:,
    name:,
    description:,
    image:,
    folder_id:,
    track_count:,
    created_at:,
    updated_at:,
  ))
}

pub fn saved_playlist_folder_decoder() -> decode.Decoder(SavedPlaylistFolder) {
  use id <- decode.field("id", decode.string)
  use name <- decode.field("name", decode.string)
  use created_at <- decode.optional_field("createdAt", 0, decode.int)
  use updated_at <- decode.optional_field("updatedAt", 0, decode.int)
  decode.success(SavedPlaylistFolder(id:, name:, created_at:, updated_at:))
}

pub fn smart_playlist_decoder() -> decode.Decoder(SmartPlaylist) {
  use id <- decode.field("id", decode.string)
  use name <- decode.field("name", decode.string)
  use description <- decode.optional_field("description", None, opt_string())
  use image <- decode.optional_field("image", None, opt_string())
  use folder_id <- decode.optional_field("folderId", None, opt_string())
  use is_system <- decode.optional_field("isSystem", False, decode.bool)
  use rules <- decode.optional_field("rules", "", decode.string)
  use created_at <- decode.optional_field("createdAt", 0, decode.int)
  use updated_at <- decode.optional_field("updatedAt", 0, decode.int)
  decode.success(SmartPlaylist(
    id:,
    name:,
    description:,
    image:,
    folder_id:,
    is_system:,
    rules:,
    created_at:,
    updated_at:,
  ))
}

pub fn track_stats_decoder() -> decode.Decoder(TrackStats) {
  use track_id <- decode.field("trackId", decode.string)
  use play_count <- decode.optional_field("playCount", 0, decode.int)
  use skip_count <- decode.optional_field("skipCount", 0, decode.int)
  use last_played <- decode.optional_field("lastPlayed", None, opt_int())
  use last_skipped <- decode.optional_field("lastSkipped", None, opt_int())
  use updated_at <- decode.optional_field("updatedAt", 0, decode.int)
  decode.success(TrackStats(
    track_id:,
    play_count:,
    skip_count:,
    last_played:,
    last_skipped:,
    updated_at:,
  ))
}

pub fn bluetooth_device_decoder() -> decode.Decoder(BluetoothDevice) {
  use address <- decode.field("address", decode.string)
  use name <- decode.optional_field("name", "", decode.string)
  use paired <- decode.optional_field("paired", False, decode.bool)
  use trusted <- decode.optional_field("trusted", False, decode.bool)
  use connected <- decode.optional_field("connected", False, decode.bool)
  use rssi <- decode.optional_field("rssi", None, opt_int())
  decode.success(BluetoothDevice(
    address:,
    name:,
    paired:,
    trusted:,
    connected:,
    rssi:,
  ))
}

pub fn volume_info_decoder() -> decode.Decoder(VolumeInfo) {
  use volume <- decode.field("volume", decode.int)
  use min <- decode.field("min", decode.int)
  use max <- decode.field("max", decode.int)
  decode.success(VolumeInfo(volume:, min:, max:))
}

pub fn device_decoder() -> decode.Decoder(Device) {
  use id <- decode.field("id", decode.string)
  use name <- decode.optional_field("name", "", decode.string)
  use host <- decode.optional_field("host", "", decode.string)
  use ip <- decode.optional_field("ip", "", decode.string)
  use port <- decode.optional_field("port", 0, decode.int)
  use service <- decode.optional_field("service", "", decode.string)
  use app <- decode.optional_field("app", "", decode.string)
  use is_connected <- decode.optional_field("isConnected", False, decode.bool)
  use base_url <- decode.optional_field("baseUrl", None, opt_string())
  use is_cast_device <- decode.optional_field(
    "isCastDevice",
    False,
    decode.bool,
  )
  use is_source_device <- decode.optional_field(
    "isSourceDevice",
    False,
    decode.bool,
  )
  use is_current_device <- decode.optional_field(
    "isCurrentDevice",
    False,
    decode.bool,
  )
  decode.success(Device(
    id:,
    name:,
    host:,
    ip:,
    port:,
    service:,
    app:,
    is_connected:,
    base_url:,
    is_cast_device:,
    is_source_device:,
    is_current_device:,
  ))
}

pub fn entry_decoder() -> decode.Decoder(Entry) {
  use name <- decode.field("name", decode.string)
  use attr <- decode.optional_field("attr", 0, decode.int)
  use time_write <- decode.optional_field("timeWrite", 0, decode.int)
  use customaction <- decode.optional_field("customaction", 0, decode.int)
  use display_name <- decode.optional_field("displayName", None, opt_string())
  decode.success(Entry(name:, attr:, time_write:, customaction:, display_name:))
}

pub fn system_status_decoder() -> decode.Decoder(SystemStatus) {
  use resume_index <- decode.optional_field("resumeIndex", 0, decode.int)
  use resume_crc32 <- decode.optional_field("resumeCrc32", 0, decode.int)
  use resume_elapsed <- decode.optional_field("resumeElapsed", 0, decode.int)
  use resume_offset <- decode.optional_field("resumeOffset", 0, decode.int)
  use runtime <- decode.optional_field("runtime", 0, decode.int)
  use topruntime <- decode.optional_field("topruntime", 0, decode.int)
  use dircache_size <- decode.optional_field("dircacheSize", 0, decode.int)
  use last_screen <- decode.optional_field("lastScreen", 0, decode.int)
  use viewer_icon_count <- decode.optional_field(
    "viewerIconCount",
    0,
    decode.int,
  )
  use last_volume_change <- decode.optional_field(
    "lastVolumeChange",
    0,
    decode.int,
  )
  decode.success(SystemStatus(
    resume_index:,
    resume_crc32:,
    resume_elapsed:,
    resume_offset:,
    runtime:,
    topruntime:,
    dircache_size:,
    last_screen:,
    viewer_icon_count:,
    last_volume_change:,
  ))
}

pub fn eq_band_setting_decoder() -> decode.Decoder(EqBandSetting) {
  use cutoff <- decode.field("cutoff", decode.int)
  use q <- decode.field("q", decode.int)
  use gain <- decode.field("gain", decode.int)
  decode.success(EqBandSetting(cutoff:, q:, gain:))
}

pub fn replaygain_settings_decoder() -> decode.Decoder(ReplaygainSettings) {
  use noclip <- decode.field("noclip", decode.bool)
  use type_ <- decode.field("type", decode.int)
  use preamp <- decode.field("preamp", decode.int)
  decode.success(ReplaygainSettings(noclip:, type_:, preamp:))
}

pub fn compressor_settings_decoder() -> decode.Decoder(CompressorSettings) {
  use threshold <- decode.field("threshold", decode.int)
  use makeup_gain <- decode.field("makeupGain", decode.int)
  use ratio <- decode.field("ratio", decode.int)
  use knee <- decode.field("knee", decode.int)
  use release_time <- decode.field("releaseTime", decode.int)
  use attack_time <- decode.field("attackTime", decode.int)
  decode.success(CompressorSettings(
    threshold:,
    makeup_gain:,
    ratio:,
    knee:,
    release_time:,
    attack_time:,
  ))
}

pub fn user_settings_decoder() -> decode.Decoder(UserSettings) {
  use music_dir <- decode.optional_field("musicDir", "", decode.string)
  use volume <- decode.optional_field("volume", 0, decode.int)
  use balance <- decode.optional_field("balance", 0, decode.int)
  use bass <- decode.optional_field("bass", 0, decode.int)
  use treble <- decode.optional_field("treble", 0, decode.int)
  use channel_config <- decode.optional_field("channelConfig", 0, decode.int)
  use stereo_width <- decode.optional_field("stereoWidth", 0, decode.int)
  use eq_enabled <- decode.optional_field("eqEnabled", False, decode.bool)
  use eq_precut <- decode.optional_field("eqPrecut", 0, decode.int)
  use eq_band_settings <- decode.optional_field(
    "eqBandSettings",
    [],
    decode.list(eq_band_setting_decoder()),
  )
  use replaygain_settings <- decode.field(
    "replaygainSettings",
    replaygain_settings_decoder(),
  )
  use compressor_settings <- decode.field(
    "compressorSettings",
    compressor_settings_decoder(),
  )
  use crossfade_enabled <- decode.optional_field(
    "crossfadeEnabled",
    0,
    decode.int,
  )
  use crossfade_fade_in_delay <- decode.optional_field(
    "crossfadeFadeInDelay",
    0,
    decode.int,
  )
  use crossfade_fade_in_duration <- decode.optional_field(
    "crossfadeFadeInDuration",
    0,
    decode.int,
  )
  use crossfade_fade_out_delay <- decode.optional_field(
    "crossfadeFadeOutDelay",
    0,
    decode.int,
  )
  use crossfade_fade_out_duration <- decode.optional_field(
    "crossfadeFadeOutDuration",
    0,
    decode.int,
  )
  use crossfade_fade_out_mixmode <- decode.optional_field(
    "crossfadeFadeOutMixmode",
    0,
    decode.int,
  )
  use crossfeed_enabled <- decode.optional_field(
    "crossfeedEnabled",
    False,
    decode.bool,
  )
  use crossfeed_direct_gain <- decode.optional_field(
    "crossfeedDirectGain",
    0,
    decode.int,
  )
  use crossfeed_cross_gain <- decode.optional_field(
    "crossfeedCrossGain",
    0,
    decode.int,
  )
  use crossfeed_hf_attenuation <- decode.optional_field(
    "crossfeedHfAttenuation",
    0,
    decode.int,
  )
  use crossfeed_hf_cutoff <- decode.optional_field(
    "crossfeedHfCutoff",
    0,
    decode.int,
  )
  use repeat_mode <- decode.optional_field("repeatMode", 0, decode.int)
  use single_mode <- decode.optional_field("singleMode", False, decode.bool)
  use party_mode <- decode.optional_field("partyMode", False, decode.bool)
  use shuffle <- decode.optional_field("shuffle", False, decode.bool)
  use player_name <- decode.optional_field("playerName", "", decode.string)
  decode.success(UserSettings(
    music_dir:,
    volume:,
    balance:,
    bass:,
    treble:,
    channel_config:,
    stereo_width:,
    eq_enabled:,
    eq_precut:,
    eq_band_settings:,
    replaygain_settings:,
    compressor_settings:,
    crossfade_enabled:,
    crossfade_fade_in_delay:,
    crossfade_fade_in_duration:,
    crossfade_fade_out_delay:,
    crossfade_fade_out_duration:,
    crossfade_fade_out_mixmode:,
    crossfeed_enabled:,
    crossfeed_direct_gain:,
    crossfeed_cross_gain:,
    crossfeed_hf_attenuation:,
    crossfeed_hf_cutoff:,
    repeat_mode:,
    single_mode:,
    party_mode:,
    shuffle:,
    player_name:,
  ))
}
