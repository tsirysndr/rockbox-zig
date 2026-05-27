pub mod bluetooth_picker;
pub mod controlbar;
pub mod device_picker;
pub mod icons;
pub mod miniplayer;
pub mod navbar;
pub mod pages;
pub mod search_input;
pub mod seek_bar;
pub mod text_input;
pub mod titlebar;

#[derive(Clone, Copy, PartialEq)]
pub enum Page {
    Library,
    Player,
    Queue,
}
impl gpui::Global for Page {}

#[derive(Clone, Copy, PartialEq)]
pub enum LibrarySection {
    Home,
    Songs,
    Albums,
    Artists,
    Genres,
    AlbumDetail,
    ArtistDetail,
    GenreDetail,
    Likes,
    Files,
    Playlists,
    PlaylistDetail,
    SmartPlaylistDetail,
    NdAlbums,
    NdAlbumDetail,
    NdArtists,
    NdArtistDetail,
    NdGenres,
    NdGenreDetail,
    NdPlaylists,
    NdPlaylistDetail,
    NdSongs,
    NdLikes,
}
impl gpui::Global for LibrarySection {}

#[derive(Clone, PartialEq)]
pub enum FilesMode {
    /// Root landing: shows "Music", "UPnP Devices", and "Plex" tiles.
    Root,
    /// Browsing the local music directory (current_path = current dir, None = music root).
    Local,
    /// Listing discovered UPnP/DLNA media servers.
    UpnpDevices,
    /// Browsing a UPnP device's ContentDirectory.
    UpnpBrowse,
    /// Listing discovered Plex Media Servers.
    PlexServers,
    /// Browsing a Plex server's library.
    PlexBrowse,
    /// Listing discovered Jellyfin media servers.
    JellyfinServers,
    /// Browsing a Jellyfin server's library.
    JellyfinBrowse,
    /// Listing manually-added Navidrome servers (no discovery).
    NavidromeServers,
    /// Browsing a Navidrome server's library.
    NavidromeBrowse,
    /// Listing discovered Kodi/XBMC media servers.
    KodiServers,
    /// Browsing a Kodi server's library.
    KodiBrowse,
}

impl Default for FilesMode {
    fn default() -> Self {
        FilesMode::Root
    }
}

#[derive(Clone, Default)]
pub struct FilesBrowseState {
    pub mode: FilesMode,
    pub current_path: Option<String>,
    pub history: Vec<(FilesMode, Option<String>)>,
    /// Path of a discovered Plex server waiting for a token before browsing.
    pub pending_plex_server: Option<String>,
}

impl FilesBrowseState {
    pub fn can_go_back(&self) -> bool {
        !self.history.is_empty()
    }

    pub fn go_back(&mut self) {
        if let Some((prev_mode, prev_path)) = self.history.pop() {
            self.mode = prev_mode;
            self.current_path = prev_path;
            self.pending_plex_server = None;
        }
    }

    pub fn navigate(&mut self, new_mode: FilesMode, new_path: Option<String>) {
        let old_mode = std::mem::replace(&mut self.mode, new_mode);
        let old_path = self.current_path.take();
        self.history.push((old_mode, old_path));
        self.current_path = new_path;
        self.pending_plex_server = None;
    }
}

#[derive(Clone, Default)]
pub struct NavidromeAuthState {
    /// base_url (percent-encoded, prefixed with "navidrome://") of the server waiting for auth.
    pub pending_server: Option<String>,
    pub authenticating: bool,
    pub error: Option<String>,
    pub show_add_manually: bool,
}
impl gpui::Global for NavidromeAuthState {}

#[derive(Clone, Default)]
pub struct KodiAuthState {
    /// base_url (percent-encoded, prefixed with "kodi://") of the server waiting for auth.
    pub pending_server: Option<String>,
    pub authenticating: bool,
    pub error: Option<String>,
}
impl gpui::Global for KodiAuthState {}

impl gpui::Global for FilesBrowseState {}

