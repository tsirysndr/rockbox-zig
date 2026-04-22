use crate::controller::Controller;
use crate::state::format_duration;
use crate::ui::components::icons::{Icon, Icons};
use crate::ui::components::miniplayer::MiniPlayer;
use crate::ui::components::search_input::SearchInput;
use crate::ui::components::LibrarySection;
use crate::ui::theme::Theme;
use gpui::prelude::FluentBuilder;
use gpui::{
    div, px, uniform_list, App, AppContext, Entity, FontWeight, InteractiveElement, IntoElement,
    MouseButton, ParentElement, Render, StatefulInteractiveElement, Styled, UniformListScrollHandle,
    Window,
};

pub struct LibraryPage {
    scroll_handle: UniformListScrollHandle,
    miniplayer: Entity<MiniPlayer>,
    search_input: Entity<SearchInput>,
}

impl LibraryPage {
    pub fn new(cx: &mut App) -> Self {
        cx.set_global(LibrarySection::Songs);
        LibraryPage {
            scroll_handle: UniformListScrollHandle::new(),
            miniplayer: cx.new(|_| MiniPlayer),
            search_input: cx.new(|cx| SearchInput::new(cx)),
        }
    }
}

impl Render for LibraryPage {
    fn render(&mut self, window: &mut Window, cx: &mut gpui::Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let section = *cx.global::<LibrarySection>();

        let content_width = f32::from(window.viewport_size().width) - 200.0;
        let album_cols = ((content_width / 200.0).floor() as u16).max(2);
        let artist_cols = ((content_width / 160.0).floor() as u16).max(2);

        // Pre-compute section data while holding state borrow
        let (n_songs, albums, artists) = {
            let state = cx.global::<Controller>().state.read(cx);

            let n_songs = state.tracks.len();

            let mut album_map: std::collections::BTreeMap<String, (String, usize)> =
                Default::default();
            for track in &state.tracks {
                let e = album_map
                    .entry(track.album.clone())
                    .or_insert((track.artist.clone(), 0));
                e.1 += 1;
            }
            let albums: Vec<(String, String, usize)> = album_map
                .into_iter()
                .map(|(name, (artist, count))| (name, artist, count))
                .collect();

            let mut artist_map: std::collections::BTreeMap<String, usize> = Default::default();
            for track in &state.tracks {
                *artist_map.entry(track.artist.clone()).or_default() += 1;
            }
            let artists: Vec<(String, usize)> = artist_map.into_iter().collect();

            (n_songs, albums, artists)
        };

        let scroll_handle = self.scroll_handle.clone();

        // Sidebar nav item builder
        let make_nav_item = move |label: &'static str, target: LibrarySection| {
            let is_active = section == target;
            div()
                .id(label)
                .w_full()
                .px_4()
                .py_2p5()
                .cursor_pointer()
                .text_sm()
                .font_weight(if is_active {
                    FontWeight(600.0)
                } else {
                    FontWeight(400.0)
                })
                .text_color(if is_active {
                    gpui::rgb(0xFFFFFF)
                } else {
                    theme.library_header_text
                })
                .when(is_active, |this| {
                    this.border_l_2().border_color(theme.switcher_active)
                })
                .hover(|this| this.text_color(theme.library_text))
                .on_click(move |_, _, cx: &mut App| {
                    *cx.global_mut::<LibrarySection>() = target;
                })
                .child(label)
        };

