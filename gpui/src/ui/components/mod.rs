pub mod controlbar;
pub mod icons;
pub mod miniplayer;
pub mod navbar;
pub mod pages;
pub mod search_input;
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
}
impl gpui::Global for LibrarySection {}

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
    pub artist: String,
    pub album: String,
}

#[derive(Clone, Default)]
pub struct LibraryContextMenuState(pub Option<LibraryContextMenu>);
impl gpui::Global for LibraryContextMenuState {}
