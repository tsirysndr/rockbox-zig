pub mod controlbar;
pub mod icons;
pub mod miniplayer;
pub mod navbar;
pub mod pages;
pub mod titlebar;

#[derive(Clone, Copy, PartialEq)]
pub enum Page {
    Library,
    Player,
    Queue,
}

impl gpui::Global for Page {}