        // Content area — switches per section
        let content = match section {
            LibrarySection::Songs => div()
                .flex_1()
                .min_h_0()
                .flex()
                .flex_col()
                .child(
                    div()
                        .w_full()
                        .flex_shrink_0()
                        .flex()
                        .items_center()
                        .px_6()
                        .py_4()
                        .border_b_1()
                        .border_color(theme.library_table_border)
                        .child(
                            div()
                                .w(px(32.0))
                                .text_xs()
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(theme.library_header_text)
                                .child("#"),
                        )
                        .child(
                            div()
                                .flex_1()
                                .text_xs()
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(theme.library_header_text)
                                .child("TITLE"),
                        )
                        .child(
                            div()
                                .w_48()
                                .text_xs()
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(theme.library_header_text)
                                .child("ARTIST"),
                        )
                        .child(
                            div()
                                .w_48()
                                .text_xs()
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(theme.library_header_text)
                                .child("ALBUM"),
                        )
                        .child(
                            div()
                                .w_16()
                                .text_xs()
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(theme.library_header_text)
                                .child("TIME"),
                        ),
                )
                .child(
                    uniform_list("library_tracks", n_songs, move |range, _window, cx| {
                        let theme = *cx.global::<Theme>();
                        let state = cx.global::<Controller>().state.read(cx);
                        let current_idx = state.current_idx;
                        range
                            .map(|idx| {
                                let track = &state.tracks[idx];
                                let is_current = current_idx == Some(idx);
                                div()
                                    .id(("track_row", idx))
                                    .w_full()
                                    .flex()
                                    .items_center()
                                    .px_6()
                                    .py_3()
                                    .cursor_pointer()
                                    .hover(|this| this.bg(theme.library_track_bg_hover))
                                    .when(is_current, |this| {
                                        this.bg(theme.library_track_bg_active)
                                            .border_b_2()
                                            .border_color(theme.switcher_active)
                                    })
                                    .on_click(move |_, _, cx: &mut App| {
                                        let state = cx.global::<Controller>().state.clone();
                                        state.update(cx, |s: &mut crate::state::AppState, _| {
                                            s.play_track(idx)
                                        });
                                    })
                                    .child(
                                        div()
                                            .w(px(32.0))
                                            .text_sm()
                                            .text_color(theme.library_header_text)
                                            .child(track.track_number.to_string()),
                                    )
                                    .child(
                                        div()
                                            .flex_1()
                                            .text_sm()
                                            .truncate()
                                            .text_color(if is_current {
                                                theme.library_track_title_active
                                            } else {
                                                theme.library_text
                                            })
                                            .font_weight(if is_current {
                                                FontWeight(600.0)
                                            } else {
                                                FontWeight(400.0)
                                            })
                                            .child(track.title.clone()),
                                    )
                                    .child(
                                        div()
                                            .w_48()
                                            .text_sm()
                                            .truncate()
                                            .text_color(theme.library_header_text)
                                            .child(track.artist.clone()),
                                    )
                                    .child(
                                        div()
                                            .w_48()
                                            .text_sm()
                                            .truncate()
                                            .text_color(theme.library_header_text)
                                            .child(track.album.clone()),
                                    )
                                    .child(
                                        div()
                                            .w_16()
                                            .text_sm()
                                            .text_color(theme.library_header_text)
                                            .child(format_duration(track.duration)),
                                    )
                            })
                            .collect()
                    })
                    .flex_1()
                    .w_full()
                    .track_scroll(scroll_handle),
                )
                .into_any_element(),

            LibrarySection::Albums => div()
                .id("albums_scroll")
                .flex_1()
                .min_h_0()
                .overflow_y_scroll()
                .child(
                    div()
                        .w_full()
                        .p_6()
                        .grid()
                        .grid_cols(album_cols)
                        .gap_6()
                        .children(albums.into_iter().enumerate().map(
                            |(idx, (name, artist, _count))| {
                                let theme = theme;
                                div()
                                    .id(("album_card", idx))
                                    .flex()
                                    .flex_col()
                                    .gap_y_2()
                                    .cursor_pointer()
                                    .hover(|this| this.opacity(0.8))
                                    .child({
                                        let mut art = div()
                                            .w_full()
                                            .rounded_lg()
                                            .bg(theme.border)
                                            .flex()
                                            .items_center()
                                            .justify_center()
                                            .text_color(theme.player_icons_text)
                                            .child(Icon::new(Icons::Music).size_8());
                                        art.style().aspect_ratio = Some(1.0_f32);
                                        art
                                    })
                                    .child(
                                        div()
                                            .flex()
                                            .flex_col()
                                            .gap_y_0p5()
                                            .child(
                                                div()
                                                    .text_sm()
                                                    .font_weight(FontWeight(500.0))
                                                    .text_color(theme.library_text)
                                                    .truncate()
                                                    .child(name),
                                            )
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .text_color(theme.library_header_text)
                                                    .truncate()
                                                    .child(artist),
                                            ),
                                    )
                            },
                        )),
                )
                .into_any_element(),

            LibrarySection::Artists => div()
                .id("artists_scroll")
                .flex_1()
                .min_h_0()
                .overflow_y_scroll()
                .child(
                    div()
                        .w_full()
                        .p_6()
                        .grid()
                        .grid_cols(artist_cols)
                        .gap_6()
                        .children(artists.into_iter().enumerate().map(|(idx, (name, count))| {
                            let theme = theme;
                            div()
                                .id(("artist_card", idx))
                                .flex()
                                .flex_col()
                                .items_center()
                                .gap_y_2()
                                .cursor_pointer()
                                .hover(|this| this.opacity(0.8))
                                .child({
                                    let mut avatar = div()
                                        .w_full()
                                        .rounded_full()
                                        .bg(theme.border)
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .text_color(theme.player_icons_text)
                                        .child(Icon::new(Icons::Music).size_8());
                                    avatar.style().aspect_ratio = Some(1.0_f32);
                                    avatar
                                })
                                .child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .items_center()
                                        .gap_y_0p5()
                                        .child(
                                            div()
                                                .text_sm()
                                                .font_weight(FontWeight(500.0))
                                                .text_color(theme.library_text)
                                                .truncate()
                                                .child(name),
                                        )
                                        .child(
                                            div()
                                                .text_xs()
                                                .text_color(theme.library_header_text)
                                                .child(format!("{count} tracks")),
                                        ),
                                )
                        })),
                )
                .into_any_element(),
        };

        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(theme.library_bg)
            .on_mouse_down(MouseButton::Left, |_, window, _| window.blur())
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .w_full()
                    .flex()
                    .overflow_hidden()
                    // Sidebar
                    .child(
                        div()
                            .w(px(200.0))
                            .h_full()
                            .flex_shrink_0()
                            .flex()
                            .flex_col()
                            .border_r_1()
                            .border_color(theme.library_table_border)
                            .pt_4()
                            // Search input
                            .child(self.search_input.clone())
                            .gap_y_1()
                            .child(make_nav_item("Songs", LibrarySection::Songs))
                            .child(make_nav_item("Albums", LibrarySection::Albums))
                            .child(make_nav_item("Artists", LibrarySection::Artists)),
                    )
                    // Content area
                    .child(content),
            )
            .child(self.miniplayer.clone())
    }
}
