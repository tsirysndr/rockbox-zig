use crate::controller::Controller;
use crate::state::format_duration;
use crate::ui::components::icons::{Icon, Icons};
use crate::ui::components::miniplayer::MiniPlayer;
use crate::ui::components::search_input::SearchInput;
use crate::ui::components::{BackSection, LibrarySection, SelectedAlbum, SelectedArtist};
use crate::ui::theme::Theme;
use gpui::prelude::FluentBuilder;
use gpui::{
    div, px, uniform_list, App, AppContext, Entity, FontWeight, InteractiveElement, IntoElement,
    MouseButton, ParentElement, Render, StatefulInteractiveElement, Styled, UniformListScrollHandle,
    Window,
};

pub struct LibraryPage {
    scroll_handle: UniformListScrollHandle,
    detail_scroll_handle: UniformListScrollHandle,
    miniplayer: Entity<MiniPlayer>,
    search_input: Entity<SearchInput>,
}

impl LibraryPage {
    pub fn new(cx: &mut App) -> Self {
        cx.set_global(LibrarySection::Songs);
        cx.set_global(SelectedAlbum(String::new()));
        cx.set_global(SelectedArtist(String::new()));
        cx.set_global(BackSection(LibrarySection::Albums));
        LibraryPage {
            scroll_handle: UniformListScrollHandle::new(),
            detail_scroll_handle: UniformListScrollHandle::new(),
            miniplayer: cx.new(|_| MiniPlayer),
            search_input: cx.new(|cx| SearchInput::new(cx)),
        }
    }
}

