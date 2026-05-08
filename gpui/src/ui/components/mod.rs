pub mod bluetooth_picker;
pub mod controlbar;
pub mod device_picker;
pub mod eq_slider;
pub mod icons;
pub mod miniplayer;
pub mod navbar;
pub mod pages;
pub mod search_input;
pub mod seek_bar;
pub mod settings_modal;
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
}
impl gpui::Global for LibrarySection {}

#[derive(Clone, PartialEq)]
pub enum FilesMode {
    /// Root landing: show "Music" and "UPnP Devices" tiles.
    Root,
    /// Browsing the local music directory (current_path = current dir, None = music root).
    Local,
    /// Listing discovered UPnP/DLNA media servers.
    UpnpDevices,
    /// Browsing a UPnP device's ContentDirectory.
    UpnpBrowse,
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
}

impl FilesBrowseState {
    pub fn can_go_back(&self) -> bool {
        !self.history.is_empty()
    }

    pub fn go_back(&mut self) {
        if let Some((prev_mode, prev_path)) = self.history.pop() {
            self.mode = prev_mode;
            self.current_path = prev_path;
        }
    }

    pub fn navigate(&mut self, new_mode: FilesMode, new_path: Option<String>) {
        let old_mode = std::mem::replace(&mut self.mode, new_mode);
        let old_path = self.current_path.take();
        self.history.push((old_mode, old_path));
        self.current_path = new_path;
    }
}

impl gpui::Global for FilesBrowseState {}

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

// ── EQ slider drag state ──────────────────────────────────────────────────────

/// Tracks which EQ slider is currently being dragged (by its layout origin_x).
/// All sliders share the same origin_y (same row), so x is the unique identifier.
#[derive(Clone, Default)]
pub struct EqSliderDrag {
    pub active_origin_x: Option<gpui::Pixels>,
}
impl gpui::Global for EqSliderDrag {}

// ── Settings modal ────────────────────────────────────────────────────────────

#[derive(Clone, PartialEq, Default)]
pub enum SettingsTab {
    #[default]
    General,
    Equalizer,
    Playback,
    Sound,
}

#[derive(Clone, Default)]
pub struct EqBandLocal {
    pub cutoff: i32,
    pub q: i32,
    pub gain: i32,
}

pub fn default_eq_bands() -> Vec<EqBandLocal> {
    // API convention: cutoff = gain (tenths dB), q = center freq (Hz), gain = Q factor
    const FREQS: [i32; 10] = [32, 64, 125, 250, 500, 1000, 2000, 4000, 8000, 16000];
    FREQS.iter().map(|&f| EqBandLocal { cutoff: 0, q: f, gain: 7 }).collect()
}

#[derive(Clone)]
pub struct SettingsModal {
    pub open: bool,
    pub active_tab: SettingsTab,
    pub loaded: bool,
    pub loading: bool,
    // General
    pub music_dir: String,
    pub player_name: String,
    // Equalizer
    pub eq_enabled: bool,
    pub eq_precut: u32,
    pub eq_bands: Vec<EqBandLocal>,
    // Playback
    pub shuffle: bool,
    pub crossfade: i32,
    pub crossfade_fade_in_delay: i32,
    pub crossfade_fade_in_duration: i32,
    pub crossfade_fade_out_delay: i32,
    pub crossfade_fade_out_duration: i32,
    pub crossfade_fade_out_mixmode: i32,
    pub replaygain_type: i32,
    pub replaygain_preamp: i32,
    pub replaygain_noclip: bool,
    // Sound
    pub balance: i32,
    pub bass: i32,
    pub treble: i32,
    pub stereo_width: i32,
    pub channel_config: i32,
    pub surround_enabled: i32,
    pub dithering_enabled: bool,
}

impl Default for SettingsModal {
    fn default() -> Self {
        SettingsModal {
            open: false,
            active_tab: SettingsTab::General,
            loaded: false,
            loading: false,
            music_dir: String::new(),
            player_name: String::new(),
            eq_enabled: false,
            eq_precut: 0,
            eq_bands: default_eq_bands(),
            shuffle: false,
            crossfade: 0,
            crossfade_fade_in_delay: 0,
            crossfade_fade_in_duration: 0,
            crossfade_fade_out_delay: 0,
            crossfade_fade_out_duration: 0,
            crossfade_fade_out_mixmode: 0,
            replaygain_type: 3,
            replaygain_preamp: 0,
            replaygain_noclip: false,
            balance: 0,
            bass: 0,
            treble: 0,
            stereo_width: 128,
            channel_config: 0,
            surround_enabled: 0,
            dithering_enabled: false,
        }
    }
}

impl gpui::Global for SettingsModal {}
