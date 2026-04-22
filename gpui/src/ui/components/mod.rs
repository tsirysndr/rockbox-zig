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
}
impl gpui::Global for LibrarySection {}

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
