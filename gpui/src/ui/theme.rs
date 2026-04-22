use gpui::{rgb, rgba, Rgba};

#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
pub struct Theme {
    // App
    pub app_bg: Rgba,

    // Titlebar
    pub titlebar_bg: Rgba,
    pub titlebar_window_icons_text: Rgba,
    pub titlebar_window_icons_bg_hover: Rgba,

    // Page Switcher
    pub switcher_bg: Rgba,
    pub switcher_active: Rgba,
    pub switcher_text: Rgba,
    pub switcher_text_hover: Rgba,
    pub switcher_text_active: Rgba,

    // Player
    pub player_bg: Rgba,
    pub player_title_text: Rgba,
    pub player_artist_text: Rgba,
    pub player_icons_text: Rgba,
    pub player_icons_text_hover: Rgba,
    pub player_icons_text_active: Rgba,
    pub player_icons_bg: Rgba,
    pub player_icons_bg_hover: Rgba,
    pub player_icons_bg_active: Rgba,
    pub player_play_pause_bg: Rgba,
    pub player_play_pause_hover: Rgba,
    pub player_play_pause_text: Rgba,

    // Queue
    pub queue_bg: Rgba,
    pub queue_heading_text: Rgba,
    pub queue_item_title: Rgba,
    pub queue_item_title_current: Rgba,
    pub queue_item_artist: Rgba,
    pub queue_item_bg_hover: Rgba,
    pub queue_item_bg_current: Rgba,

    // Controlbar
    pub playback_slider_track: Rgba,
    pub playback_slider_fill: Rgba,
    pub playback_position_text: Rgba,
    pub volume_icon: Rgba,
    pub volume_slider_track: Rgba,
    pub volume_slider_fill: Rgba,

    // Library
    pub library_bg: Rgba,
    pub library_text: Rgba,
    pub library_header_text: Rgba,
    pub library_table_border: Rgba,
    pub library_track_border: Rgba,
    pub library_track_bg_hover: Rgba,
    pub library_track_bg_active: Rgba,
    pub library_track_title_active: Rgba,
    pub library_empty_text: Rgba,

    // Common
    pub border: Rgba,
    pub scrollbar_thumb: Rgba,

    // Library art / avatar placeholder
    pub library_art_bg: Rgba,
}

impl Default for Theme {
    #[allow(clippy::unreadable_literal)]
    fn default() -> Self {
        Theme {
            app_bg: rgb(0x0F1117),

            titlebar_bg: rgb(0x0F1117),
            titlebar_window_icons_text: rgba(0xFFFFFFCC),
            titlebar_window_icons_bg_hover: rgba(0xFFFFFF29),

            switcher_bg: rgba(0xFFFFFF0D),
            switcher_active: rgb(0x6F00FF),
            switcher_text: rgba(0xFFFFFFCC),
            switcher_text_hover: rgb(0xFFFFFF),
            switcher_text_active: rgb(0x0F1117),

            player_bg: rgb(0x0F1117),
            player_title_text: rgb(0xFFFFFF),
            player_artist_text: rgb(0x9898A8),

            player_icons_text: rgb(0x9090A3),
            player_icons_text_hover: rgb(0x6F00FF),
            player_icons_text_active: rgb(0x6F00FF),

            player_icons_bg: rgba(0xFFFFFF00),
            player_icons_bg_hover: rgba(0x6F00FF14),
            player_icons_bg_active: rgba(0x6F00FF14),

            player_play_pause_bg: rgb(0x6F00FF),
            player_play_pause_hover: rgba(0x6F00FFE6),
            player_play_pause_text: rgb(0x0F1117),

            queue_bg: rgb(0x0F1117),
            queue_heading_text: rgb(0xFFFFFF),
            queue_item_title: rgb(0xFFFFFF),
            queue_item_title_current: rgb(0xFFFFFF),
            queue_item_artist: rgb(0x9090A3),
            queue_item_bg_hover: rgba(0xFFFFFF0D),
            queue_item_bg_current: rgb(0x1A0E3D),

            playback_slider_track: rgba(0xFFFFFF1A),
            playback_slider_fill: rgb(0x6F00FF),
            playback_position_text: rgb(0x9898A8),

            volume_icon: rgb(0x9898A8),
            volume_slider_track: rgba(0xFFFFFF1A),
            volume_slider_fill: rgb(0x6F00FF),

            library_bg: rgb(0x0F1117),
            library_text: rgb(0xFFFFFF),
            library_header_text: rgb(0x9090A3),
            library_table_border: rgba(0xFFFFFF1A),
            library_track_border: rgba(0xFFFFFF1A),
            library_track_bg_hover: rgba(0xFFFFFF0D),
            library_track_bg_active: rgb(0x1A0E3D),
            library_track_title_active: rgb(0xFFFFFF),
            library_empty_text: rgb(0x9090A3),

            border: rgba(0xFFFFFF29),
            scrollbar_thumb: rgb(0x6F00FF),

            library_art_bg: rgba(0xFFFFFF0D),
        }
    }
}

impl gpui::Global for Theme {}