impl Render for LibraryPage {
    fn render(&mut self, window: &mut Window, cx: &mut gpui::Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let section = *cx.global::<LibrarySection>();
        let back_section = cx.global::<BackSection>().0;
        let selected_album = cx.global::<SelectedAlbum>().0.clone();
        let selected_artist = cx.global::<SelectedArtist>().0.clone();

        let content_width = f32::from(window.viewport_size().width) - 200.0;
        let album_cols = ((content_width / 200.0).floor() as u16).max(2);
        let artist_cols = ((content_width / 160.0).floor() as u16).max(2);
        let detail_album_cols = ((content_width / 180.0).floor() as u16).max(2);

        // Pre-compute all section data in a single state borrow
        let (n_songs, albums, artists, current_idx, album_tracks, album_artist, artist_tracks, artist_albums_detail) = {
            let state = cx.global::<Controller>().state.read(cx);

            let current_idx = state.current_idx;
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

            // Album detail: tracks filtered by selected album
            let album_tracks: Vec<(usize, String, String, u64)> = state
                .tracks
                .iter()
                .enumerate()
                .filter(|(_, t)| t.album == selected_album)
                .map(|(idx, t)| (idx, t.title.clone(), t.track_number.to_string(), t.duration))
                .collect();

            let album_artist = state
                .tracks
                .iter()
                .find(|t| t.album == selected_album)
                .map(|t| t.artist.clone())
                .unwrap_or_default();

            // Artist detail: tracks and albums filtered by selected artist
            let artist_tracks: Vec<(usize, String, String, u64)> = state
                .tracks
                .iter()
                .enumerate()
                .filter(|(_, t)| t.artist == selected_artist)
                .map(|(idx, t)| (idx, t.title.clone(), t.album.clone(), t.duration))
                .collect();

            let mut artist_album_map: std::collections::BTreeMap<String, usize> =
                Default::default();
            for track in &state.tracks {
                if track.artist == selected_artist {
                    *artist_album_map.entry(track.album.clone()).or_default() += 1;
                }
            }
            let artist_albums_detail: Vec<(String, usize)> =
                artist_album_map.into_iter().collect();

            (
                n_songs,
                albums,
                artists,
                current_idx,
                album_tracks,
                album_artist,
                artist_tracks,
                artist_albums_detail,
            )
        };

        let n_album_tracks = album_tracks.len();
        let n_artist_tracks = artist_tracks.len();
        let scroll_handle = self.scroll_handle.clone();
        let _detail_scroll_handle = self.detail_scroll_handle.clone();

        // Sidebar nav item — Albums/Artists stay active while in their detail view
        let make_nav_item = move |label: &'static str, target: LibrarySection| {
            let is_active = section == target
                || (section == LibrarySection::AlbumDetail && target == LibrarySection::Albums)
                || (section == LibrarySection::ArtistDetail && target == LibrarySection::Artists)
                || (section == LibrarySection::AlbumDetail
                    && back_section == LibrarySection::ArtistDetail
                    && target == LibrarySection::Artists);
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

        // ── track row helper (shared between Songs, AlbumDetail, ArtistDetail) ──────
        let track_row =
            move |row_id: (&'static str, usize),
                  global_idx: usize,
                  num: String,
                  title: String,
                  secondary: String, // artist or album depending on context
                  show_secondary: bool,
                  duration: u64,
                  is_current: bool| {
                div()
                    .id(row_id)
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
                            s.play_track(global_idx)
                        });
                    })
                    .child(
                        div()
                            .w(px(32.0))
                            .text_sm()
                            .text_color(theme.library_header_text)
                            .child(num),
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
                            .child(title),
                    )
                    .when(show_secondary, |this| {
                        this.child(
                            div()
                                .w_48()
                                .text_sm()
                                .truncate()
                                .text_color(theme.library_header_text)
                                .child(secondary),
                        )
                    })
                    .child(
                        div()
                            .w_16()
                            .text_sm()
                            .text_color(theme.library_header_text)
                            .child(format_duration(duration)),
                    )
            };

        let content = match section {
            // ── Songs ─────────────────────────────────────────────────────────────
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
                        let _theme = *cx.global::<Theme>();
                        let state = cx.global::<Controller>().state.read(cx);
                        let current_idx = state.current_idx;
                        range
                            .map(|idx| {
                                let track = &state.tracks[idx];
                                let is_current = current_idx == Some(idx);
                                track_row(
                                    ("track_row", idx),
                                    idx,
                                    track.track_number.to_string(),
                                    track.title.clone(),
                                    track.artist.clone(),
                                    true,
                                    track.duration,
                                    is_current,
                                )
                            })
                            .collect()
                    })
                    .flex_1()
                    .w_full()
                    .track_scroll(scroll_handle),
                )
                .into_any_element(),

            // ── Albums grid ───────────────────────────────────────────────────────
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
                                let name_clone = name.clone();
                                div()
                                    .id(("album_card", idx))
                                    .flex()
                                    .flex_col()
                                    .gap_y_2()
                                    .cursor_pointer()
                                    .hover(|this| this.opacity(0.8))
                                    .on_click(move |_, _, cx: &mut App| {
                                        *cx.global_mut::<SelectedAlbum>() =
                                            SelectedAlbum(name_clone.clone());
                                        *cx.global_mut::<BackSection>() =
                                            BackSection(LibrarySection::Albums);
                                        *cx.global_mut::<LibrarySection>() =
                                            LibrarySection::AlbumDetail;
                                    })
                                    .child({
                                        let mut art = div()
                                            .w_full()
                                            .rounded_lg()
                                            .bg(theme.library_art_bg)
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

            // ── Artists grid ──────────────────────────────────────────────────────
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
                            let name_clone = name.clone();
                            div()
                                .id(("artist_card", idx))
                                .flex()
                                .flex_col()
                                .items_center()
                                .gap_y_2()
                                .cursor_pointer()
                                .hover(|this| this.opacity(0.8))
                                .on_click(move |_, _, cx: &mut App| {
                                    *cx.global_mut::<SelectedArtist>() =
                                        SelectedArtist(name_clone.clone());
                                    *cx.global_mut::<LibrarySection>() =
                                        LibrarySection::ArtistDetail;
                                })
                                .child({
                                    let mut avatar = div()
                                        .w_full()
                                        .rounded_full()
                                        .bg(theme.library_art_bg)
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

            // ── Album Detail ──────────────────────────────────────────────────────
            LibrarySection::AlbumDetail => {
                let back_label = if back_section == LibrarySection::ArtistDetail {
                    format!("← {}", selected_artist)
                } else {
                    "← Albums".to_string()
                };
                let n_tracks_label = format!(
                    "{} track{}",
                    n_album_tracks,
                    if n_album_tracks == 1 { "" } else { "s" }
                );
                let album_name_display = selected_album.clone();
                let album_artist_display = album_artist.clone();

                div()
                    .id("album_detail_scroll")
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .child(
                        div()
                            .w_full()
                            .flex()
                            .flex_col()
                            // Header
                            .child(
                                div()
                                    .px_6()
                                    .pt_5()
                                    .pb_6()
                                    .flex()
                                    .flex_col()
                                    .gap_y_5()
                                    // Back button
                                    .child(
                                        div()
                                            .id("album_detail_back")
                                            .cursor_pointer()
                                            .text_sm()
                                            .text_color(theme.library_header_text)
                                            .hover(|this| this.text_color(theme.library_text))
                                            .on_click(move |_, _, cx: &mut App| {
                                                *cx.global_mut::<LibrarySection>() = back_section;
                                            })
                                            .child(back_label),
                                    )
                                    // Album info row
                                    .child(
                                        div()
                                            .flex()
                                            .items_center()
                                            .gap_x_6()
                                            .child(
                                                div()
                                                    .w(px(128.0))
                                                    .h(px(128.0))
                                                    .rounded_lg()
                                                    .flex_shrink_0()
                                                    .bg(theme.library_art_bg)
                                                    .flex()
                                                    .items_center()
                                                    .justify_center()
                                                    .text_color(theme.player_icons_text)
                                                    .child(Icon::new(Icons::Music).size_10()),
                                            )
                                            .child(
                                                div()
                                                    .flex()
                                                    .flex_col()
                                                    .gap_y_1()
                                                    .child(
                                                        div()
                                                            .text_2xl()
                                                            .font_weight(FontWeight(700.0))
                                                            .text_color(theme.library_text)
                                                            .child(album_name_display),
                                                    )
                                                    .child(
                                                        div()
                                                            .text_base()
                                                            .text_color(theme.library_header_text)
                                                            .child(album_artist_display),
                                                    )
                                                    .child(
                                                        div()
                                                            .text_sm()
                                                            .text_color(theme.library_header_text)
                                                            .child(n_tracks_label),
                                                    ),
                                            ),
                                    ),
                            )
                            // Track list header
                            .child(
                                div()
                                    .w_full()
                                    .flex()
                                    .items_center()
                                    .px_6()
                                    .py_3()
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
                                            .w_16()
                                            .text_xs()
                                            .font_weight(FontWeight::MEDIUM)
                                            .text_color(theme.library_header_text)
                                            .child("TIME"),
                                    ),
                            )
                            // Track rows
                            .children(album_tracks.into_iter().enumerate().map(
                                |(i, (global_idx, title, num, duration))| {
                                    let is_current = current_idx == Some(global_idx);
                                    track_row(
                                        ("album_detail_row", i),
                                        global_idx,
                                        num,
                                        title,
                                        String::new(),
                                        false,
                                        duration,
                                        is_current,
                                    )
                                },
                            )),
                    )
                    .into_any_element()
            }

            // ── Artist Detail ─────────────────────────────────────────────────────
            LibrarySection::ArtistDetail => {
                let n_tracks_label = format!(
                    "{} track{}",
                    n_artist_tracks,
                    if n_artist_tracks == 1 { "" } else { "s" }
                );
                let artist_name_display = selected_artist.clone();
                let sa_clone = selected_artist.clone();

                div()
                    .id("artist_detail_scroll")
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .child(
                        div()
                            .w_full()
                            .flex()
                            .flex_col()
                            // Header
                            .child(
                                div()
                                    .px_6()
                                    .pt_5()
                                    .pb_6()
                                    .flex()
                                    .flex_col()
                                    .gap_y_5()
                                    // Back button
                                    .child(
                                        div()
                                            .id("artist_detail_back")
                                            .cursor_pointer()
                                            .text_sm()
                                            .text_color(theme.library_header_text)
                                            .hover(|this| this.text_color(theme.library_text))
                                            .on_click(|_, _, cx: &mut App| {
                                                *cx.global_mut::<LibrarySection>() =
                                                    LibrarySection::Artists;
                                            })
                                            .child("← Artists"),
                                    )
                                    // Artist info row
                                    .child(
                                        div()
                                            .flex()
                                            .items_center()
                                            .gap_x_6()
                                            .child(
                                                div()
                                                    .w(px(96.0))
                                                    .h(px(96.0))
                                                    .rounded_full()
                                                    .flex_shrink_0()
                                                    .bg(theme.library_art_bg)
                                                    .flex()
                                                    .items_center()
                                                    .justify_center()
                                                    .text_color(theme.player_icons_text)
                                                    .child(Icon::new(Icons::Music).size_8()),
                                            )
                                            .child(
                                                div()
                                                    .flex()
                                                    .flex_col()
                                                    .gap_y_1()
                                                    .child(
                                                        div()
                                                            .text_2xl()
                                                            .font_weight(FontWeight(700.0))
                                                            .text_color(theme.library_text)
                                                            .child(artist_name_display),
                                                    )
                                                    .child(
                                                        div()
                                                            .text_sm()
                                                            .text_color(theme.library_header_text)
                                                            .child(n_tracks_label),
                                                    ),
                                            ),
                                    ),
                            )
                            // Albums section
                            .child(
                                div()
                                    .px_6()
                                    .pb_2()
                                    .text_sm()
                                    .font_weight(FontWeight(600.0))
                                    .text_color(theme.library_text)
                                    .child("Albums"),
                            )
                            .child(
                                div()
                                    .w_full()
                                    .px_6()
                                    .pb_6()
                                    .grid()
                                    .grid_cols(detail_album_cols)
                                    .gap_4()
                                    .children(
                                        artist_albums_detail.into_iter().enumerate().map(
                                            |(idx, (album_name, _count))| {
                                                let album_name_clone = album_name.clone();
                                                let sa = sa_clone.clone();
                                                div()
                                                    .id(("artist_album_card", idx))
                                                    .flex()
                                                    .flex_col()
                                                    .gap_y_2()
                                                    .cursor_pointer()
                                                    .hover(|this| this.opacity(0.8))
                                                    .on_click(move |_, _, cx: &mut App| {
                                                        *cx.global_mut::<SelectedAlbum>() =
                                                            SelectedAlbum(album_name_clone.clone());
                                                        *cx.global_mut::<SelectedArtist>() =
                                                            SelectedArtist(sa.clone());
                                                        *cx.global_mut::<BackSection>() =
                                                            BackSection(LibrarySection::ArtistDetail);
                                                        *cx.global_mut::<LibrarySection>() =
                                                            LibrarySection::AlbumDetail;
                                                    })
                                                    .child({
                                                        let mut art = div()
                                                            .w_full()
                                                            .rounded_lg()
                                                            .bg(theme.library_art_bg)
                                                            .flex()
                                                            .items_center()
                                                            .justify_center()
                                                            .text_color(theme.player_icons_text)
                                                            .child(Icon::new(Icons::Music).size_6());
                                                        art.style().aspect_ratio = Some(1.0_f32);
                                                        art
                                                    })
                                                    .child(
                                                        div()
                                                            .text_xs()
                                                            .font_weight(FontWeight(500.0))
                                                            .text_color(theme.library_text)
                                                            .truncate()
                                                            .child(album_name),
                                                    )
                                            },
                                        ),
                                    ),
                            )
                            // Songs section header
                            .child(
                                div()
                                    .px_6()
                                    .pb_2()
                                    .text_sm()
                                    .font_weight(FontWeight(600.0))
                                    .text_color(theme.library_text)
                                    .child("Songs"),
                            )
                            .child(
                                div()
                                    .w_full()
                                    .flex()
                                    .items_center()
                                    .px_6()
                                    .py_3()
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
                            // Artist track rows
                            .children(artist_tracks.into_iter().enumerate().map(
                                |(i, (global_idx, title, album, duration))| {
                                    let is_current = current_idx == Some(global_idx);
                                    track_row(
                                        ("artist_detail_row", i),
                                        global_idx,
                                        format!("{}", i + 1),
                                        title,
                                        album,
                                        true,
                                        duration,
                                        is_current,
                                    )
                                },
                            )),
                    )
                    .into_any_element()
            }
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
                            .child(self.search_input.clone())
                            .gap_y_1()
                            .child(make_nav_item("Songs", LibrarySection::Songs))
                            .child(make_nav_item("Albums", LibrarySection::Albums))
                            .child(make_nav_item("Artists", LibrarySection::Artists)),
                    )
                    .child(content),
            )
            .child(self.miniplayer.clone())
    }
}
