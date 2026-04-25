use crate::client::{
    insert_directory, insert_track_last, insert_track_next, insert_tracks, play_directory,
    INSERT_FIRST, INSERT_LAST, INSERT_LAST_SHUFFLED, INSERT_SHUFFLED,
};
use crate::controller::Controller;
use crate::state::{format_duration, PlaybackStatus};
use crate::ui::animations::equalizer_bars;
use crate::ui::components::icons::{Icon, Icons};
use crate::ui::components::miniplayer::MiniPlayer;
use crate::ui::components::search_input::SearchInput;
use crate::ui::components::pages::files::{menu_item, FilesView};
use crate::ui::components::{
    AlbumContextMenu, AlbumContextMenuState, BackSection, FileContextMenuState,
    HoveredAlbumIdx, LibraryContextMenu, LibraryContextMenuState, LibrarySection, LikedOrder,
    LikedSongs, SelectedAlbum, SelectedArtist,
};
use crate::ui::theme::Theme;
use gpui::prelude::FluentBuilder;
use gpui::{
    div, img, px, uniform_list, AnyElement, App, AppContext, Entity, FontWeight,
    InteractiveElement, IntoElement, MouseButton, ObjectFit, ParentElement, Render,
    StatefulInteractiveElement, Styled, StyledImage, Subscription, UniformListScrollHandle, Window,
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
    files_view: Entity<FilesView>,
    _search_sub: Option<Subscription>,
}

impl LibraryPage {
    pub fn new(cx: &mut App) -> Self {
        cx.set_global(LibrarySection::Songs);
        cx.set_global(SelectedAlbum(String::new()));
        cx.set_global(SelectedArtist(String::new()));
        cx.set_global(BackSection(LibrarySection::Albums));
        cx.set_global(LibraryContextMenuState::default());
        cx.set_global(AlbumContextMenuState::default());
        cx.set_global(HoveredAlbumIdx::default());
        cx.set_global(LikedSongs::default());
        cx.set_global(LikedOrder::default());
        LibraryPage {
            scroll_handle: UniformListScrollHandle::new(),
            detail_scroll_handle: UniformListScrollHandle::new(),
            miniplayer: cx.new(|_| MiniPlayer),
            search_input: cx.new(|cx| SearchInput::new(cx)),
            files_view: cx.new(|cx| FilesView::new(cx)),
            _search_sub: None,
        }
    }
}

