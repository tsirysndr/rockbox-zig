use crate::client::{
    insert_directory, insert_track_last, insert_track_next, insert_tracks, play_directory,
    INSERT_FIRST, INSERT_LAST, INSERT_LAST_SHUFFLED, INSERT_SHUFFLED,
};
use crate::controller::Controller;
use crate::state::{format_duration, DevicesState, PlaybackStatus};
use crate::ui::animations::{
    equalizer_bars, skeleton_album_grid, skeleton_artist_grid, skeleton_rect, skeleton_track_list,
};
use crate::ui::components::icons::{Icon, Icons};
use crate::ui::components::miniplayer::MiniPlayer;
use crate::ui::components::pages::files::{menu_item, FilesView};
use crate::ui::components::search_input::SearchInput;
use crate::ui::components::text_input::TextInput;
use crate::ui::components::{
    AddToPlaylistMenuState, AlbumContextMenu, AlbumContextMenuState, BackSection,
    CreatePlaylistModal, DeletePlaylistModal, DiscoveredServers, EditPlaylistModal,
    FileContextMenuState, GenresState, HoveredAlbumIdx, LibraryContextMenu,
    LibraryContextMenuState, LibrarySection, LikedOrder, LikedSongs, NavidromeAddModal, NavidromeServerState,
    NdAlbumItem, NdArtistItem, NdContextMenu, NdContextMenuState, NdCurrentCoverArt,
    NdGenreItem, NdLibraryData, NdLikesState, NdPlaylistItem, NdSelectedAlbum, NdSelectedArtist,
    NdSelectedGenre, NdSelectedPlaylist, NdSongItem, NdSongsState, NdStarredIds,
    PlaylistsSidebarCollapsed,
    PlaylistsState, SelectedAlbum, SelectedAlbumMeta, SelectedArtist, SelectedGenre,
    SelectedPlaylist, ServerPickerOpen,
};
use crate::ui::theme::Theme;
use gpui::prelude::FluentBuilder;
use gpui::{
    div, img, px, uniform_list, AnyElement, App, AppContext, Entity, FontWeight,
    InteractiveElement, IntoElement, MouseButton, ObjectFit, ParentElement, Render, ScrollHandle,
    StatefulInteractiveElement, Styled, StyledImage, Subscription, UniformListScrollHandle, Window,
};

fn covers_base() -> String {
    crate::server::get_covers_base()
}

/// Parse "yyyy-MM-dd" into "9 December 2014". Falls back to the raw string on any parse failure.
fn format_release_date(s: &str) -> String {
    const MONTHS: [&str; 12] = [
        "January",
        "February",
        "March",
        "April",
        "May",
        "June",
        "July",
        "August",
        "September",
        "October",
        "November",
        "December",
    ];
    let parts: Vec<&str> = s.splitn(3, '-').collect();
    if parts.len() == 3 {
        if let (Ok(y), Ok(m), Ok(d)) = (
            parts[0].parse::<u32>(),
            parts[1].parse::<usize>(),
            parts[2].parse::<u32>(),
        ) {
            if m >= 1 && m <= 12 {
                return format!("{} {} {}", d, MONTHS[m - 1], y);
            }
        }
    }
    s.to_string()
}

