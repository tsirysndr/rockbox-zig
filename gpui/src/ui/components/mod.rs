pub mod controlbar;
pub mod icons;
pub mod miniplayer;
pub mod navbar;
pub mod pages;
pub mod search_input;
pub mod seek_bar;
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
    Songs,
    Albums,
    Artists,
    AlbumDetail,
    ArtistDetail,
    Likes,
    Files,
}
impl gpui::Global for LibrarySection {}

#[derive(Clone, Default)]
pub struct FilesBrowseState {
    pub current_path: Option<String>,
    pub path_history: Vec<Option<String>>,
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

#[derive(Clone, PartialEq)]
pub struct SelectedAlbum(pub String);
impl gpui::Global for SelectedAlbum {}

#[derive(Clone, PartialEq)]
pub struct SelectedArtist(pub String);
impl gpui::Global for SelectedArtist {}

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