#[derive(Clone, Default, PartialEq)]
pub enum JellyfinAuthMode {
    #[default]
    Credentials,
    ApiKey,
}

#[derive(Clone, Default)]
pub struct JellyfinAuthState {
    /// base_url (percent-encoded, prefixed with "jellyfin://") of the server waiting for auth.
    pub pending_server: Option<String>,
    pub authenticating: bool,
    pub error: Option<String>,
    pub auth_mode: JellyfinAuthMode,
    pub show_add_manually: bool,
}
impl gpui::Global for JellyfinAuthState {}

#[derive(Clone)]
pub struct FileContextMenu {
    pub pos: gpui::Point<gpui::Pixels>,
    pub path: String,
    pub name: String,
    pub is_dir: bool,
    pub current_dir: String,
    pub dir_idx: usize,
}

#[derive(Clone, Default)]
pub struct FileContextMenuState(pub Option<FileContextMenu>);
impl gpui::Global for FileContextMenuState {}

#[derive(Clone, Default)]
pub struct LikedSongs(pub std::collections::HashSet<String>);
impl gpui::Global for LikedSongs {}

#[derive(Clone, Default)]
pub struct LikedOrder(pub Vec<String>);
impl gpui::Global for LikedOrder {}

#[derive(Clone, PartialEq)]
pub struct SelectedAlbum(pub String);
impl gpui::Global for SelectedAlbum {}

#[derive(Clone, Default)]
pub struct SelectedAlbumMeta {
    pub album_id: String,
    pub year_string: String,
    pub copyright_message: Option<String>,
}
impl gpui::Global for SelectedAlbumMeta {}

#[derive(Clone, PartialEq)]
pub struct SelectedArtist(pub String);
impl gpui::Global for SelectedArtist {}

#[derive(Clone, Default)]
pub struct GenreItem {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub image: Option<String>,
    pub track_count: i64,
}

#[derive(Clone, Default)]
pub struct GenresState(pub Vec<GenreItem>);
impl gpui::Global for GenresState {}

#[derive(Clone, Default)]
pub struct SelectedGenre {
    pub id: String,
    pub name: String,
    pub tracks: Vec<crate::state::Track>,
    pub albums: Vec<crate::state::SearchAlbum>,
    pub artists: Vec<crate::state::SearchArtist>,
}
impl gpui::Global for SelectedGenre {}

/// Where the back button in a detail view should navigate to.
#[derive(Clone, Copy, PartialEq)]
pub struct BackSection(pub LibrarySection);
impl gpui::Global for BackSection {}

#[derive(Clone)]
pub struct LibraryContextMenu {
    pub pos: gpui::Point<gpui::Pixels>,
    pub path: String,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub album_art: Option<String>,
    pub track_id: String,
}

#[derive(Clone, Default)]
pub struct LibraryContextMenuState(pub Option<LibraryContextMenu>);
impl gpui::Global for LibraryContextMenuState {}

#[derive(Clone)]
pub struct AlbumContextMenu {
    pub pos: gpui::Point<gpui::Pixels>,
    pub album_id: String,
    pub album_name: String,
    pub album_art: Option<String>,
    pub artist_name: String,
    pub track_paths: Vec<String>,
}

#[derive(Clone, Default)]
pub struct AlbumContextMenuState(pub Option<AlbumContextMenu>);
impl gpui::Global for AlbumContextMenuState {}

#[derive(Clone, Default)]
pub struct HoveredAlbumIdx(pub Option<usize>);
impl gpui::Global for HoveredAlbumIdx {}

// ── Playlist state types ──────────────────────────────────────────────────────

#[derive(Clone, Default)]
pub struct SavedPlaylistItem {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub image: Option<String>,
    pub folder_id: Option<String>,
    pub track_count: i64,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Clone, Default)]
pub struct SmartPlaylistItem {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub is_system: bool,
    pub rules: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Clone, Default)]