/// Square art tile that fills its container (used in grids). `icon_size` is the size_N shorthand number.
fn art_tile(
    art: Option<String>,
    theme: crate::ui::theme::Theme,
    fallback: Icons,
    icon_size: u8,
) -> AnyElement {
    let art_url = art
        .filter(|s| !s.is_empty())
        .map(|id| format!("{}{id}", covers_base()));
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
        .map(|id| format!("{}{id}", covers_base()));
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

/// Deterministic neon-ish palette for the Genres grid. Same `seed` always
/// returns the same colour so a genre keeps its identity across renders.
fn neon_color_for(seed: &str) -> gpui::Rgba {
    const PALETTE: &[u32] = &[
        0xFF1F6E, // hot pink
        0x00E5FF, // electric cyan
        0xB1FF34, // lime
        0xFFC400, // amber
        0x9D4EFF, // violet
        0x00C853, // fluorescent green
        0xFF6E40, // tangerine
        0x536DFE, // cobalt
        0xFF4081, // magenta
        0x18FFFF, // aqua
        0xFFD600, // yellow
        0x7C4DFF, // deep violet
        0xFF1744, // crimson
        0x00BFA5, // teal
        0xF50057, // pink
        0x1DE9B6, // mint
    ];
    let mut h: u32 = 0;
    for b in seed.bytes() {
        h = h.wrapping_mul(31).wrapping_add(b as u32);
    }
    gpui::rgb(PALETTE[(h as usize) % PALETTE.len()])
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

fn nd_search_sections(
    albums: &[NdAlbumItem],
    artists: &[NdArtistItem],
    playlists: &[NdPlaylistItem],
    creds: &crate::ui::components::NdSavedServer,
    search_input: Entity<crate::ui::components::search_input::SearchInput>,
    theme: crate::ui::theme::Theme,
) -> gpui::Div {
    let base_url = creds.base_url.clone();
    let user = creds.user.clone();
    let token = creds.token.clone();
    let salt = creds.salt.clone();

    let albums_section: Option<gpui::AnyElement> = if !albums.is_empty() {
        let cards: Vec<gpui::AnyElement> = albums.iter().enumerate().map(|(idx, album)| {
            let aid = album.id.clone();
            let aname = album.name.clone();
            let aartist = album.artist.clone();
            let cover_url = album.cover_art.as_ref().map(|cid| {
                crate::navidrome::cover_art_url(&base_url, &user, &token, &salt, cid, Some(200))
            });
            let si = search_input.clone();
            let mut art_box = div()
                .id(("nd_srch_album_art", idx))
                .w_full()
                .rounded_lg()
                .overflow_hidden();
            art_box.style().aspect_ratio = Some(1.0_f32);
            let art_inner: gpui::AnyElement = if let Some(url) = cover_url {
                art_box.child(
                    img(url).w_full().h_full().object_fit(ObjectFit::Cover),
                ).into_any_element()
            } else {
                art_box.bg(theme.library_art_bg)
                    .flex().items_center().justify_center()
                    .text_color(theme.player_icons_text)
                    .child(Icon::new(Icons::Music).size_8())
                    .into_any_element()
            };
            div()
                .id(("nd_srch_album", idx))
                .w(px(130.0))
                .flex_shrink_0()
                .flex().flex_col().gap_y_1()
                .cursor_pointer()
                .on_click(move |_, _, cx: &mut App| {
                    let sel = cx.global_mut::<NdSelectedAlbum>();
                    sel.id = aid.clone();
                    sel.name = aname.clone();
                    sel.songs = vec![];
                    sel.loading = false;
                    *cx.global_mut::<LibrarySection>() = LibrarySection::NdAlbumDetail;
                    si.update(cx, |this, cx| { this.query.clear(); cx.notify(); });
                })
                .child(art_inner)
                .child(div().text_xs().font_weight(FontWeight(500.0)).text_color(theme.library_text).truncate().child(album.name.clone()))
                .child(div().text_xs().text_color(theme.library_header_text).truncate().child(aartist))
                .into_any_element()
        }).collect();
        Some(div()
            .flex().flex_col().gap_y_4()
            .child(div().text_base().font_weight(FontWeight(600.0)).text_color(theme.library_text).child("Albums"))
            .child(div().flex().items_start().gap_x_4().children(cards))
            .into_any_element())
    } else { None };

    let artists_section: Option<gpui::AnyElement> = if !artists.is_empty() {
        let cards: Vec<gpui::AnyElement> = artists.iter().enumerate().map(|(idx, artist)| {
            let aname = artist.name.clone();
            let cover_url = artist.cover_art.as_ref().map(|cid| {
                crate::navidrome::cover_art_url(&base_url, &user, &token, &salt, cid, Some(200))
            });
            let si = search_input.clone();
            let mut c = div()
                .id(("nd_srch_artist_art", idx))
                .w(px(88.0)).rounded_full().overflow_hidden().flex_shrink_0();
            c.style().aspect_ratio = Some(1.0_f32);
            let avatar: gpui::AnyElement = if let Some(url) = cover_url {
                c.child(img(url).w_full().h_full().rounded_full().object_fit(ObjectFit::Cover)).into_any_element()
            } else {
                c.bg(theme.library_art_bg).flex().items_center().justify_center()
                    .text_color(theme.player_icons_text)
                    .child(Icon::new(Icons::Artist).size_8())
                    .into_any_element()
            };
            div()
                .id(("nd_srch_artist", idx))
                .w(px(112.0)).flex_shrink_0()
                .flex().flex_col().items_center().gap_y_2()
                .cursor_pointer()
                .on_click(move |_, _, cx: &mut App| {
                    let sel = cx.global_mut::<NdSelectedArtist>();
                    sel.id = aname.clone();
                    sel.name = aname.clone();
                    sel.albums = vec![];
                    sel.loading = false;
                    *cx.global_mut::<LibrarySection>() = LibrarySection::NdArtistDetail;
                    si.update(cx, |this, cx| { this.query.clear(); cx.notify(); });
                })
                .child(avatar)
                .child(div().w_full().text_xs().font_weight(FontWeight(500.0)).text_color(theme.library_text).text_center().truncate().child(artist.name.clone()))
                .into_any_element()
        }).collect();
        Some(div()
            .flex().flex_col().gap_y_4()
            .child(div().text_base().font_weight(FontWeight(600.0)).text_color(theme.library_text).child("Artists"))
            .child(div().flex().items_start().gap_x_4().children(cards))
            .into_any_element())
    } else { None };

    let playlists_section: Option<gpui::AnyElement> = if !playlists.is_empty() {
        let rows: Vec<gpui::AnyElement> = playlists.iter().enumerate().map(|(idx, pl)| {
            let pid = pl.id.clone();
            let pname = pl.name.clone();
            let si = search_input.clone();
            div()
                .id(("nd_srch_pl", idx))
                .flex().items_center().gap_x_3()
                .px_2().py_2().rounded_md()
                .cursor_pointer()
                .hover(|s| s.bg(theme.library_table_border))
                .on_click(move |_, _, cx: &mut App| {
                    let sel = cx.global_mut::<NdSelectedPlaylist>();
                    sel.id = pid.clone();
                    sel.name = pname.clone();
                    sel.tracks = vec![];
                    sel.loading = false;
                    *cx.global_mut::<LibrarySection>() = LibrarySection::NdPlaylistDetail;
                    si.update(cx, |this, cx| { this.query.clear(); cx.notify(); });
                })
                .child(
                    div().flex().items_center().justify_center()
                        .w_8().h_8().rounded_md().bg(theme.library_art_bg)
                        .text_color(theme.player_icons_text)
                        .child(Icon::new(Icons::Playlist).size_4()),
                )
                .child(div().text_sm().font_weight(FontWeight(500.0)).text_color(theme.library_text).truncate().child(pl.name.clone()))
                .into_any_element()
        }).collect();
        Some(div()
            .flex().flex_col().gap_y_4()
            .child(div().text_base().font_weight(FontWeight(600.0)).text_color(theme.library_text).child("Playlists"))
            .child(div().flex().flex_col().gap_y_1().children(rows))
            .into_any_element())
    } else { None };

    div()
        .w_full()
        .p_6()
        .flex().flex_col().gap_y_8()
        .when_some(albums_section, |t, s| t.child(s))
        .when_some(artists_section, |t, s| t.child(s))
        .when_some(playlists_section, |t, s| t.child(s))
}

pub struct LibraryPage {
    scroll_handle: UniformListScrollHandle,
    detail_scroll_handle: UniformListScrollHandle,
    miniplayer: Entity<MiniPlayer>,
    search_input: Entity<SearchInput>,
    files_view: Entity<FilesView>,
    modal_name_input: Entity<TextInput>,
    modal_desc_input: Entity<TextInput>,
    edit_name_input: Entity<TextInput>,
    edit_desc_input: Entity<TextInput>,
    // Persisted scroll state for horizontal rows on Home / GenreDetail. We
    // wire these via `track_scroll(...)` and an explicit `on_scroll_wheel`
    // handler that converts vertical mouse-wheel deltas into horizontal
    // scroll — gpui's auto-conversion is preempted by the parent's
    // `overflow_y_scroll`, so without this nothing scrolls on a normal
    // (non-trackpad) mouse.
    home_recent_scroll: ScrollHandle,
    home_popular_scroll: ScrollHandle,
    home_artists_scroll: ScrollHandle,
    home_saved_scroll: ScrollHandle,
    genre_albums_scroll: ScrollHandle,
    genre_artists_scroll: ScrollHandle,
    nd_url_input: Entity<TextInput>,
    nd_user_input: Entity<TextInput>,
    nd_pass_input: Entity<TextInput>,
    _search_sub: Option<Subscription>,
    _playlists_sub: Subscription,
    _edit_modal_sub: Subscription,
    _delete_modal_sub: Subscription,
    _album_meta_sub: Subscription,
}

impl LibraryPage {
    pub fn new(cx: &mut gpui::Context<Self>) -> Self {
        cx.set_global(LibrarySection::Home);
        cx.set_global(SelectedAlbum(String::new()));
        cx.set_global(SelectedArtist(String::new()));
        cx.set_global(SelectedGenre::default());
        cx.set_global(GenresState::default());
        cx.set_global(BackSection(LibrarySection::Albums));
        cx.set_global(LibraryContextMenuState::default());
        cx.set_global(AlbumContextMenuState::default());
        cx.set_global(HoveredAlbumIdx::default());
        cx.set_global(LikedSongs::default());
        cx.set_global(LikedOrder::default());
        cx.set_global(PlaylistsState::default());
        cx.set_global(SelectedPlaylist::default());
        cx.set_global(PlaylistsSidebarCollapsed(false));
        cx.set_global(CreatePlaylistModal::default());
        cx.set_global(NavidromeAddModal::default());
        cx.set_global(AddToPlaylistMenuState::default());
        cx.set_global(EditPlaylistModal::default());
        cx.set_global(DeletePlaylistModal::default());
        cx.set_global(SelectedAlbumMeta::default());
        let (nd_servers, _) = crate::nd_persist::load_servers();
        cx.set_global(NavidromeServerState {
            servers: nd_servers,
            active_id: None, // always start in local library mode; user connects manually
            ..NavidromeServerState::default()
        });
        cx.set_global(NdLibraryData::default());
        cx.set_global(NdSelectedAlbum::default());
        cx.set_global(NdSelectedArtist::default());
        cx.set_global(NdSelectedGenre::default());
        cx.set_global(NdSelectedPlaylist::default());
        cx.set_global(NdSongsState::default());
        cx.set_global(NdLikesState::default());
        cx.set_global(NdStarredIds::default());
        cx.set_global(NdContextMenuState::default());
        cx.set_global(NdCurrentCoverArt::default());
        cx.set_global(crate::ui::components::NdScrobbleState::default());
        cx.set_global(crate::ui::components::NdCoverFetchState::default());

        // Re-render whenever PlaylistsState changes (initial load, post-create refresh, etc.)
        let _playlists_sub = cx.observe_global::<PlaylistsState>(|_, cx| cx.notify());
        let _edit_modal_sub = cx.observe_global::<EditPlaylistModal>(|this, cx| {
            let modal = cx.global::<EditPlaylistModal>().clone();
            if modal.open {
                this.edit_name_input.update(cx, |input, cx| {
                    input.value = modal.name.clone();
                    cx.notify();
                });
                this.edit_desc_input.update(cx, |input, cx| {
                    input.value = modal.description.clone();
                    cx.notify();
                });
            }
            cx.notify();
        });
        let _delete_modal_sub = cx.observe_global::<DeletePlaylistModal>(|_, cx| cx.notify());

        // Fetch album metadata (year + copyright) whenever the selected album changes.
        let _album_meta_sub = cx.observe_global::<SelectedAlbum>(|_this, cx| {
            let album_name = cx.global::<SelectedAlbum>().0.clone();
            let album_id = cx
                .global::<Controller>()
                .state
                .read(cx)
                .tracks
                .iter()
                .find(|t| t.album == album_name)
                .map(|t| t.album_id.clone())
                .unwrap_or_default();
            if album_id.is_empty() {
                return;
            }
            let tokio = cx.global::<crate::state::TokioHandle>().0.clone();
            cx.spawn(async move |_, cx| {
                let result = cx
                    .background_executor()
                    .spawn(async move {
                        tokio.block_on(async { crate::client::get_album(&album_id).await })
                    })
                    .await;
                let _ = cx.update(|app: &mut gpui::App| {
                    if let Ok((year_string, copyright_message)) = result {
                        let meta = app.global_mut::<SelectedAlbumMeta>();
                        meta.year_string = year_string;
                        meta.copyright_message = copyright_message;
                    }
                });
            })
            .detach();
        });

        // Kick off the initial playlist load so the sidebar is populated from the start.
        let tokio = cx.global::<crate::state::TokioHandle>().0.clone();
        cx.spawn(async move |_this, cx| {
            let (saved, smart) = cx
                .background_executor()
                .spawn(async move {
                    tokio.block_on(async {
                        let saved = crate::client::fetch_saved_playlists()
                            .await
                            .unwrap_or_default();
                        let smart = crate::client::fetch_smart_playlists()
                            .await
                            .unwrap_or_default();
                        (saved, smart)
                    })
                })
                .await;
            let _ = cx.update(|app: &mut gpui::App| {
                let state = app.global_mut::<PlaylistsState>();
                state.saved = saved;
                state.smart = smart;
            });
        })
        .detach();

        // Initial genres load.
        let tokio_g = cx.global::<crate::state::TokioHandle>().0.clone();
        cx.spawn(async move |_this, cx| {
            let genres = cx
                .background_executor()
                .spawn(async move {
                    tokio_g.block_on(async {
                        crate::client::fetch_genres().await.unwrap_or_default()
                    })
                })
                .await;
            let _ = cx.update(|app: &mut gpui::App| {
                app.global_mut::<GenresState>().0 = genres;
            });
        })
        .detach();

        // Whenever a genre id is selected, fetch its detail (tracks/albums/artists).
        let _genre_sub = cx.observe_global::<SelectedGenre>(|_, cx| {
            let id = cx.global::<SelectedGenre>().id.clone();
            if id.is_empty() {
                return;
            }
            // Avoid re-fetching if we already loaded this id.
            if !cx.global::<SelectedGenre>().tracks.is_empty()
                || !cx.global::<SelectedGenre>().albums.is_empty()
            {
                return;
            }
            let tokio = cx.global::<crate::state::TokioHandle>().0.clone();
            cx.spawn(async move |_, cx| {
                let result = cx
                    .background_executor()
                    .spawn(async move {
                        tokio.block_on(async { crate::client::fetch_genre(id).await })
                    })
                    .await;
                if let Ok((name, tracks, albums, artists)) = result {
                    let _ = cx.update(|app: &mut gpui::App| {
                        let g = app.global_mut::<SelectedGenre>();
                        g.name = name;
                        g.tracks = tracks;
                        g.albums = albums;
                        g.artists = artists;
                    });
                }
            })
            .detach();
        });
        // We don't actually need to keep the subscription alive in `self` —
        // the closure references the global by id and re-fires on each change.
        std::mem::forget(_genre_sub);

        // Background mDNS scan on startup so the server list is pre-populated.
        cx.global_mut::<DiscoveredServers>().scanning = true;
        cx.spawn(async move |_, cx| {
            let found = cx
                .background_executor()
                .spawn(async move { crate::server::scan_mdns(std::time::Duration::from_secs(3)) })
                .await;
            let _ = cx.update(|app: &mut gpui::App| {
                let state = app.global_mut::<DiscoveredServers>();
                state.scanning = false;
                state.servers = found;
            });
        })
        .detach();

        LibraryPage {
            scroll_handle: UniformListScrollHandle::new(),
            detail_scroll_handle: UniformListScrollHandle::new(),
            home_recent_scroll: ScrollHandle::new(),
            home_popular_scroll: ScrollHandle::new(),
            home_artists_scroll: ScrollHandle::new(),
            home_saved_scroll: ScrollHandle::new(),
            genre_albums_scroll: ScrollHandle::new(),
            genre_artists_scroll: ScrollHandle::new(),
            miniplayer: {
                cx.new(|cx| {
                    cx.observe_global::<DevicesState>(|_, cx| cx.notify())
                        .detach();
                    MiniPlayer
                })
            },
            search_input: cx.new(|cx| SearchInput::new(cx)),
            files_view: cx.new(|cx| FilesView::new(cx)),
            modal_name_input: cx.new(|cx| TextInput::new("Title", cx)),
            modal_desc_input: cx.new(|cx| TextInput::new("Description (optional)", cx)),
            edit_name_input: cx.new(|cx| TextInput::new("Title", cx)),
            edit_desc_input: cx.new(|cx| TextInput::new("Description (optional)", cx)),
            nd_url_input: cx.new(|cx| TextInput::new("http://192.168.1.x:4533", cx)),
            nd_user_input: cx.new(|cx| TextInput::new("Username", cx)),
            nd_pass_input: cx.new(|cx| TextInput::new("Password", cx).masked()),
            _search_sub: None,
            _playlists_sub,
            _edit_modal_sub,
            _delete_modal_sub,
            _album_meta_sub,
        }
    }

    fn spawn_nd_connect(&self, cx: &mut gpui::Context<Self>) {
        let raw_url = self.nd_url_input.read(cx).value.clone();
        let username = self.nd_user_input.read(cx).value.clone();
        let password = self.nd_pass_input.read(cx).value.clone();

        if raw_url.trim().is_empty() || username.trim().is_empty() {
            cx.global_mut::<NavidromeServerState>().connect_error =
                Some("Server URL and username are required.".to_string());
            return;
        }

        let base_url = raw_url.trim().to_string();
        let salt = crate::navidrome::random_salt();
        let token = crate::navidrome::compute_token(&password, &salt);
        let base_url2 = base_url.clone();
        let user2 = username.clone();
        let token2 = token.clone();
        let salt2 = salt.clone();

        cx.global_mut::<NavidromeServerState>().connecting = true;
        cx.global_mut::<NavidromeServerState>().connect_error = None;

        let (tx, rx) = tokio::sync::oneshot::channel::<bool>();
        cx.global::<Controller>().rt().spawn(async move {
            let ok = crate::navidrome::ping(&base_url, &username, &token, &salt).await;
            let _ = tx.send(ok);
        });
        cx.spawn(async move |this: gpui::WeakEntity<LibraryPage>, cx| {
            if let Ok(ok) = rx.await {
                if ok {
                    let _ = this.update(cx, |this, cx| {
                        let new_id = crate::navidrome::random_salt();
                        let new_server = crate::ui::components::NdSavedServer {
                            id: new_id.clone(),
                            base_url: base_url2.clone(),
                            user: user2.clone(),
                            token: token2.clone(),
                            salt: salt2.clone(),
                        };
                        {
                            let state = cx.global_mut::<NavidromeServerState>();
                            state.servers.push(new_server);
                            state.active_id = Some(new_id);
                            state.connecting = false;
                            state.connect_error = None;
                        }
                        cx.global_mut::<NavidromeAddModal>().open = false;
                        let servers = cx.global::<NavidromeServerState>().servers.clone();
                        let active_id = cx.global::<NavidromeServerState>().active_id.clone();
                        crate::nd_persist::save_servers(
                            &servers,
                            active_id.as_deref(),
                        );
                        // clear inputs
                        this.nd_url_input.update(cx, |i, cx| {
                            i.value.clear();
                            cx.notify();
                        });
                        this.nd_user_input.update(cx, |i, cx| {
                            i.value.clear();
                            cx.notify();
                        });
                        this.nd_pass_input.update(cx, |i, cx| {
                            i.value.clear();
                            cx.notify();
                        });
                        *cx.global_mut::<NdLibraryData>() = NdLibraryData::default();
                        *cx.global_mut::<LibrarySection>() = LibrarySection::NdAlbums;
                        this.load_nd_library(cx);
                    });
                } else {
                    let _ = this.update(cx, |_, cx| {
                        let state = cx.global_mut::<NavidromeServerState>();
                        state.connecting = false;
                        state.connect_error =
                            Some("Authentication failed. Check URL, username, and password.".to_string());
                    });
                }
            }
        })
        .detach();
    }

    fn load_nd_library(&self, cx: &mut gpui::Context<Self>) {
        let nd = cx.global::<NavidromeServerState>().clone();
        let srv = match nd.active_server() {
            Some(s) => s.clone(),
            None => return,
        };
        let (base_url, user, token, salt) =
            (srv.base_url.clone(), srv.user.clone(), srv.token.clone(), srv.salt.clone());
        let b1 = base_url.clone();
        let u1 = user.clone();
        let t1 = token.clone();
        let s1 = salt.clone();
        let b2 = base_url.clone();
        let u2 = user.clone();
        let t2 = token.clone();
        let s2 = salt.clone();
        let b3 = base_url.clone();
        let u3 = user.clone();
        let t3 = token.clone();
        let s3 = salt.clone();
        let b4 = base_url.clone();
        let u4 = user.clone();
        let t4 = token.clone();
        let s4 = salt.clone();

        cx.global_mut::<NdLibraryData>().loading = true;

        let (tx, rx) = tokio::sync::oneshot::channel::<(
            Vec<NdAlbumItem>,
            Vec<NdArtistItem>,
            Vec<NdGenreItem>,
            Vec<NdPlaylistItem>,
        )>();
        cx.global::<Controller>().rt().spawn(async move {
            let (albums_raw, artists_raw, genres_raw, playlists_raw) = tokio::join!(
                crate::navidrome::get_albums(&b1, &u1, &t1, &s1, "alphabeticalByName", 500, 0),
                crate::navidrome::get_artists(&b2, &u2, &t2, &s2),
                crate::navidrome::get_genres(&b3, &u3, &t3, &s3),
                crate::navidrome::get_playlists(&b4, &u4, &t4, &s4),
            );
            let albums = albums_raw
                .into_iter()
                .map(|a| NdAlbumItem {
                    id: a.id,
                    name: a.name,
                    artist: a.artist,
                    artist_id: a.artist_id,
                    year: a.year,
                    cover_art: a.cover_art,
                    song_count: a.song_count,
                })
                .collect();
            let artists = artists_raw
                .into_iter()
                .map(|a| NdArtistItem {
                    id: a.id,
                    name: a.name,
                    cover_art: a.cover_art,
                    album_count: a.album_count,
                })
                .collect();
            let genres = genres_raw
                .into_iter()
                .map(|g| NdGenreItem {
                    name: g.name,
                    song_count: g.song_count,
                    album_count: g.album_count,
                })
                .collect();
            let playlists = playlists_raw
                .into_iter()
                .map(|p| NdPlaylistItem {
                    id: p.id,
                    name: p.name,
                    comment: p.comment,
                    song_count: p.song_count,
                    cover_art: p.cover_art,
                })
                .collect();
            let _ = tx.send((albums, artists, genres, playlists));
        });
        cx.spawn(async move |_, cx| {
            if let Ok((albums, artists, genres, playlists)) = rx.await {
                let _ = cx.update(|app: &mut gpui::App| {
                    let data = app.global_mut::<NdLibraryData>();
                    data.albums = albums;
                    data.artists = artists;
                    data.genres = genres;
                    data.playlists = playlists;
                    data.loading = false;
                });
            }
        })
        .detach();
    }

    fn load_nd_album(&self, cx: &mut gpui::Context<Self>, album_id: String) {
        let nd = cx.global::<NavidromeServerState>().clone();
        let srv = match nd.active_server() {
            Some(s) => s.clone(),
            None => return,
        };
        let (base_url, user, token, salt) =
            (srv.base_url.clone(), srv.user.clone(), srv.token.clone(), srv.salt.clone());
        let b = base_url.clone();
        let u = user.clone();
        let t = token.clone();
        let s = salt.clone();
        let aid = album_id.clone();

        cx.global_mut::<NdSelectedAlbum>().loading = true;
        cx.global_mut::<NdSelectedAlbum>().songs = vec![];

        let (tx, rx) = tokio::sync::oneshot::channel::<Option<(
            crate::navidrome::NavidromeAlbum,
            Vec<crate::navidrome::NavidromeSong>,
        )>>();
        cx.global::<Controller>().rt().spawn(async move {
            let result = crate::navidrome::get_album_with_songs(&b, &u, &t, &s, &aid).await;
            let _ = tx.send(result);
        });
        cx.spawn(async move |_, cx| {
            if let Ok(Some((album, songs))) = rx.await {
                let _ = cx.update(|app: &mut gpui::App| {
                    let (bu, un, tok, sa) = app
                        .global::<NavidromeServerState>()
                        .active_server()
                        .map(|s| {
                            (s.base_url.clone(), s.user.clone(), s.token.clone(), s.salt.clone())
                        })
                        .unwrap_or_default();
                    // Navidrome often omits coverArt from individual songs;
                    // fall back to the album's cover art ID.
                    let album_cover = album.cover_art.clone();
                    let song_items = songs
                        .into_iter()
                        .map(|s| {
                            let surl = crate::navidrome::stream_url(&bu, &un, &tok, &sa, &s.id);
                            NdSongItem {
                                id: s.id,
                                title: s.title,
                                artist: s.artist,
                                artist_id: s.artist_id,
                                album: s.album,
                                album_id: s.album_id,
                                cover_art: s.cover_art.or_else(|| album_cover.clone()),
                                duration: s.duration,
                                track: s.track,
                                stream_url: surl,
                            }
                        })
                        .collect();
                    let sel = app.global_mut::<NdSelectedAlbum>();
                    sel.id = album.id;
                    sel.name = album.name;
                    sel.artist = album.artist;
                    sel.cover_art = album.cover_art;
                    sel.songs = song_items;
                    sel.loading = false;
                });
            }
        })
        .detach();
    }

    fn load_nd_artist(&self, cx: &mut gpui::Context<Self>, artist_id: String) {
        let nd = cx.global::<NavidromeServerState>().clone();
        let srv = match nd.active_server() {
            Some(s) => s.clone(),
            None => return,
        };
        let b = srv.base_url.clone();
        let u = srv.user.clone();
        let t = srv.token.clone();
        let s = srv.salt.clone();

        cx.global_mut::<NdSelectedArtist>().loading = true;
        cx.global_mut::<NdSelectedArtist>().albums = vec![];

        let (tx, rx) = tokio::sync::oneshot::channel::<Option<(
            crate::navidrome::NavidromeArtist,
            Vec<crate::navidrome::NavidromeAlbum>,
        )>>();
        cx.global::<Controller>().rt().spawn(async move {
            let result = crate::navidrome::get_artist_with_albums(&b, &u, &t, &s, &artist_id).await;
            let _ = tx.send(result);
        });
        cx.spawn(async move |_, cx| {
            if let Ok(Some((artist, albums))) = rx.await {
                let _ = cx.update(|app: &mut gpui::App| {
                    let album_items = albums
                        .into_iter()
                        .map(|a| NdAlbumItem {
                            id: a.id,
                            name: a.name,
                            artist: a.artist,
                            artist_id: a.artist_id,
                            year: a.year,
                            cover_art: a.cover_art,
                            song_count: a.song_count,
                        })
                        .collect();
                    let sel = app.global_mut::<NdSelectedArtist>();
                    sel.id = artist.id;
                    sel.name = artist.name;
                    sel.cover_art = artist.cover_art;
                    sel.albums = album_items;
                    sel.loading = false;
                });
            }
        })
        .detach();
    }

    fn load_nd_genre(&self, cx: &mut gpui::Context<Self>, genre_name: String) {
        let nd = cx.global::<NavidromeServerState>().clone();
        let srv = match nd.active_server() {
            Some(s) => s.clone(),
            None => return,
        };
        let b = srv.base_url.clone();
        let u = srv.user.clone();
        let t = srv.token.clone();
        let s = srv.salt.clone();

        cx.global_mut::<NdSelectedGenre>().loading = true;
        cx.global_mut::<NdSelectedGenre>().songs = vec![];

        let gname = genre_name.clone();
        let (tx, rx) =
            tokio::sync::oneshot::channel::<Vec<crate::navidrome::NavidromeSong>>();
        cx.global::<Controller>().rt().spawn(async move {
            let songs =
                crate::navidrome::get_songs_by_genre(&b, &u, &t, &s, &gname, 500, 0).await;
            let _ = tx.send(songs);
        });
        cx.spawn(async move |_, cx| {
            if let Ok(songs) = rx.await {
                let _ = cx.update(|app: &mut gpui::App| {
                    let (bu, un, tok, sa) = app
                        .global::<NavidromeServerState>()
                        .active_server()
                        .map(|s| {
                            (s.base_url.clone(), s.user.clone(), s.token.clone(), s.salt.clone())
                        })
                        .unwrap_or_default();
                    let items = songs
                        .into_iter()
                        .map(|s| {
                            let surl = crate::navidrome::stream_url(&bu, &un, &tok, &sa, &s.id);
                            NdSongItem {
                                id: s.id,
                                title: s.title,
                                artist: s.artist,
                                artist_id: s.artist_id,
                                album: s.album,
                                album_id: s.album_id,
                                cover_art: s.cover_art,
                                duration: s.duration,
                                track: s.track,
                                stream_url: surl,
                            }
                        })
                        .collect();
                    let sel = app.global_mut::<NdSelectedGenre>();
                    sel.name = genre_name.clone();
                    sel.songs = items;
                    sel.loading = false;
                });
            }
        })
        .detach();
    }

    fn load_nd_playlist(&self, cx: &mut gpui::Context<Self>, playlist_id: String) {
        let nd = cx.global::<NavidromeServerState>().clone();
        let srv = match nd.active_server() {
            Some(s) => s.clone(),
            None => return,
        };
        let b = srv.base_url.clone();
        let u = srv.user.clone();
        let t = srv.token.clone();
        let s = srv.salt.clone();

        cx.global_mut::<NdSelectedPlaylist>().loading = true;
        cx.global_mut::<NdSelectedPlaylist>().tracks = vec![];

        let pid = playlist_id.clone();
        let (tx, rx) = tokio::sync::oneshot::channel::<Option<(
            crate::navidrome::NavidromePlaylist,
            Vec<crate::navidrome::NavidromeSong>,
        )>>();
        cx.global::<Controller>().rt().spawn(async move {
            let result = crate::navidrome::get_playlist_with_tracks(&b, &u, &t, &s, &pid).await;
            let _ = tx.send(result);
        });
        cx.spawn(async move |_, cx| {
            if let Ok(Some((playlist, tracks))) = rx.await {
                let _ = cx.update(|app: &mut gpui::App| {
                    let (bu, un, tok, sa) = app
                        .global::<NavidromeServerState>()
                        .active_server()
                        .map(|s| {
                            (s.base_url.clone(), s.user.clone(), s.token.clone(), s.salt.clone())
                        })
                        .unwrap_or_default();
                    let items = tracks
                        .into_iter()
                        .map(|s| {
                            let surl = crate::navidrome::stream_url(&bu, &un, &tok, &sa, &s.id);
                            NdSongItem {
                                id: s.id,
                                title: s.title,
                                artist: s.artist,
                                artist_id: s.artist_id,
                                album: s.album,
                                album_id: s.album_id,
                                cover_art: s.cover_art,
                                duration: s.duration,
                                track: s.track,
                                stream_url: surl,
                            }
                        })
                        .collect();
                    let sel = app.global_mut::<NdSelectedPlaylist>();
                    sel.id = playlist.id;
                    sel.name = playlist.name;
                    sel.tracks = items;
                    sel.loading = false;
                });
            }
        })
        .detach();
    }

    fn load_nd_songs(&self, cx: &mut gpui::Context<Self>) {
        let nd = cx.global::<NavidromeServerState>().clone();
        let srv = match nd.active_server() {
            Some(s) => s.clone(),
            None => return,
        };
        let (b, u, t, s) = (srv.base_url.clone(), srv.user.clone(), srv.token.clone(), srv.salt.clone());
        cx.global_mut::<NdSongsState>().loading = true;
        cx.global_mut::<NdSongsState>().songs = vec![];
        let (tx, rx) = tokio::sync::oneshot::channel::<Vec<crate::navidrome::NavidromeSong>>();
        cx.global::<Controller>().rt().spawn(async move {
            let songs = crate::navidrome::search_songs(&b, &u, &t, &s, "", 500).await;
            let _ = tx.send(songs);
        });
        cx.spawn(async move |_, cx| {
            if let Ok(songs) = rx.await {
                let _ = cx.update(|app: &mut gpui::App| {
                    let (bu, un, tok, sa) = app.global::<NavidromeServerState>()
                        .active_server()
                        .map(|s| (s.base_url.clone(), s.user.clone(), s.token.clone(), s.salt.clone()))
                        .unwrap_or_default();
                    let items = songs.into_iter().map(|s| {
                        let surl = crate::navidrome::stream_url(&bu, &un, &tok, &sa, &s.id);
                        NdSongItem { id: s.id, title: s.title, artist: s.artist, artist_id: s.artist_id,
                            album: s.album, album_id: s.album_id, cover_art: s.cover_art,
                            duration: s.duration, track: s.track, stream_url: surl }
                    }).collect();
                    let state = app.global_mut::<NdSongsState>();
                    state.songs = items;
                    state.loading = false;
                });
            }
        }).detach();
    }

    fn load_nd_likes(&self, cx: &mut gpui::Context<Self>) {
        let nd = cx.global::<NavidromeServerState>().clone();
        let srv = match nd.active_server() {
            Some(s) => s.clone(),
            None => return,
        };
        let (b, u, t, s) = (srv.base_url.clone(), srv.user.clone(), srv.token.clone(), srv.salt.clone());
        cx.global_mut::<NdLikesState>().loading = true;
        cx.global_mut::<NdLikesState>().songs = vec![];
        let (tx, rx) = tokio::sync::oneshot::channel::<Vec<crate::navidrome::NavidromeSong>>();
        cx.global::<Controller>().rt().spawn(async move {
            let songs = crate::navidrome::get_starred(&b, &u, &t, &s).await;
            let _ = tx.send(songs);
        });
        cx.spawn(async move |_, cx| {
            if let Ok(songs) = rx.await {
                let _ = cx.update(|app: &mut gpui::App| {
                    let (bu, un, tok, sa) = app.global::<NavidromeServerState>()
                        .active_server()
                        .map(|s| (s.base_url.clone(), s.user.clone(), s.token.clone(), s.salt.clone()))
                        .unwrap_or_default();
                    let items = songs.into_iter().map(|s| {
                        let surl = crate::navidrome::stream_url(&bu, &un, &tok, &sa, &s.id);
                        NdSongItem { id: s.id.clone(), title: s.title, artist: s.artist, artist_id: s.artist_id,
                            album: s.album, album_id: s.album_id, cover_art: s.cover_art,
                            duration: s.duration, track: s.track, stream_url: surl }
                    }).collect::<Vec<_>>();
                    // Also update the starred ID set so like buttons reflect state
                    let starred: std::collections::HashSet<String> = items.iter().map(|s| s.id.clone()).collect();
                    app.global_mut::<NdStarredIds>().0 = starred;
                    let state = app.global_mut::<NdLikesState>();
                    state.songs = items;
                    state.loading = false;
                });
            }
        }).detach();
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
        let playlists_collapsed = cx.global::<PlaylistsSidebarCollapsed>().0;
        let saved_playlists = cx.global::<PlaylistsState>().saved.clone();
        let smart_playlists = cx.global::<PlaylistsState>().smart.clone();
        let playlist_tracks = cx.global::<PlaylistsState>().playlist_tracks.clone();
        let selected_playlist = cx.global::<SelectedPlaylist>().clone();
        let create_modal = cx.global::<CreatePlaylistModal>().clone();
        let edit_modal = cx.global::<EditPlaylistModal>().clone();
        let delete_modal = cx.global::<DeletePlaylistModal>().clone();
        let add_to_playlist_menu = cx.global::<AddToPlaylistMenuState>().0.clone();
        let album_meta = cx.global::<SelectedAlbumMeta>().clone();
        let genres_state = cx.global::<GenresState>().0.clone();
        let selected_genre = cx.global::<SelectedGenre>().clone();
        let home_recent_scroll = self.home_recent_scroll.clone();
        let home_popular_scroll = self.home_popular_scroll.clone();
        let home_artists_scroll = self.home_artists_scroll.clone();
        let home_saved_scroll = self.home_saved_scroll.clone();
        let genre_albums_scroll = self.genre_albums_scroll.clone();
        let genre_artists_scroll = self.genre_artists_scroll.clone();
        let cur_server = crate::server::current_server();
        let picker_open = cx.global::<ServerPickerOpen>().0;
        let server_scanning = cx.global::<DiscoveredServers>().scanning;
        let discovered_servers = cx.global::<DiscoveredServers>().servers.clone();
        let nd_state = cx.global::<NavidromeServerState>().clone();
        let nd_add_modal = cx.global::<NavidromeAddModal>().clone();
        let nd_data = cx.global::<NdLibraryData>().clone();
        let nd_sel_album = cx.global::<NdSelectedAlbum>().clone();
        let nd_sel_artist = cx.global::<NdSelectedArtist>().clone();
        let nd_sel_genre = cx.global::<NdSelectedGenre>().clone();
        let nd_sel_playlist = cx.global::<NdSelectedPlaylist>().clone();
        let nd_songs_state = cx.global::<NdSongsState>().clone();
        let nd_likes_state = cx.global::<NdLikesState>().clone();
        let nd_starred_ids = cx.global::<NdStarredIds>().0.clone();

        // Trigger playlist tracks load when in detail views
        if (section == LibrarySection::PlaylistDetail
            || section == LibrarySection::SmartPlaylistDetail)
            && playlist_tracks.is_empty()
            && !selected_playlist.id.is_empty()
        {
            let pid = selected_playlist.id.clone();
            let is_smart = selected_playlist.is_smart;
            let all_tracks = cx.global::<Controller>().state.read(cx).tracks.clone();
            let tokio = cx.global::<crate::state::TokioHandle>().0.clone();
            cx.spawn(async move |_this: gpui::WeakEntity<LibraryPage>, cx| {
                let track_ids = cx
                    .background_executor()
                    .spawn(async move {
                        tokio.block_on(async move {
                            if is_smart {
                                crate::client::fetch_smart_playlist_track_ids(pid)
                                    .await
                                    .unwrap_or_default()
                            } else {
                                crate::client::fetch_saved_playlist_track_ids(pid)
                                    .await
                                    .unwrap_or_default()
                            }
                        })
                    })
                    .await;
                let id_set: std::collections::HashSet<String> = track_ids.iter().cloned().collect();
                let mut resolved: Vec<crate::state::Track> = all_tracks
                    .into_iter()
                    .filter(|t| id_set.contains(&t.id))
                    .collect();
                let order_map: std::collections::HashMap<String, usize> = track_ids
                    .into_iter()
                    .enumerate()
                    .map(|(i, id)| (id, i))
                    .collect();
                resolved.sort_by_key(|t| order_map.get(&t.id).copied().unwrap_or(usize::MAX));
                let _ = cx.update(|app: &mut gpui::App| {
                    app.global_mut::<PlaylistsState>().playlist_tracks = resolved;
                });
            })
            .detach();
        }

        // Trigger Navidrome library load when needed
        if nd_state.connected() && nd_data.albums.is_empty() && !nd_data.loading {
            self.load_nd_library(cx);
        }
        if section == LibrarySection::NdAlbumDetail
            && nd_sel_album.songs.is_empty()
            && !nd_sel_album.loading
            && !nd_sel_album.id.is_empty()
        {
            let aid = nd_sel_album.id.clone();
            self.load_nd_album(cx, aid);
        }
        if section == LibrarySection::NdArtistDetail
            && nd_sel_artist.albums.is_empty()
            && !nd_sel_artist.loading
            && !nd_sel_artist.id.is_empty()
        {
            let aid = nd_sel_artist.id.clone();
            self.load_nd_artist(cx, aid);
        }
        if section == LibrarySection::NdGenreDetail
            && nd_sel_genre.songs.is_empty()
            && !nd_sel_genre.loading
            && !nd_sel_genre.name.is_empty()
        {
            let gname = nd_sel_genre.name.clone();
            self.load_nd_genre(cx, gname);
        }
        if section == LibrarySection::NdPlaylistDetail
            && nd_sel_playlist.tracks.is_empty()
            && !nd_sel_playlist.loading
            && !nd_sel_playlist.id.is_empty()
        {
            let pid = nd_sel_playlist.id.clone();
            self.load_nd_playlist(cx, pid);
        }
        if section == LibrarySection::NdSongs
            && nd_songs_state.songs.is_empty()
            && !nd_songs_state.loading
        {
            self.load_nd_songs(cx);
        }
        if section == LibrarySection::NdLikes
            && nd_likes_state.songs.is_empty()
            && !nd_likes_state.loading
        {
            self.load_nd_likes(cx);
        }

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
                (
                    String,
                    u32,
                    usize,
                    Option<String>,
                    String,
                    Vec<(u32, String)>,
                ),
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
            let albums: Vec<(
                String,
                String,
                u32,
                usize,
                Option<String>,
                String,
                Vec<String>,
            )> = album_map
                .into_iter()
                .map(
                    |(name, (artist, year, count, art, album_id, mut numbered_paths))| {
                        numbered_paths.sort_by_key(|(num, _)| *num);
                        let paths: Vec<String> =
                            numbered_paths.into_iter().map(|(_, p)| p).collect();
                        (name, artist, year, count, art, album_id, paths)
                    },
                )
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

            // Album detail: tracks filtered by selected album — (global_idx, path, title, num, dur, id, album_art, disc_number)
            let mut album_tracks: Vec<(
                usize,
                String,
                String,
                String,
                u64,
                String,
                Option<String>,
                u32,
            )> = state
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
                        t.disc_number,
                    )
                })
                .collect();
            album_tracks.sort_by_key(|(_, _, _, num, _, _, _, disc)| {
                (*disc, num.parse::<u32>().unwrap_or(0))
            });

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
            let artist_tracks: Vec<(usize, String, String, String, u64, String, Option<String>)> =
                state
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
            let mut liked_tracks: Vec<(
                usize,
                String,
                String,
                String,
                String,
                u64,
                String,
                Option<String>,
            )> = state
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
                liked_order_map
                    .get(id.as_str())
                    .copied()
                    .unwrap_or(usize::MAX)
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
                let query = si_entity.read(cx).query.trim().to_string();
                cx.global::<Controller>().search(query);
                cx.notify();
            }));
        }
        let query = self.search_input.read(cx).query.trim().to_string();
        let search_input = self.search_input.clone();

        // Active Navidrome credentials (for cover art URL generation in search)
        let nd_creds_search = nd_state.active_server().cloned().unwrap_or_default();
        let nd_is_connected = nd_state.connected();

        // Navidrome in-memory search results (filtered from already-loaded library data)
        let nd_q = query.to_lowercase();
        let nd_search_albums: Vec<crate::ui::components::NdAlbumItem> = if nd_is_connected && !nd_q.is_empty() {
            nd_data.albums.iter().filter(|a| {
                a.name.to_lowercase().contains(&nd_q) || a.artist.to_lowercase().contains(&nd_q)
            }).take(8).cloned().collect()
        } else { vec![] };
        let nd_search_artists: Vec<crate::ui::components::NdArtistItem> = if nd_is_connected && !nd_q.is_empty() {
            nd_data.artists.iter().filter(|a| a.name.to_lowercase().contains(&nd_q)).take(8).cloned().collect()
        } else { vec![] };
        let nd_search_playlists: Vec<crate::ui::components::NdPlaylistItem> = if nd_is_connected && !nd_q.is_empty() {
            nd_data.playlists.iter().filter(|p| p.name.to_lowercase().contains(&nd_q)).take(10).cloned().collect()
        } else { vec![] };
        let nd_search_has_results = !nd_search_albums.is_empty() || !nd_search_artists.is_empty() || !nd_search_playlists.is_empty();

        let context_menu = cx.global::<LibraryContextMenuState>().0.clone();
        let album_context_menu = cx.global::<AlbumContextMenuState>().0.clone();
        let file_context_menu = cx.global::<FileContextMenuState>().0.clone();
        let nd_context_menu = cx.global::<NdContextMenuState>().0.clone();
        let n_album_tracks = album_tracks.len();
        let n_artist_tracks = artist_tracks.len();
        let scroll_handle = self.scroll_handle.clone();
        let modal_name_input = self.modal_name_input.clone();
        let modal_desc_input = self.modal_desc_input.clone();
        let modal_name_value = self.modal_name_input.read(cx).value.trim().to_string();
        let modal_desc_value = self.modal_desc_input.read(cx).value.trim().to_string();
        let edit_name_input = self.edit_name_input.clone();
        let edit_desc_input = self.edit_desc_input.clone();
        let edit_name_value = self.edit_name_input.read(cx).value.trim().to_string();
        let edit_desc_value = self.edit_desc_input.read(cx).value.trim().to_string();
        let _detail_scroll_handle = self.detail_scroll_handle.clone();

        // Sidebar nav item — Albums/Artists stay active while in their detail view.
        // When a Navidrome server is active, Albums/Artists/Genres route to Nd* sections.
        let nd_connected = nd_state.connected();
        let make_nav_item =
            move |icon: Icons, icon_size: u8, label: &'static str, target: LibrarySection| {
                let nd_target = if nd_connected {
                    match target {
                        LibrarySection::Albums => Some(LibrarySection::NdAlbums),
                        LibrarySection::Artists => Some(LibrarySection::NdArtists),
                        LibrarySection::Genres => Some(LibrarySection::NdGenres),
                        LibrarySection::Songs => Some(LibrarySection::NdSongs),
                        LibrarySection::Likes => Some(LibrarySection::NdLikes),
                        _ => None,
                    }
                } else {
                    None
                };
                let effective = nd_target.unwrap_or(target);
                let is_active = section == target
                    || section == effective
                    || (section == LibrarySection::AlbumDetail && target == LibrarySection::Albums)
                    || (section == LibrarySection::NdAlbumDetail && target == LibrarySection::Albums)
                    || (section == LibrarySection::ArtistDetail
                        && target == LibrarySection::Artists)
                    || (section == LibrarySection::NdArtistDetail
                        && target == LibrarySection::Artists)
                    || (section == LibrarySection::AlbumDetail
                        && back_section == LibrarySection::ArtistDetail
                        && target == LibrarySection::Artists)
                    || (section == LibrarySection::GenreDetail
                        && target == LibrarySection::Genres)
                    || (section == LibrarySection::NdGenreDetail
                        && target == LibrarySection::Genres)
                    || (section == LibrarySection::AlbumDetail
                        && back_section == LibrarySection::GenreDetail
                        && target == LibrarySection::Genres)
                    || (section == LibrarySection::ArtistDetail
                        && back_section == LibrarySection::GenreDetail
                        && target == LibrarySection::Genres)
                    || (section == LibrarySection::NdSongs && target == LibrarySection::Songs)
                    || (section == LibrarySection::NdLikes && target == LibrarySection::Likes);
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
                        *cx.global_mut::<LibrarySection>() = effective;
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
                              track_art: Option<String>,
                              remove_playlist_id: Option<String>| {
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
            let track_id_for_remove = track_id.clone();
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
                            .min_w_0()
                            .overflow_hidden()
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
                            .min_w_0()
                            .overflow_hidden()
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
                                    track_id: track_id.clone(),
                                });
                        })
                        .child(Icon::new(Icons::Options).size_4()),
                )
                .when_some(remove_playlist_id, |this, pid| {
                    let remove_id: gpui::SharedString =
                        format!("{}_remove_{}", row_id.0, row_id.1).into();
                    let tid_for_remove = track_id_for_remove.clone();
                    this.child(
                        div()
                            .id(remove_id)
                            .w(px(28.0))
                            .flex_shrink_0()
                            .flex()
                            .items_center()
                            .justify_center()
                            .cursor_pointer()
                            .text_color(theme.library_header_text)
                            .hover(|s| s.text_color(gpui::rgb(0xef4444)))
                            .on_click(move |_, _, cx: &mut App| {
                                cx.stop_propagation();
                                let tokio =
                                    cx.global::<crate::state::TokioHandle>().0.clone();
                                let pid2 = pid.clone();
                                let tid2 = tid_for_remove.clone();
                                cx.spawn(async move |cx| {
                                    let result = cx
                                        .background_executor()
                                        .spawn(async move {
                                            tokio.block_on(async move {
                                                let _ = crate::client::remove_track_from_saved_playlist(pid2, tid2).await;
                                                let saved = crate::client::fetch_saved_playlists()
                                                    .await
                                                    .ok();
                                                saved
                                            })
                                        })
                                        .await;
                                    let _ = cx.update(|app: &mut gpui::App| {
                                        if let Some(saved) = result {
                                            app.global_mut::<PlaylistsState>().saved = saved;
                                        }
                                        app.global_mut::<PlaylistsState>().playlist_tracks =
                                            vec![];
                                    });
                                })
                                .detach();
                            })
                            .child(Icon::new(Icons::Trash).size_4()),
                    )
                })
        };

        // ── nd_song_row helper ─────────────────────────────────────────────────────
        let nd_song_row = {
            let starred = nd_starred_ids.clone();
            let nd_creds_row = nd_state.active_server().cloned().unwrap_or_default();
            move |row_id: (&'static str, usize),
                  song_id: String,
                  title: String,
                  artist: String,
                  album: String,
                  artist_id: String,
                  album_id: String,
                  stream_url: String,
                  duration: u32,
                  track_num: Option<u32>,
                  cover_art: Option<String>| {
                let is_starred = starred.contains(&song_id);
                let surl_play = stream_url.clone();
                let surl_ctx = stream_url.clone();
                let sid_heart = song_id.clone();
                let sid_ctx = song_id.clone();
                let title_ctx = title.clone();
                let artist_ctx = artist.clone();
                let album_ctx = album.clone();
                let artist_id_ctx = artist_id.clone();
                let album_id_ctx = album_id.clone();
                // Captures for NdLikesState live-update
                let heart_song_item = NdSongItem {
                    id: song_id.clone(),
                    title: title.clone(),
                    artist: artist.clone(),
                    artist_id: artist_id.clone(),
                    album: album.clone(),
                    album_id: album_id.clone(),
                    cover_art: cover_art.clone(),
                    duration,
                    track: None,
                    stream_url: stream_url.clone(),
                };
                let heart_id: gpui::SharedString =
                    format!("{}_nd_heart_{}", row_id.0, row_id.1).into();
                let opts_id: gpui::SharedString =
                    format!("{}_nd_opts_{}", row_id.0, row_id.1).into();
                let group: gpui::SharedString =
                    format!("{}_nd_grp_{}", row_id.0, row_id.1).into();
                let group2 = group.clone();
                let cov = cover_art.clone();
                // Pre-compute cover art URL for the miniplayer/player when this track plays
                let cover_art_url_for_play = cover_art.as_ref().map(|cid| {
                    crate::navidrome::cover_art_url(
                        &nd_creds_row.base_url,
                        &nd_creds_row.user,
                        &nd_creds_row.token,
                        &nd_creds_row.salt,
                        cid,
                        Some(300),
                    )
                });
                div()
                    .id(row_id)
                    .group(group.clone())
                    .w_full()
                    .flex()
                    .items_center()
                    .gap_x_4()
                    .px_6()
                    .py_3()
                    .cursor_pointer()
                    .hover(|t| t.bg(theme.library_track_bg_hover))
                    .on_click(move |_, _, cx: &mut App| {
                        cx.global_mut::<NdCurrentCoverArt>().0 = cover_art_url_for_play.clone();
                        let rt = cx.global::<Controller>().rt();
                        let url = surl_play.clone();
                        rt.spawn(async move {
                            let _ = crate::client::play_track(url).await;
                        });
                    })
                    .child(
                        div()
                            .w(px(28.0))
                            .flex_shrink_0()
                            .text_sm()
                            .text_color(theme.library_header_text)
                            .child(
                                track_num
                                    .map(|n| n.to_string())
                                    .unwrap_or_else(|| (row_id.1 + 1).to_string()),
                            ),
                    )
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .flex()
                            .flex_col()
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(theme.library_text)
                                    .truncate()
                                    .child(title),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(theme.library_header_text)
                                    .truncate()
                                    .child(format!("{} — {}", artist, album)),
                            ),
                    )
                    .child(
                        div()
                            .w(px(56.0))
                            .flex_shrink_0()
                            .text_sm()
                            .text_color(theme.library_header_text)
                            .child(crate::state::format_duration(duration as u64)),
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
                            .text_color(if is_starred {
                                gpui::rgb(0xFFFFFF)
                            } else {
                                theme.library_header_text
                            })
                            .on_click(move |_, _, cx: &mut App| {
                                cx.stop_propagation();
                                let nd = cx.global::<NavidromeServerState>().clone();
                                if let Some(srv) = nd.active_server() {
                                    let (b, u, t, s) = (
                                        srv.base_url.clone(),
                                        srv.user.clone(),
                                        srv.token.clone(),
                                        srv.salt.clone(),
                                    );
                                    let id = sid_heart.clone();
                                    let starred_ids = &mut cx.global_mut::<NdStarredIds>().0;
                                    if starred_ids.contains(&id) {
                                        starred_ids.remove(&id);
                                        cx.global_mut::<NdLikesState>().songs.retain(|s| s.id != id);
                                        cx.global::<Controller>().rt().spawn(async move {
                                            crate::navidrome::unstar_song(&b, &u, &t, &s, &id).await;
                                        });
                                    } else {
                                        starred_ids.insert(id.clone());
                                        let item = heart_song_item.clone();
                                        cx.global_mut::<NdLikesState>().songs.insert(0, item);
                                        cx.global::<Controller>().rt().spawn(async move {
                                            crate::navidrome::star_song(&b, &u, &t, &s, &id).await;
                                        });
                                    }
                                }
                            })
                            .child(Icon::new(if is_starred {
                                Icons::Heart
                            } else {
                                Icons::HeartOutline
                            }).size_5()),
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
                            .group_hover(group2, |s| s.opacity(1.0))
                            .cursor_pointer()
                            .text_color(theme.library_header_text)
                            .on_click(move |event, _, cx: &mut App| {
                                cx.stop_propagation();
                                cx.global_mut::<NdContextMenuState>().0 = Some(NdContextMenu {
                                    pos: event.position(),
                                    song_id: sid_ctx.clone(),
                                    title: title_ctx.clone(),
                                    artist_id: artist_id_ctx.clone(),
                                    artist: artist_ctx.clone(),
                                    album_id: album_id_ctx.clone(),
                                    album: album_ctx.clone(),
                                    cover_art: cov.clone(),
                                    stream_url: surl_ctx.clone(),
                                });
                            })
                            .child(Icon::new(Icons::Options).size_4()),
                    )
            }
        };

        let show_search = !query.is_empty();
        let content: AnyElement = if show_search {
            // ── Search results ────────────────────────────────────────────────────
            match search_results {
                None if !nd_search_has_results => div().flex_1().into_any_element(),
                None => {
                    // No local results yet, but Navidrome has matches — fall through to nd-only view
                    div()
                        .id("search_results_scroll")
                        .flex_1()
                        .min_h_0()
                        .overflow_y_scroll()
                        .child(nd_search_sections(
                            &nd_search_albums,
                            &nd_search_artists,
                            &nd_search_playlists,
                            &nd_creds_search,
                            search_input.clone(),
                            theme,
                        ))
                        .into_any_element()
                }
                Some(ref r)
                    if r.tracks.is_empty()
                        && r.albums.is_empty()
                        && r.artists.is_empty()
                        && r.playlists.is_empty()
                        && !nd_search_has_results =>
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
                                            .child(div().flex().items_start().gap_x_4().children(
                                                r.artists.iter().take(8).enumerate().map(
                                                    |(idx, artist)| {
                                                        let name_clone = artist.name.clone();
                                                        let si = search_input.clone();
                                                        let img_url = artist
                                                            .image
                                                            .as_deref()
                                                            .filter(|s| !s.is_empty())
                                                            .map(|s| {
                                                                if s.starts_with("http") {
                                                                    s.to_string()
                                                                } else {
                                                                    format!("{}{s}", covers_base())
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
                                                                *cx.global_mut::<SelectedArtist>(
                                                                ) = SelectedArtist(
                                                                    name_clone.clone(),
                                                                );
                                                                *cx.global_mut::<LibrarySection>(
                                                                ) = LibrarySection::ArtistDetail;
                                                                si.update(cx, |this, cx| {
                                                                    this.query.clear();
                                                                    cx.notify();
                                                                });
                                                            })
                                                            .child({
                                                                let mut c = div()
                                                                    .w(px(88.0))
                                                                    .rounded_full()
                                                                    .overflow_hidden()
                                                                    .flex_shrink_0();
                                                                c.style().aspect_ratio =
                                                                    Some(1.0_f32);
                                                                if let Some(url) = img_url {
                                                                    c.child(
                                                                        img(url)
                                                                            .w_full()
                                                                            .h_full()
                                                                            .rounded_full()
                                                                            .object_fit(
                                                                                ObjectFit::Cover,
                                                                            ),
                                                                    )
                                                                } else {
                                                                    c.bg(theme.library_art_bg)
                                                                        .flex()
                                                                        .items_center()
                                                                        .justify_center()
                                                                        .text_color(
                                                                            theme.player_icons_text,
                                                                        )
                                                                        .child(
                                                                            Icon::new(
                                                                                Icons::Artist,
                                                                            )
                                                                            .size_8(),
                                                                        )
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
                                                ),
                                            )),
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
                                            .child(div().flex().items_start().gap_x_4().children(
                                                r.albums.iter().take(8).enumerate().map(
                                                    |(idx, album)| {
                                                        let title_clone = album.title.clone();
                                                        let si = search_input.clone();
                                                        let art_url = album
                                                            .album_art
                                                            .as_deref()
                                                            .filter(|s| !s.is_empty())
                                                            .map(|id| format!("{}{id}", covers_base()));
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
                                                                    SelectedAlbum(
                                                                        title_clone.clone(),
                                                                    );
                                                                *cx.global_mut::<BackSection>() =
                                                                    BackSection(
                                                                        LibrarySection::Albums,
                                                                    );
                                                                *cx.global_mut::<LibrarySection>(
                                                                ) = LibrarySection::AlbumDetail;
                                                                si.update(cx, |this, cx| {
                                                                    this.query.clear();
                                                                    cx.notify();
                                                                });
                                                            })
                                                            .child({
                                                                let mut c = div()
                                                                    .w_full()
                                                                    .rounded_lg()
                                                                    .overflow_hidden();
                                                                c.style().aspect_ratio =
                                                                    Some(1.0_f32);
                                                                if let Some(url) = art_url {
                                                                    c.child(
                                                                        img(url)
                                                                            .w_full()
                                                                            .h_full()
                                                                            .object_fit(
                                                                                ObjectFit::Cover,
                                                                            ),
                                                                    )
                                                                } else {
                                                                    c.bg(theme.library_art_bg)
                                                                        .flex()
                                                                        .items_center()
                                                                        .justify_center()
                                                                        .text_color(
                                                                            theme.player_icons_text,
                                                                        )
                                                                        .child(
                                                                            Icon::new(Icons::Music)
                                                                                .size_8(),
                                                                        )
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
                                                                    .text_color(
                                                                        theme.library_header_text,
                                                                    )
                                                                    .truncate()
                                                                    .child(album.artist.clone()),
                                                            )
                                                    },
                                                ),
                                            )),
                                    )
                                })
                                // ── Songs ─────────────────────────────────────────
                                .when(!r.tracks.is_empty(), |this| {
                                    let rows =
                                        r.tracks.iter().take(20).enumerate().map(|(i, track)| {
                                            let is_current = current_path.as_deref()
                                                == Some(track.path.as_str());
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
                                                None,
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
                                })
                                // ── Playlists ─────────────────────────────────────
                                .when(!r.playlists.is_empty(), |this| {
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
                                                    .child("Playlists"),
                                            )
                                            .child(div().flex().flex_col().gap_y_1().children(
                                                r.playlists.iter().take(10).enumerate().map(
                                                    |(idx, pl)| {
                                                        let pl_id = pl.id.clone();
                                                        let pl_name = pl.name.clone();
                                                        let pl_is_smart = pl.is_smart;
                                                        let si = search_input.clone();
                                                        div()
                                                            .id(("spl", idx))
                                                            .flex()
                                                            .items_center()
                                                            .gap_x_3()
                                                            .px_2()
                                                            .py_2()
                                                            .rounded_md()
                                                            .cursor_pointer()
                                                            .hover(|s| s.bg(theme.library_table_border))
                                                            .on_click(move |_, _, cx: &mut App| {
                                                                *cx.global_mut::<SelectedPlaylist>() =
                                                                    SelectedPlaylist {
                                                                        id: pl_id.clone(),
                                                                        name: pl_name.clone(),
                                                                        is_smart: pl_is_smart,
                                                                    };
                                                                *cx.global_mut::<LibrarySection>() =
                                                                    if pl_is_smart {
                                                                        LibrarySection::SmartPlaylistDetail
                                                                    } else {
                                                                        LibrarySection::PlaylistDetail
                                                                    };
                                                                si.update(cx, |this, cx| {
                                                                    this.query.clear();
                                                                    cx.notify();
                                                                });
                                                            })
                                                            .child(
                                                                div()
                                                                    .flex()
                                                                    .items_center()
                                                                    .justify_center()
                                                                    .w_8()
                                                                    .h_8()
                                                                    .rounded_md()
                                                                    .bg(theme.library_art_bg)
                                                                    .text_color(theme.player_icons_text)
                                                                    .child(
                                                                        Icon::new(Icons::Playlist)
                                                                            .size_4(),
                                                                    ),
                                                            )
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
                                                                            .child(pl.name.clone()),
                                                                    )
                                                                    .child(
                                                                        div()
                                                                            .text_xs()
                                                                            .text_color(theme.library_header_text)
                                                                            .child(if pl_is_smart {
                                                                                "Smart Playlist"
                                                                            } else {
                                                                                "Playlist"
                                                                            }),
                                                                    ),
                                                            )
                                                    },
                                                ),
                                            )),
                                    )
                                })
                                .when(nd_search_has_results, |this| {
                                    this.child(nd_search_sections(
                                        &nd_search_albums,
                                        &nd_search_artists,
                                        &nd_search_playlists,
                                        &nd_creds_search,
                                        search_input.clone(),
                                        theme,
                                    ))
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
                                    .min_w_0()
                                    .overflow_hidden()
                                    .text_xs()
                                    .font_weight(FontWeight::MEDIUM)
                                    .text_color(theme.library_header_text)
                                    .child("ARTIST"),
                            )
                            .child(
                                div()
                                    .w_40()
                                    .min_w_0()
                                    .overflow_hidden()
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
                                        None,
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
                                |(
                                    idx,
                                    (name, artist, year, _count, album_art, album_id, track_paths),
                                )| {
                                    let name_clone = name.clone();
                                    let name_clone_opts = name.clone();
                                    let art_url = album_art
                                        .filter(|s| !s.is_empty())
                                        .map(|id| format!("{}{id}", covers_base()));
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
                                            cx.set_global(HoveredAlbumIdx(if *hovered {
                                                Some(idx)
                                            } else {
                                                None
                                            }));
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
                                            div().flex_1().flex().justify_center().child(
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
                                                        cx.global::<Controller>().play_album(
                                                            album_id_play.clone(),
                                                            false,
                                                        );
                                                    })
                                                    .child(Icon::new(Icons::Play).size_4()),
                                            ),
                                        )
                                        .child(
                                            // Right half — Options button centered within it
                                            div().flex_1().flex().justify_center().child(
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
                                                        cx.global_mut::<AlbumContextMenuState>()
                                                            .0 = Some(AlbumContextMenu {
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
                                            let img_url =
                                                image.filter(|s| !s.is_empty()).map(|s| {
                                                    if s.starts_with("http") {
                                                        s
                                                    } else {
                                                        format!("{}{s}", covers_base())
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
                    .min_w_0()
                    .min_h_0()
                    .overflow_y_scroll()
                    .child(
                        div()
                            .w_full()
                            .min_w_0()
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
                            // Track rows (with disc headers for multi-disc albums)
                            .child({
                                let has_multiple_discs = album_tracks
                                    .iter()
                                    .any(|(_, _, _, _, _, _, _, disc)| *disc > 1);
                                let mut rows: Vec<AnyElement> = Vec::new();
                                let mut current_disc = 0u32;
                                for (i, (global_idx, path, title, num, duration, track_id, art, disc)) in
                                    album_tracks.into_iter().enumerate()
                                {
                                    if has_multiple_discs && disc != current_disc {
                                        current_disc = disc;
                                        rows.push(
                                            div()
                                                .w_full()
                                                .flex()
                                                .items_center()
                                                .gap_x_3()
                                                .px_6()
                                                .pt_4()
                                                .pb_2()
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .font_weight(FontWeight::MEDIUM)
                                                        .text_color(theme.library_header_text)
                                                        .child(format!("DISC {}", disc)),
                                                )
                                                .into_any_element(),
                                        );
                                    }
                                    let is_current = current_idx == Some(global_idx);
                                    let is_liked = liked_songs.contains(&track_id);
                                    let row_album = selected_album.clone();
                                    let row_artist = album_artist.clone();
                                    rows.push(
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
                                            None,
                                        )
                                        .into_any_element(),
                                    );
                                }
                                div().children(rows)
                            })
                            // Footer: release year and copyright
                            .child({
                                let year = album_meta.year_string.clone();
                                let copyright = album_meta.copyright_message.clone();
                                let show_footer = !year.is_empty() || copyright.is_some();
                                div()
                                    .when(show_footer, |d| {
                                        d.flex()
                                            .flex_col()
                                            .gap_y_1()
                                            .px_6()
                                            .pt_6()
                                            .pb_8()
                                            .when(!year.is_empty(), |d| {
                                                d.child(
                                                    div()
                                                        .text_sm()
                                                        .text_color(theme.library_header_text)
                                                        .child(format_release_date(&year)),
                                                )
                                            })
                                            .when_some(copyright, |d, msg| {
                                                d.child(
                                                    div()
                                                        .text_sm()
                                                        .text_color(theme.library_header_text)
                                                        .child(msg),
                                                )
                                            })
                                    })
                            }),
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
                    .min_w_0()
                    .min_h_0()
                    .overflow_y_scroll()
                    .child(
                        div()
                            .w_full()
                            .min_w_0()
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
                                                            format!("{}{s}", covers_base())
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
                                            .min_w_0()
                                            .overflow_hidden()
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
                                        None,
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
                        .min_w_0()
                        .min_h_0()
                        .overflow_y_scroll()
                        .child(
                            div()
                                .w_full()
                                .min_w_0()
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
                                                                .text_color(
                                                                    theme.library_header_text,
                                                                )
                                                                .child(format!(
                                                                    "{} track{}",
                                                                    n_liked,
                                                                    if n_liked == 1 {
                                                                        ""
                                                                    } else {
                                                                        "s"
                                                                    }
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
                                        .flex_shrink_0()
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
                                                .min_w_0()
                                                .overflow_hidden()
                                                .text_xs()
                                                .font_weight(FontWeight::MEDIUM)
                                                .text_color(theme.library_header_text)
                                                .child("ARTIST"),
                                        )
                                        .child(
                                            div()
                                                .w_40()
                                                .min_w_0()
                                                .overflow_hidden()
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
                                    |(
                                        i,
                                        (
                                            global_idx,
                                            path,
                                            title,
                                            artist,
                                            album,
                                            duration,
                                            track_id,
                                            art,
                                        ),
                                    )| {
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
                                            None,
                                        )
                                    },
                                )),
                        )
                        .into_any_element()
                }

                // ── Files ─────────────────────────────────────────────────────────────
                LibrarySection::Files => self.files_view.clone().into_any_element(),

                // ── Playlists ──────────────────────────────────────────────────────────
                LibrarySection::Playlists => {
                    div()
                        .id("playlists_scroll")
                        .flex_1()
                        .min_h_0()
                        .overflow_y_scroll()
                        .child(
                            div()
                                .w_full()
                                .flex()
                                .flex_col()
                                // ── Header ──────────────────────────────────
                                .child(
                                    div()
                                        .px_6()
                                        .pt_5()
                                        .pb_4()
                                        .flex()
                                        .items_center()
                                        .justify_between()
                                        .child(
                                            div()
                                                .flex()
                                                .items_center()
                                                .gap_x_3()
                                                .child(
                                                    Icon::new(Icons::Playlist)
                                                        .size_8()
                                                        .text_color(theme.library_text),
                                                )
                                                .child(
                                                    div()
                                                        .text_2xl()
                                                        .font_weight(FontWeight(700.0))
                                                        .text_color(theme.library_text)
                                                        .child("Playlists"),
                                                ),
                                        )
                                        .child(
                                            div()
                                                .id("new_playlist_header_btn")
                                                .flex()
                                                .items_center()
                                                .gap_x_1()
                                                .px_3()
                                                .py_1p5()
                                                .rounded_md()
                                                .cursor_pointer()
                                                .bg(theme.player_play_pause_bg)
                                                .text_color(theme.player_play_pause_text)
                                                .hover(|this| {
                                                    this.bg(theme.player_play_pause_hover)
                                                })
                                                .on_click(|_, _, cx: &mut App| {
                                                    cx.global_mut::<CreatePlaylistModal>().open =
                                                        true;
                                                })
                                                .child(Icon::new(Icons::CirclePlus).size_4())
                                                .child(
                                                    div()
                                                        .text_sm()
                                                        .font_weight(FontWeight(600.0))
                                                        .child("New Playlist"),
                                                ),
                                        ),
                                )
                                // ── Saved playlists grid ─────────────────────
                                .when(!saved_playlists.is_empty(), |this| {
                                    this.child(
                                        div()
                                            .px_6()
                                            .pb_2()
                                            .text_xs()
                                            .font_weight(FontWeight(600.0))
                                            .text_color(theme.library_header_text)
                                            .child("MY PLAYLISTS"),
                                    )
                                    .child(
                                        div()
                                            .px_6()
                                            .pb_6()
                                            .grid()
                                            .grid_cols(album_cols)
                                            .gap_4()
                                            .children(
                                                saved_playlists.clone().into_iter().enumerate().map(
                                                    |(i, pl)| {
                                                        let pl_id_click = pl.id.clone();
                                                        let pl_name = pl.name.clone();
                                                        let pl_name_click = pl.name.clone();
                                                        let group_name: gpui::SharedString =
                                                            format!("pl_card_{}", i).into();
                                                        div()
                                                            .id(("saved_pl", i))
                                                            .group(group_name.clone())
                                                            .flex()
                                                            .flex_col()
                                                            .gap_y_2()
                                                            .cursor_pointer()
                                                            .p_2()
                                                            .rounded_lg()
                                                            .hover(|this| {
                                                                this.bg(theme.library_track_bg_hover)
                                                            })
                                                            .on_click(move |_, _, cx: &mut App| {
                                                                *cx.global_mut::<SelectedPlaylist>() =
                                                                    SelectedPlaylist {
                                                                        id: pl_id_click.clone(),
                                                                        name: pl_name_click.clone(),
                                                                        is_smart: false,
                                                                    };
                                                                cx.global_mut::<PlaylistsState>()
                                                                    .playlist_tracks = vec![];
                                                                *cx.global_mut::<LibrarySection>() =
                                                                    LibrarySection::PlaylistDetail;
                                                            })
                                                            .child(
                                                                div()
                                                                    .relative()
                                                                    .child(art_tile(
                                                                        pl.image.clone(),
                                                                        theme,
                                                                        Icons::Playlist,
                                                                        8,
                                                                    ))
                                                                    .child(
                                                                        div()
                                                                            .absolute()
                                                                            .bottom(px(4.0))
                                                                            .right(px(4.0))
                                                                            .flex()
                                                                            .gap_x_1()
                                                                            .opacity(0.0)
                                                                            .group_hover(group_name.clone(), |s| s.opacity(1.0))
                                                                            .child(
                                                                                div()
                                                                                    .id(("pl_edit_btn", i))
                                                                                    .p_1()
                                                                                    .rounded_md()
                                                                                    .bg(theme.titlebar_bg)
                                                                                    .cursor_pointer()
                                                                                    .text_color(theme.library_header_text)
                                                                                    .hover(|s| s.text_color(theme.library_text))
                                                                                    .on_click({
                                                                                        let pl_id = pl.id.clone();
                                                                                        let pl_name2 = pl.name.clone();
                                                                                        let pl_desc = pl.description.clone().unwrap_or_default();
                                                                                        move |_, _, cx: &mut App| {
                                                                                            cx.stop_propagation();
                                                                                            let modal = cx.global_mut::<EditPlaylistModal>();
                                                                                            modal.open = true;
                                                                                            modal.id = pl_id.clone();
                                                                                            modal.name = pl_name2.clone();
                                                                                            modal.description = pl_desc.clone();
                                                                                        }
                                                                                    })
                                                                                    .child(Icon::new(Icons::Pencil).size_4()),
                                                                            )
                                                                            .child(
                                                                                div()
                                                                                    .id(("pl_del_btn", i))
                                                                                    .p_1()
                                                                                    .rounded_md()
                                                                                    .bg(theme.titlebar_bg)
                                                                                    .cursor_pointer()
                                                                                    .text_color(theme.library_header_text)
                                                                                    .hover(|s| s.text_color(gpui::rgb(0xef4444)))
                                                                                    .on_click({
                                                                                        let pl_id = pl.id.clone();
                                                                                        let pl_name2 = pl.name.clone();
                                                                                        move |_, _, cx: &mut App| {
                                                                                            cx.stop_propagation();
                                                                                            let modal = cx.global_mut::<DeletePlaylistModal>();
                                                                                            modal.open = true;
                                                                                            modal.id = pl_id.clone();
                                                                                            modal.name = pl_name2.clone();
                                                                                        }
                                                                                    })
                                                                                    .child(Icon::new(Icons::Trash).size_4()),
                                                                            ),
                                                                    ),
                                                            )
                                                            .child(
                                                                div()
                                                                    .flex()
                                                                    .flex_col()
                                                                    .gap_y_0p5()
                                                                    .child(
                                                                        div()
                                                                            .text_sm()
                                                                            .font_weight(
                                                                                FontWeight(600.0),
                                                                            )
                                                                            .text_color(
                                                                                theme.library_text,
                                                                            )
                                                                            .truncate()
                                                                            .child(pl_name.clone()),
                                                                    )
                                                                    .child(
                                                                        div()
                                                                            .text_xs()
                                                                            .text_color(
                                                                                theme
                                                                                    .library_header_text,
                                                                            )
                                                                            .child(format!(
                                                                                "{} tracks",
                                                                                pl.track_count
                                                                            )),
                                                                    ),
                                                            )
                                                    },
                                                ),
                                            ),
                                    )
                                })
                                // ── Smart playlists ──────────────────────────
                                .when(!smart_playlists.is_empty(), |this| {
                                    this.child(
                                        div()
                                            .px_6()
                                            .pb_2()
                                            .text_xs()
                                            .font_weight(FontWeight(600.0))
                                            .text_color(theme.library_header_text)
                                            .child("SMART PLAYLISTS"),
                                    )
                                    .child(
                                        div()
                                            .px_6()
                                            .pb_6()
                                            .grid()
                                            .grid_cols(album_cols)
                                            .gap_4()
                                            .children(
                                                smart_playlists
                                                    .clone()
                                                    .into_iter()
                                                    .enumerate()
                                                    .map(|(i, pl)| {
                                                        let pl_id_click = pl.id.clone();
                                                        let pl_name = pl.name.clone();
                                                        let pl_name_click = pl.name.clone();
                                                        div()
                                                            .id(("smart_pl", i))
                                                            .flex()
                                                            .flex_col()
                                                            .gap_y_2()
                                                            .cursor_pointer()
                                                            .p_2()
                                                            .rounded_lg()
                                                            .hover(|this| {
                                                                this.bg(theme.library_track_bg_hover)
                                                            })
                                                            .on_click(move |_, _, cx: &mut App| {
                                                                *cx.global_mut::<SelectedPlaylist>() =
                                                                    SelectedPlaylist {
                                                                        id: pl_id_click.clone(),
                                                                        name: pl_name_click.clone(),
                                                                        is_smart: true,
                                                                    };
                                                                cx.global_mut::<PlaylistsState>()
                                                                    .playlist_tracks = vec![];
                                                                *cx.global_mut::<LibrarySection>() =
                                                                    LibrarySection::SmartPlaylistDetail;
                                                            })
                                                            .child(art_tile(
                                                                None,
                                                                theme,
                                                                Icons::MusicList,
                                                                8,
                                                            ))
                                                            .child(
                                                                div()
                                                                    .flex()
                                                                    .flex_col()
                                                                    .gap_y_0p5()
                                                                    .child(
                                                                        div()
                                                                            .text_sm()
                                                                            .font_weight(
                                                                                FontWeight(600.0),
                                                                            )
                                                                            .text_color(
                                                                                theme.library_text,
                                                                            )
                                                                            .truncate()
                                                                            .child(pl_name.clone()),
                                                                    )
                                                                    .child(
                                                                        div()
                                                                            .text_xs()
                                                                            .text_color(
                                                                                theme
                                                                                    .library_header_text,
                                                                            )
                                                                            .child("Smart Playlist"),
                                                                    ),
                                                            )
                                                    }),
                                            ),
                                    )
                                })
                                .when(
                                    saved_playlists.is_empty() && smart_playlists.is_empty(),
                                    |this| {
                                        this.child(
                                            div()
                                                .flex_1()
                                                .flex()
                                                .flex_col()
                                                .items_center()
                                                .justify_center()
                                                .gap_y_3()
                                                .py_16()
                                                .text_color(theme.library_header_text)
                                                .child(
                                                    Icon::new(Icons::Playlist)
                                                        .size_10()
                                                        .text_color(theme.library_header_text),
                                                )
                                                .child(
                                                    div()
                                                        .text_sm()
                                                        .child("No playlists yet"),
                                                )
                                                .child(
                                                    div()
                                                        .id("create_first_playlist_btn")
                                                        .px_4()
                                                        .py_2()
                                                        .rounded_md()
                                                        .cursor_pointer()
                                                        .bg(theme.player_play_pause_bg)
                                                        .text_color(theme.player_play_pause_text)
                                                        .text_sm()
                                                        .hover(|this| {
                                                            this.bg(theme.player_play_pause_hover)
                                                        })
                                                        .on_click(|_, _, cx: &mut App| {
                                                            cx.global_mut::<CreatePlaylistModal>()
                                                                .open = true;
                                                        })
                                                        .child("Create your first playlist"),
                                                ),
                                        )
                                    },
                                ),
                        )
                        .into_any_element()
                }

                // ── PlaylistDetail ─────────────────────────────────────────────────────
                LibrarySection::PlaylistDetail | LibrarySection::SmartPlaylistDetail => {
                    let pl_name = selected_playlist.name.clone();
                    let pl_id_play = selected_playlist.id.clone();
                    let pl_id_shuffled = selected_playlist.id.clone();
                    let pl_is_smart = selected_playlist.is_smart;
                    let selected_playlist_id_for_remove = selected_playlist.id.clone();
                    let n_pl_tracks = playlist_tracks.len();
                    div()
                        .id("playlist_detail_scroll")
                        .flex_1()
                        .min_w_0()
                        .min_h_0()
                        .overflow_y_scroll()
                        .child(
                            div()
                                .w_full()
                                .min_w_0()
                                .flex()
                                .flex_col()
                                // ── Back + Header ─────────────────────────────
                                .child(
                                    div()
                                        .px_6()
                                        .pt_5()
                                        .pb_4()
                                        .flex()
                                        .flex_col()
                                        .gap_y_4()
                                        .child(
                                            div()
                                                .id("pl_back_btn")
                                                .flex()
                                                .items_center()
                                                .gap_x_1()
                                                .cursor_pointer()
                                                .text_xs()
                                                .text_color(theme.library_header_text)
                                                .hover(|this| this.text_color(theme.library_text))
                                                .on_click(|_, _, cx: &mut App| {
                                                    *cx.global_mut::<LibrarySection>() =
                                                        LibrarySection::Playlists;
                                                })
                                                .child(
                                                    Icon::new(Icons::ChevronLeft)
                                                        .size_3()
                                                        .text_color(theme.library_header_text),
                                                )
                                                .child("Playlists"),
                                        )
                                        .child(
                                            div()
                                                .flex()
                                                .items_center()
                                                .gap_x_4()
                                                .child(art_fixed(
                                                    None,
                                                    theme,
                                                    if pl_is_smart {
                                                        Icons::MusicList
                                                    } else {
                                                        Icons::Playlist
                                                    },
                                                    px(120.0),
                                                ))
                                                .child(
                                                    div()
                                                        .flex()
                                                        .flex_col()
                                                        .gap_y_2()
                                                        .child(
                                                            div()
                                                                .text_2xl()
                                                                .font_weight(FontWeight(700.0))
                                                                .text_color(theme.library_text)
                                                                .child(pl_name.clone()),
                                                        )
                                                        .child(
                                                            div()
                                                                .text_sm()
                                                                .text_color(
                                                                    theme.library_header_text,
                                                                )
                                                                .child(format!(
                                                                    "{} track{}",
                                                                    n_pl_tracks,
                                                                    if n_pl_tracks == 1 { "" } else { "s" }
                                                                )),
                                                        )
                                                        .child(
                                                            div()
                                                                .flex()
                                                                .items_center()
                                                                .gap_x_3()
                                                                .child(
                                                                    div()
                                                                        .id("pl_play_btn")
                                                                        .flex()
                                                                        .items_center()
                                                                        .gap_x_2()
                                                                        .px_4()
                                                                        .py_2()
                                                                        .rounded_md()
                                                                        .bg(
                                                                            theme
                                                                                .player_play_pause_bg,
                                                                        )
                                                                        .text_color(
                                                                            theme
                                                                                .player_play_pause_text,
                                                                        )
                                                                        .when(n_pl_tracks == 0, |this| {
                                                                            this.opacity(0.5).cursor_default()
                                                                        })
                                                                        .when(n_pl_tracks > 0, |this| {
                                                                            this.cursor_pointer()
                                                                                .hover(|s| {
                                                                                    s.bg(theme.player_play_pause_hover)
                                                                                })
                                                                                .on_click(
                                                                                    move |_, _, cx: &mut App| {
                                                                                        let rt = cx
                                                                                            .global::<Controller>()
                                                                                            .rt();
                                                                                        let pid = pl_id_play.clone();
                                                                                        if pl_is_smart {
                                                                                            rt.spawn(
                                                                                                crate::client::play_smart_playlist(pid),
                                                                                            );
                                                                                        } else {
                                                                                            rt.spawn(
                                                                                                crate::client::play_saved_playlist(pid),
                                                                                            );
                                                                                        }
                                                                                    },
                                                                                )
                                                                        })
                                                                        .child(
                                                                            Icon::new(Icons::Play)
                                                                                .size_4(),
                                                                        )
                                                                        .child(
                                                                            div()
                                                                                .text_sm()
                                                                                .font_weight(
                                                                                    FontWeight(600.0),
                                                                                )
                                                                                .child("Play"),
                                                                        ),
                                                                )
                                                                .child(
                                                                    div()
                                                                        .id("pl_shuffle_btn")
                                                                        .flex()
                                                                        .items_center()
                                                                        .gap_x_2()
                                                                        .px_4()
                                                                        .py_2()
                                                                        .rounded_md()
                                                                        .bg(theme.player_icons_bg_active)
                                                                        .text_color(theme.library_text)
                                                                        .when(n_pl_tracks == 0, |this| {
                                                                            this.opacity(0.5).cursor_default()
                                                                        })
                                                                        .when(n_pl_tracks > 0, |this| {
                                                                            this.cursor_pointer()
                                                                                .hover(|s| s.bg(theme.player_icons_bg_hover))
                                                                                .on_click({
                                                                                    let pid = pl_id_shuffled.clone();
                                                                                    move |_, _, cx: &mut App| {
                                                                                        let rt = cx.global::<Controller>().rt();
                                                                                        rt.spawn(crate::client::play_saved_playlist_shuffled(pid.clone()));
                                                                                    }
                                                                                })
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
                                                ),
                                        ),
                                )
                                // ── Column headers ──────────────────────────────
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
                                                .min_w_0()
                                                .overflow_hidden()
                                                .text_xs()
                                                .font_weight(FontWeight::MEDIUM)
                                                .text_color(theme.library_header_text)
                                                .child("ARTIST"),
                                        )
                                        .child(
                                            div()
                                                .w_40()
                                                .min_w_0()
                                                .overflow_hidden()
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
                                        .child(div().w(px(28.0)).flex_shrink_0())
                                        .when(!pl_is_smart, |this| {
                                            this.child(div().w(px(28.0)).flex_shrink_0())
                                        }),
                                )
                                // ── Track rows ──────────────────────────────
                                .children(
                                    playlist_tracks
                                        .into_iter()
                                        .enumerate()
                                        .map(|(i, t)| {
                                            let is_current = current_idx
                                                == cx
                                                    .global::<Controller>()
                                                    .state
                                                    .read(cx)
                                                    .tracks
                                                    .iter()
                                                    .position(|tr| tr.path == t.path);
                                            let is_liked = liked_songs.contains(&t.id);
                                            let row_artist = t.artist.clone();
                                            let row_album = t.album.clone();
                                            track_row(
                                                ("pl_detail_row", i),
                                                t.path,
                                                (i + 1).to_string(),
                                                t.title,
                                                Some(t.artist),
                                                Some(t.album),
                                                t.duration,
                                                is_current,
                                                is_liked,
                                                row_artist,
                                                row_album,
                                                t.id,
                                                t.album_art,
                                                if pl_is_smart {
                                                    None
                                                } else {
                                                    Some(selected_playlist_id_for_remove.clone())
                                                },
                                            )
                                        }),
                                ),
                        )
                        .into_any_element()
                }

                // ── Home ───────────────────────────────────────────────────────────────
                LibrarySection::Home => {
                    // Build per-album aggregates from the local track list.
                    // We don't yet have play_count / last_played wired into the
                    // GPUI client (track_stats lives in the backend smart-playlist
                    // store) — TODO: expose via gRPC. For now we proxy:
                    //   • "Recently played" → newest track added (cuid is
                    //     time-ordered, so MAX(id) approximates date_added DESC)
                    //   • "Popular albums"  → most tracks in the album (weak
                    //     stand-in for play count until stats land)
                    //
                    // Album NAME is the key (matches what `SelectedAlbum`
                    // stores in the regular Albums grid). Album art is taken
                    // from the first track that actually has it, since the
                    // first track in iteration order doesn't always carry it.
                    struct Agg {
                        artist: String,
                        art: Option<String>,
                        track_count: usize,
                        max_id: String,
                    }
                    let agg_map: std::collections::HashMap<String, Agg> = {
                        let state = cx.global::<Controller>().state.read(cx);
                        let mut map: std::collections::HashMap<String, Agg> =
                            Default::default();
                        for t in &state.tracks {
                            if t.album.is_empty() {
                                continue;
                            }
                            let display_artist = if t.album_artist.is_empty() {
                                t.artist.clone()
                            } else {
                                t.album_artist.clone()
                            };
                            let entry = map.entry(t.album.clone()).or_insert(Agg {
                                artist: display_artist.clone(),
                                art: None,
                                track_count: 0,
                                max_id: String::new(),
                            });
                            entry.track_count += 1;
                            if entry.art.as_deref().filter(|s| !s.is_empty()).is_none() {
                                if let Some(a) =
                                    t.album_art.clone().filter(|s| !s.is_empty())
                                {
                                    entry.art = Some(a);
                                }
                            }
                            if t.id > entry.max_id {
                                entry.max_id = t.id.clone();
                            }
                        }
                        map
                    };

                    // (album_name, album_artist, album_art)
                    let mut by_recent: Vec<(String, String, Option<String>, String)> =
                        agg_map
                            .iter()
                            .map(|(name, a)| {
                                (
                                    name.clone(),
                                    a.artist.clone(),
                                    a.art.clone(),
                                    a.max_id.clone(),
                                )
                            })
                            .collect();
                    by_recent.sort_by(|a, b| b.3.cmp(&a.3));
                    let recent: Vec<(String, String, Option<String>)> = by_recent
                        .into_iter()
                        .take(20)
                        .map(|(n, a, art, _)| (n, a, art))
                        .collect();

                    let mut by_popular: Vec<(String, String, Option<String>, usize)> =
                        agg_map
                            .iter()
                            .map(|(name, a)| {
                                (
                                    name.clone(),
                                    a.artist.clone(),
                                    a.art.clone(),
                                    a.track_count,
                                )
                            })
                            .collect();
                    by_popular.sort_by(|a, b| b.3.cmp(&a.3).then(a.0.cmp(&b.0)));
                    let popular: Vec<(String, String, Option<String>)> = by_popular
                        .into_iter()
                        .take(20)
                        .map(|(n, a, art, _)| (n, a, art))
                        .collect();

                    // Top artists: aggregate track counts per artist (proxy for
                    // popularity until track stats are wired up); pull image
                    // from `state.artist_images`.
                    let top_artists: Vec<(String, Option<String>)> = {
                        let state = cx.global::<Controller>().state.read(cx);
                        let mut counts: std::collections::HashMap<String, usize> =
                            Default::default();
                        for t in &state.tracks {
                            if t.artist.is_empty() {
                                continue;
                            }
                            *counts.entry(t.artist.clone()).or_default() += 1;
                        }
                        let mut sorted: Vec<(String, usize)> = counts.into_iter().collect();
                        sorted.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
                        sorted
                            .into_iter()
                            .take(20)
                            .map(|(name, _)| {
                                let img = state.artist_images.get(&name).cloned();
                                (name, img)
                            })
                            .collect()
                    };

                    let smart_playlists_home = smart_playlists.clone();
                    let saved_playlists_home = saved_playlists.clone();

                    div()
                        .id("home_scroll")
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
                                .child(
                                    div()
                                        .text_3xl()
                                        .font_weight(FontWeight(800.0))
                                        .text_color(theme.library_text)
                                        .child("Home"),
                                )
                                // Quick picks (top of home)
                                .when(!smart_playlists_home.is_empty(), |this| {
                                    let picks: Vec<_> =
                                        smart_playlists_home.iter().take(6).cloned().collect();
                                    {
                                        // Build a card for one pick — used by both columns below.
                                        let make_card = move |idx: usize,
                                                              p: crate::ui::components::SmartPlaylistItem,
                                                              theme: Theme| {
                                            let pid = p.id.clone();
                                            let pname = p.name.clone();
                                            div()
                                                .id(("quick_pick", idx))
                                                .w_full()
                                                .min_w_0()
                                                .flex()
                                                .items_center()
                                                .gap_x_3()
                                                .p_2()
                                                .rounded_md()
                                                .bg(theme.library_art_bg)
                                                .overflow_hidden()
                                                .cursor_pointer()
                                                .hover(|t| t.bg(theme.library_track_bg_hover))
                                                .on_click(move |_, _, cx: &mut App| {
                                                    *cx.global_mut::<SelectedPlaylist>() =
                                                        SelectedPlaylist {
                                                            id: pid.clone(),
                                                            name: pname.clone(),
                                                            is_smart: true,
                                                        };
                                                    cx.global_mut::<PlaylistsState>()
                                                        .playlist_tracks
                                                        .clear();
                                                    *cx.global_mut::<LibrarySection>() =
                                                        LibrarySection::SmartPlaylistDetail;
                                                })
                                                .child(
                                                    div()
                                                        .w(px(48.0))
                                                        .h(px(48.0))
                                                        .rounded_sm()
                                                        .flex_shrink_0()
                                                        .flex()
                                                        .items_center()
                                                        .justify_center()
                                                        .text_color(theme.player_icons_text)
                                                        .child(Icon::new(Icons::Playlist).size_5()),
                                                )
                                                .child(
                                                    div()
                                                        .flex_1()
                                                        .min_w_0()
                                                        .text_sm()
                                                        .font_weight(FontWeight(600.0))
                                                        .text_color(theme.library_text)
                                                        .truncate()
                                                        .child(p.name.clone()),
                                                )
                                        };

                                        // Partition picks into left (even idx) and right (odd idx)
                                        // columns. flex_1 + min_w_0 on each column guarantees a
                                        // 50/50 split that doesn't fight content width.
                                        let (left, right): (Vec<_>, Vec<_>) = picks
                                            .into_iter()
                                            .enumerate()
                                            .partition(|(idx, _)| idx % 2 == 0);

                                        this.child(
                                            div()
                                                .w_full()
                                                .min_w_0()
                                                .flex()
                                                .gap_x_2()
                                                .child(
                                                    div()
                                                        .flex_1()
                                                        .min_w_0()
                                                        .flex()
                                                        .flex_col()
                                                        .gap_y_2()
                                                        .children(left.into_iter().map(
                                                            |(idx, p)| make_card(idx, p, theme),
                                                        )),
                                                )
                                                .child(
                                                    div()
                                                        .flex_1()
                                                        .min_w_0()
                                                        .flex()
                                                        .flex_col()
                                                        .gap_y_2()
                                                        .children(right.into_iter().map(
                                                            |(idx, p)| make_card(idx, p, theme),
                                                        )),
                                                ),
                                        )
                                    }
                                })
                                // Recently played (album row)
                                .when(!recent.is_empty(), |this| {
                                    this.child(
                                        div()
                                            .flex()
                                            .flex_col()
                                            .gap_y_3()
                                            .child(
                                                div()
                                                    .text_lg()
                                                    .font_weight(FontWeight(700.0))
                                                    .text_color(theme.library_text)
                                                    .child("Recently played"),
                                            )
                                            .child(
                                                div()
                                                    .id("home_recent")
                                                    .w_full()
                                                    .min_w_0()
                                                    .flex()
                                                    .gap_x_4()
                                                    .overflow_x_scroll()
                                                    .track_scroll(&home_recent_scroll)
                                                    .children(recent.into_iter().enumerate().map(
                                                        |(idx, (title, artist, art))| {
                                                            let name_for_click = title.clone();
                                                            div()
                                                                .id(("home_recent_card", idx))
                                                                .w(px(160.0))
                                                                .flex_shrink_0()
                                                                .flex()
                                                                .flex_col()
                                                                .gap_y_2()
                                                                .cursor_pointer()
                                                                .on_click(move |_, _, cx: &mut App| {
                                                                    *cx.global_mut::<SelectedAlbum>() =
                                                                        SelectedAlbum(name_for_click.clone());
                                                                    *cx.global_mut::<BackSection>() =
                                                                        BackSection(LibrarySection::Home);
                                                                    *cx.global_mut::<LibrarySection>() =
                                                                        LibrarySection::AlbumDetail;
                                                                })
                                                                .child(
                                                                    div()
                                                                        .w(px(160.0))
                                                                        .h(px(160.0))
                                                                        .child(art_tile(art, theme, Icons::Disc, 10)),
                                                                )
                                                                .child(
                                                                    div()
                                                                        .text_sm()
                                                                        .font_weight(FontWeight(600.0))
                                                                        .text_color(theme.library_text)
                                                                        .truncate()
                                                                        .child(title),
                                                                )
                                                                .child(
                                                                    div()
                                                                        .text_xs()
                                                                        .text_color(theme.library_header_text)
                                                                        .truncate()
                                                                        .child(artist),
                                                                )
                                                        },
                                                    )),
                                            ),
                                    )
                                })
                                .when(!popular.is_empty(), |this| {
                                    this.child(
                                        div()
                                            .flex()
                                            .flex_col()
                                            .gap_y_3()
                                            .child(
                                                div()
                                                    .text_lg()
                                                    .font_weight(FontWeight(700.0))
                                                    .text_color(theme.library_text)
                                                    .child("Popular albums"),
                                            )
                                            .child(
                                                div()
                                                    .id("home_popular")
                                                    .w_full()
                                                    .min_w_0()
                                                    .flex()
                                                    .gap_x_4()
                                                    .overflow_x_scroll()
                                                    .track_scroll(&home_popular_scroll)
                                                    .children(popular.into_iter().enumerate().map(
                                                        |(idx, (title, artist, art))| {
                                                            let name_for_click = title.clone();
                                                            div()
                                                                .id(("home_popular_card", idx))
                                                                .w(px(160.0))
                                                                .flex_shrink_0()
                                                                .flex()
                                                                .flex_col()
                                                                .gap_y_2()
                                                                .cursor_pointer()
                                                                .on_click(move |_, _, cx: &mut App| {
                                                                    *cx.global_mut::<SelectedAlbum>() =
                                                                        SelectedAlbum(name_for_click.clone());
                                                                    *cx.global_mut::<BackSection>() =
                                                                        BackSection(LibrarySection::Home);
                                                                    *cx.global_mut::<LibrarySection>() =
                                                                        LibrarySection::AlbumDetail;
                                                                })
                                                                .child(
                                                                    div()
                                                                        .w(px(160.0))
                                                                        .h(px(160.0))
                                                                        .child(art_tile(art, theme, Icons::Disc, 10)),
                                                                )
                                                                .child(
                                                                    div()
                                                                        .text_sm()
                                                                        .font_weight(FontWeight(600.0))
                                                                        .text_color(theme.library_text)
                                                                        .truncate()
                                                                        .child(title),
                                                                )
                                                                .child(
                                                                    div()
                                                                        .text_xs()
                                                                        .text_color(theme.library_header_text)
                                                                        .truncate()
                                                                        .child(artist),
                                                                )
                                                        },
                                                    )),
                                            ),
                                    )
                                })
                                .when(!top_artists.is_empty(), |this| {
                                    this.child(
                                        div()
                                            .flex()
                                            .flex_col()
                                            .gap_y_3()
                                            .child(
                                                div()
                                                    .text_lg()
                                                    .font_weight(FontWeight(700.0))
                                                    .text_color(theme.library_text)
                                                    .child("Your top artists"),
                                            )
                                            .child(
                                                div()
                                                    .id("home_top_artists")
                                                    .w_full()
                                                    .min_w_0()
                                                    .flex()
                                                    .gap_x_4()
                                                    .overflow_x_scroll()
                                                    .track_scroll(&home_artists_scroll)
                                                    .children(
                                                        top_artists.into_iter().enumerate().map(
                                                            |(idx, (name, image))| {
                                                                let name_for_click = name.clone();
                                                                // Artist images may already be absolute (http(s)://…),
                                                                // upstream providers return full URLs. `covers_base()`
                                                                // is only correct for relative paths.
                                                                let img_url = image
                                                                    .filter(|s| !s.is_empty())
                                                                    .map(|s| {
                                                                        if s.starts_with("http") {
                                                                            s
                                                                        } else {
                                                                            format!("{}{s}", covers_base())
                                                                        }
                                                                    });
                                                                div()
                                                                    .id(("home_top_artist", idx))
                                                                    .w(px(130.0))
                                                                    .flex_shrink_0()
                                                                    .flex()
                                                                    .flex_col()
                                                                    .items_center()
                                                                    .gap_y_2()
                                                                    .cursor_pointer()
                                                                    .on_click(move |_, _, cx: &mut App| {
                                                                        *cx.global_mut::<SelectedArtist>() =
                                                                            SelectedArtist(name_for_click.clone());
                                                                        *cx.global_mut::<BackSection>() =
                                                                            BackSection(LibrarySection::Home);
                                                                        *cx.global_mut::<LibrarySection>() =
                                                                            LibrarySection::ArtistDetail;
                                                                    })
                                                                    .child(if let Some(url) = img_url {
                                                                        div()
                                                                            .w(px(130.0))
                                                                            .h(px(130.0))
                                                                            .rounded_full()
                                                                            .overflow_hidden()
                                                                            .flex_shrink_0()
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
                                                                            .w(px(130.0))
                                                                            .h(px(130.0))
                                                                            .rounded_full()
                                                                            .flex_shrink_0()
                                                                            .bg(theme.library_art_bg)
                                                                            .flex()
                                                                            .items_center()
                                                                            .justify_center()
                                                                            .text_color(theme.player_icons_text)
                                                                            .child(Icon::new(Icons::Artist).size_10())
                                                                            .into_any_element()
                                                                    })
                                                                    .child(
                                                                        div()
                                                                            .text_sm()
                                                                            .font_weight(FontWeight(600.0))
                                                                            .text_color(theme.library_text)
                                                                            .truncate()
                                                                            .child(name.clone()),
                                                                    )
                                                                    .child(
                                                                        div()
                                                                            .text_xs()
                                                                            .text_color(theme.library_header_text)
                                                                            .child("Artist"),
                                                                    )
                                                            },
                                                        ),
                                                    ),
                                            ),
                                    )
                                })
                                .when(!saved_playlists_home.is_empty(), |this| {
                                    this.child(
                                        div()
                                            .flex()
                                            .flex_col()
                                            .gap_y_3()
                                            .child(
                                                div()
                                                    .text_lg()
                                                    .font_weight(FontWeight(700.0))
                                                    .text_color(theme.library_text)
                                                    .child("Your playlists"),
                                            )
                                            .child(
                                                div()
                                                    .id("home_saved_pl")
                                                    .w_full()
                                                    .min_w_0()
                                                    .flex()
                                                    .gap_x_4()
                                                    .overflow_x_scroll()
                                                    .track_scroll(&home_saved_scroll)
                                                    .children(
                                                        saved_playlists_home.iter().take(20).cloned().enumerate().map(
                                                            |(idx, p)| {
                                                                let pid = p.id.clone();
                                                                let pname = p.name.clone();
                                                                div()
                                                                    .id(("home_saved_card", idx))
                                                                    .w(px(160.0))
                                                                    .flex_shrink_0()
                                                                    .flex()
                                                                    .flex_col()
                                                                    .gap_y_2()
                                                                    .cursor_pointer()
                                                                    .on_click(move |_, _, cx: &mut App| {
                                                                        *cx.global_mut::<SelectedPlaylist>() =
                                                                            SelectedPlaylist {
                                                                                id: pid.clone(),
                                                                                name: pname.clone(),
                                                                                is_smart: false,
                                                                            };
                                                                        cx.global_mut::<PlaylistsState>()
                                                                            .playlist_tracks
                                                                            .clear();
                                                                        *cx.global_mut::<LibrarySection>() =
                                                                            LibrarySection::PlaylistDetail;
                                                                    })
                                                                    .child(
                                                                        div()
                                                                            .w(px(160.0))
                                                                            .h(px(160.0))
                                                                            .child(art_tile(p.image.clone(), theme, Icons::Playlist, 10)),
                                                                    )
                                                                    .child(
                                                                        div()
                                                                            .text_sm()
                                                                            .font_weight(FontWeight(600.0))
                                                                            .text_color(theme.library_text)
                                                                            .truncate()
                                                                            .child(p.name.clone()),
                                                                    )
                                                            },
                                                        ),
                                                    ),
                                            ),
                                    )
                                }),
                        )
                        .into_any_element()
                }

                // ── Genres ─────────────────────────────────────────────────────────────
                LibrarySection::Genres => {
                    div()
                        .id("genres_scroll")
                        .flex_1()
                        .min_h_0()
                        .overflow_y_scroll()
                        .child(
                            div()
                                .w_full()
                                .p_6()
                                .flex()
                                .flex_col()
                                .gap_y_5()
                                .child(
                                    div()
                                        .text_3xl()
                                        .font_weight(FontWeight(800.0))
                                        .text_color(theme.library_text)
                                        .child("Genres"),
                                )
                                .child(
                                    div()
                                        .grid()
                                        .grid_cols(4)
                                        .gap_3()
                                        .children(genres_state.iter().cloned().enumerate().map(|(idx, g)| {
                                            let gid_click = g.id.clone();
                                            let gname_click = g.name.clone();
                                            let tile_color = neon_color_for(if g.id.is_empty() { &g.name } else { &g.id });
                                            div()
                                                .id(("genre_tile", idx))
                                                .relative()
                                                .h(px(120.0))
                                                .rounded_md()
                                                .overflow_hidden()
                                                .bg(tile_color)
                                                .cursor_pointer()
                                                .hover(|this| this.opacity(0.92))
                                                .on_click(move |_, _, cx: &mut App| {
                                                    *cx.global_mut::<SelectedGenre>() =
                                                        SelectedGenre {
                                                            id: gid_click.clone(),
                                                            name: gname_click.clone(),
                                                            tracks: vec![],
                                                            albums: vec![],
                                                            artists: vec![],
                                                        };
                                                    *cx.global_mut::<BackSection>() =
                                                        BackSection(LibrarySection::Genres);
                                                    *cx.global_mut::<LibrarySection>() =
                                                        LibrarySection::GenreDetail;
                                                })
                                                .child(
                                                    div()
                                                        .absolute()
                                                        .top_3()
                                                        .left_3()
                                                        .text_lg()
                                                        .font_weight(FontWeight(700.0))
                                                        .text_color(gpui::rgb(0xFFFFFF))
                                                        .child(g.name.clone()),
                                                )
                                                .child(
                                                    div()
                                                        .absolute()
                                                        .bottom_2()
                                                        .left_3()
                                                        .text_xs()
                                                        .text_color(gpui::rgb(0xFFFFFFB0))
                                                        .child(format!("{} tracks", g.track_count)),
                                                )
                                        })),
                                ),
                        )
                        .into_any_element()
                }

                // ── GenreDetail ────────────────────────────────────────────────────────
                LibrarySection::GenreDetail => {
                    let g = selected_genre.clone();
                    let gid_for_play = g.id.clone();
                    let gid_for_shuffle = g.id.clone();
                    div()
                        .id("genre_detail_scroll")
                        .flex_1()
                        .min_h_0()
                        .overflow_y_scroll()
                        .child(
                            div()
                                .w_full()
                                .p_6()
                                .flex()
                                .flex_col()
                                .gap_y_5()
                                // Back + title
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap_x_3()
                                        .child(
                                            div()
                                                .id("genre_back")
                                                .w(px(28.0))
                                                .h(px(28.0))
                                                .flex()
                                                .items_center()
                                                .justify_center()
                                                .rounded_full()
                                                .bg(theme.library_art_bg)
                                                .cursor_pointer()
                                                .text_color(theme.library_text)
                                                .on_click(|_, _, cx: &mut App| {
                                                    *cx.global_mut::<LibrarySection>() =
                                                        LibrarySection::Genres;
                                                    *cx.global_mut::<SelectedGenre>() =
                                                        SelectedGenre::default();
                                                })
                                                .child(Icon::new(Icons::ChevronLeft).size_5()),
                                        )
                                        .child(
                                            div()
                                                .text_3xl()
                                                .font_weight(FontWeight(800.0))
                                                .text_color(theme.library_text)
                                                .child(g.name.clone()),
                                        ),
                                )
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(theme.library_header_text)
                                        .child(format!(
                                            "{} tracks · {} albums · {} artists",
                                            g.tracks.len(),
                                            g.albums.len(),
                                            g.artists.len()
                                        )),
                                )
                                // Play / Shuffle buttons
                                .child(
                                    div()
                                        .flex()
                                        .gap_x_3()
                                        .child(
                                            div()
                                                .id("genre_play_btn")
                                                .px_4()
                                                .py_2()
                                                .rounded_full()
                                                .bg(theme.switcher_active)
                                                .text_color(gpui::rgb(0xFFFFFF))
                                                .text_sm()
                                                .font_weight(FontWeight(600.0))
                                                .cursor_pointer()
                                                .on_click(move |_, _, cx: &mut App| {
                                                    let id = gid_for_play.clone();
                                                    let rt = cx.global::<Controller>().rt();
                                                    rt.spawn(async move {
                                                        let _ = crate::client::play_genre_tracks(
                                                            id, false,
                                                        )
                                                        .await;
                                                    });
                                                })
                                                .child("Play"),
                                        )
                                        .child(
                                            div()
                                                .id("genre_shuffle_btn")
                                                .px_4()
                                                .py_2()
                                                .rounded_full()
                                                .bg(theme.library_art_bg)
                                                .text_color(theme.library_text)
                                                .text_sm()
                                                .font_weight(FontWeight(600.0))
                                                .cursor_pointer()
                                                .on_click(move |_, _, cx: &mut App| {
                                                    let id = gid_for_shuffle.clone();
                                                    let rt = cx.global::<Controller>().rt();
                                                    rt.spawn(async move {
                                                        let _ = crate::client::play_genre_tracks(
                                                            id, true,
                                                        )
                                                        .await;
                                                    });
                                                })
                                                .child("Shuffle"),
                                        ),
                                )
                                // Albums row
                                .when(!g.albums.is_empty(), |this| {
                                    this.child(
                                        div()
                                            .flex()
                                            .flex_col()
                                            .gap_y_3()
                                            .child(
                                                div()
                                                    .text_lg()
                                                    .font_weight(FontWeight(700.0))
                                                    .text_color(theme.library_text)
                                                    .child("Albums"),
                                            )
                                            .child(
                                                div()
                                                    .id("genre_albums_row")
                                                    .w_full()
                                                    .min_w_0()
                                                    .flex()
                                                    .gap_x_4()
                                                    .overflow_x_scroll()
                                                    .track_scroll(&genre_albums_scroll)
                                                    .children(g.albums.iter().cloned().enumerate().map(|(idx, a)| {
                                                        let aname = a.title.clone();
                                                        div()
                                                            .id(("genre_album_card", idx))
                                                            .w(px(150.0))
                                                            .flex_shrink_0()
                                                            .flex()
                                                            .flex_col()
                                                            .gap_y_2()
                                                            .cursor_pointer()
                                                            .on_click(move |_, _, cx: &mut App| {
                                                                *cx.global_mut::<SelectedAlbum>() =
                                                                    SelectedAlbum(aname.clone());
                                                                *cx.global_mut::<BackSection>() =
                                                                    BackSection(LibrarySection::GenreDetail);
                                                                *cx.global_mut::<LibrarySection>() =
                                                                    LibrarySection::AlbumDetail;
                                                            })
                                                            .child(
                                                                div()
                                                                    .w(px(150.0))
                                                                    .h(px(150.0))
                                                                    .child(art_tile(
                                                                        a.album_art.clone(),
                                                                        theme,
                                                                        Icons::Disc,
                                                                        10,
                                                                    )),
                                                            )
                                                            .child(
                                                                div()
                                                                    .text_sm()
                                                                    .font_weight(FontWeight(600.0))
                                                                    .text_color(theme.library_text)
                                                                    .truncate()
                                                                    .child(a.title.clone()),
                                                            )
                                                            .child(
                                                                div()
                                                                    .text_xs()
                                                                    .text_color(theme.library_header_text)
                                                                    .truncate()
                                                                    .child(a.artist.clone()),
                                                            )
                                                    })),
                                            ),
                                    )
                                })
                                // Artists row
                                .when(!g.artists.is_empty(), |this| {
                                    this.child(
                                        div()
                                            .flex()
                                            .flex_col()
                                            .gap_y_3()
                                            .child(
                                                div()
                                                    .text_lg()
                                                    .font_weight(FontWeight(700.0))
                                                    .text_color(theme.library_text)
                                                    .child("Artists"),
                                            )
                                            .child(
                                                div()
                                                    .id("genre_artists_row")
                                                    .w_full()
                                                    .min_w_0()
                                                    .flex()
                                                    .gap_x_4()
                                                    .overflow_x_scroll()
                                                    .track_scroll(&genre_artists_scroll)
                                                    .children(g.artists.iter().cloned().enumerate().map(|(idx, ar)| {
                                                        let arname = ar.name.clone();
                                                        // Same URL handling as the Artists grid: leave http(s) URLs
                                                        // untouched, only prepend covers_base() for relative paths.
                                                        let img_url = ar
                                                            .image
                                                            .clone()
                                                            .filter(|s| !s.is_empty())
                                                            .map(|s| {
                                                                if s.starts_with("http") {
                                                                    s
                                                                } else {
                                                                    format!("{}{s}", covers_base())
                                                                }
                                                            });
                                                        div()
                                                            .id(("genre_artist_card", idx))
                                                            .w(px(120.0))
                                                            .flex_shrink_0()
                                                            .flex()
                                                            .flex_col()
                                                            .items_center()
                                                            .gap_y_2()
                                                            .cursor_pointer()
                                                            .on_click(move |_, _, cx: &mut App| {
                                                                *cx.global_mut::<SelectedArtist>() =
                                                                    SelectedArtist(arname.clone());
                                                                *cx.global_mut::<BackSection>() =
                                                                    BackSection(LibrarySection::GenreDetail);
                                                                *cx.global_mut::<LibrarySection>() =
                                                                    LibrarySection::ArtistDetail;
                                                            })
                                                            .child(if let Some(url) = img_url {
                                                                div()
                                                                    .w(px(120.0))
                                                                    .h(px(120.0))
                                                                    .rounded_full()
                                                                    .overflow_hidden()
                                                                    .flex_shrink_0()
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
                                                                    .w(px(120.0))
                                                                    .h(px(120.0))
                                                                    .rounded_full()
                                                                    .flex_shrink_0()
                                                                    .bg(theme.library_art_bg)
                                                                    .flex()
                                                                    .items_center()
                                                                    .justify_center()
                                                                    .text_color(theme.player_icons_text)
                                                                    .child(Icon::new(Icons::Artist).size_10())
                                                                    .into_any_element()
                                                            })
                                                            .child(
                                                                div()
                                                                    .text_sm()
                                                                    .font_weight(FontWeight(600.0))
                                                                    .text_color(theme.library_text)
                                                                    .truncate()
                                                                    .child(ar.name.clone()),
                                                            )
                                                    })),
                                            ),
                                    )
                                }),
                        )
                        .into_any_element()
                }

                // ── Navidrome Albums ───────────────────────────────────────
                LibrarySection::NdAlbums => {
                    let nd = nd_state.clone();
                    let nd_creds = nd.active_server().cloned().unwrap_or_default();
                    let albums = nd_data.albums.clone();
                    let loading = nd_data.loading;
                    div()
                        .flex_1()
                        .min_h_0()
                        .flex()
                        .flex_col()
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .px_6()
                                .py_4()
                                .gap_x_3()
                                .child(
                                    Icon::new(Icons::Navidrome)
                                        .size_5()
                                        .text_color(theme.library_text),
                                )
                                .child(
                                    div()
                                        .text_xl()
                                        .font_weight(FontWeight(700.0))
                                        .text_color(theme.library_text)
                                        .child("Albums"),
                                )
                        )
                        .child(
                            div()
                                .id("nd_albums_scroll")
                                .flex_1()
                                .min_h_0()
                                .overflow_y_scroll()
                                .px_6()
                                .pb_6()
                                .when(loading, |t| {
                                    t.child(skeleton_album_grid(album_cols as usize, 3, 0x1E2235))
                                })
                                .when(!loading, |t| t.child(
                                    div()
                                        .w_full()
                                        .p_6()
                                        .grid()
                                        .grid_cols(album_cols)
                                        .gap_6()
                                        .children(albums.into_iter().enumerate().map(|(idx, album)| {
                                            let aid = album.id.clone();
                                            let aname = album.name.clone();
                                            let aartist = album.artist.clone();
                                            let cover_url = album.cover_art.as_ref().map(|cid| {
                                                crate::navidrome::cover_art_url(
                                                    &nd_creds.base_url, &nd_creds.user, &nd_creds.token, &nd_creds.salt,
                                                    cid, Some(300),
                                                )
                                            });
                                            let mut art_box = div()
                                                .id(("nd_album_art", idx))
                                                .w_full()
                                                .rounded_lg()
                                                .overflow_hidden();
                                            art_box.style().aspect_ratio = Some(1.0_f32);
                                            let art_inner: AnyElement = if let Some(url) = cover_url {
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
                                                    .child(Icon::new(Icons::Disc).size_8())
                                                    .into_any_element()
                                            };
                                            div()
                                                .id(("nd_album_card", idx))
                                                .flex()
                                                .flex_col()
                                                .gap_y_2()
                                                .cursor_pointer()
                                                .on_click(move |_, _, cx: &mut App| {
                                                    let sel = cx.global_mut::<NdSelectedAlbum>();
                                                    sel.id = aid.clone();
                                                    sel.name = aname.clone();
                                                    sel.songs = vec![];
                                                    sel.loading = false;
                                                    *cx.global_mut::<LibrarySection>() =
                                                        LibrarySection::NdAlbumDetail;
                                                })
                                                .child(art_box.child(art_inner))
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
                                                                .child(album.name.clone()),
                                                        )
                                                        .child(
                                                            div()
                                                                .text_xs()
                                                                .text_color(theme.library_header_text)
                                                                .truncate()
                                                                .child(aartist),
                                                        ),
                                                )
                                        })),
                                ))
                        )
                        .into_any_element()
                }

                // ── Navidrome Album Detail ─────────────────────────────────
                LibrarySection::NdAlbumDetail => {
                    let nd = nd_state.clone();
                    let nd_creds = nd.active_server().cloned().unwrap_or_default();
                    let sel = nd_sel_album.clone();
                    let cover_url = sel.cover_art.as_ref().map(|cid| {
                        crate::navidrome::cover_art_url(
                            &nd_creds.base_url, &nd_creds.user, &nd_creds.token, &nd_creds.salt, cid, Some(300),
                        )
                    });
                    div()
                        .flex_1()
                        .min_h_0()
                        .flex()
                        .flex_col()
                        .child(
                            // Back header
                            div()
                                .flex()
                                .items_center()
                                .gap_x_2()
                                .px_6()
                                .py_3()
                                .border_b_1()
                                .border_color(theme.library_table_border)
                                .child(
                                    div()
                                        .id("nd_album_back")
                                        .p_1p5()
                                        .rounded_md()
                                        .cursor_pointer()
                                        .text_color(theme.library_text)
                                        .hover(|t| t.bg(theme.library_track_bg_hover))
                                        .on_click(|_, _, cx: &mut App| {
                                            *cx.global_mut::<LibrarySection>() =
                                                LibrarySection::NdAlbums;
                                        })
                                        .child(Icon::new(Icons::ChevronLeft).size_4()),
                                )
                                .child(
                                    div()
                                        .text_sm()
                                        .font_weight(FontWeight(600.0))
                                        .text_color(theme.library_text)
                                        .child(sel.name.clone()),
                                ),
                        )
                        .child(
                            div()
                                .id("nd_album_detail_scroll")
                                .flex_1()
                                .min_h_0()
                                .overflow_y_scroll()
                                .child(
                                    div()
                                        .flex()
                                        .gap_x_6()
                                        .p_6()
                                        .child(if sel.loading {
                                            div()
                                                .flex_shrink_0()
                                                .child(skeleton_rect(
                                                    "nd_album_detail_cover_sk",
                                                    160.0,
                                                    160.0,
                                                    8.0,
                                                    0x1E2235,
                                                ))
                                                .into_any_element()
                                        } else {
                                            div()
                                                .w(px(160.0))
                                                .h(px(160.0))
                                                .flex_shrink_0()
                                                .rounded_lg()
                                                .overflow_hidden()
                                                .bg(theme.library_art_bg)
                                                .flex()
                                                .items_center()
                                                .justify_center()
                                                .when_some(cover_url.clone(), |t, url| {
                                                    t.child(
                                                        img(url)
                                                            .w_full()
                                                            .h_full()
                                                            .object_fit(ObjectFit::Cover),
                                                    )
                                                })
                                                .when(sel.cover_art.is_none(), |t| {
                                                    t.child(
                                                        Icon::new(Icons::Disc)
                                                            .size_10()
                                                            .text_color(theme.player_icons_text),
                                                    )
                                                })
                                                .into_any_element()
                                        })
                                        .child(
                                            div()
                                                .flex()
                                                .flex_col()
                                                .justify_end()
                                                .gap_y_1()
                                                .child(
                                                    div()
                                                        .text_2xl()
                                                        .font_weight(FontWeight(700.0))
                                                        .text_color(theme.library_text)
                                                        .child(sel.name.clone()),
                                                )
                                                .child(
                                                    div()
                                                        .text_sm()
                                                        .text_color(theme.library_header_text)
                                                        .child(sel.artist.clone()),
                                                )
                                                .child(
                                                    div()
                                                        .flex()
                                                        .items_center()
                                                        .gap_x_3()
                                                        .mt_2()
                                                        .child({
                                                            let songs = sel.songs.clone();
                                                            let rt = cx.global::<Controller>().rt();
                                                            let album_art = cover_url.clone();
                                                            div()
                                                                .id("nd_album_play_btn")
                                                                .flex()
                                                                .items_center()
                                                                .gap_x_2()
                                                                .px_4()
                                                                .py_2()
                                                                .rounded_md()
                                                                .cursor_pointer()
                                                                .bg(theme.player_play_pause_bg)
                                                                .text_color(theme.player_play_pause_text)
                                                                .hover(|t| t.bg(theme.player_play_pause_hover))
                                                                .on_click(move |_, _, cx: &mut App| {
                                                                    cx.global_mut::<NdCurrentCoverArt>().0 = album_art.clone();
                                                                    let urls: Vec<String> = songs.iter().map(|s| s.stream_url.clone()).collect();
                                                                    let rt2 = rt.clone();
                                                                    rt2.spawn(async move {
                                                                        if let Some(first_url) = urls.first() {
                                                                            let _ = crate::client::play_track(first_url.clone()).await;
                                                                        }
                                                                        for url in urls.iter().skip(1) {
                                                                            let _ = crate::client::insert_track_last(url.clone()).await;
                                                                        }
                                                                    });
                                                                })
                                                                .child(Icon::new(Icons::Play).size_4())
                                                                .child(div().text_sm().font_weight(FontWeight(600.0)).child("Play"))
                                                        })
                                                        .child({
                                                            let songs = sel.songs.clone();
                                                            let rt = cx.global::<Controller>().rt();
                                                            let album_art = cover_url.clone();
                                                            div()
                                                                .id("nd_album_shuffle_btn")
                                                                .flex()
                                                                .items_center()
                                                                .gap_x_2()
                                                                .px_4()
                                                                .py_2()
                                                                .rounded_md()
                                                                .cursor_pointer()
                                                                .bg(theme.player_icons_bg_active)
                                                                .text_color(theme.library_text)
                                                                .hover(|t| t.bg(theme.player_icons_bg_hover))
                                                                .on_click(move |_, _, cx: &mut App| {
                                                                    cx.global_mut::<NdCurrentCoverArt>().0 = album_art.clone();
                                                                    let mut urls: Vec<String> = songs.iter().map(|s| s.stream_url.clone()).collect();
                                                                    use std::time::{SystemTime, UNIX_EPOCH};
                                                                    let mut x = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.subsec_nanos() as u64).unwrap_or(42) | 1;
                                                                    let n = urls.len();
                                                                    for i in (1..n).rev() {
                                                                        x ^= x << 13; x ^= x >> 7; x ^= x << 17;
                                                                        let j = (x % (i as u64 + 1)) as usize;
                                                                        urls.swap(i, j);
                                                                    }
                                                                    let rt2 = rt.clone();
                                                                    rt2.spawn(async move {
                                                                        if let Some(first_url) = urls.first() {
                                                                            let _ = crate::client::play_track(first_url.clone()).await;
                                                                        }
                                                                        for url in urls.iter().skip(1) {
                                                                            let _ = crate::client::insert_track_last(url.clone()).await;
                                                                        }
                                                                    });
                                                                })
                                                                .child(Icon::new(Icons::Shuffle).size_4())
                                                                .child(div().text_sm().font_weight(FontWeight(500.0)).child("Shuffle"))
                                                        }),
                                                ),
                                        ),
                                )
                                .when(sel.loading, |t| {
                                    t.child(skeleton_track_list(10, 0x1E2235))
                                })
                                .when(!sel.loading, |t| {
                                    let mut sorted_songs = sel.songs.clone();
                                    sorted_songs.sort_by_key(|s| s.track.unwrap_or(u32::MAX));
                                    t.child(
                                        div()
                                            .pb_6()
                                            .flex()
                                            .flex_col()
                                            .children(sorted_songs.into_iter().enumerate().map(|(i, song)| {
                                                nd_song_row(
                                                    ("nd_album_song", i),
                                                    song.id.clone(),
                                                    song.title.clone(),
                                                    song.artist.clone(),
                                                    song.album.clone(),
                                                    song.artist_id.clone(),
                                                    song.album_id.clone(),
                                                    song.stream_url.clone(),
                                                    song.duration,
                                                    song.track,
                                                    song.cover_art.clone(),
                                                )
                                            })),
                                    )
                                }),
                        )
                        .into_any_element()
                }

                // ── Navidrome Artists ──────────────────────────────────────
                LibrarySection::NdArtists => {
                    let nd = nd_state.clone();
                    let nd_creds = nd.active_server().cloned().unwrap_or_default();
                    let artists = nd_data.artists.clone();
                    let artists_loading = nd_data.loading;
                    div()
                        .id("nd_artists_scroll")
                        .flex_1()
                        .min_h_0()
                        .overflow_y_scroll()
                        .when(artists_loading, |t| {
                            t.child(skeleton_artist_grid(artist_cols as usize, 3, 0x1E2235))
                        })
                        .when(!artists_loading, |t| t.child(
                            div()
                                .w_full()
                                .p_6()
                                .grid()
                                .grid_cols(artist_cols)
                                .gap_6()
                                .children(artists.into_iter().enumerate().map(|(idx, artist)| {
                                    let aid = artist.id.clone();
                                    let aname = artist.name.clone();
                                    let cover_url =
                                        artist.cover_art.as_ref().map(|cid| {
                                            crate::navidrome::cover_art_url(
                                                &nd_creds.base_url, &nd_creds.user, &nd_creds.token, &nd_creds.salt,
                                                cid, Some(200),
                                            )
                                        });
                                    div()
                                        .id(("nd_artist_card", idx))
                                        .flex()
                                        .flex_col()
                                        .items_center()
                                        .gap_y_2()
                                        .cursor_pointer()
                                        .hover(|this| this.opacity(0.8))
                                        .on_click(move |_, _, cx: &mut App| {
                                            let sel = cx.global_mut::<NdSelectedArtist>();
                                            sel.id = aid.clone();
                                            sel.name = aname.clone();
                                            sel.albums = vec![];
                                            sel.loading = false;
                                            *cx.global_mut::<LibrarySection>() =
                                                LibrarySection::NdArtistDetail;
                                        })
                                        .child({
                                            let mut container = div()
                                                .w_full()
                                                .rounded_full()
                                                .overflow_hidden()
                                                .flex_shrink_0();
                                            container.style().aspect_ratio = Some(1.0_f32);
                                            if let Some(url) = cover_url {
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
                                                        .child(artist.name.clone()),
                                                )
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .text_color(theme.library_header_text)
                                                        .child(format!("{} albums", artist.album_count)),
                                                ),
                                        )
                                })),
                        ))
                        .into_any_element()
                }

                // ── Navidrome Artist Detail ────────────────────────────────
                LibrarySection::NdArtistDetail => {
                    let nd = nd_state.clone();
                    let nd_creds = nd.active_server().cloned().unwrap_or_default();
                    let sel = nd_sel_artist.clone();
                    let cover_url = sel.cover_art.as_ref().map(|cid| {
                        crate::navidrome::cover_art_url(
                            &nd_creds.base_url, &nd_creds.user, &nd_creds.token, &nd_creds.salt, cid, Some(300),
                        )
                    });
                    div()
                        .flex_1()
                        .min_h_0()
                        .flex()
                        .flex_col()
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_x_2()
                                .px_6()
                                .py_3()
                                .border_b_1()
                                .border_color(theme.library_table_border)
                                .child(
                                    div()
                                        .id("nd_artist_back")
                                        .p_1p5()
                                        .rounded_md()
                                        .cursor_pointer()
                                        .text_color(theme.library_text)
                                        .hover(|t| t.bg(theme.library_track_bg_hover))
                                        .on_click(|_, _, cx: &mut App| {
                                            *cx.global_mut::<LibrarySection>() =
                                                LibrarySection::NdArtists;
                                        })
                                        .child(Icon::new(Icons::ChevronLeft).size_4()),
                                )
                                .child(
                                    div()
                                        .text_sm()
                                        .font_weight(FontWeight(600.0))
                                        .text_color(theme.library_text)
                                        .child(sel.name.clone()),
                                ),
                        )
                        .child(
                            div()
                                .id("nd_artist_detail_scroll")
                                .flex_1()
                                .min_h_0()
                                .overflow_y_scroll()
                                .p_6()
                                .when(sel.loading, |t| t.child(skeleton_album_grid(3, 2, 0x1E2235)))
                                .when(!sel.loading, |t| t.flex().flex_col().gap_y_6()
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap_x_4()
                                        .child(if let Some(url) = cover_url {
                                            div()
                                                .w(px(80.0))
                                                .h(px(80.0))
                                                .flex_shrink_0()
                                                .rounded_full()
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
                                                .w(px(80.0))
                                                .h(px(80.0))
                                                .flex_shrink_0()
                                                .rounded_full()
                                                .bg(theme.library_art_bg)
                                                .flex()
                                                .items_center()
                                                .justify_center()
                                                .text_color(theme.player_icons_text)
                                                .child(Icon::new(Icons::Artist).size_8())
                                                .into_any_element()
                                        })
                                        .child(
                                            div()
                                                .text_2xl()
                                                .font_weight(FontWeight(700.0))
                                                .text_color(theme.library_text)
                                                .child(sel.name.clone()),
                                        ),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .gap_y_3()
                                        .child(
                                            div()
                                                .text_lg()
                                                .font_weight(FontWeight(700.0))
                                                .text_color(theme.library_text)
                                                .child("Albums"),
                                        )
                                        .child(
                                            div()
                                                .flex()
                                                .flex_wrap()
                                                .gap_4()
                                                .children(sel.albums.into_iter().enumerate().map(
                                                    |(idx, album)| {
                                                        let aid = album.id.clone();
                                                        let aname = album.name.clone();
                                                        let cover_url = album.cover_art.as_ref().map(|cid| {
                                                            crate::navidrome::cover_art_url(
                                                                &nd_creds.base_url, &nd_creds.user, &nd_creds.token, &nd_creds.salt,
                                                                cid, Some(300),
                                                            )
                                                        });
                                                        div()
                                                            .id(("nd_artist_album", idx))
                                                            .w(px(140.0))
                                                            .flex()
                                                            .flex_col()
                                                            .gap_y_2()
                                                            .cursor_pointer()
                                                            .on_click(move |_, _, cx: &mut App| {
                                                                let sel2 = cx.global_mut::<NdSelectedAlbum>();
                                                                sel2.id = aid.clone();
                                                                sel2.name = aname.clone();
                                                                sel2.songs = vec![];
                                                                sel2.loading = false;
                                                                *cx.global_mut::<LibrarySection>() =
                                                                    LibrarySection::NdAlbumDetail;
                                                            })
                                                            .child(
                                                                div()
                                                                    .w(px(140.0))
                                                                    .h(px(140.0))
                                                                    .rounded_lg()
                                                                    .overflow_hidden()
                                                                    .bg(theme.library_art_bg)
                                                                    .flex()
                                                                    .items_center()
                                                                    .justify_center()
                                                                    .when_some(cover_url, |t, url| {
                                                                        t.child(
                                                                            img(url)
                                                                                .w_full()
                                                                                .h_full()
                                                                                .object_fit(
                                                                                    ObjectFit::Cover,
                                                                                ),
                                                                        )
                                                                    })
                                                                    .when(album.cover_art.is_none(), |t| {
                                                                        t.child(
                                                                            Icon::new(Icons::Disc)
                                                                                .size_8()
                                                                                .text_color(
                                                                                    theme.player_icons_text,
                                                                                ),
                                                                        )
                                                                    }),
                                                            )
                                                            .child(
                                                                div()
                                                                    .text_sm()
                                                                    .font_weight(FontWeight(600.0))
                                                                    .text_color(theme.library_text)
                                                                    .truncate()
                                                                    .child(album.name.clone()),
                                                            )
                                                            .when_some(album.year, |t, y| {
                                                                t.child(
                                                                    div()
                                                                        .text_xs()
                                                                        .text_color(theme.library_header_text)
                                                                        .child(y.to_string()),
                                                                )
                                                            })
                                                    },
                                                )),
                                        ),
                                ))
                        )
                        .into_any_element()
                }

                // ── Navidrome Genres ───────────────────────────────────────
                LibrarySection::NdGenres => {
                    let genres = nd_data.genres.clone();
                    div()
                        .flex_1()
                        .min_h_0()
                        .flex()
                        .flex_col()
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .px_6()
                                .py_4()
                                .gap_x_3()
                                .child(
                                    Icon::new(Icons::Navidrome)
                                        .size_5()
                                        .text_color(theme.library_text),
                                )
                                .child(
                                    div()
                                        .text_xl()
                                        .font_weight(FontWeight(700.0))
                                        .text_color(theme.library_text)
                                        .child("Genres"),
                                ),
                        )
                        .child(
                            div()
                                .id("nd_genres_grid_scroll")
                                .flex_1()
                                .min_h_0()
                                .overflow_y_scroll()
                                .px_6()
                                .pb_6()
                                .child(
                                    div()
                                        .w_full()
                                        .flex()
                                        .flex_wrap()
                                        .gap_3()
                                        .children(genres.into_iter().enumerate().map(|(idx, genre)| {
                                            let gname = genre.name.clone();
                                            let gname2 = genre.name.clone();
                                            let color = neon_color_for(&genre.name);
                                            div()
                                                .id(("nd_genre_card", idx))
                                                .w(px(160.0))
                                                .h(px(80.0))
                                                .rounded_lg()
                                                .overflow_hidden()
                                                .bg(color)
                                                .flex()
                                                .flex_col()
                                                .justify_end()
                                                .px_3()
                                                .pb_3()
                                                .cursor_pointer()
                                                .hover(|t| t.opacity(0.85))
                                                .on_click(move |_, _, cx: &mut App| {
                                                    let sel = cx.global_mut::<NdSelectedGenre>();
                                                    sel.name = gname.clone();
                                                    sel.songs = vec![];
                                                    sel.loading = false;
                                                    *cx.global_mut::<LibrarySection>() =
                                                        LibrarySection::NdGenreDetail;
                                                })
                                                .child(
                                                    div()
                                                        .text_sm()
                                                        .font_weight(FontWeight(700.0))
                                                        .text_color(gpui::black())
                                                        .truncate()
                                                        .child(gname2),
                                                )
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .text_color(gpui::black())
                                                        .child(format!(
                                                            "{} songs",
                                                            genre.song_count
                                                        )),
                                                )
                                        })),
                                ),
                        )
                        .into_any_element()
                }

                // ── Navidrome Genre Detail ─────────────────────────────────
                LibrarySection::NdGenreDetail => {
                    let sel = nd_sel_genre.clone();
                    div()
                        .flex_1()
                        .min_h_0()
                        .flex()
                        .flex_col()
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_x_2()
                                .px_6()
                                .py_3()
                                .border_b_1()
                                .border_color(theme.library_table_border)
                                .child(
                                    div()
                                        .id("nd_genre_back")
                                        .p_1p5()
                                        .rounded_md()
                                        .cursor_pointer()
                                        .text_color(theme.library_text)
                                        .hover(|t| t.bg(theme.library_track_bg_hover))
                                        .on_click(|_, _, cx: &mut App| {
                                            *cx.global_mut::<LibrarySection>() =
                                                LibrarySection::NdGenres;
                                        })
                                        .child(Icon::new(Icons::ChevronLeft).size_4()),
                                )
                                .child(
                                    div()
                                        .text_sm()
                                        .font_weight(FontWeight(600.0))
                                        .text_color(theme.library_text)
                                        .child(sel.name.clone()),
                                )
                        )
                        .child(
                            div()
                                .id("nd_genre_detail_scroll")
                                .flex_1()
                                .min_h_0()
                                .overflow_y_scroll()
                                .pb_6()
                                .when(sel.loading, |t| t.child(skeleton_track_list(12, 0x1E2235)))
                                .when(!sel.loading, |t| {
                                    let mut sorted = sel.songs.clone();
                                    sorted.sort_by(|a, b| a.title.to_lowercase().cmp(&b.title.to_lowercase()));
                                    t.children(sorted.into_iter().enumerate().map(|(i, song)| {
                                        nd_song_row(
                                            ("nd_genre_song", i),
                                            song.id.clone(),
                                            song.title.clone(),
                                            song.artist.clone(),
                                            song.album.clone(),
                                            song.artist_id.clone(),
                                            song.album_id.clone(),
                                            song.stream_url.clone(),
                                            song.duration,
                                            None,
                                            song.cover_art.clone(),
                                        )
                                    }))
                                })
                        )
                        .into_any_element()
                }

                // ── Navidrome Playlists ────────────────────────────────────
                LibrarySection::NdPlaylists => {
                    let nd = nd_state.clone();
                    let nd_creds = nd.active_server().cloned().unwrap_or_default();
                    let playlists = nd_data.playlists.clone();
                    div()
                        .flex_1()
                        .min_h_0()
                        .flex()
                        .flex_col()
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .px_6()
                                .py_4()
                                .gap_x_3()
                                .child(
                                    Icon::new(Icons::Navidrome)
                                        .size_5()
                                        .text_color(theme.library_text),
                                )
                                .child(
                                    div()
                                        .text_xl()
                                        .font_weight(FontWeight(700.0))
                                        .text_color(theme.library_text)
                                        .child("Playlists"),
                                ),
                        )
                        .child(
                            div()
                                .id("nd_genres_scroll")
                                .flex_1()
                                .min_h_0()
                                .overflow_y_scroll()
                                .px_6()
                                .pb_6()
                                .child(
                                    div()
                                        .w_full()
                                        .flex()
                                        .flex_wrap()
                                        .gap_4()
                                        .children(playlists.into_iter().enumerate().map(|(idx, pl)| {
                                            let pid = pl.id.clone();
                                            let pname = pl.name.clone();
                                            let pname2 = pl.name.clone();
                                            let cover_url = pl.cover_art.as_ref().map(|cid| {
                                                crate::navidrome::cover_art_url(
                                                    &nd_creds.base_url, &nd_creds.user, &nd_creds.token, &nd_creds.salt,
                                                    cid, Some(300),
                                                )
                                            });
                                            div()
                                                .id(("nd_pl_card", idx))
                                                .w(px(160.0))
                                                .flex()
                                                .flex_col()
                                                .gap_y_2()
                                                .cursor_pointer()
                                                .on_click(move |_, _, cx: &mut App| {
                                                    let sel = cx.global_mut::<NdSelectedPlaylist>();
                                                    sel.id = pid.clone();
                                                    sel.name = pname.clone();
                                                    sel.tracks = vec![];
                                                    sel.loading = false;
                                                    *cx.global_mut::<LibrarySection>() =
                                                        LibrarySection::NdPlaylistDetail;
                                                })
                                                .child(
                                                    div()
                                                        .w(px(160.0))
                                                        .h(px(160.0))
                                                        .rounded_lg()
                                                        .overflow_hidden()
                                                        .bg(theme.library_art_bg)
                                                        .flex()
                                                        .items_center()
                                                        .justify_center()
                                                        .when_some(cover_url, |t, url| {
                                                            t.child(
                                                                img(url)
                                                                    .w_full()
                                                                    .h_full()
                                                                    .object_fit(ObjectFit::Cover),
                                                            )
                                                        })
                                                        .when(pl.cover_art.is_none(), |t| {
                                                            t.child(
                                                                Icon::new(Icons::Playlist)
                                                                    .size_10()
                                                                    .text_color(theme.player_icons_text),
                                                            )
                                                        }),
                                                )
                                                .child(
                                                    div()
                                                        .text_sm()
                                                        .font_weight(FontWeight(600.0))
                                                        .text_color(theme.library_text)
                                                        .truncate()
                                                        .child(pname2),
                                                )
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .text_color(theme.library_header_text)
                                                        .child(format!("{} songs", pl.song_count)),
                                                )
                                                .when_some(pl.comment.clone(), |t, c| {
                                                    t.child(
                                                        div()
                                                            .text_xs()
                                                            .text_color(theme.library_header_text)
                                                            .truncate()
                                                            .child(c),
                                                    )
                                                })
                                        })),
                                ),
                        )
                        .into_any_element()
                }

                // ── Navidrome Playlist Detail ──────────────────────────────
                LibrarySection::NdPlaylistDetail => {
                    let sel = nd_sel_playlist.clone();
                    div()
                        .flex_1()
                        .min_h_0()
                        .flex()
                        .flex_col()
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_x_2()
                                .px_6()
                                .py_3()
                                .border_b_1()
                                .border_color(theme.library_table_border)
                                .child(
                                    div()
                                        .id("nd_pl_back")
                                        .p_1p5()
                                        .rounded_md()
                                        .cursor_pointer()
                                        .text_color(theme.library_text)
                                        .hover(|t| t.bg(theme.library_track_bg_hover))
                                        .on_click(|_, _, cx: &mut App| {
                                            *cx.global_mut::<LibrarySection>() =
                                                LibrarySection::NdPlaylists;
                                        })
                                        .child(Icon::new(Icons::ChevronLeft).size_4()),
                                )
                                .child(
                                    div()
                                        .text_sm()
                                        .font_weight(FontWeight(600.0))
                                        .text_color(theme.library_text)
                                        .child(sel.name.clone()),
                                )
                                .child(
                                    div()
                                        .ml_auto()
                                        .child(
                                            div()
                                                .id("nd_pl_play_btn")
                                                .px_4()
                                                .py_1p5()
                                                .rounded_full()
                                                .bg(gpui::rgb(0x6F00FF))
                                                .text_sm()
                                                .text_color(gpui::white())
                                                .cursor_pointer()
                                                .hover(|t| t.bg(gpui::rgb(0x5900DD)))
                                                .on_click({
                                                    let tracks = sel.tracks.clone();
                                                    let rt = cx.global::<Controller>().rt();
                                                    move |_, _, _| {
                                                        let urls: Vec<String> = tracks.iter().map(|s| s.stream_url.clone()).collect();
                                                        let rt2 = rt.clone();
                                                        rt2.spawn(async move {
                                                            if let Some(first_url) = urls.first() {
                                                                let _ = crate::client::play_track(first_url.clone()).await;
                                                            }
                                                            for url in urls.iter().skip(1) {
                                                                let _ = crate::client::insert_track_last(url.clone()).await;
                                                            }
                                                        });
                                                    }
                                                })
                                                .child("Play all"),
                                        ),
                                ),
                        )
                        .child(
                            div()
                                .id("nd_playlist_detail_scroll")
                                .flex_1()
                                .min_h_0()
                                .overflow_y_scroll()
                                .pb_6()
                                .when(sel.loading, |t| t.child(skeleton_track_list(12, 0x1E2235)))
                                .when(!sel.loading, |t| t.children(sel.tracks.iter().enumerate().map(|(i, song)| {
                                    nd_song_row(
                                        ("nd_pl_track", i),
                                        song.id.clone(),
                                        song.title.clone(),
                                        song.artist.clone(),
                                        song.album.clone(),
                                        song.artist_id.clone(),
                                        song.album_id.clone(),
                                        song.stream_url.clone(),
                                        song.duration,
                                        Some((i + 1) as u32),
                                        song.cover_art.clone(),
                                    )
                                }))),
                        )
                        .into_any_element()
                }

                // ── Navidrome Songs ────────────────────────────────────────
                LibrarySection::NdSongs => {
                    let songs = nd_songs_state.songs.clone();
                    let loading = nd_songs_state.loading;
                    div()
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
                            div()
                                .id("nd_songs_scroll")
                                .flex_1()
                                .min_h_0()
                                .overflow_y_scroll()
                                .pb_6()
                                .when(loading, |t| t.child(skeleton_track_list(15, 0x1E2235)))
                                .when(!loading, |t| {
                                    let mut sorted = songs.clone();
                                    sorted.sort_by(|a, b| a.title.to_lowercase().cmp(&b.title.to_lowercase()));
                                    t.children(sorted.into_iter().enumerate().map(|(i, song)| {
                                        nd_song_row(
                                            ("nd_songs_row", i),
                                            song.id.clone(),
                                            song.title.clone(),
                                            song.artist.clone(),
                                            song.album.clone(),
                                            song.artist_id.clone(),
                                            song.album_id.clone(),
                                            song.stream_url.clone(),
                                            song.duration,
                                            None,
                                            song.cover_art.clone(),
                                        )
                                    }))
                                }),
                        )
                        .into_any_element()
                }

                // ── Navidrome Likes ────────────────────────────────────────
                LibrarySection::NdLikes => {
                    let songs = nd_likes_state.songs.clone();
                    let songs_play = songs.clone();
                    let songs_shuffle = songs.clone();
                    let loading = nd_likes_state.loading;
                    let n_liked = songs.len();
                    div()
                        .id("nd_likes_scroll")
                        .flex_1()
                        .min_w_0()
                        .min_h_0()
                        .overflow_y_scroll()
                        .child(
                            div()
                                .w_full()
                                .min_w_0()
                                .flex()
                                .flex_col()
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
                                                        .id("nd_likes_play_btn")
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
                                                        .on_click({
                                                            let rt = cx.global::<Controller>().rt();
                                                            move |_, _, _| {
                                                                let urls: Vec<String> = songs_play.iter().map(|s| s.stream_url.clone()).collect();
                                                                let rt2 = rt.clone();
                                                                rt2.spawn(async move {
                                                                    if let Some(first) = urls.first() {
                                                                        let _ = crate::client::play_track(first.clone()).await;
                                                                    }
                                                                    for url in urls.iter().skip(1) {
                                                                        let _ = crate::client::insert_track_last(url.clone()).await;
                                                                    }
                                                                });
                                                            }
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
                                                        .id("nd_likes_shuffle_btn")
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
                                                        .on_click({
                                                            let rt = cx.global::<Controller>().rt();
                                                            move |_, _, _| {
                                                                let mut urls: Vec<String> = songs_shuffle.iter().map(|s| s.stream_url.clone()).collect();
                                                                let n = urls.len();
                                                                use std::time::{SystemTime, UNIX_EPOCH};
                                                                let mut x = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.subsec_nanos() as u64).unwrap_or(42) | 1;
                                                                for i in (1..n).rev() {
                                                                    x ^= x << 13; x ^= x >> 7; x ^= x << 17;
                                                                    let j = (x % (i as u64 + 1)) as usize;
                                                                    urls.swap(i, j);
                                                                }
                                                                let rt2 = rt.clone();
                                                                rt2.spawn(async move {
                                                                    if let Some(first) = urls.first() {
                                                                        let _ = crate::client::play_track(first.clone()).await;
                                                                    }
                                                                    for url in urls.iter().skip(1) {
                                                                        let _ = crate::client::insert_track_last(url.clone()).await;
                                                                    }
                                                                });
                                                            }
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
                                .child(
                                    div()
                                        .w_full()
                                        .flex_shrink_0()
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
                                .when(loading, |t| t.child(skeleton_track_list(15, 0x1E2235)))
                                .when(!loading, |t| t.children(songs.iter().enumerate().map(|(i, song)| {
                                    nd_song_row(
                                        ("nd_likes_row", i),
                                        song.id.clone(),
                                        song.title.clone(),
                                        song.artist.clone(),
                                        song.album.clone(),
                                        song.artist_id.clone(),
                                        song.album_id.clone(),
                                        song.stream_url.clone(),
                                        song.duration,
                                        None,
                                        song.cover_art.clone(),
                                    )
                                }))),
                        )
                        .into_any_element()
                }
            };
            content_inner
        }; // end if/else search

        let cur_is_local = cur_server.is_localhost();
        let server_is_empty = !server_scanning && discovered_servers.is_empty();
        let servers_for_picker = discovered_servers.clone();
        let cur_host_for_picker = cur_server.host.clone();
        let server_footer = div()
            .w_full()
            .border_t_1()
            .border_color(theme.library_table_border)
            .flex()
            .flex_col()
            .when(picker_open, |this| {
                let servers = servers_for_picker;
                let cur_host = cur_host_for_picker;
                let scanning = server_scanning;
                let is_empty = server_is_empty;
                this.child(
                    div()
                        .id("server-panel")
                        .w_full()
                        .bg(theme.titlebar_bg)
                        .flex()
                        .flex_col()
                        .max_h(px(200.0))
                        .overflow_y_scroll()
                        .child(
                            div()
                                .w_full()
                                .flex()
                                .items_center()
                                .justify_between()
                                .px(px(10.0))
                                .py(px(6.0))
                                .child(
                                    div()
                                        .text_xs()
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(theme.library_header_text)
                                        .child("Servers"),
                                )
                                .child(
                                    div()
                                        .id("server-scan-btn")
                                        .px(px(6.0))
                                        .py(px(2.0))
                                        .rounded_md()
                                        .text_xs()
                                        .cursor_pointer()
                                        .text_color(if scanning {
                                            gpui::rgb(0x6F00FF)
                                        } else {
                                            theme.library_header_text
                                        })
                                        .hover(|s| s.text_color(theme.library_text))
                                        .on_click(cx.listener(|_, _, _, cx| {
                                            if cx.global::<DiscoveredServers>().scanning {
                                                return;
                                            }
                                            cx.global_mut::<DiscoveredServers>().scanning = true;
                                            cx.global_mut::<DiscoveredServers>().servers = vec![];
                                            cx.notify();
                                            cx.spawn(async move |this, cx| {
                                                let found = cx
                                                    .background_executor()
                                                    .spawn(async move {
                                                        crate::server::scan_mdns(
                                                            std::time::Duration::from_secs(3),
                                                        )
                                                    })
                                                    .await;
                                                let _ = this.update(cx, |_, cx| {
                                                    cx.global_mut::<DiscoveredServers>().scanning =
                                                        false;
                                                    cx.global_mut::<DiscoveredServers>().servers =
                                                        found;
                                                    cx.notify();
                                                });
                                            })
                                            .detach();
                                        }))
                                        .child(if scanning { "Scanning…" } else { "Scan" }),
                                ),
                        )
                        .child(
                            div()
                                .id("server-localhost")
                                .w_full()
                                .flex()
                                .items_center()
                                .gap_x_2()
                                .px(px(10.0))
                                .py(px(5.0))
                                .cursor_pointer()
                                .bg(if cur_is_local {
                                    gpui::rgba(0x6F00FF20)
                                } else {
                                    theme.titlebar_bg
                                })
                                .hover(|s| s.bg(theme.library_table_border))
                                .on_click(cx.listener(|_, _, _, cx| {
                                    crate::server::set_server(
                                        crate::server::ServerInfo::localhost(),
                                    );
                                    cx.global_mut::<ServerPickerOpen>().0 = false;
                                    let tokio = cx.global::<crate::state::TokioHandle>().0.clone();
                                    cx.spawn(async move |_, cx| {
                                        let (saved, smart) = cx
                                            .background_executor()
                                            .spawn(async move {
                                                tokio.block_on(async {
                                                    let saved =
                                                        crate::client::fetch_saved_playlists()
                                                            .await
                                                            .unwrap_or_default();
                                                    let smart =
                                                        crate::client::fetch_smart_playlists()
                                                            .await
                                                            .unwrap_or_default();
                                                    (saved, smart)
                                                })
                                            })
                                            .await;
                                        let _ = cx.update(|app: &mut gpui::App| {
                                            let state = app.global_mut::<PlaylistsState>();
                                            state.saved = saved;
                                            state.smart = smart;
                                        });
                                    })
                                    .detach();
                                    cx.notify();
                                }))
                                .child(div().w(px(6.0)).h(px(6.0)).rounded_full().bg(
                                    if cur_is_local {
                                        gpui::rgb(0x39FF14)
                                    } else {
                                        theme.library_header_text
                                    },
                                ))
                                .child(
                                    div()
                                        .flex_1()
                                        .min_w_0()
                                        .truncate()
                                        .text_xs()
                                        .text_color(if cur_is_local {
                                            theme.library_text
                                        } else {
                                            theme.library_header_text
                                        })
                                        .child("localhost"),
                                ),
                        )
                        .children(servers.into_iter().enumerate().map(|(idx, server)| {
                            let is_active = server.host == cur_host;
                            let s = server.clone();
                            let label = server.display_name();
                            div()
                                .id(gpui::SharedString::from(format!("server-row-{idx}")))
                                .w_full()
                                .flex()
                                .items_center()
                                .gap_x_2()
                                .px(px(10.0))
                                .py(px(5.0))
                                .cursor_pointer()
                                .bg(if is_active {
                                    gpui::rgba(0x6F00FF20)
                                } else {
                                    theme.titlebar_bg
                                })
                                .hover(|sv| sv.bg(theme.library_table_border))
                                .on_click(cx.listener(move |_, _, _, cx| {
                                    crate::server::set_server(s.clone());
                                    cx.global_mut::<ServerPickerOpen>().0 = false;
                                    let tokio = cx.global::<crate::state::TokioHandle>().0.clone();
                                    cx.spawn(async move |_, cx| {
                                        let (saved, smart) = cx
                                            .background_executor()
                                            .spawn(async move {
                                                tokio.block_on(async {
                                                    let saved =
                                                        crate::client::fetch_saved_playlists()
                                                            .await
                                                            .unwrap_or_default();
                                                    let smart =
                                                        crate::client::fetch_smart_playlists()
                                                            .await
                                                            .unwrap_or_default();
                                                    (saved, smart)
                                                })
                                            })
                                            .await;
                                        let _ = cx.update(|app: &mut gpui::App| {
                                            let state = app.global_mut::<PlaylistsState>();
                                            state.saved = saved;
                                            state.smart = smart;
                                        });
                                    })
                                    .detach();
                                    cx.notify();
                                }))
                                .child(div().w(px(6.0)).h(px(6.0)).rounded_full().bg(
                                    if is_active {
                                        gpui::rgb(0x39FF14)
                                    } else {
                                        theme.library_header_text
                                    },
                                ))
                                .child(
                                    div()
                                        .flex_1()
                                        .min_w_0()
                                        .truncate()
                                        .text_xs()
                                        .text_color(if is_active {
                                            theme.library_text
                                        } else {
                                            theme.library_header_text
                                        })
                                        .child(label),
                                )
                        }))
                        .when(scanning, |s| {
                            s.child(
                                div()
                                    .w_full()
                                    .px(px(10.0))
                                    .py(px(6.0))
                                    .text_xs()
                                    .text_color(theme.library_header_text)
                                    .child("Scanning network…"),
                            )
                        })
                        .when(is_empty, |s| {
                            s.child(
                                div()
                                    .w_full()
                                    .px(px(10.0))
                                    .py(px(6.0))
                                    .text_xs()
                                    .text_color(theme.library_header_text)
                                    .child("No servers found. Press Scan."),
                            )
                        }),
                )
            })
            .child(
                div()
                    .id("server-picker-toggle")
                    .w_full()
                    .flex()
                    .items_center()
                    .gap_x_2()
                    .px(px(10.0))
                    .py(px(8.0))
                    .cursor_pointer()
                    .hover(|s| s.bg(theme.library_table_border))
                    .on_click(cx.listener(|_, _, _, cx| {
                        let open = !cx.global::<ServerPickerOpen>().0;
                        cx.global_mut::<ServerPickerOpen>().0 = open;
                        cx.notify();
                    }))
                    .child(
                        Icon::new(Icons::Device)
                            .size_3()
                            .text_color(if picker_open {
                                gpui::rgb(0x6F00FF)
                            } else {
                                theme.library_header_text
                            }),
                    )
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .truncate()
                            .text_xs()
                            .text_color(theme.library_header_text)
                            .child(cur_server.display_name()),
                    ),
            );

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
                            .child(div()
                                .id("sidebar_scroll")
                                .flex_1()
                                .min_h_0()
                                .flex()
                                .flex_col()
                                .overflow_y_scroll()
                                .pt_4()
                                .child(self.search_input.clone())
                                .gap_y_1()
                            .child(make_nav_item(
                                Icons::Home,
                                5,
                                "Home",
                                LibrarySection::Home,
                            ))
                            .child(make_nav_item(
                                Icons::Music,
                                5,
                                "Songs",
                                LibrarySection::Songs,
                            ))
                            .child(make_nav_item(
                                Icons::Disc,
                                5,
                                "Albums",
                                LibrarySection::Albums,
                            ))
                            .child(make_nav_item(
                                Icons::Artist,
                                5,
                                "Artists",
                                LibrarySection::Artists,
                            ))
                            .child(make_nav_item(
                                Icons::Genre,
                                5,
                                "Genres",
                                LibrarySection::Genres,
                            ))
                            .child(make_nav_item(
                                Icons::HeartOutline,
                                5,
                                "Likes",
                                LibrarySection::Likes,
                            ))
                            .child(make_nav_item(
                                Icons::HardDrive,
                                4,
                                "Files",
                                LibrarySection::Files,
                            ))
                            // ── Playlists sidebar section ──────────────────
                            .child(
                                div()
                                    .w_full()
                                    .flex()
                                    .flex_col()
                                    .child(
                                        // Section header row: label navigates, chevron toggles collapse
                                        div()
                                            .id("playlists_sidebar_header")
                                            .w_full()
                                            .flex()
                                            .items_center()
                                            .justify_between()
                                            .px_4()
                                            .py_2p5()
                                            .cursor_pointer()
                                            .hover(|this| this.text_color(theme.library_text))
                                            .on_click(move |_, _, cx: &mut App| {
                                                if nd_connected
                                                    && cx
                                                        .global::<NavidromeServerState>()
                                                        .connected()
                                                {
                                                    *cx.global_mut::<LibrarySection>() =
                                                        LibrarySection::NdPlaylists;
                                                } else {
                                                    *cx.global_mut::<LibrarySection>() =
                                                        LibrarySection::Playlists;
                                                    cx.global_mut::<PlaylistsSidebarCollapsed>().0 =
                                                        false;
                                                }
                                            })
                                            .child(
                                                div()
                                                    .flex()
                                                    .items_center()
                                                    .gap_x_2()
                                                    .text_sm()
                                                    .text_color(theme.library_header_text)
                                                    .child(
                                                        // Chevron is its own button — stops propagation
                                                        // so it only toggles collapse without navigating.
                                                        div()
                                                            .id("playlists_collapse_btn")
                                                            .cursor_pointer()
                                                            .on_click(|_, _, cx: &mut App| {
                                                                cx.stop_propagation();
                                                                let c = cx.global_mut::<PlaylistsSidebarCollapsed>();
                                                                c.0 = !c.0;
                                                            })
                                                            .child(
                                                                Icon::new(Icons::ChevronLeft)
                                                                    .size_3()
                                                                    .rotate(if playlists_collapsed {
                                                                        gpui::Radians(
                                                                            -std::f32::consts::PI,
                                                                        )
                                                                    } else {
                                                                        gpui::Radians(
                                                                            -std::f32::consts::FRAC_PI_2,
                                                                        )
                                                                    })
                                                                    .text_color(theme.library_header_text),
                                                            ),
                                                    )
                                                    .child(
                                                        Icon::new(Icons::Playlist)
                                                            .size_4()
                                                            .text_color(theme.library_header_text),
                                                    )
                                                    .child("Playlists"),
                                            )
                                            .child(
                                                div()
                                                    .id("sidebar_new_playlist_btn")
                                                    .cursor_pointer()
                                                    .text_color(theme.library_header_text)
                                                    .hover(|this| {
                                                        this.text_color(theme.library_text)
                                                    })
                                                    .on_click(|_, _, cx: &mut App| {
                                                        cx.stop_propagation();
                                                        cx.global_mut::<CreatePlaylistModal>()
                                                            .open = true;
                                                    })
                                                    .child(Icon::new(Icons::CirclePlus).size_4()),
                                            ),
                                    )
                                    // Playlist items (when not collapsed)
                                    .when(!playlists_collapsed, |this| {
                                        this.children(
                                            saved_playlists
                                                .iter()
                                                .enumerate()
                                                .map(|(i, pl)| {
                                                    let pl_id = pl.id.clone();
                                                    let pl_name = pl.name.clone();
                                                    let pl_name_label = pl.name.clone();
                                                    let is_active = section
                                                        == LibrarySection::PlaylistDetail
                                                        && selected_playlist.id == pl.id;
                                                    div()
                                                        .id(("sidebar_pl", i))
                                                        .w_full()
                                                        .px_4()
                                                        .pl_8()
                                                        .py_2()
                                                        .cursor_pointer()
                                                        .text_xs()
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
                                                            this.border_l_2()
                                                                .border_color(theme.switcher_active)
                                                        })
                                                        .hover(|this| {
                                                            this.text_color(theme.library_text)
                                                        })
                                                        .on_click(move |_, _, cx: &mut App| {
                                                            *cx.global_mut::<SelectedPlaylist>() =
                                                                SelectedPlaylist {
                                                                    id: pl_id.clone(),
                                                                    name: pl_name.clone(),
                                                                    is_smart: false,
                                                                };
                                                            cx.global_mut::<PlaylistsState>()
                                                                .playlist_tracks = vec![];
                                                            *cx.global_mut::<LibrarySection>() =
                                                                LibrarySection::PlaylistDetail;
                                                        })
                                                        .child(
                                                            div()
                                                                .flex()
                                                                .items_center()
                                                                .gap_x_2()
                                                                .child(
                                                                    Icon::new(Icons::Playlist)
                                                                        .size_3(),
                                                                )
                                                                .child(
                                                                    div()
                                                                        .flex_1()
                                                                        .min_w_0()
                                                                        .truncate()
                                                                        .child(pl_name_label),
                                                                ),
                                                        )
                                                })
                                                .collect::<Vec<_>>(),
                                        )
                                        .children(
                                            smart_playlists
                                                .iter()
                                                .enumerate()
                                                .map(|(i, pl)| {
                                                    let pl_id = pl.id.clone();
                                                    let pl_name = pl.name.clone();
                                                    let pl_name_label = pl.name.clone();
                                                    let is_active = section
                                                        == LibrarySection::SmartPlaylistDetail
                                                        && selected_playlist.id == pl.id;
                                                    div()
                                                        .id(("sidebar_smart_pl", i))
                                                        .w_full()
                                                        .px_4()
                                                        .pl_8()
                                                        .py_2()
                                                        .cursor_pointer()
                                                        .text_xs()
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
                                                            this.border_l_2()
                                                                .border_color(theme.switcher_active)
                                                        })
                                                        .hover(|this| {
                                                            this.text_color(theme.library_text)
                                                        })
                                                        .on_click(move |_, _, cx: &mut App| {
                                                            *cx.global_mut::<SelectedPlaylist>() =
                                                                SelectedPlaylist {
                                                                    id: pl_id.clone(),
                                                                    name: pl_name.clone(),
                                                                    is_smart: true,
                                                                };
                                                            cx.global_mut::<PlaylistsState>()
                                                                .playlist_tracks = vec![];
                                                            *cx.global_mut::<LibrarySection>() =
                                                                LibrarySection::SmartPlaylistDetail;
                                                        })
                                                        .child(
                                                            div()
                                                                .flex()
                                                                .items_center()
                                                                .gap_x_2()
                                                                .child(
                                                                    Icon::new(Icons::MusicList)
                                                                        .size_3(),
                                                                )
                                                                .child(
                                                                    div()
                                                                        .flex_1()
                                                                        .min_w_0()
                                                                        .truncate()
                                                                        .child(pl_name_label),
                                                                ),
                                                        )
                                                })
                                                .collect::<Vec<_>>(),
                                        )
                                    }),
                            ),
                            )
                            // ── Navidrome sidebar section ──────────────────
                            .child({
                                let nd = nd_state.clone();
                                let has_servers = !nd.servers.is_empty();
                                div()
                                    .w_full()
                                    .flex()
                                    .flex_col()
                                    // ── Section header ───────────────────────
                                    .child(
                                        div()
                                            .id("nd_sidebar_header")
                                            .w_full()
                                            .flex()
                                            .items_center()
                                            .justify_between()
                                            .px_4()
                                            .py_2p5()
                                            .child(
                                                div()
                                                    .flex()
                                                    .items_center()
                                                    .gap_x_2()
                                                    .text_sm()
                                                    .text_color(theme.library_header_text)
                                                    .when(has_servers, |t| {
                                                        t.child(
                                                            div()
                                                                .id("nd_collapse_btn")
                                                                .cursor_pointer()
                                                                .on_click(|_, _, cx: &mut App| {
                                                                    cx.stop_propagation();
                                                                    let c = cx.global_mut::<NavidromeServerState>();
                                                                    c.sidebar_collapsed = !c.sidebar_collapsed;
                                                                })
                                                                .child(
                                                                    Icon::new(Icons::ChevronLeft)
                                                                        .size_3()
                                                                        .rotate(if nd.sidebar_collapsed {
                                                                            gpui::Radians(-std::f32::consts::PI)
                                                                        } else {
                                                                            gpui::Radians(-std::f32::consts::FRAC_PI_2)
                                                                        })
                                                                        .text_color(theme.library_header_text),
                                                                ),
                                                        )
                                                    })
                                                    .child(
                                                        Icon::new(Icons::Navidrome)
                                                            .size_4()
                                                            .text_color(theme.library_header_text),
                                                    )
                                                    .child("Navidrome"),
                                            )
                                            // "Add server" (+) button always visible
                                            .child(
                                                div()
                                                    .id("nd_add_btn")
                                                    .cursor_pointer()
                                                    .text_color(theme.library_header_text)
                                                    .hover(|t| t.text_color(theme.library_text))
                                                    .on_click(move |_, _, cx: &mut App| {
                                                        cx.stop_propagation();
                                                        cx.global_mut::<NavidromeAddModal>().open = true;
                                                    })
                                                    .child(Icon::new(Icons::CirclePlus).size_4())
                                            ),
                                    )
                                    // ── Saved server list ────────────────────
                                    .when(has_servers && !nd.sidebar_collapsed, |outer| {
                                        outer.children(nd.servers.iter().enumerate().map(|(idx, srv)| {
                                            let srv_id = srv.id.clone();
                                            let srv_id2 = srv.id.clone();
                                            let is_active = nd.active_id.as_deref() == Some(&srv.id);
                                            // derive a short display label from the URL
                                            let label = srv.base_url
                                                .trim_start_matches("https://")
                                                .trim_start_matches("http://")
                                                .to_string();
                                            div()
                                                .id(("nd_srv_row", idx))
                                                .w_full()
                                                .flex()
                                                .items_center()
                                                .justify_between()
                                                .px_4()
                                                .py_1()
                                                .cursor_pointer()
                                                .hover(|t| t.bg(theme.library_track_bg_hover))
                                                .on_click({
                                                    let srv_id3 = srv_id.clone();
                                                    move |_, _, cx: &mut App| {
                                                        let state = cx.global_mut::<NavidromeServerState>();
                                                        if state.active_id.as_deref() != Some(&srv_id3) {
                                                            state.active_id = Some(srv_id3.clone());
                                                            let servers = state.servers.clone();
                                                            let active_id = state.active_id.clone();
                                                            crate::nd_persist::save_servers(
                                                                &servers,
                                                                active_id.as_deref(),
                                                            );
                                                            *cx.global_mut::<NdLibraryData>() = NdLibraryData::default();
                                                            *cx.global_mut::<NdSelectedAlbum>() = NdSelectedAlbum::default();
                                                            *cx.global_mut::<NdSelectedArtist>() = NdSelectedArtist::default();
                                                            *cx.global_mut::<NdSelectedGenre>() = NdSelectedGenre::default();
                                                            *cx.global_mut::<NdSelectedPlaylist>() = NdSelectedPlaylist::default();
                                                            *cx.global_mut::<LibrarySection>() = LibrarySection::NdAlbums;
                                                        }
                                                    }
                                                })
                                                .child(
                                                    div()
                                                        .flex()
                                                        .items_center()
                                                        .gap_x_1p5()
                                                        .flex_1()
                                                        .min_w_0()
                                                        .when(is_active, |t| {
                                                            t.child(
                                                                div()
                                                                    .w_1p5()
                                                                    .h_1p5()
                                                                    .rounded_full()
                                                                    .bg(gpui::rgb(0x6F00FF)),
                                                            )
                                                        })
                                                        .when(!is_active, |t| {
                                                            t.child(div().w_1p5().h_1p5())
                                                        })
                                                        .child(
                                                            div()
                                                                .text_xs()
                                                                .truncate()
                                                                .text_color(if is_active {
                                                                    theme.library_text
                                                                } else {
                                                                    theme.library_header_text
                                                                })
                                                                .child(label),
                                                        ),
                                                )
                                                .child(
                                                    div()
                                                        .id(("nd_srv_remove", idx))
                                                        .cursor_pointer()
                                                        .text_color(theme.library_header_text)
                                                        .hover(|t| t.text_color(gpui::rgb(0xFF4444)))
                                                        .on_click(move |_, _, cx: &mut App| {
                                                            cx.stop_propagation();
                                                            let state = cx.global_mut::<NavidromeServerState>();
                                                            let was_active = state.active_id.as_deref() == Some(&srv_id2);
                                                            state.remove_server(&srv_id2);
                                                            let servers = state.servers.clone();
                                                            let active_id = state.active_id.clone();
                                                            crate::nd_persist::save_servers(
                                                                &servers,
                                                                active_id.as_deref(),
                                                            );
                                                            if was_active {
                                                                *cx.global_mut::<NdLibraryData>() = NdLibraryData::default();
                                                                if cx.global::<NavidromeServerState>().active_id.is_none() {
                                                                    *cx.global_mut::<LibrarySection>() = LibrarySection::Home;
                                                                }
                                                            }
                                                        })
                                                        .child(Icon::new(Icons::WinClose).size_3()),
                                                )
                                        }).collect::<Vec<_>>())
                                    })
                                    // ── Disconnect button (reset to local) ───
                                    .when(nd.connected(), |t| {
                                        t.child(
                                            div()
                                                .id("nd_disconnect_btn")
                                                .w_full()
                                                .px_4()
                                                .pb_3()
                                                .child(
                                                    div()
                                                        .id("nd_disconnect_inner")
                                                        .cursor_pointer()
                                                        .text_xs()
                                                        .px_2()
                                                        .py_1()
                                                        .rounded_md()
                                                        .text_color(theme.library_header_text)
                                                        .hover(|t| t.text_color(gpui::rgb(0xFF4444)))
                                                        .on_click(|_, _, cx: &mut App| {
                                                            let state = cx.global_mut::<NavidromeServerState>();
                                                            state.active_id = None;
                                                            let servers = state.servers.clone();
                                                            crate::nd_persist::save_servers(&servers, None);
                                                            *cx.global_mut::<NdLibraryData>() = NdLibraryData::default();
                                                            *cx.global_mut::<NdSongsState>() = NdSongsState::default();
                                                            *cx.global_mut::<NdLikesState>() = NdLikesState::default();
                                                            *cx.global_mut::<NdStarredIds>() = NdStarredIds::default();
                                                            cx.global_mut::<NdContextMenuState>().0 = None;
                                                            *cx.global_mut::<LibrarySection>() = LibrarySection::Home;
                                                        })
                                                        .child("Use local library"),
                                                ),
                                        )
                                    })
                            })
                            .child(server_footer)
                    )
                    .child(content),
            )
            .child(self.miniplayer.clone())
            .when_some(context_menu, |this, menu| {
                let path_for_next = menu.path.clone();
                let path_for_last = menu.path.clone();
                let menu_artist = menu.artist.clone();
                let menu_album = menu.album.clone();
                let menu_track_id = menu.track_id.clone();
                let menu_track_path = menu.path.clone();
                let header_art_url = menu
                    .album_art
                    .as_deref()
                    .filter(|s| !s.is_empty())
                    .map(|id| format!("{}{id}", covers_base()));
                // header ~64px + 6 items × ~33px + separator + borders
                let menu_w = px(240.0);
                let menu_h = px(264.0);
                let margin = px(8.0);
                let max_x = viewport.width - menu_w - margin;
                let menu_x = if menu.pos.x > max_x {
                    max_x
                } else {
                    menu.pos.x
                };
                let menu_x = if menu_x < margin { margin } else { menu_x };
                // Flip above cursor when the menu would overflow the bottom edge.
                let overflows_bottom = (menu.pos.y + menu_h + margin) > viewport.height;
                let menu_y = if overflows_bottom {
                    menu.pos.y - menu_h
                } else {
                    menu.pos.y
                };
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
                                        .child(
                                            img(url).w_full().h_full().object_fit(ObjectFit::Cover),
                                        )
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
                                .on_hover(|hovered, _window, cx: &mut App| {
                                    if *hovered {
                                        cx.global_mut::<AddToPlaylistMenuState>().0 = None;
                                    }
                                })
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
                                .on_hover(|hovered, _window, cx: &mut App| {
                                    if *hovered {
                                        cx.global_mut::<AddToPlaylistMenuState>().0 = None;
                                    }
                                })
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
                                .on_hover(|hovered, _window, cx: &mut App| {
                                    if *hovered {
                                        cx.global_mut::<AddToPlaylistMenuState>().0 = None;
                                    }
                                })
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
                                .on_hover(|hovered, _window, cx: &mut App| {
                                    if *hovered {
                                        cx.global_mut::<AddToPlaylistMenuState>().0 = None;
                                    }
                                })
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
                        )
                        .child(div().h(px(1.0)).bg(theme.library_table_border).mx_2())
                        .child(
                            div()
                                .id("ctx_add_to_playlist")
                                .px_4()
                                .py_2()
                                .text_sm()
                                .cursor_pointer()
                                .text_color(theme.library_text)
                                .hover(|this| this.bg(theme.library_track_bg_hover))
                                .on_hover(move |hovered, _window, cx: &mut App| {
                                    if *hovered {
                                        // item starts after: header(65) + 4 items(132) + separator(1)
                                        let item_y = menu_y + px(198.0);
                                        cx.global_mut::<AddToPlaylistMenuState>().0 =
                                            Some(crate::ui::components::AddToPlaylistMenu {
                                                anchor_x: menu_x + menu_w,
                                                flip_x: menu_x,
                                                anchor_y: item_y,
                                                track_path: menu_track_path.clone(),
                                                track_id: menu_track_id.clone(),
                                            });
                                    }
                                })
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .justify_between()
                                        .child("Add to Playlist")
                                        .child(Icon::new(Icons::ChevronLeft).size_3().rotate(
                                            gpui::Radians(std::f32::consts::PI),
                                        )),
                                ),
                        ),
                )
            })
            // ── Add-to-playlist submenu ────────────────────────────────────────────
            .when_some(add_to_playlist_menu, |this, atpm| {
                let saved_for_menu = saved_playlists.clone();
                let sub_menu_w = px(220.0);
                let sub_menu_h = px((saved_for_menu.len() as f32 * 33.0 + 50.0).max(80.0));
                let margin = px(8.0);
                // Flyout to the right of the parent menu; flip left when right edge overflows.
                let sub_x = if atpm.anchor_x + sub_menu_w + margin > viewport.width {
                    (atpm.flip_x - sub_menu_w).max(margin)
                } else {
                    atpm.anchor_x
                };
                // Align with the hovered row, clamped to stay within the viewport.
                let sub_y = atpm
                    .anchor_y
                    .min(viewport.height - sub_menu_h - margin)
                    .max(margin);
                this.child(
                    div()
                        .id("atp_backdrop")
                        .absolute()
                        .top_0()
                        .left_0()
                        .size_full()
                        .occlude()
                        .on_click(|_, _, cx: &mut App| {
                            cx.stop_propagation();
                            cx.global_mut::<AddToPlaylistMenuState>().0 = None;
                            cx.global_mut::<LibraryContextMenuState>().0 = None;
                        }),
                )
                .child(
                    div()
                        .absolute()
                        .left(sub_x)
                        .top(sub_y)
                        .w(sub_menu_w)
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
                                .py_2()
                                .border_b_1()
                                .border_color(theme.library_table_border)
                                .text_xs()
                                .font_weight(FontWeight(600.0))
                                .text_color(theme.library_header_text)
                                .child("ADD TO PLAYLIST"),
                        )
                        .child(
                            div()
                                .id("atp_new")
                                .px_4()
                                .py_2()
                                .text_sm()
                                .cursor_pointer()
                                .text_color(theme.library_text)
                                .hover(|this| this.bg(theme.library_track_bg_hover))
                                .on_click({
                                    let pending_id = atpm.track_id.clone();
                                    move |_, _, cx: &mut App| {
                                    cx.global_mut::<AddToPlaylistMenuState>().0 = None;
                                    cx.global_mut::<LibraryContextMenuState>().0 = None;
                                    let modal = cx.global_mut::<CreatePlaylistModal>();
                                    modal.open = true;
                                    modal.pending_track_id = Some(pending_id.clone());
                                }})
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap_x_2()
                                        .child(Icon::new(Icons::CirclePlus).size_4())
                                        .child("New Playlist..."),
                                ),
                        )
                        .children(saved_for_menu.into_iter().enumerate().map(|(i, pl)| {
                            let track_id_for_add = atpm.track_id.clone();
                            let pl_id_for_add = pl.id.clone();
                            div()
                                .id(("atp_item", i))
                                .px_4()
                                .py_2()
                                .text_sm()
                                .cursor_pointer()
                                .text_color(theme.library_text)
                                .hover(|this| this.bg(theme.library_track_bg_hover))
                                .on_click(move |_, _, cx: &mut App| {
                                    let tokio =
                                        cx.global::<crate::state::TokioHandle>().0.clone();
                                    let tid = track_id_for_add.clone();
                                    let pid = pl_id_for_add.clone();
                                    cx.global_mut::<AddToPlaylistMenuState>().0 = None;
                                    cx.global_mut::<LibraryContextMenuState>().0 = None;
                                    cx.spawn(async move |cx| {
                                        let saved = cx
                                            .background_executor()
                                            .spawn(async move {
                                                tokio.block_on(async move {
                                                    let _ = crate::client::add_track_to_playlist(
                                                        pid, tid,
                                                    )
                                                    .await;
                                                    crate::client::fetch_saved_playlists()
                                                        .await
                                                        .ok()
                                                })
                                            })
                                            .await;
                                        if let Some(saved) = saved {
                                            let _ = cx.update(|app: &mut gpui::App| {
                                                app.global_mut::<PlaylistsState>().saved = saved;
                                            });
                                        }
                                    })
                                    .detach();
                                })
                                .child(pl.name.clone())
                        })),
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
                let alb_header_art_url =
                    menu.album_art
                        .as_deref()
                        .filter(|s| !s.is_empty())
                        .map(|s| {
                            if s.starts_with("http") {
                                s.to_string()
                            } else {
                                format!("{}{s}", covers_base())
                            }
                        });
                // header ~64px + 7 items × ~33px + borders
                let menu_w = px(250.0);
                let menu_h = px(296.0);
                let margin = px(8.0);
                let max_x = viewport.width - menu_w - margin;
                let menu_x = if menu.pos.x > max_x {
                    max_x
                } else {
                    menu.pos.x
                };
                let menu_x = if menu_x < margin { margin } else { menu_x };
                // Flip above cursor when the menu would overflow the bottom edge.
                let overflows_bottom = (menu.pos.y + menu_h + margin) > viewport.height;
                let menu_y = if overflows_bottom {
                    menu.pos.y - menu_h
                } else {
                    menu.pos.y
                };
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
                                        .child(
                                            img(url).w_full().h_full().object_fit(ObjectFit::Cover),
                                        )
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
                                    cx.global::<Controller>()
                                        .play_album(album_id_play.clone(), false);
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
                let menu_x = if menu.pos.x > max_x {
                    max_x
                } else {
                    menu.pos.x
                };
                let menu_x = if menu_x < margin { margin } else { menu_x };
                let overflows_bottom = (menu.pos.y + menu_h + margin) > viewport.height;
                let menu_y = if overflows_bottom {
                    menu.pos.y - menu_h
                } else {
                    menu.pos.y
                };
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
                                        Icon::new(if is_dir {
                                            Icons::Directory
                                        } else {
                                            Icons::Music
                                        })
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
                                    rt.spawn(insert_directory(
                                        path_shuffled.clone(),
                                        INSERT_SHUFFLED,
                                    ));
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
            // ── Navidrome Song Context Menu ───────────────────────────────────────
            .when_some(nd_context_menu, |this, menu| {
                let nd_creds_ctx = nd_state.active_server().cloned().unwrap_or_default();
                let header_art_url = menu.cover_art.as_ref().map(|cid| {
                    crate::navidrome::cover_art_url(
                        &nd_creds_ctx.base_url,
                        &nd_creds_ctx.user,
                        &nd_creds_ctx.token,
                        &nd_creds_ctx.salt,
                        cid,
                        Some(200),
                    )
                });
                let menu_w = px(240.0);
                let menu_h = px(240.0);
                let margin = px(8.0);
                let max_x = viewport.width - menu_w - margin;
                let menu_x = if menu.pos.x > max_x { max_x } else { menu.pos.x };
                let menu_x = if menu_x < margin { margin } else { menu_x };
                let overflows_bottom = (menu.pos.y + menu_h + margin) > viewport.height;
                let menu_y = if overflows_bottom {
                    menu.pos.y - menu_h
                } else {
                    menu.pos.y
                };
                let menu_y = if menu_y < margin { margin } else { menu_y };
                let surl_play = menu.stream_url.clone();
                let surl_next = menu.stream_url.clone();
                let surl_last = menu.stream_url.clone();
                let artist_id_nav = menu.artist_id.clone();
                let artist_nav = menu.artist.clone();
                let album_id_nav = menu.album_id.clone();
                let album_nav = menu.album.clone();
                this.child(
                    div()
                        .id("nd_ctx_backdrop")
                        .absolute()
                        .top_0()
                        .left_0()
                        .size_full()
                        .occlude()
                        .on_click(|_, _, cx: &mut App| {
                            cx.stop_propagation();
                            cx.global_mut::<NdContextMenuState>().0 = None;
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
                        // Header
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
                                        .child(
                                            img(url).w_full().h_full().object_fit(ObjectFit::Cover),
                                        )
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
                        // Play Now
                        .child(
                            div()
                                .id("nd_ctx_play_now")
                                .px_4()
                                .py_2()
                                .text_sm()
                                .cursor_pointer()
                                .text_color(theme.library_text)
                                .hover(|this| this.bg(theme.library_track_bg_hover))
                                .on_click(move |_, _, cx: &mut App| {
                                    let rt = cx.global::<Controller>().rt();
                                    let url = surl_play.clone();
                                    rt.spawn(async move {
                                        let _ = crate::client::play_track(url).await;
                                    });
                                    cx.global_mut::<NdContextMenuState>().0 = None;
                                })
                                .child("Play Now"),
                        )
                        // Play Next
                        .child(
                            div()
                                .id("nd_ctx_play_next")
                                .px_4()
                                .py_2()
                                .text_sm()
                                .cursor_pointer()
                                .text_color(theme.library_text)
                                .hover(|this| this.bg(theme.library_track_bg_hover))
                                .on_click(move |_, _, cx: &mut App| {
                                    let rt = cx.global::<Controller>().rt();
                                    let url = surl_next.clone();
                                    rt.spawn(insert_track_next(url));
                                    cx.global_mut::<NdContextMenuState>().0 = None;
                                })
                                .child("Play Next"),
                        )
                        // Play Last
                        .child(
                            div()
                                .id("nd_ctx_play_last")
                                .px_4()
                                .py_2()
                                .text_sm()
                                .cursor_pointer()
                                .text_color(theme.library_text)
                                .hover(|this| this.bg(theme.library_track_bg_hover))
                                .on_click(move |_, _, cx: &mut App| {
                                    let rt = cx.global::<Controller>().rt();
                                    let url = surl_last.clone();
                                    rt.spawn(insert_track_last(url));
                                    cx.global_mut::<NdContextMenuState>().0 = None;
                                })
                                .child("Play Last"),
                        )
                        .child(div().h(px(1.0)).bg(theme.library_table_border).mx_2())
                        // Go to Artist
                        .child(
                            div()
                                .id("nd_ctx_go_artist")
                                .px_4()
                                .py_2()
                                .text_sm()
                                .cursor_pointer()
                                .text_color(theme.library_text)
                                .hover(|this| this.bg(theme.library_track_bg_hover))
                                .on_click(move |_, _, cx: &mut App| {
                                    let sel = cx.global_mut::<NdSelectedArtist>();
                                    sel.id = artist_id_nav.clone();
                                    sel.name = artist_nav.clone();
                                    sel.albums = vec![];
                                    sel.loading = false;
                                    *cx.global_mut::<LibrarySection>() =
                                        LibrarySection::NdArtistDetail;
                                    cx.global_mut::<NdContextMenuState>().0 = None;
                                })
                                .child("Go to Artist"),
                        )
                        // Go to Album
                        .child(
                            div()
                                .id("nd_ctx_go_album")
                                .px_4()
                                .py_2()
                                .text_sm()
                                .cursor_pointer()
                                .text_color(theme.library_text)
                                .hover(|this| this.bg(theme.library_track_bg_hover))
                                .on_click(move |_, _, cx: &mut App| {
                                    let sel = cx.global_mut::<NdSelectedAlbum>();
                                    sel.id = album_id_nav.clone();
                                    sel.name = album_nav.clone();
                                    sel.songs = vec![];
                                    sel.loading = false;
                                    *cx.global_mut::<LibrarySection>() =
                                        LibrarySection::NdAlbumDetail;
                                    cx.global_mut::<NdContextMenuState>().0 = None;
                                })
                                .child("Go to Album"),
                        ),
                )
            })
            // ── Create Playlist Modal ─────────────────────────────────────────────���
            .when(create_modal.open, |this| {
                this.child(
                    // Backdrop
                    div()
                        .id("modal_backdrop")
                        .absolute()
                        .top_0()
                        .left_0()
                        .size_full()
                        .bg(gpui::rgba(0x00000099))
                        .occlude()
                        .on_click(|_, _, cx: &mut App| {
                            cx.global_mut::<CreatePlaylistModal>().open = false;
                        }),
                )
                .child(
                    // Modal card — centred
                    div()
                        .id("create_pl_modal_card")
                        .absolute()
                        .top(viewport.height * 0.25)
                        .left((viewport.width - px(380.0)) / 2.0)
                        .w(px(380.0))
                        .bg(theme.titlebar_bg)
                        .border_1()
                        .border_color(theme.library_table_border)
                        .rounded_lg()
                        .p_6()
                        .flex()
                        .flex_col()
                        .gap_y_4()
                        .on_click(|_, _, cx: &mut App| cx.stop_propagation())
                        // Title
                        .child(
                            div()
                                .text_lg()
                                .font_weight(FontWeight(700.0))
                                .text_color(theme.library_text)
                                .child("New Playlist"),
                        )
                        // Name field
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap_y_1()
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(theme.library_header_text)
                                        .child("Title"),
                                )
                                .child(modal_name_input.clone()),
                        )
                        // Description field
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap_y_1()
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(theme.library_header_text)
                                        .child("Description (optional)"),
                                )
                                .child(modal_desc_input.clone()),
                        )
                        // Buttons
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .justify_end()
                                .gap_x_3()
                                .child(
                                    div()
                                        .id("modal_cancel_btn")
                                        .px_4()
                                        .py_2()
                                        .rounded_md()
                                        .cursor_pointer()
                                        .text_sm()
                                        .text_color(theme.library_header_text)
                                        .hover(|this| this.text_color(theme.library_text))
                                        .on_click(|_, _, cx: &mut App| {
                                            let modal = cx.global_mut::<CreatePlaylistModal>();
                                            modal.open = false;
                                            modal.pending_track_id = None;
                                        })
                                        .child("Cancel"),
                                )
                                .child(
                                    div()
                                        .id("modal_create_btn")
                                        .px_4()
                                        .py_2()
                                        .rounded_md()
                                        .cursor_pointer()
                                        .text_sm()
                                        .font_weight(FontWeight(600.0))
                                        .bg(theme.player_play_pause_bg)
                                        .text_color(theme.player_play_pause_text)
                                        .hover(|this| this.bg(theme.player_play_pause_hover))
                                        .on_click({
                                            let modal_name_input = modal_name_input.clone();
                                            let modal_desc_input = modal_desc_input.clone();
                                            move |_, _, cx: &mut App| {
                                            let name = modal_name_value.clone();
                                            if name.trim().is_empty() {
                                                return;
                                            }
                                            let desc = if modal_desc_value.is_empty() {
                                                None
                                            } else {
                                                Some(modal_desc_value.clone())
                                            };
                                            let modal = cx.global_mut::<CreatePlaylistModal>();
                                            modal.open = false;
                                            let pending_id = modal.pending_track_id.take();
                                            // Clear inputs
                                            modal_name_input.update(cx, |i, cx| {
                                                i.value.clear();
                                                cx.notify();
                                            });
                                            modal_desc_input.update(cx, |i, cx| {
                                                i.value.clear();
                                                cx.notify();
                                            });
                                            let track_ids = pending_id
                                                .map(|id| vec![id])
                                                .unwrap_or_default();
                                            let tokio = cx.global::<crate::state::TokioHandle>().0.clone();
                                            cx.spawn(async move |cx| {
                                                let saved = cx
                                                    .background_executor()
                                                    .spawn(async move {
                                                        tokio.block_on(async move {
                                                            let _ = crate::client::create_saved_playlist(name, desc, track_ids).await;
                                                            crate::client::fetch_saved_playlists().await.ok()
                                                        })
                                                    })
                                                    .await;
                                                if let Some(saved) = saved {
                                                    let _ = cx.update(|app: &mut gpui::App| {
                                                        app.global_mut::<PlaylistsState>().saved = saved;
                                                    });
                                                }
                                            })
                                            .detach();
                                        }})
                                        .child("Create"),
                                ),
                        ),
                )
            })
            // ── Edit Playlist Modal ───────────────────────────────────────────────
            .when(edit_modal.open, |this| {
                let edit_id = edit_modal.id.clone();
                this.child(
                    div()
                        .id("edit_modal_backdrop")
                        .absolute()
                        .top_0()
                        .left_0()
                        .size_full()
                        .bg(gpui::rgba(0x00000099))
                        .occlude()
                        .on_click(|_, _, cx: &mut App| {
                            cx.global_mut::<EditPlaylistModal>().open = false;
                        }),
                )
                .child(
                    div()
                        .id("edit_pl_modal_card")
                        .absolute()
                        .top(viewport.height * 0.25)
                        .left((viewport.width - px(380.0)) / 2.0)
                        .w(px(380.0))
                        .bg(theme.titlebar_bg)
                        .border_1()
                        .border_color(theme.library_table_border)
                        .rounded_lg()
                        .p_6()
                        .flex()
                        .flex_col()
                        .gap_y_4()
                        .on_click(|_, _, cx: &mut App| cx.stop_propagation())
                        .child(
                            div()
                                .text_lg()
                                .font_weight(FontWeight(700.0))
                                .text_color(theme.library_text)
                                .child("Edit Playlist"),
                        )
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap_y_1()
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(theme.library_header_text)
                                        .child("Title"),
                                )
                                .child(edit_name_input.clone()),
                        )
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap_y_1()
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(theme.library_header_text)
                                        .child("Description (optional)"),
                                )
                                .child(edit_desc_input.clone()),
                        )
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .justify_end()
                                .gap_x_3()
                                .child(
                                    div()
                                        .id("edit_modal_cancel_btn")
                                        .px_4()
                                        .py_2()
                                        .rounded_md()
                                        .cursor_pointer()
                                        .text_sm()
                                        .text_color(theme.library_header_text)
                                        .hover(|this| this.text_color(theme.library_text))
                                        .on_click(|_, _, cx: &mut App| {
                                            cx.global_mut::<EditPlaylistModal>().open = false;
                                        })
                                        .child("Cancel"),
                                )
                                .child(
                                    div()
                                        .id("edit_modal_save_btn")
                                        .px_4()
                                        .py_2()
                                        .rounded_md()
                                        .cursor_pointer()
                                        .text_sm()
                                        .font_weight(FontWeight(600.0))
                                        .bg(theme.player_play_pause_bg)
                                        .text_color(theme.player_play_pause_text)
                                        .hover(|this| this.bg(theme.player_play_pause_hover))
                                        .on_click({
                                            let edit_name_input2 = edit_name_input.clone();
                                            let edit_desc_input2 = edit_desc_input.clone();
                                            move |_, _, cx: &mut App| {
                                                let name = edit_name_value.clone();
                                                if name.trim().is_empty() {
                                                    return;
                                                }
                                                let desc = if edit_desc_value.is_empty() {
                                                    None
                                                } else {
                                                    Some(edit_desc_value.clone())
                                                };
                                                let id = edit_id.clone();
                                                cx.global_mut::<EditPlaylistModal>().open = false;
                                                edit_name_input2.update(cx, |i, cx| {
                                                    i.value.clear();
                                                    cx.notify();
                                                });
                                                edit_desc_input2.update(cx, |i, cx| {
                                                    i.value.clear();
                                                    cx.notify();
                                                });
                                                let tokio =
                                                    cx.global::<crate::state::TokioHandle>()
                                                        .0
                                                        .clone();
                                                cx.spawn(async move |cx| {
                                                    let saved = cx
                                                        .background_executor()
                                                        .spawn(async move {
                                                            tokio.block_on(async move {
                                                                let _ = crate::client::update_saved_playlist(id, name, desc).await;
                                                                crate::client::fetch_saved_playlists().await.ok()
                                                            })
                                                        })
                                                        .await;
                                                    if let Some(saved) = saved {
                                                        let _ = cx.update(|app: &mut gpui::App| {
                                                            app.global_mut::<PlaylistsState>()
                                                                .saved = saved;
                                                        });
                                                    }
                                                })
                                                .detach();
                                            }
                                        })
                                        .child("Save"),
                                ),
                        ),
                )
            })
            // ── Delete Playlist Modal ─────────────────────────────────────────────
            .when(delete_modal.open, |this| {
                let del_id = delete_modal.id.clone();
                let del_name = delete_modal.name.clone();
                let del_name_msg = delete_modal.name.clone();
                this.child(
                    div()
                        .id("delete_modal_backdrop")
                        .absolute()
                        .top_0()
                        .left_0()
                        .size_full()
                        .bg(gpui::rgba(0x00000099))
                        .occlude()
                        .on_click(|_, _, cx: &mut App| {
                            cx.global_mut::<DeletePlaylistModal>().open = false;
                        }),
                )
                .child(
                    div()
                        .id("delete_pl_modal_card")
                        .absolute()
                        .top(viewport.height * 0.3)
                        .left((viewport.width - px(360.0)) / 2.0)
                        .w(px(360.0))
                        .bg(theme.titlebar_bg)
                        .border_1()
                        .border_color(theme.library_table_border)
                        .rounded_lg()
                        .p_6()
                        .flex()
                        .flex_col()
                        .gap_y_4()
                        .on_click(|_, _, cx: &mut App| cx.stop_propagation())
                        .child(
                            div()
                                .text_lg()
                                .font_weight(FontWeight(700.0))
                                .text_color(theme.library_text)
                                .child("Delete Playlist"),
                        )
                        .child(
                            div()
                                .text_sm()
                                .text_color(theme.library_header_text)
                                .child(format!(
                                    "{} will be permanently deleted.",
                                    del_name_msg
                                )),
                        )
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .justify_end()
                                .gap_x_3()
                                .child(
                                    div()
                                        .id("delete_modal_cancel_btn")
                                        .px_4()
                                        .py_2()
                                        .rounded_md()
                                        .cursor_pointer()
                                        .text_sm()
                                        .text_color(theme.library_header_text)
                                        .hover(|this| this.text_color(theme.library_text))
                                        .on_click(|_, _, cx: &mut App| {
                                            cx.global_mut::<DeletePlaylistModal>().open = false;
                                        })
                                        .child("Cancel"),
                                )
                                .child(
                                    div()
                                        .id("delete_modal_delete_btn")
                                        .px_4()
                                        .py_2()
                                        .rounded_md()
                                        .cursor_pointer()
                                        .text_sm()
                                        .font_weight(FontWeight(600.0))
                                        .bg(gpui::rgb(0xef4444))
                                        .text_color(gpui::rgb(0xFFFFFF))
                                        .hover(|this| this.bg(gpui::rgb(0xdc2626)))
                                        .on_click(move |_, _, cx: &mut App| {
                                            let id = del_id.clone();
                                            let name = del_name.clone();
                                            cx.global_mut::<DeletePlaylistModal>().open = false;
                                            // If currently viewing this playlist, navigate back
                                            let cur_id =
                                                cx.global::<SelectedPlaylist>().id.clone();
                                            if cur_id == id {
                                                *cx.global_mut::<LibrarySection>() =
                                                    LibrarySection::Playlists;
                                            }
                                            let tokio =
                                                cx.global::<crate::state::TokioHandle>()
                                                    .0
                                                    .clone();
                                            let _ = name;
                                            cx.spawn(async move |cx| {
                                                let saved = cx
                                                    .background_executor()
                                                    .spawn(async move {
                                                        tokio.block_on(async move {
                                                            let _ = crate::client::delete_saved_playlist(id).await;
                                                            crate::client::fetch_saved_playlists().await.ok()
                                                        })
                                                    })
                                                    .await;
                                                if let Some(saved) = saved {
                                                    let _ = cx.update(|app: &mut gpui::App| {
                                                        app.global_mut::<PlaylistsState>()
                                                            .saved = saved;
                                                    });
                                                }
                                            })
                                            .detach();
                                        })
                                        .child("Delete"),
                                ),
                        ),
                )
            })
            // ── Add Navidrome Server Modal ────────────────────────────────────────
            .when(nd_add_modal.open, |this| {
                let nd_url_input = self.nd_url_input.clone();
                let nd_user_input = self.nd_user_input.clone();
                let nd_pass_input = self.nd_pass_input.clone();
                let nd = nd_state.clone();
                this.child(
                    div()
                        .id("nd_modal_backdrop")
                        .absolute()
                        .top_0()
                        .left_0()
                        .size_full()
                        .bg(gpui::rgba(0x00000099))
                        .occlude()
                        .on_click(|_, _, cx: &mut App| {
                            cx.global_mut::<NavidromeAddModal>().open = false;
                            cx.global_mut::<NavidromeServerState>().connect_error = None;
                        }),
                )
                .child(
                    div()
                        .id("nd_modal_card")
                        .absolute()
                        .top(viewport.height * 0.25)
                        .left((viewport.width - px(380.0)) / 2.0)
                        .w(px(380.0))
                        .bg(theme.titlebar_bg)
                        .border_1()
                        .border_color(theme.library_table_border)
                        .rounded_lg()
                        .p_6()
                        .flex()
                        .flex_col()
                        .gap_y_4()
                        .on_click(|_, _, cx: &mut App| cx.stop_propagation())
                        .child(
                            div()
                                .text_lg()
                                .font_weight(FontWeight(700.0))
                                .text_color(theme.library_text)
                                .child("Add Navidrome Server"),
                        )
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap_y_1()
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(theme.library_header_text)
                                        .child("Server URL"),
                                )
                                .child(nd_url_input),
                        )
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap_y_1()
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(theme.library_header_text)
                                        .child("Username"),
                                )
                                .child(nd_user_input),
                        )
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap_y_1()
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(theme.library_header_text)
                                        .child("Password"),
                                )
                                .child(nd_pass_input),
                        )
                        .when_some(nd.connect_error.clone(), |t, err| {
                            t.child(
                                div()
                                    .text_sm()
                                    .text_color(gpui::rgb(0xFF4444))
                                    .child(err),
                            )
                        })
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .justify_end()
                                .gap_x_3()
                                .child(
                                    div()
                                        .id("nd_modal_cancel_btn")
                                        .px_4()
                                        .py_2()
                                        .rounded_md()
                                        .cursor_pointer()
                                        .text_sm()
                                        .text_color(theme.library_header_text)
                                        .hover(|t| t.text_color(theme.library_text))
                                        .on_click(|_, _, cx: &mut App| {
                                            cx.global_mut::<NavidromeAddModal>().open = false;
                                            cx.global_mut::<NavidromeServerState>().connect_error = None;
                                        })
                                        .child("Cancel"),
                                )
                                .child(
                                    div()
                                        .id("nd_modal_connect_btn")
                                        .px_4()
                                        .py_2()
                                        .rounded_md()
                                        .cursor_pointer()
                                        .text_sm()
                                        .font_weight(FontWeight(600.0))
                                        .bg(gpui::rgb(0x6F00FF))
                                        .text_color(gpui::white())
                                        .hover(|t| t.bg(gpui::rgb(0x5900DD)))
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.spawn_nd_connect(cx);
                                        }))
                                        .child(if nd.connecting { "Connecting…" } else { "Add" }),
                                ),
                        ),
                )
            })
    }
}
