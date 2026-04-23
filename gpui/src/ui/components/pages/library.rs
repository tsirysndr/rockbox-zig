use crate::controller::Controller;
use crate::state::format_duration;
use crate::ui::components::icons::{Icon, Icons};
use crate::ui::components::miniplayer::MiniPlayer;
use crate::ui::components::search_input::SearchInput;
use crate::ui::components::{BackSection, LibrarySection, SelectedAlbum, SelectedArtist};
use crate::ui::theme::Theme;
use gpui::prelude::FluentBuilder;
use gpui::{
    div, img, px, uniform_list, AnyElement, App, AppContext, Entity, FontWeight,
    InteractiveElement, IntoElement, MouseButton, ObjectFit, ParentElement, Render,
    StatefulInteractiveElement, Styled, StyledImage, UniformListScrollHandle, Window,
};

const COVERS_BASE: &str = "http://localhost:6062/covers/";

/// Square art tile that fills its container (used in grids). `icon_size` is the size_N shorthand number.
fn art_tile(
    art: Option<String>,
    theme: crate::ui::theme::Theme,
    fallback: Icons,
    icon_size: u8,
) -> AnyElement {
    let art_url = art
        .filter(|s| !s.is_empty())
        .map(|id| format!("{COVERS_BASE}{id}"));
    let mut container = div().w_full().rounded_lg().overflow_hidden();
    container.style().aspect_ratio = Some(1.0_f32);
    if let Some(url) = art_url {
        container
            .child(img(url).w_full().h_full().object_fit(ObjectFit::Cover))
            .into_any_element()
    } else {
        container
            .bg(theme.library_art_bg)
            .flex()
            .items_center()
            .justify_center()
            .text_color(theme.player_icons_text)
            .child(icon_sized(fallback, icon_size))
            .into_any_element()
    }
}

/// Fixed-size square art element (used in detail headers).
fn art_fixed(
    art: Option<String>,
    theme: crate::ui::theme::Theme,
    fallback: Icons,
    size: gpui::Pixels,
) -> AnyElement {
    let art_url = art
        .filter(|s| !s.is_empty())
        .map(|id| format!("{COVERS_BASE}{id}"));
    if let Some(url) = art_url {
        div()
            .w(size)
            .h(size)
            .rounded_lg()
            .flex_shrink_0()
            .overflow_hidden()
            .child(img(url).w_full().h_full().object_fit(ObjectFit::Cover))
            .into_any_element()
    } else {
        div()
            .w(size)
            .h(size)
            .rounded_lg()
            .flex_shrink_0()
            .bg(theme.library_art_bg)
            .flex()
            .items_center()
            .justify_center()
            .text_color(theme.player_icons_text)
            .child(icon_sized(fallback, 10))
            .into_any_element()
    }
}