pub struct PlaylistsState {
    pub saved: Vec<SavedPlaylistItem>,
    pub smart: Vec<SmartPlaylistItem>,
    pub playlist_tracks: Vec<crate::state::Track>,
}
impl gpui::Global for PlaylistsState {}

#[derive(Clone, Default)]
pub struct SelectedPlaylist {
    pub id: String,
    pub name: String,
    pub is_smart: bool,
}
impl gpui::Global for SelectedPlaylist {}

#[derive(Clone, Default)]
pub struct PlaylistsSidebarCollapsed(pub bool);
impl gpui::Global for PlaylistsSidebarCollapsed {}

#[derive(Clone, Default)]
pub struct CreatePlaylistModal {
    pub open: bool,
    pub name: String,
    pub description: String,
    /// Track to add to the newly created playlist (set when opened from "Add to Playlist" submenu).
    pub pending_track_id: Option<String>,
}
impl gpui::Global for CreatePlaylistModal {}

#[derive(Clone, Default)]
pub struct EditPlaylistModal {
    pub open: bool,
    pub id: String,
    pub name: String,
    pub description: String,
}
impl gpui::Global for EditPlaylistModal {}

#[derive(Clone, Default)]
pub struct DeletePlaylistModal {
    pub open: bool,
    pub id: String,
    pub name: String,
}
impl gpui::Global for DeletePlaylistModal {}

#[derive(Clone)]
pub struct AddToPlaylistMenu {
    /// Right edge of the parent context menu (flyout opens here by default).
    pub anchor_x: gpui::Pixels,
    /// Left edge of parent context menu (used when flipping left on overflow).
    pub flip_x: gpui::Pixels,
    /// Y of the "Add to Playlist" row (submenu aligns here).
    pub anchor_y: gpui::Pixels,
    pub track_path: String,
    pub track_id: String,
}

#[derive(Clone, Default)]
pub struct AddToPlaylistMenuState(pub Option<AddToPlaylistMenu>);
impl gpui::Global for AddToPlaylistMenuState {}

// ── Server picker ─────────────────────────────────────────────────────────────

#[derive(Clone, Default)]
pub struct ServerPickerOpen(pub bool);
impl gpui::Global for ServerPickerOpen {}

#[derive(Clone, Default)]
pub struct DiscoveredServers {
    pub servers: Vec<crate::server::ServerInfo>,
    pub scanning: bool,
}
impl gpui::Global for DiscoveredServers {}

// ── Navidrome / Subsonic integration ─────────────────────────────────────────

#[derive(Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct NdSavedServer {
    pub id: String,
    pub base_url: String,
    pub user: String,
    pub token: String,
    pub salt: String,
}

#[derive(Clone, Default)]
pub struct NavidromeServerState {
    pub servers: Vec<NdSavedServer>,
    pub active_id: Option<String>,
    pub connecting: bool,
    pub connect_error: Option<String>,
    pub show_add_form: bool,
    pub sidebar_collapsed: bool,
}

impl NavidromeServerState {
    pub fn connected(&self) -> bool {
        self.active_id.is_some()
    }

    pub fn active_server(&self) -> Option<&NdSavedServer> {
        self.active_id
            .as_ref()
            .and_then(|id| self.servers.iter().find(|s| &s.id == id))
    }

    pub fn remove_server(&mut self, id: &str) {
        self.servers.retain(|s| s.id != id);
        if self.active_id.as_deref() == Some(id) {
            self.active_id = self.servers.first().map(|s| s.id.clone());
        }
    }
}

impl gpui::Global for NavidromeServerState {}

#[derive(Clone, Default)]
pub struct NdAlbumItem {
    pub id: String,
    pub name: String,
    pub artist: String,
    pub artist_id: String,
    pub year: Option<u32>,
    pub cover_art: Option<String>,
    pub song_count: u32,
}

#[derive(Clone, Default)]
pub struct NdArtistItem {
    pub id: String,
    pub name: String,
    pub cover_art: Option<String>,
    pub album_count: u32,
}

#[derive(Clone, Default)]
pub struct NdGenreItem {
    pub name: String,
    pub song_count: u32,
    pub album_count: u32,
}