impl Render for LibraryPage {
    fn render(&mut self, window: &mut Window, cx: &mut gpui::Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let section = *cx.global::<LibrarySection>();
        let back_section = cx.global::<BackSection>().0;
        let hovered_album_idx = cx.global::<HoveredAlbumIdx>().0;
        let selected_album = cx.global::<SelectedAlbum>().0.clone();
        let selected_artist = cx.global::<SelectedArtist>().0.clone();

        let viewport = window.viewport_size();
        let liked_songs = cx.global::<LikedSongs>().0.clone();
        let liked_order = cx.global::<LikedOrder>().0.clone();
        let content_width = f32::from(viewport.width) - 200.0;
        let album_cols = ((content_width / 200.0).floor() as u16).max(2);
        let artist_cols = ((content_width / 160.0).floor() as u16).max(2);
        let detail_album_cols = ((content_width / 180.0).floor() as u16).max(2);

        // Pre-compute all section data in a single state borrow
        let (
            n_songs,
            albums,
            artists,
            current_idx,
            current_path,
            album_tracks,
            album_artist,
            album_detail_art,
            album_id,
            artist_tracks,
            artist_albums_detail,
            artist_detail_image,
            artist_id,
            liked_tracks,
            search_results,
            is_playing,
        ) = {
            let state = cx.global::<Controller>().state.read(cx);

            let current_idx = state.current_library_idx();
            let n_songs = state.tracks.len();

            let mut album_map: std::collections::BTreeMap<
                String,
                (String, u32, usize, Option<String>, String, Vec<(u32, String)>),
            > = Default::default();
            for track in &state.tracks {
                let display_artist = if track.album_artist.is_empty() {
                    track.artist.clone()
                } else {
                    track.album_artist.clone()
                };
                let e = album_map.entry(track.album.clone()).or_insert((
                    display_artist,
                    track.year,
                    0,
                    track.album_art.clone(),
                    track.album_id.clone(),
                    Vec::new(),
                ));
                e.2 += 1;
                e.5.push((track.track_number, track.path.clone()));
            }
            let albums: Vec<(String, String, u32, usize, Option<String>, String, Vec<String>)> =
                album_map
                    .into_iter()
                    .map(|(name, (artist, year, count, art, album_id, mut numbered_paths))| {
                        numbered_paths.sort_by_key(|(num, _)| *num);
                        let paths: Vec<String> =
                            numbered_paths.into_iter().map(|(_, p)| p).collect();
                        (name, artist, year, count, art, album_id, paths)
                    })
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

            // Album detail: tracks filtered by selected album — (global_idx, path, title, num, dur, id, album_art)
            let mut album_tracks: Vec<(usize, String, String, String, u64, String, Option<String>)> = state
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
                        t.id.clone(),
                        t.album_art.clone(),
                    )
                })
                .collect();
            album_tracks.sort_by_key(|(_, _, _, num, _, _, _)| num.parse::<u32>().unwrap_or(0));

            let album_first_track = state.tracks.iter().find(|t| t.album == selected_album);
            let album_artist = album_first_track
                .map(|t| {
                    if t.album_artist.is_empty() {
                        t.artist.clone()
                    } else {
                        t.album_artist.clone()
                    }
                })
                .unwrap_or_default();
            let album_detail_art = album_first_track.and_then(|t| t.album_art.clone());
            let album_id = album_first_track
                .map(|t| t.album_id.clone())
                .unwrap_or_default();

            let artist_id = state
                .tracks
                .iter()
                .find(|t| t.artist == selected_artist)
                .map(|t| t.artist_id.clone())
                .unwrap_or_default();

            // Artist detail: tracks filtered by selected artist — (global_idx, path, title, album, dur, id, album_art)
            let artist_tracks: Vec<(usize, String, String, String, u64, String, Option<String>)> = state
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
                        t.id.clone(),
                        t.album_art.clone(),
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

            // (global_idx, path, title, artist, album, duration, id, album_art)
            let liked_order_map: std::collections::HashMap<&str, usize> = liked_order
                .iter()
                .enumerate()
                .map(|(i, id)| (id.as_str(), i))
                .collect();
            let mut liked_tracks: Vec<(usize, String, String, String, String, u64, String, Option<String>)> = state
                .tracks
                .iter()
                .enumerate()
                .filter(|(_, t)| liked_songs.contains(&t.id))
                .map(|(idx, t)| {
                    (
                        idx,
                        t.path.clone(),
                        t.title.clone(),
                        t.artist.clone(),
                        t.album.clone(),
                        t.duration,
                        t.id.clone(),
                        t.album_art.clone(),
                    )
                })
                .collect();
            liked_tracks.sort_by_key(|(_, _, _, _, _, _, id, _)| {
                liked_order_map.get(id.as_str()).copied().unwrap_or(usize::MAX)
            });

            let current_path = state.current_track().map(|t| t.path.clone());
            let search_results = state.search_results.clone();
            let is_playing = state.status == PlaybackStatus::Playing;

            (
                n_songs,
                albums,
                artists,
                current_idx,
                current_path,
                album_tracks,
                album_artist,
                album_detail_art,
                album_id,
                artist_tracks,
                artist_albums_detail,
                artist_detail_image,
                artist_id,
                liked_tracks,
                search_results,
                is_playing,
            )
        };

        // Lazily subscribe to search input changes
        if self._search_sub.is_none() {
            let si = self.search_input.clone();
            self._search_sub = Some(cx.observe(&si, |_this, si_entity, cx| {
                let query = si_entity.read(cx).query.clone();
                cx.global::<Controller>().search(query);
                cx.notify();
            }));
        }
        let query = self.search_input.read(cx).query.clone();

        let context_menu = cx.global::<LibraryContextMenuState>().0.clone();
        let album_context_menu = cx.global::<AlbumContextMenuState>().0.clone();
        let file_context_menu = cx.global::<FileContextMenuState>().0.clone();
        let n_album_tracks = album_tracks.len();
        let n_artist_tracks = artist_tracks.len();
        let scroll_handle = self.scroll_handle.clone();
        let _detail_scroll_handle = self.detail_scroll_handle.clone();

        // Sidebar nav item — Albums/Artists stay active while in their detail view
        let make_nav_item = move |icon: Icons, icon_size: u8, label: &'static str, target: LibrarySection| {
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
                        .child(icon_sized(icon, icon_size))
                        .child(label),
                )
        };

        // ── track row helper (shared between Songs, AlbumDetail, ArtistDetail, Likes) ──────
        let track_row = move |row_id: (&'static str, usize),
                              path: String,
                              num: String,
                              title: String,
                              artist: Option<String>,
                              album: Option<String>,
                              duration: u64,
                              is_current: bool,
                              is_liked: bool,
                              track_artist: String,
                              track_album: String,
                              track_id: String,
                              track_art: Option<String>| {
            let show_artist = artist.is_some();
            let show_album = album.is_some();
            let artist_text = artist.unwrap_or_default();
            let album_text = album.unwrap_or_default();
            let path_for_opts = path.clone();
            let opts_title = title.clone();
            let opts_art = track_art;
            let heart_track_id = track_id.clone();
            let opts_artist = track_artist.clone();
            let opts_album = track_album.clone();
            let group_name: gpui::SharedString =
                format!("track_group_{}_{}", row_id.0, row_id.1).into();
            let group_name2 = group_name.clone();
            let group_name_for_num = group_name.clone();
            let group_name_for_play = group_name.clone();
            let opts_id: gpui::SharedString = format!("{}_opts_{}", row_id.0, row_id.1).into();
            let heart_id: gpui::SharedString = format!("{}_heart_{}", row_id.0, row_id.1).into();
            let play_id: gpui::SharedString = format!("{}_play_{}", row_id.0, row_id.1).into();
            let path_for_play = path.clone();
            div()
                .id(row_id)
                .group(group_name)
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
                .child(
                    div()
                        .w(px(28.0))
                        .h(px(20.0))
                        .flex_shrink_0()
                        .relative()
                        // Track number / equalizer bars — hidden on row hover
                        .child(
                            div()
                                .absolute()
                                .top_0()
                                .left_0()
                                .w_full()
                                .h_full()
                                .flex()
                                .items_center()
                                .group_hover(group_name_for_num, |s| s.opacity(0.0))
                                .when(!is_current, |this| {
                                    this.text_sm()
                                        .text_color(theme.library_header_text)
                                        .child(num)
                                })
                                .when(is_current, |this| {
                                    this.child(equalizer_bars(row_id.1, is_playing))
                                }),
                        )
                        // Play icon — appears on row hover
                        .child(
                            div()
                                .id(play_id)
                                .absolute()
                                .top_0()
                                .left(px(-3.0))
                                .w_full()
                                .h_full()
                                .flex()
                                .items_center()
                                .opacity(0.0)
                                .group_hover(group_name_for_play, |s| s.opacity(1.0))
                                .cursor_pointer()
                                .text_color(theme.library_text)
                                .on_click(move |_, _, cx: &mut App| {
                                    cx.stop_propagation();
                                    let rt = cx.global::<Controller>().rt();
                                    rt.spawn(crate::client::play_track(path_for_play.clone()));
                                })
                                .child(Icon::new(Icons::Play).size_4()),
                        ),
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
                .child(
                    div()
                        .id(heart_id)
                        .w(px(28.0))
                        .flex_shrink_0()
                        .flex()
                        .items_center()
                        .justify_center()
                        .cursor_pointer()
                        .text_color(gpui::rgb(0xFFFFFF))
                        .on_click(move |_, _, cx: &mut App| {
                            cx.stop_propagation();
                            let rt = cx.global::<Controller>().rt();
                            let liked = &mut cx.global_mut::<LikedSongs>().0;
                            if liked.contains(&heart_track_id) {
                                liked.remove(&heart_track_id);
                                rt.spawn(crate::client::unlike_track(heart_track_id.clone()));
                            } else {
                                liked.insert(heart_track_id.clone());
                                rt.spawn(crate::client::like_track(heart_track_id.clone()));
                            }
                        })
                        .child(
                            Icon::new(if is_liked {
                                Icons::Heart
                            } else {
                                Icons::HeartOutline
                            })
                            .size_5(),
                        ),
                )
                .child(
                    div()
                        .id(opts_id)
                        .w(px(28.0))
                        .flex_shrink_0()
                        .flex()
                        .items_center()
                        .justify_center()
                        .opacity(0.0)
                        .group_hover(group_name2, |s| s.opacity(1.0))
                        .cursor_pointer()
                        .text_color(theme.library_header_text)
                        .on_click(move |event, _, cx: &mut App| {
                            cx.stop_propagation();
                            cx.global_mut::<LibraryContextMenuState>().0 =
                                Some(LibraryContextMenu {
                                    pos: event.position(),
                                    path: path_for_opts.clone(),
                                    title: opts_title.clone(),
                                    artist: opts_artist.clone(),
                                    album: opts_album.clone(),
                                    album_art: opts_art.clone(),
                                });
                        })
                        .child(Icon::new(Icons::Options).size_4()),
                )
        };

        let show_search = !query.is_empty()
            && !matches!(
                section,
                LibrarySection::AlbumDetail | LibrarySection::ArtistDetail | LibrarySection::Files
            );
        let content: AnyElement = if show_search {
            // ── Search results ────────────────────────────────────────────────────
            match search_results {
                None => div().flex_1().into_any_element(),
                Some(ref r)
                    if r.tracks.is_empty() && r.albums.is_empty() && r.artists.is_empty() =>
                {
                    div()
                        .flex_1()
                        .flex()
                        .flex_col()
                        .items_center()
                        .justify_center()
                        .gap_y_4()
                        .child(
                            div()
                                .text_color(theme.library_header_text)
                                .child(Icon::new(Icons::Search).size_16()),
                        )
                        .child(
                            div()
                                .text_lg()
                                .font_weight(FontWeight(600.0))
                                .text_color(theme.library_text)
                                .child("No results found"),
                        )
                        .child(
                            div()
                                .text_sm()
                                .text_color(theme.library_header_text)
                                .child("Try searching for something else"),
                        )
                        .into_any_element()
                }
                Some(r) => {
                    div()
                        .id("search_results_scroll")
                        .flex_1()
                        .min_h_0()
                        .overflow_y_scroll()
                        .child(
                            div()
                                .w_full()
                                .p_6()
                                .flex()
                                .flex_col()
                                .gap_y_8()
                                // ── Artists ───────────────────────────────────────
                                .when(!r.artists.is_empty(), |this| {
                                    this.child(
                                        div()
                                            .flex()
                                            .flex_col()
                                            .gap_y_4()
                                            .child(
                                                div()
                                                    .text_base()
                                                    .font_weight(FontWeight(600.0))
                                                    .text_color(theme.library_text)
                                                    .child("Artists"),
                                            )
                                            .child(
                                                div()
                                                    .flex()
                                                    .items_start()
                                                    .gap_x_4()
                                                    .children(r.artists.iter().take(8).enumerate().map(
                                                        |(idx, artist)| {
                                                            let name_clone = artist.name.clone();
                                                            let img_url = artist
                                                                .image
                                                                .as_deref()
                                                                .filter(|s| !s.is_empty())
                                                                .map(|s| {
                                                                    if s.starts_with("http") {
                                                                        s.to_string()
                                                                    } else {
                                                                        format!("{COVERS_BASE}{s}")
                                                                    }
                                                                });
                                                            div()
                                                                .id(("sa", idx))
                                                                .w(px(112.0))
                                                                .flex_shrink_0()
                                                                .flex()
                                                                .flex_col()
                                                                .items_center()
                                                                .gap_y_2()
                                                                .cursor_pointer()
                                                                .on_click(move |_, _, cx: &mut App| {
                                                                    *cx.global_mut::<SelectedArtist>() =
                                                                        SelectedArtist(name_clone.clone());
                                                                    *cx.global_mut::<LibrarySection>() =
                                                                        LibrarySection::ArtistDetail;
                                                                })
                                                                .child({
                                                                    let mut c = div()
                                                                        .w(px(88.0))
                                                                        .rounded_full()
                                                                        .overflow_hidden()
                                                                        .flex_shrink_0();
                                                                    c.style().aspect_ratio = Some(1.0_f32);
                                                                    if let Some(url) = img_url {
                                                                        c.child(
                                                                            img(url)
                                                                                .w_full()
                                                                                .h_full()
                                                                                .rounded_full()
                                                                                .object_fit(ObjectFit::Cover),
                                                                        )
                                                                    } else {
                                                                        c.bg(theme.library_art_bg)
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
                                                                        .text_xs()
                                                                        .font_weight(FontWeight(500.0))
                                                                        .text_color(theme.library_text)
                                                                        .text_center()
                                                                        .truncate()
                                                                        .child(artist.name.clone()),
                                                                )
                                                        },
                                                    )),
                                            ),
                                    )
                                })
                                // ── Albums ────────────────────────────────────────
                                .when(!r.albums.is_empty(), |this| {
                                    this.child(
                                        div()
                                            .flex()
                                            .flex_col()
                                            .gap_y_4()
                                            .child(
                                                div()
                                                    .text_base()
                                                    .font_weight(FontWeight(600.0))
                                                    .text_color(theme.library_text)
                                                    .child("Albums"),
                                            )
                                            .child(
                                                div()
                                                    .flex()
                                                    .items_start()
                                                    .gap_x_4()
                                                    .children(r.albums.iter().take(8).enumerate().map(
                                                        |(idx, album)| {
                                                            let title_clone = album.title.clone();
                                                            let art_url = album
                                                                .album_art
                                                                .as_deref()
                                                                .filter(|s| !s.is_empty())
                                                                .map(|id| {
                                                                    format!("{COVERS_BASE}{id}")
                                                                });
                                                            div()
                                                                .id(("sab", idx))
                                                                .w(px(130.0))
                                                                .flex_shrink_0()
                                                                .flex()
                                                                .flex_col()
                                                                .gap_y_1()
                                                                .cursor_pointer()
                                                                .on_click(move |_, _, cx: &mut App| {
                                                                    *cx.global_mut::<SelectedAlbum>() =
                                                                        SelectedAlbum(title_clone.clone());
                                                                    *cx.global_mut::<BackSection>() =
                                                                        BackSection(LibrarySection::Albums);
                                                                    *cx.global_mut::<LibrarySection>() =
                                                                        LibrarySection::AlbumDetail;
                                                                })
                                                                .child({
                                                                    let mut c = div()
                                                                        .w_full()
                                                                        .rounded_lg()
                                                                        .overflow_hidden();
                                                                    c.style().aspect_ratio = Some(1.0_f32);
                                                                    if let Some(url) = art_url {
                                                                        c.child(
                                                                            img(url)
                                                                                .w_full()
                                                                                .h_full()
                                                                                .object_fit(ObjectFit::Cover),
                                                                        )
                                                                    } else {
                                                                        c.bg(theme.library_art_bg)
                                                                            .flex()
                                                                            .items_center()
                                                                            .justify_center()
                                                                            .text_color(theme.player_icons_text)
                                                                            .child(Icon::new(Icons::Music).size_8())
                                                                    }
                                                                })
                                                                .child(
                                                                    div()
                                                                        .text_xs()
                                                                        .font_weight(FontWeight(500.0))
                                                                        .text_color(theme.library_text)
                                                                        .truncate()
                                                                        .child(album.title.clone()),
                                                                )
                                                                .child(
                                                                    div()
                                                                        .text_xs()
                                                                        .text_color(theme.library_header_text)
                                                                        .truncate()
                                                                        .child(album.artist.clone()),
                                                                )
                                                        },
                                                    )),
                                            ),
                                    )
                                })
                                // ── Songs ─────────────────────────────────────────
                                .when(!r.tracks.is_empty(), |this| {
                                    let rows = r.tracks.iter().take(20).enumerate().map(|(i, track)| {
                                        let is_current =
                                            current_path.as_deref() == Some(track.path.as_str());
                                        let is_liked = liked_songs.contains(&track.id);
                                        track_row(
                                            ("search_track", i),
                                            track.path.clone(),
                                            (i + 1).to_string(),
                                            track.title.clone(),
                                            Some(track.artist.clone()),
                                            Some(track.album.clone()),
                                            track.duration,
                                            is_current,
                                            is_liked,
                                            track.artist.clone(),
                                            track.album.clone(),
                                            track.id.clone(),
                                            track.album_art.clone(),
                                        )
                                    });
                                    this.child(
                                        div()
                                            .flex()
                                            .flex_col()
                                            .gap_y_2()
                                            .child(
                                                div()
                                                    .text_base()
                                                    .font_weight(FontWeight(600.0))
                                                    .text_color(theme.library_text)
                                                    .child("Songs"),
                                            )
                                            .children(rows),
                                    )
                                }),
                        )
                        .into_any_element()
                }
            }
        } else {
        let content_inner = match section {
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
                        )
                        .child(div().w(px(28.0)).flex_shrink_0())
                        .child(div().w(px(28.0)).flex_shrink_0()),
                )
                .child(
                    uniform_list("library_tracks", n_songs, move |range, _window, cx| {
                        let state = cx.global::<Controller>().state.read(cx);
                        let current_idx = state.current_library_idx();
                        let liked = cx.global::<LikedSongs>().0.clone();
                        range
                            .map(|idx| {
                                let track = &state.tracks[idx];
                                let is_current = current_idx == Some(idx);
                                let is_liked = liked.contains(&track.id);
                                track_row(
                                    ("track_row", idx),
                                    track.path.clone(),
                                    (idx + 1).to_string(),
                                    track.title.clone(),
                                    Some(track.artist.clone()),
                                    Some(track.album.clone()),
                                    track.duration,
                                    is_current,
                                    is_liked,
                                    track.artist.clone(),
                                    track.album.clone(),
                                    track.id.clone(),
                                    track.album_art.clone(),
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
                            |(idx, (name, artist, year, _count, album_art, album_id, track_paths))| {
                                let name_clone = name.clone();
                                let name_clone_opts = name.clone();
                                let art_url = album_art
                                    .filter(|s| !s.is_empty())
                                    .map(|id| format!("{COVERS_BASE}{id}"));
                                let art_url_opts = art_url.clone();
                                let album_id_play = album_id.clone();
                                let album_id_opts = album_id.clone();
                                let artist_for_opts = artist.clone();
                                let paths_for_opts = track_paths.clone();
                                let is_hovered = hovered_album_idx == Some(idx);
                                let mut art_container = div()
                                    .id(("album_art_hover", idx))
                                    .w_full()
                                    .rounded_lg()
                                    .overflow_hidden()
                                    .relative()
                                    .on_hover(move |hovered, _window, cx: &mut App| {
                                        cx.set_global(HoveredAlbumIdx(
                                            if *hovered { Some(idx) } else { None },
                                        ));
                                    });
                                art_container.style().aspect_ratio = Some(1.0_f32);
                                let art_content: AnyElement = if let Some(url) = art_url {
                                    img(url)
                                        .w_full()
                                        .h_full()
                                        .object_fit(ObjectFit::Cover)
                                        .into_any_element()
                                } else {
                                    div()
                                        .w_full()
                                        .h_full()
                                        .bg(theme.library_art_bg)
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .text_color(theme.player_icons_text)
                                        .child(Icon::new(Icons::Music).size_8())
                                        .into_any_element()
                                };
                                let art_overlay = div()
                                    .absolute()
                                    .bottom_0()
                                    .left_0()
                                    .right_0()
                                    .pb_3()
                                    .flex()
                                    .items_center()
                                    .opacity(if is_hovered { 1.0 } else { 0.0 })
                                    .child(
                                        // Left half — Play button centered within it
                                        div()
                                            .flex_1()
                                            .flex()
                                            .justify_center()
                                            .child(
                                                div()
                                                    .id(("album_play_btn", idx))
                                                    .w(px(36.0))
                                                    .h(px(36.0))
                                                    .rounded_full()
                                                    .flex()
                                                    .items_center()
                                                    .justify_center()
                                                    .cursor_pointer()
                                                    .bg(gpui::Rgba {
                                                        r: 0.0,
                                                        g: 0.0,
                                                        b: 0.0,
                                                        a: 0.65,
                                                    })
                                                    .text_color(gpui::rgb(0xFFFFFF))
                                                    .hover(|this| {
                                                        this.bg(gpui::Rgba {
                                                            r: 0.0,
                                                            g: 0.0,
                                                            b: 0.0,
                                                            a: 0.85,
                                                        })
                                                    })
                                                    .on_click(move |_, _, cx: &mut App| {
                                                        cx.stop_propagation();
                                                        cx.global::<Controller>()
                                                            .play_album(album_id_play.clone(), false);
                                                    })
                                                    .child(Icon::new(Icons::Play).size_4()),
                                            ),
                                    )
                                    .child(
                                        // Right half — Options button centered within it
                                        div()
                                            .flex_1()
                                            .flex()
                                            .justify_center()
                                            .child(
                                                div()
                                                    .id(("album_opts_btn", idx))
                                                    .w(px(36.0))
                                                    .h(px(36.0))
                                                    .rounded_full()
                                                    .flex()
                                                    .items_center()
                                                    .justify_center()
                                                    .cursor_pointer()
                                                    .bg(gpui::Rgba {
                                                        r: 0.0,
                                                        g: 0.0,
                                                        b: 0.0,
                                                        a: 0.65,
                                                    })
                                                    .text_color(gpui::rgb(0xFFFFFF))
                                                    .hover(|this| {
                                                        this.bg(gpui::Rgba {
                                                            r: 0.0,
                                                            g: 0.0,
                                                            b: 0.0,
                                                            a: 0.85,
                                                        })
                                                    })
                                                    .on_click(move |event, _, cx: &mut App| {
                                                        cx.stop_propagation();
                                                        cx.global_mut::<AlbumContextMenuState>().0 =
                                                            Some(AlbumContextMenu {
                                                                pos: event.position(),
                                                                album_id: album_id_opts.clone(),
                                                                album_name: name_clone_opts.clone(),
                                                                album_art: art_url_opts.clone(),
                                                                artist_name: artist_for_opts.clone(),
                                                                track_paths: paths_for_opts.clone(),
                                                            });
                                                    })
                                                    .child(Icon::new(Icons::Options).size_4()),
                                            ),
                                    );
                                let art_tile_overlay = art_container
                                    .child(art_content)
                                    .child(art_overlay)
                                    .into_any_element();
                                div()
                                    .id(("album_card", idx))
                                    .flex()
                                    .flex_col()
                                    .gap_y_2()
                                    .cursor_pointer()
                                    .on_click(move |_, _, cx: &mut App| {
                                        *cx.global_mut::<SelectedAlbum>() =
                                            SelectedAlbum(name_clone.clone());
                                        *cx.global_mut::<BackSection>() =
                                            BackSection(LibrarySection::Albums);
                                        *cx.global_mut::<LibrarySection>() =
                                            LibrarySection::AlbumDetail;
                                    })
                                    .child(art_tile_overlay)
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
                                                    .child(if year > 0 {
                                                        format!("{artist} · {year}")
                                                    } else {
                                                        artist
                                                    }),
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
                                                    )
                                                    .child(
                                                        div()
                                                            .flex()
                                                            .items_center()
                                                            .gap_x_3()
                                                            .mt_2()
                                                            .child({
                                                                let aid = album_id.clone();
                                                                div()
                                                                    .id("album_play_btn")
                                                                    .flex()
                                                                    .items_center()
                                                                    .gap_x_2()
                                                                    .px_4()
                                                                    .py_2()
                                                                    .rounded_md()
                                                                    .cursor_pointer()
                                                                    .bg(theme.player_play_pause_bg)
                                                                    .text_color(theme.player_play_pause_text)
                                                                    .hover(|this| this.bg(theme.player_play_pause_hover))
                                                                    .on_click(move |_, _, cx: &mut App| {
                                                                        cx.global::<Controller>().play_album(aid.clone(), false);
                                                                    })
                                                                    .child(Icon::new(Icons::Play).size_4())
                                                                    .child(div().text_sm().font_weight(FontWeight(600.0)).child("Play"))
                                                            })
                                                            .child({
                                                                let aid = album_id.clone();
                                                                div()
                                                                    .id("album_shuffle_btn")
                                                                    .flex()
                                                                    .items_center()
                                                                    .gap_x_2()
                                                                    .px_4()
                                                                    .py_2()
                                                                    .rounded_md()
                                                                    .cursor_pointer()
                                                                    .bg(theme.player_icons_bg_active)
                                                                    .text_color(theme.library_text)
                                                                    .hover(|this| this.bg(theme.player_icons_bg_hover))
                                                                    .on_click(move |_, _, cx: &mut App| {
                                                                        cx.global::<Controller>().play_album(aid.clone(), true);
                                                                    })
                                                                    .child(Icon::new(Icons::Shuffle).size_4())
                                                                    .child(div().text_sm().font_weight(FontWeight(500.0)).child("Shuffle"))
                                                            }),
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
                                    )
                                    .child(div().w(px(28.0)).flex_shrink_0())
                                    .child(div().w(px(28.0)).flex_shrink_0()),
                            )
                            // Track rows
                            .children(album_tracks.into_iter().enumerate().map(
                                |(i, (global_idx, path, title, num, duration, track_id, art))| {
                                    let is_current = current_idx == Some(global_idx);
                                    let is_liked = liked_songs.contains(&track_id);
                                    let row_album = selected_album.clone();
                                    let row_artist = album_artist.clone();
                                    track_row(
                                        ("album_detail_row", i),
                                        path,
                                        num,
                                        title,
                                        None,
                                        None,
                                        duration,
                                        is_current,
                                        is_liked,
                                        row_artist,
                                        row_album,
                                        track_id,
                                        art,
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
                                                    )
                                                    .child(
                                                        div()
                                                            .flex()
                                                            .items_center()
                                                            .gap_x_3()
                                                            .mt_2()
                                                            .child({
                                                                let aid = artist_id.clone();
                                                                div()
                                                                    .id("artist_play_btn")
                                                                    .flex()
                                                                    .items_center()
                                                                    .gap_x_2()
                                                                    .px_4()
                                                                    .py_2()
                                                                    .rounded_md()
                                                                    .cursor_pointer()
                                                                    .bg(theme.player_play_pause_bg)
                                                                    .text_color(theme.player_play_pause_text)
                                                                    .hover(|this| this.bg(theme.player_play_pause_hover))
                                                                    .on_click(move |_, _, cx: &mut App| {
                                                                        cx.global::<Controller>().play_artist_tracks(aid.clone(), false);
                                                                    })
                                                                    .child(Icon::new(Icons::Play).size_4())
                                                                    .child(div().text_sm().font_weight(FontWeight(600.0)).child("Play"))
                                                            })
                                                            .child({
                                                                let aid = artist_id.clone();
                                                                div()
                                                                    .id("artist_shuffle_btn")
                                                                    .flex()
                                                                    .items_center()
                                                                    .gap_x_2()
                                                                    .px_4()
                                                                    .py_2()
                                                                    .rounded_md()
                                                                    .cursor_pointer()
                                                                    .bg(theme.player_icons_bg_active)
                                                                    .text_color(theme.library_text)
                                                                    .hover(|this| this.bg(theme.player_icons_bg_hover))
                                                                    .on_click(move |_, _, cx: &mut App| {
                                                                        cx.global::<Controller>().play_artist_tracks(aid.clone(), true);
                                                                    })
                                                                    .child(Icon::new(Icons::Shuffle).size_4())
                                                                    .child(div().text_sm().font_weight(FontWeight(500.0)).child("Shuffle"))
                                                            }),
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
                                    )
                                    .child(div().w(px(28.0)).flex_shrink_0())
                                    .child(div().w(px(28.0)).flex_shrink_0()),
                            )
                            // Artist track rows
                            .children(artist_tracks.into_iter().enumerate().map(
                                |(i, (global_idx, path, title, album, duration, track_id, art))| {
                                    let is_current = current_idx == Some(global_idx);
                                    let is_liked = liked_songs.contains(&track_id);
                                    let row_artist = selected_artist.clone();
                                    let row_album = album.clone();
                                    track_row(
                                        ("artist_detail_row", i),
                                        path,
                                        format!("{}", i + 1),
                                        title,
                                        None,
                                        Some(album),
                                        duration,
                                        is_current,
                                        is_liked,
                                        row_artist,
                                        row_album,
                                        track_id,
                                        art,
                                    )
                                },
                            )),
                    )
                    .into_any_element()
            }

            // ── Likes ─────────────────────────────────────────────────────────────
            LibrarySection::Likes => {
                let n_liked = liked_tracks.len();
                let liked_paths: Vec<String> =
                    liked_tracks.iter().map(|(_, p, ..)| p.clone()).collect();
                let liked_paths_shuffle = liked_paths.clone();
                div()
                    .id("likes_scroll")
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
                                    .gap_y_4()
                                    .child(
                                        div()
                                            .flex()
                                            .items_center()
                                            .gap_x_3()
                                            .child(
                                                Icon::new(Icons::Heart)
                                                    .size_8()
                                                    .text_color(gpui::rgb(0xFFFFFF)),
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
                                                            .child("Liked Songs"),
                                                    )
                                                    .child(
                                                        div()
                                                            .text_sm()
                                                            .text_color(theme.library_header_text)
                                                            .child(format!(
                                                                "{} track{}",
                                                                n_liked,
                                                                if n_liked == 1 { "" } else { "s" }
                                                            )),
                                                    ),
                                            ),
                                    )
                                    .child(
                                        div()
                                            .flex()
                                            .items_center()
                                            .gap_x_3()
                                            .child(
                                                div()
                                                    .id("likes_play_btn")
                                                    .flex()
                                                    .items_center()
                                                    .gap_x_2()
                                                    .px_4()
                                                    .py_2()
                                                    .rounded_md()
                                                    .cursor_pointer()
                                                    .bg(theme.player_play_pause_bg)
                                                    .text_color(theme.player_play_pause_text)
                                                    .hover(|this| {
                                                        this.bg(theme.player_play_pause_hover)
                                                    })
                                                    .on_click(move |_, _, cx: &mut App| {
                                                        cx.global::<Controller>()
                                                            .play_liked_tracks(
                                                                liked_paths.clone(),
                                                                false,
                                                            );
                                                    })
                                                    .child(Icon::new(Icons::Play).size_4())
                                                    .child(
                                                        div()
                                                            .text_sm()
                                                            .font_weight(FontWeight(600.0))
                                                            .child("Play"),
                                                    ),
                                            )
                                            .child(
                                                div()
                                                    .id("likes_shuffle_btn")
                                                    .flex()
                                                    .items_center()
                                                    .gap_x_2()
                                                    .px_4()
                                                    .py_2()
                                                    .rounded_md()
                                                    .cursor_pointer()
                                                    .bg(theme.player_icons_bg_active)
                                                    .text_color(theme.library_text)
                                                    .hover(|this| {
                                                        this.bg(theme.player_icons_bg_hover)
                                                    })
                                                    .on_click(move |_, _, cx: &mut App| {
                                                        cx.global::<Controller>()
                                                            .play_liked_tracks(
                                                                liked_paths_shuffle.clone(),
                                                                true,
                                                            );
                                                    })
                                                    .child(Icon::new(Icons::Shuffle).size_4())
                                                    .child(
                                                        div()
                                                            .text_sm()
                                                            .font_weight(FontWeight(500.0))
                                                            .child("Shuffle"),
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
                                    )
                                    .child(div().w(px(28.0)).flex_shrink_0())
                                    .child(div().w(px(28.0)).flex_shrink_0()),
                            )
                            // Liked track rows
                            .children(liked_tracks.into_iter().enumerate().map(
                                |(i, (global_idx, path, title, artist, album, duration, track_id, art))| {
                                    let is_current = current_idx == Some(global_idx);
                                    let row_artist = artist.clone();
                                    let row_album = album.clone();
                                    track_row(
                                        ("liked_row", i),
                                        path,
                                        (i + 1).to_string(),
                                        title,
                                        Some(artist),
                                        Some(album),
                                        duration,
                                        is_current,
                                        true,
                                        row_artist,
                                        row_album,
                                        track_id,
                                        art,
                                    )
                                },
                            )),
                    )
                    .into_any_element()
            }

            // ── Files ─────────────────────────────────────────────────────────────
            LibrarySection::Files => self.files_view.clone().into_any_element(),
        };
        content_inner
        }; // end if/else search

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
                            .child(make_nav_item(Icons::Music, 5, "Songs", LibrarySection::Songs))
                            .child(make_nav_item(Icons::Disc, 5, "Albums", LibrarySection::Albums))
                            .child(make_nav_item(Icons::Artist, 5, "Artists", LibrarySection::Artists))
                            .child(make_nav_item(Icons::HeartOutline, 5, "Likes", LibrarySection::Likes))
                            .child(make_nav_item(Icons::HardDrive, 4, "Files", LibrarySection::Files)),
                    )
                    .child(content),
            )
            .child(self.miniplayer.clone())
            .when_some(context_menu, |this, menu| {
                let path_for_next = menu.path.clone();
                let path_for_last = menu.path.clone();
                let menu_artist = menu.artist.clone();
                let menu_album = menu.album.clone();
                let header_art_url = menu
                    .album_art
                    .as_deref()
                    .filter(|s| !s.is_empty())
                    .map(|id| format!("{COVERS_BASE}{id}"));
                // header ~64px + 4 items × ~33px + borders
                let menu_w = px(240.0);
                let menu_h = px(198.0);
                let margin = px(8.0);
                let max_x = viewport.width - menu_w - margin;
                let menu_x = if menu.pos.x > max_x { max_x } else { menu.pos.x };
                let menu_x = if menu_x < margin { margin } else { menu_x };
                // Flip above cursor when the menu would overflow the bottom edge.
                let overflows_bottom = (menu.pos.y + menu_h + margin) > viewport.height;
                let menu_y = if overflows_bottom { menu.pos.y - menu_h } else { menu.pos.y };
                let menu_y = if menu_y < margin { margin } else { menu_y };
                this.child(
                    div()
                        .id("ctx_backdrop")
                        .absolute()
                        .top_0()
                        .left_0()
                        .size_full()
                        .occlude()
                        .on_click(|_, _, cx: &mut App| {
                            cx.stop_propagation();
                            cx.global_mut::<LibraryContextMenuState>().0 = None;
                        }),
                )
                .child(
                    div()
                        .absolute()
                        .left(menu_x)
                        .top(menu_y)
                        .bg(theme.titlebar_bg)
                        .border_1()
                        .border_color(theme.library_table_border)
                        .rounded_md()
                        .overflow_hidden()
                        .w(px(240.0))
                        .flex()
                        .flex_col()
                        // ── Header ──────────────────────────────────────
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_x_3()
                                .px_3()
                                .py_3()
                                .border_b_1()
                                .border_color(theme.library_table_border)
                                .child(if let Some(url) = header_art_url {
                                    div()
                                        .w(px(40.0))
                                        .h(px(40.0))
                                        .rounded_md()
                                        .flex_shrink_0()
                                        .overflow_hidden()
                                        .child(img(url).w_full().h_full().object_fit(ObjectFit::Cover))
                                        .into_any_element()
                                } else {
                                    div()
                                        .w(px(40.0))
                                        .h(px(40.0))
                                        .rounded_md()
                                        .flex_shrink_0()
                                        .bg(theme.library_art_bg)
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .text_color(theme.player_icons_text)
                                        .child(Icon::new(Icons::Music).size_4())
                                        .into_any_element()
                                })
                                .child(
                                    div()
                                        .flex_1()
                                        .min_w_0()
                                        .flex()
                                        .flex_col()
                                        .gap_y_0p5()
                                        .child(
                                            div()
                                                .text_sm()
                                                .font_weight(FontWeight(600.0))
                                                .text_color(theme.library_text)
                                                .truncate()
                                                .child(menu.title.clone()),
                                        )
                                        .child(
                                            div()
                                                .text_xs()
                                                .text_color(theme.library_header_text)
                                                .truncate()
                                                .child(menu.artist.clone()),
                                        ),
                                ),
                        )
                        .child(
                            div()
                                .id("ctx_play_next")
                                .px_4()
                                .py_2()
                                .text_sm()
                                .cursor_pointer()
                                .text_color(theme.library_text)
                                .hover(|this| this.bg(theme.library_track_bg_hover))
                                .on_click(move |_, _, cx: &mut App| {
                                    cx.global::<Controller>()
                                        .insert_track_next(path_for_next.clone());
                                    cx.global_mut::<LibraryContextMenuState>().0 = None;
                                })
                                .child("Play Next"),
                        )
                        .child(
                            div()
                                .id("ctx_play_last")
                                .px_4()
                                .py_2()
                                .text_sm()
                                .cursor_pointer()
                                .text_color(theme.library_text)
                                .hover(|this| this.bg(theme.library_track_bg_hover))
                                .on_click(move |_, _, cx: &mut App| {
                                    cx.global::<Controller>()
                                        .insert_track_last(path_for_last.clone());
                                    cx.global_mut::<LibraryContextMenuState>().0 = None;
                                })
                                .child("Play Last"),
                        )
                        .child(
                            div()
                                .id("ctx_go_artist")
                                .px_4()
                                .py_2()
                                .text_sm()
                                .cursor_pointer()
                                .text_color(theme.library_text)
                                .hover(|this| this.bg(theme.library_track_bg_hover))
                                .on_click(move |_, _, cx: &mut App| {
                                    *cx.global_mut::<SelectedArtist>() =
                                        SelectedArtist(menu_artist.clone());
                                    *cx.global_mut::<LibrarySection>() =
                                        LibrarySection::ArtistDetail;
                                    cx.global_mut::<LibraryContextMenuState>().0 = None;
                                })
                                .child("Go to Artist"),
                        )
                        .child(
                            div()
                                .id("ctx_go_album")
                                .px_4()
                                .py_2()
                                .text_sm()
                                .cursor_pointer()
                                .text_color(theme.library_text)
                                .hover(|this| this.bg(theme.library_track_bg_hover))
                                .on_click(move |_, _, cx: &mut App| {
                                    *cx.global_mut::<SelectedAlbum>() =
                                        SelectedAlbum(menu_album.clone());
                                    *cx.global_mut::<BackSection>() =
                                        BackSection(LibrarySection::Albums);
                                    *cx.global_mut::<LibrarySection>() =
                                        LibrarySection::AlbumDetail;
                                    cx.global_mut::<LibraryContextMenuState>().0 = None;
                                })
                                .child("Go to Album"),
                        ),
                )
            })
            .when_some(album_context_menu, |this, menu| {
                let album_id_play = menu.album_id.clone();
                let album_id_shuffled = menu.album_id.clone();
                let paths_next = menu.track_paths.clone();
                let paths_last = menu.track_paths.clone();
                let paths_add_shuffled = menu.track_paths.clone();
                let paths_last_shuffled = menu.track_paths.clone();
                let artist_nav = menu.artist_name.clone();
                let alb_header_art_url = menu
                    .album_art
                    .as_deref()
                    .filter(|s| !s.is_empty())
                    .map(|s| {
                        if s.starts_with("http") {
                            s.to_string()
                        } else {
                            format!("{COVERS_BASE}{s}")
                        }
                    });
                // header ~64px + 7 items × ~33px + borders
                let menu_w = px(250.0);
                let menu_h = px(296.0);
                let margin = px(8.0);
                let max_x = viewport.width - menu_w - margin;
                let menu_x = if menu.pos.x > max_x { max_x } else { menu.pos.x };
                let menu_x = if menu_x < margin { margin } else { menu_x };
                // Flip above cursor when the menu would overflow the bottom edge.
                let overflows_bottom = (menu.pos.y + menu_h + margin) > viewport.height;
                let menu_y = if overflows_bottom { menu.pos.y - menu_h } else { menu.pos.y };
                let menu_y = if menu_y < margin { margin } else { menu_y };
                this.child(
                    div()
                        .id("album_ctx_backdrop")
                        .absolute()
                        .top_0()
                        .left_0()
                        .size_full()
                        .occlude()
                        .on_click(|_, _, cx: &mut App| {
                            cx.stop_propagation();
                            cx.global_mut::<AlbumContextMenuState>().0 = None;
                        }),
                )
                .child(
                    div()
                        .absolute()
                        .left(menu_x)
                        .top(menu_y)
                        .bg(theme.titlebar_bg)
                        .border_1()
                        .border_color(theme.library_table_border)
                        .rounded_md()
                        .overflow_hidden()
                        .w(px(250.0))
                        .flex()
                        .flex_col()
                        // ── Header ──────────────────────────────────────
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_x_3()
                                .px_3()
                                .py_3()
                                .border_b_1()
                                .border_color(theme.library_table_border)
                                .child(if let Some(url) = alb_header_art_url {
                                    div()
                                        .w(px(40.0))
                                        .h(px(40.0))
                                        .rounded_md()
                                        .flex_shrink_0()
                                        .overflow_hidden()
                                        .child(img(url).w_full().h_full().object_fit(ObjectFit::Cover))
                                        .into_any_element()
                                } else {
                                    div()
                                        .w(px(40.0))
                                        .h(px(40.0))
                                        .rounded_md()
                                        .flex_shrink_0()
                                        .bg(theme.library_art_bg)
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .text_color(theme.player_icons_text)
                                        .child(Icon::new(Icons::Disc).size_4())
                                        .into_any_element()
                                })
                                .child(
                                    div()
                                        .flex_1()
                                        .min_w_0()
                                        .flex()
                                        .flex_col()
                                        .gap_y_0p5()
                                        .child(
                                            div()
                                                .text_sm()
                                                .font_weight(FontWeight(600.0))
                                                .text_color(theme.library_text)
                                                .truncate()
                                                .child(menu.album_name.clone()),
                                        )
                                        .child(
                                            div()
                                                .text_xs()
                                                .text_color(theme.library_header_text)
                                                .truncate()
                                                .child(menu.artist_name.clone()),
                                        ),
                                ),
                        )
                        .child(
                            div()
                                .id("alb_ctx_play")
                                .px_4()
                                .py_2()
                                .text_sm()
                                .cursor_pointer()
                                .text_color(theme.library_text)
                                .hover(|this| this.bg(theme.library_track_bg_hover))
                                .on_click(move |_, _, cx: &mut App| {
                                    cx.global::<Controller>().play_album(album_id_play.clone(), false);
                                    cx.global_mut::<AlbumContextMenuState>().0 = None;
                                })
                                .child("Play"),
                        )
                        .child(
                            div()
                                .id("alb_ctx_play_shuffled")
                                .px_4()
                                .py_2()
                                .text_sm()
                                .cursor_pointer()
                                .text_color(theme.library_text)
                                .hover(|this| this.bg(theme.library_track_bg_hover))
                                .on_click(move |_, _, cx: &mut App| {
                                    cx.global::<Controller>()
                                        .play_album(album_id_shuffled.clone(), true);
                                    cx.global_mut::<AlbumContextMenuState>().0 = None;
                                })
                                .child("Play Shuffled"),
                        )
                        .child(
                            div()
                                .id("alb_ctx_play_next")
                                .px_4()
                                .py_2()
                                .text_sm()
                                .cursor_pointer()
                                .text_color(theme.library_text)
                                .hover(|this| this.bg(theme.library_track_bg_hover))
                                .on_click(move |_, _, cx: &mut App| {
                                    cx.global::<Controller>().insert_tracks(
                                        paths_next.clone(),
                                        crate::client::INSERT_FIRST,
                                        false,
                                    );
                                    cx.global_mut::<AlbumContextMenuState>().0 = None;
                                })
                                .child("Play Next"),
                        )
                        .child(
                            div()
                                .id("alb_ctx_play_last")
                                .px_4()
                                .py_2()
                                .text_sm()
                                .cursor_pointer()
                                .text_color(theme.library_text)
                                .hover(|this| this.bg(theme.library_track_bg_hover))
                                .on_click(move |_, _, cx: &mut App| {
                                    cx.global::<Controller>().insert_tracks(
                                        paths_last.clone(),
                                        crate::client::INSERT_LAST,
                                        false,
                                    );
                                    cx.global_mut::<AlbumContextMenuState>().0 = None;
                                })
                                .child("Play Last"),
                        )
                        .child(
                            div()
                                .id("alb_ctx_add_shuffled")
                                .px_4()
                                .py_2()
                                .text_sm()
                                .cursor_pointer()
                                .text_color(theme.library_text)
                                .hover(|this| this.bg(theme.library_track_bg_hover))
                                .on_click(move |_, _, cx: &mut App| {
                                    cx.global::<Controller>().insert_tracks(
                                        paths_add_shuffled.clone(),
                                        crate::client::INSERT_SHUFFLED,
                                        false,
                                    );
                                    cx.global_mut::<AlbumContextMenuState>().0 = None;
                                })
                                .child("Add Shuffled"),
                        )
                        .child(
                            div()
                                .id("alb_ctx_play_last_shuffled")
                                .px_4()
                                .py_2()
                                .text_sm()
                                .cursor_pointer()
                                .text_color(theme.library_text)
                                .hover(|this| this.bg(theme.library_track_bg_hover))
                                .on_click(move |_, _, cx: &mut App| {
                                    cx.global::<Controller>().insert_tracks(
                                        paths_last_shuffled.clone(),
                                        crate::client::INSERT_LAST_SHUFFLED,
                                        false,
                                    );
                                    cx.global_mut::<AlbumContextMenuState>().0 = None;
                                })
                                .child("Play Last Shuffled"),
                        )
                        .child(
                            div()
                                .id("alb_ctx_go_artist")
                                .px_4()
                                .py_2()
                                .text_sm()
                                .cursor_pointer()
                                .text_color(theme.library_text)
                                .hover(|this| this.bg(theme.library_track_bg_hover))
                                .on_click(move |_, _, cx: &mut App| {
                                    *cx.global_mut::<SelectedArtist>() =
                                        SelectedArtist(artist_nav.clone());
                                    *cx.global_mut::<LibrarySection>() =
                                        LibrarySection::ArtistDetail;
                                    cx.global_mut::<AlbumContextMenuState>().0 = None;
                                })
                                .child("Go to Artist"),
                        ),
                )
            })
            .when_some(file_context_menu, |this, menu| {
                let path_next = menu.path.clone();
                let path_last = menu.path.clone();
                let path_shuffled = menu.path.clone();
                let path_last_shuffled = menu.path.clone();
                let path_play_shuffled = menu.path.clone();
                let is_dir = menu.is_dir;

                let menu_w = px(220.0);
                let menu_h = if is_dir { px(230.0) } else { px(140.0) };
                let margin = px(8.0);
                let max_x = viewport.width - menu_w - margin;
                let menu_x = if menu.pos.x > max_x { max_x } else { menu.pos.x };
                let menu_x = if menu_x < margin { margin } else { menu_x };
                let overflows_bottom = (menu.pos.y + menu_h + margin) > viewport.height;
                let menu_y = if overflows_bottom { menu.pos.y - menu_h } else { menu.pos.y };
                let menu_y = if menu_y < margin { margin } else { menu_y };

                this.child(
                    div()
                        .id("file_ctx_backdrop")
                        .absolute()
                        .top_0()
                        .left_0()
                        .size_full()
                        .occlude()
                        .on_click(|_, _, cx: &mut App| {
                            cx.stop_propagation();
                            cx.global_mut::<FileContextMenuState>().0 = None;
                        }),
                )
                .child(
                    div()
                        .absolute()
                        .left(menu_x)
                        .top(menu_y)
                        .w(menu_w)
                        .bg(theme.titlebar_bg)
                        .border_1()
                        .border_color(theme.library_table_border)
                        .rounded_md()
                        .overflow_hidden()
                        .flex()
                        .flex_col()
                        .child(
                            div()
                                .px_3()
                                .py_2p5()
                                .border_b_1()
                                .border_color(theme.library_table_border)
                                .flex()
                                .items_center()
                                .gap_x_2()
                                .child(
                                    div().text_color(theme.library_header_text).child(
                                        Icon::new(if is_dir { Icons::Directory } else { Icons::Music })
                                            .size_4(),
                                    ),
                                )
                                .child(
                                    div()
                                        .flex_1()
                                        .min_w_0()
                                        .text_sm()
                                        .font_weight(FontWeight(600.0))
                                        .text_color(theme.library_text)
                                        .truncate()
                                        .child(menu.name.clone()),
                                ),
                        )
                        .child(menu_item(
                            "file_ctx_next",
                            "Play Next",
                            theme,
                            move |_, _, cx: &mut App| {
                                let rt = cx.global::<Controller>().rt();
                                if is_dir {
                                    rt.spawn(insert_directory(path_next.clone(), INSERT_FIRST));
                                } else {
                                    rt.spawn(insert_track_next(path_next.clone()));
                                }
                                cx.global_mut::<FileContextMenuState>().0 = None;
                            },
                        ))
                        .child(menu_item(
                            "file_ctx_last",
                            "Play Last",
                            theme,
                            move |_, _, cx: &mut App| {
                                let rt = cx.global::<Controller>().rt();
                                if is_dir {
                                    rt.spawn(insert_directory(path_last.clone(), INSERT_LAST));
                                } else {
                                    rt.spawn(insert_track_last(path_last.clone()));
                                }
                                cx.global_mut::<FileContextMenuState>().0 = None;
                            },
                        ))
                        .child(menu_item(
                            "file_ctx_shuffled",
                            "Add Shuffled",
                            theme,
                            move |_, _, cx: &mut App| {
                                let rt = cx.global::<Controller>().rt();
                                if is_dir {
                                    rt.spawn(insert_directory(path_shuffled.clone(), INSERT_SHUFFLED));
                                } else {
                                    rt.spawn(insert_tracks(
                                        vec![path_shuffled.clone()],
                                        INSERT_SHUFFLED,
                                        false,
                                    ));
                                }
                                cx.global_mut::<FileContextMenuState>().0 = None;
                            },
                        ))
                        .when(is_dir, |this| {
                            this.child(div().h(px(1.0)).bg(theme.library_table_border).mx_2())
                                .child(menu_item(
                                    "file_ctx_last_shuffled",
                                    "Play Last Shuffled",
                                    theme,
                                    move |_, _, cx: &mut App| {
                                        cx.global::<Controller>().rt().spawn(insert_directory(
                                            path_last_shuffled.clone(),
                                            INSERT_LAST_SHUFFLED,
                                        ));
                                        cx.global_mut::<FileContextMenuState>().0 = None;
                                    },
                                ))
                                .child(menu_item(
                                    "file_ctx_play_shuffled",
                                    "Play Shuffled",
                                    theme,
                                    move |_, _, cx: &mut App| {
                                        cx.global::<Controller>().rt().spawn(play_directory(
                                            path_play_shuffled.clone(),
                                            true,
                                        ));
                                        cx.global_mut::<FileContextMenuState>().0 = None;
                                    },
                                ))
                        }),
                )
            })
    }
}