fn icon_sized(icon: Icons, size: u8) -> Icon {
    use crate::ui::components::icons::Icon;
    let i = Icon::new(icon);
    match size {
        4 => i.size_4(),
        5 => i.size_5(),
        6 => i.size_6(),
        8 => i.size_8(),
        10 => i.size_10(),
        16 => i.size_16(),
        _ => i.size_8(),
    }
}

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
        let (
            n_songs,
            albums,
            artists,
            current_idx,
            album_tracks,
            album_artist,
            album_detail_art,
            artist_tracks,
            artist_albums_detail,
            artist_detail_image,
        ) = {
            let state = cx.global::<Controller>().state.read(cx);

            let current_idx = state.current_library_idx();
            let n_songs = state.tracks.len();

            // (name, artist, count, album_art)
            let mut album_map: std::collections::BTreeMap<String, (String, usize, Option<String>)> =
                Default::default();
            for track in &state.tracks {
                let e = album_map.entry(track.album.clone()).or_insert((
                    track.artist.clone(),
                    0,
                    track.album_art.clone(),
                ));
                e.1 += 1;
            }
            let albums: Vec<(String, String, usize, Option<String>)> = album_map
                .into_iter()
                .map(|(name, (artist, count, art))| (name, artist, count, art))
                .collect();

            let mut artist_map: std::collections::BTreeMap<String, usize> = Default::default();
            for track in &state.tracks {
                *artist_map.entry(track.artist.clone()).or_default() += 1;
            }
            // (name, count, image)
            let artists: Vec<(String, usize, Option<String>)> = artist_map
                .into_iter()
                .map(|(name, count)| {
                    let img = state.artist_images.get(&name).cloned();
                    (name, count, img)
                })
                .collect();

            // Album detail: tracks filtered by selected album — (global_idx, path, title, num, dur)
            let mut album_tracks: Vec<(usize, String, String, String, u64)> = state
                .tracks
                .iter()
                .enumerate()
                .filter(|(_, t)| t.album == selected_album)
                .map(|(idx, t)| {
                    (
                        idx,
                        t.path.clone(),
                        t.title.clone(),
                        t.track_number.to_string(),
                        t.duration,
                    )
                })
                .collect();
            album_tracks.sort_by_key(|(_, _, _, num, _)| num.parse::<u32>().unwrap_or(0));

            let album_first_track = state.tracks.iter().find(|t| t.album == selected_album);
            let album_artist = album_first_track
                .map(|t| t.artist.clone())
                .unwrap_or_default();
            let album_detail_art = album_first_track.and_then(|t| t.album_art.clone());

            // Artist detail: tracks filtered by selected artist — (global_idx, path, title, album, dur)
            let artist_tracks: Vec<(usize, String, String, String, u64)> = state
                .tracks
                .iter()
                .enumerate()
                .filter(|(_, t)| t.artist == selected_artist)
                .map(|(idx, t)| {
                    (
                        idx,
                        t.path.clone(),
                        t.title.clone(),
                        t.album.clone(),
                        t.duration,
                    )
                })
                .collect();

            // (album_name, count, album_art)
            let mut artist_album_map: std::collections::BTreeMap<String, (usize, Option<String>)> =
                Default::default();
            for track in &state.tracks {
                if track.artist == selected_artist {
                    let e = artist_album_map
                        .entry(track.album.clone())
                        .or_insert((0, track.album_art.clone()));
                    e.0 += 1;
                }
            }
            let artist_albums_detail: Vec<(String, usize, Option<String>)> = artist_album_map
                .into_iter()
                .map(|(n, (c, a))| (n, c, a))
                .collect();

            let artist_detail_image = state.artist_images.get(&selected_artist).cloned();

            (
                n_songs,
                albums,
                artists,
                current_idx,
                album_tracks,
                album_artist,
                album_detail_art,
                artist_tracks,
                artist_albums_detail,
                artist_detail_image,
            )
        };

        let n_album_tracks = album_tracks.len();
        let n_artist_tracks = artist_tracks.len();
        let scroll_handle = self.scroll_handle.clone();
        let _detail_scroll_handle = self.detail_scroll_handle.clone();

        // Sidebar nav item — Albums/Artists stay active while in their detail view
        let make_nav_item = move |icon: Icons, label: &'static str, target: LibrarySection| {
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
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_x_2()
                        .child(Icon::new(icon).size_5())
                        .child(label),
                )
        };

        // ── track row helper (shared between Songs, AlbumDetail, ArtistDetail) ──────
        let track_row = move |row_id: (&'static str, usize),
                              path: String,
                              num: String,
                              title: String,
                              artist: Option<String>,
                              album: Option<String>,
                              duration: u64,
                              is_current: bool| {
            let show_artist = artist.is_some();
            let show_album = album.is_some();
            let artist_text = artist.unwrap_or_default();
            let album_text = album.unwrap_or_default();
            div()
                .id(row_id)
                .w_full()
                .flex()
                .items_center()
                .gap_x_4()
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
                    let rt = cx.global::<Controller>().rt();
                    rt.spawn(crate::client::play_track(path.clone()));
                })
                .child(
                    div()
                        .w(px(28.0))
                        .flex_shrink_0()
                        .text_sm()
                        .text_color(theme.library_header_text)
                        .child(num),
                )
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
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
                .when(show_artist, move |this| {
                    this.child(
                        div()
                            .w_40()
                            .flex_shrink_0()
                            .text_sm()
                            .truncate()
                            .text_color(theme.library_header_text)
                            .child(artist_text),
                    )
                })
                .when(show_album, move |this| {
                    this.child(
                        div()
                            .w_40()
                            .flex_shrink_0()
                            .text_sm()
                            .truncate()
                            .text_color(theme.library_header_text)
                            .child(album_text),
                    )
                })
                .child(
                    div()
                        .w(px(56.0))
                        .flex_shrink_0()
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
                        .gap_x_4()
                        .px_6()
                        .py_4()
                        .border_b_1()
                        .border_color(theme.library_table_border)
                        .child(
                            div()
                                .w(px(28.0))
                                .flex_shrink_0()
                                .text_xs()
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(theme.library_header_text)
                                .child("#"),
                        )
                        .child(
                            div()
                                .flex_1()
                                .min_w_0()
                                .text_xs()
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(theme.library_header_text)
                                .child("TITLE"),
                        )
                        .child(
                            div()
                                .w_40()
                                .flex_shrink_0()
                                .text_xs()
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(theme.library_header_text)
                                .child("ARTIST"),
                        )
                        .child(
                            div()
                                .w_40()
                                .flex_shrink_0()
                                .text_xs()
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(theme.library_header_text)
                                .child("ALBUM"),
                        )
                        .child(
                            div()
                                .w(px(56.0))
                                .flex_shrink_0()
                                .text_xs()
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(theme.library_header_text)
                                .child("TIME"),
                        ),
                )
                .child(
                    uniform_list("library_tracks", n_songs, move |range, _window, cx| {
                        let state = cx.global::<Controller>().state.read(cx);
                        let current_idx = state.current_library_idx();
                        range
                            .map(|idx| {
                                let track = &state.tracks[idx];
                                let is_current = current_idx == Some(idx);
                                track_row(
                                    ("track_row", idx),
                                    track.path.clone(),
                                    (idx + 1).to_string(),
                                    track.title.clone(),
                                    Some(track.artist.clone()),
                                    Some(track.album.clone()),
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
                            |(idx, (name, artist, _count, album_art))| {
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
                                    .child(art_tile(album_art, theme, Icons::Music, 8))
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
                        .children(artists.into_iter().enumerate().map(
                            |(idx, (name, count, image))| {
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
                                        let img_url = image.filter(|s| !s.is_empty()).map(|s| {
                                            if s.starts_with("http") {
                                                s
                                            } else {
                                                format!("{COVERS_BASE}{s}")
                                            }
                                        });
                                        let mut container = div()
                                            .w_full()
                                            .rounded_full()
                                            .overflow_hidden()
                                            .flex_shrink_0();
                                        container.style().aspect_ratio = Some(1.0_f32);
                                        if let Some(url) = img_url {
                                            container.child(
                                                img(url)
                                                    .w_full()
                                                    .h_full()
                                                    .rounded_full()
                                                    .object_fit(ObjectFit::Cover),
                                            )
                                        } else {
                                            container
                                                .bg(theme.library_art_bg)
                                                .flex()
                                                .items_center()
                                                .justify_center()
                                                .text_color(theme.player_icons_text)
                                                .child(Icon::new(Icons::Artist).size_8())
                                        }
                                    })
                                    .child(
                                        div()
                                            .w_full()
                                            .flex()
                                            .flex_col()
                                            .items_center()
                                            .gap_y_0p5()
                                            .child(
                                                div()
                                                    .w_full()
                                                    .text_sm()
                                                    .font_weight(FontWeight(500.0))
                                                    .text_color(theme.library_text)
                                                    .text_center()
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
                            },
                        )),
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
                                            .child(art_fixed(
                                                album_detail_art,
                                                theme,
                                                Icons::Music,
                                                px(128.0),
                                            ))
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
                                    .gap_x_4()
                                    .px_6()
                                    .py_3()
                                    .border_b_1()
                                    .border_color(theme.library_table_border)
                                    .child(
                                        div()
                                            .w(px(28.0))
                                            .flex_shrink_0()
                                            .text_xs()
                                            .font_weight(FontWeight::MEDIUM)
                                            .text_color(theme.library_header_text)
                                            .child("#"),
                                    )
                                    .child(
                                        div()
                                            .flex_1()
                                            .min_w_0()
                                            .text_xs()
                                            .font_weight(FontWeight::MEDIUM)
                                            .text_color(theme.library_header_text)
                                            .child("TITLE"),
                                    )
                                    .child(
                                        div()
                                            .w(px(56.0))
                                            .flex_shrink_0()
                                            .text_xs()
                                            .font_weight(FontWeight::MEDIUM)
                                            .text_color(theme.library_header_text)
                                            .child("TIME"),
                                    ),
                            )
                            // Track rows
                            .children(album_tracks.into_iter().enumerate().map(
                                |(i, (global_idx, path, title, num, duration))| {
                                    let is_current = current_idx == Some(global_idx);
                                    track_row(
                                        ("album_detail_row", i),
                                        path,
                                        num,
                                        title,
                                        None,
                                        None,
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
                                            .child({
                                                let img_url = artist_detail_image
                                                    .filter(|s| !s.is_empty())
                                                    .map(|s| {
                                                        if s.starts_with("http") {
                                                            s
                                                        } else {
                                                            format!("{COVERS_BASE}{s}")
                                                        }
                                                    });
                                                if let Some(url) = img_url {
                                                    div()
                                                        .w(px(96.0))
                                                        .h(px(96.0))
                                                        .rounded_full()
                                                        .flex_shrink_0()
                                                        .overflow_hidden()
                                                        .child(
                                                            img(url)
                                                                .w_full()
                                                                .h_full()
                                                                .rounded_full()
                                                                .object_fit(ObjectFit::Cover),
                                                        )
                                                        .into_any_element()
                                                } else {
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
                                                        .child(Icon::new(Icons::Artist).size_8())
                                                        .into_any_element()
                                                }
                                            })
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
                                    .children(artist_albums_detail.into_iter().enumerate().map(
                                        |(idx, (album_name, _count, album_art))| {
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
                                                .child(art_tile(album_art, theme, Icons::Music, 6))
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .font_weight(FontWeight(500.0))
                                                        .text_color(theme.library_text)
                                                        .truncate()
                                                        .child(album_name),
                                                )
                                        },
                                    )),
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
                                    .gap_x_4()
                                    .px_6()
                                    .py_3()
                                    .border_b_1()
                                    .border_color(theme.library_table_border)
                                    .child(
                                        div()
                                            .w(px(28.0))
                                            .flex_shrink_0()
                                            .text_xs()
                                            .font_weight(FontWeight::MEDIUM)
                                            .text_color(theme.library_header_text)
                                            .child("#"),
                                    )
                                    .child(
                                        div()
                                            .flex_1()
                                            .min_w_0()
                                            .text_xs()
                                            .font_weight(FontWeight::MEDIUM)
                                            .text_color(theme.library_header_text)
                                            .child("TITLE"),
                                    )
                                    .child(
                                        div()
                                            .w_40()
                                            .flex_shrink_0()
                                            .text_xs()
                                            .font_weight(FontWeight::MEDIUM)
                                            .text_color(theme.library_header_text)
                                            .child("ALBUM"),
                                    )
                                    .child(
                                        div()
                                            .w(px(56.0))
                                            .flex_shrink_0()
                                            .text_xs()
                                            .font_weight(FontWeight::MEDIUM)
                                            .text_color(theme.library_header_text)
                                            .child("TIME"),
                                    ),
                            )
                            // Artist track rows
                            .children(artist_tracks.into_iter().enumerate().map(
                                |(i, (global_idx, path, title, album, duration))| {
                                    let is_current = current_idx == Some(global_idx);
                                    track_row(
                                        ("artist_detail_row", i),
                                        path,
                                        format!("{}", i + 1),
                                        title,
                                        None,
                                        Some(album),
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
                            .child(make_nav_item(Icons::Music, "Songs", LibrarySection::Songs))
                            .child(make_nav_item(Icons::Disc, "Albums", LibrarySection::Albums))
                            .child(make_nav_item(
                                Icons::Artist,
                                "Artists",
                                LibrarySection::Artists,
                            )),
                    )
                    .child(content),
            )
            .child(self.miniplayer.clone())
    }
}