#[derive(Clone, Default)]
pub struct NdPlaylistItem {
    pub id: String,
    pub name: String,
    pub comment: Option<String>,
    pub song_count: u32,
    pub cover_art: Option<String>,
}

#[derive(Clone, Default)]
pub struct NdSongItem {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub artist_id: String,
    pub album: String,
    pub album_id: String,
    pub cover_art: Option<String>,
    pub duration: u32,
    pub track: Option<u32>,
    pub stream_url: String,
}

#[derive(Clone, Default)]
pub struct NdLibraryData {
    pub albums: Vec<NdAlbumItem>,
    pub artists: Vec<NdArtistItem>,
    pub genres: Vec<NdGenreItem>,
    pub playlists: Vec<NdPlaylistItem>,
    pub loading: bool,
}
impl gpui::Global for NdLibraryData {}

#[derive(Clone, Default)]
pub struct NdSelectedAlbum {
    pub id: String,
    pub name: String,
    pub artist: String,
    pub cover_art: Option<String>,
    pub songs: Vec<NdSongItem>,
    pub loading: bool,
}
impl gpui::Global for NdSelectedAlbum {}

#[derive(Clone, Default)]
pub struct NdSelectedArtist {
    pub id: String,
    pub name: String,
    pub cover_art: Option<String>,
    pub albums: Vec<NdAlbumItem>,
    pub loading: bool,
}
impl gpui::Global for NdSelectedArtist {}

#[derive(Clone, Default)]
pub struct NdSelectedGenre {
    pub name: String,
    pub songs: Vec<NdSongItem>,
    pub loading: bool,
}
impl gpui::Global for NdSelectedGenre {}

#[derive(Clone, Default)]
pub struct NdSelectedPlaylist {
    pub id: String,
    pub name: String,
    pub tracks: Vec<NdSongItem>,
    pub loading: bool,
}
impl gpui::Global for NdSelectedPlaylist {}

#[derive(Clone, Default)]
pub struct NdSongsState {
    pub songs: Vec<NdSongItem>,
    pub loading: bool,
}
impl gpui::Global for NdSongsState {}

#[derive(Clone, Default)]
pub struct NdLikesState {
    pub songs: Vec<NdSongItem>,
    pub loading: bool,
}
impl gpui::Global for NdLikesState {}

/// Set of starred song IDs for the active Navidrome server.
#[derive(Clone, Default)]
pub struct NdStarredIds(pub std::collections::HashSet<String>);
impl gpui::Global for NdStarredIds {}

/// Full cover art URL for the currently playing Navidrome track.
/// Set when a Navidrome track is played; cleared when switching to a local track.
#[derive(Clone, Default)]
pub struct NdCurrentCoverArt(pub Option<String>);
impl gpui::Global for NdCurrentCoverArt {}

/// Tracks scrobble state for the current Navidrome track.
/// `scrobbled_id` is the song ID that has already been scrobbled in this session;
/// reset to None when the track changes.
#[derive(Clone, Default)]
pub struct NdScrobbleState {
    pub scrobbled_id: Option<String>,
}
impl gpui::Global for NdScrobbleState {}

/// Tracks the song ID for which we have already initiated a cover art fetch,
/// to avoid duplicate API calls when NdCurrentCoverArt is None for the playing track.
#[derive(Clone, Default)]
pub struct NdCoverFetchState {
    pub fetched_id: Option<String>,
}
impl gpui::Global for NdCoverFetchState {}

#[derive(Clone)]
pub struct NdContextMenu {
    pub pos: gpui::Point<gpui::Pixels>,
    pub song_id: String,
    pub title: String,
    pub artist_id: String,
    pub artist: String,
    pub album_id: String,
    pub album: String,
    pub cover_art: Option<String>,
    pub stream_url: String,
}

#[derive(Clone, Default)]
pub struct NdContextMenuState(pub Option<NdContextMenu>);
impl gpui::Global for NdContextMenuState {}
