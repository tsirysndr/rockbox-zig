use crate::client::{save_repeat, save_shuffle};
use crate::controller::Controller;
use crate::state::{
    format_duration, BluetoothState, DevicesState, PlaybackStatus,
};
use crate::ui::components::bluetooth_picker::fetch_and_update_bluetooth_devices;
use crate::ui::components::device_picker::device_icon;
use crate::ui::components::icons::{Icon, Icons};
use crate::ui::components::seek_bar::SeekBar;
use crate::ui::components::{
    LikedSongs, NavidromeServerState, NdCurrentCoverArt, NdLikesState, NdSelectedAlbum,
    NdSelectedGenre, NdSelectedPlaylist, NdSongsState, NdStarredIds, Page,
};
use crate::ui::global_keybinds::play_pause;
use crate::ui::theme::Theme;
use gpui::prelude::FluentBuilder;
use gpui::{
    div, img, px, App, Context, FontWeight, InteractiveElement, IntoElement, ObjectFit,
    ParentElement, Render, StatefulInteractiveElement, Styled, StyledImage,
    Window,
};

pub struct MiniPlayer;

impl Render for MiniPlayer {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let liked_songs = cx.global::<LikedSongs>().0.clone();

        // Pre-fetch Navidrome data before state.read(cx) borrows cx.
        let nd_current_cover = cx.global::<NdCurrentCoverArt>().0.clone();
        let nd_creds_cover = cx
            .global::<NavidromeServerState>()
            .active_server()
            .map(|s| (s.base_url.clone(), s.user.clone(), s.token.clone(), s.salt.clone()));
        let nd_song_cover_map: std::collections::HashMap<String, String> = {
            let mut map = std::collections::HashMap::new();
            for s in &cx.global::<NdSelectedAlbum>().songs {
                if let Some(cid) = &s.cover_art { map.insert(s.id.clone(), cid.clone()); }
            }
            for s in &cx.global::<NdSongsState>().songs {
                if let Some(cid) = &s.cover_art { map.insert(s.id.clone(), cid.clone()); }
            }
            for s in &cx.global::<NdLikesState>().songs {
                if let Some(cid) = &s.cover_art { map.insert(s.id.clone(), cid.clone()); }
            }
            for s in &cx.global::<NdSelectedPlaylist>().tracks {
                if let Some(cid) = &s.cover_art { map.insert(s.id.clone(), cid.clone()); }
            }
            for s in &cx.global::<NdSelectedGenre>().songs {
                if let Some(cid) = &s.cover_art { map.insert(s.id.clone(), cid.clone()); }
            }
            map
        };
        // Note: NdSelectedAlbum.cover_art is intentionally NOT used as fallback here —
        // it reflects whatever album the user last browsed, not necessarily the playing track's album.
        let nd_starred_ids = cx.global::<NdStarredIds>().0.clone();
        let nd_creds_heart = cx
            .global::<NavidromeServerState>()
            .active_server()
            .map(|s| (s.base_url.clone(), s.user.clone(), s.token.clone(), s.salt.clone()));
        let nd_song_item_map: std::collections::HashMap<String, crate::ui::components::NdSongItem> = {
            let mut map = std::collections::HashMap::new();
            for s in &cx.global::<NdSelectedAlbum>().songs { map.insert(s.id.clone(), s.clone()); }
            for s in &cx.global::<NdSongsState>().songs { map.insert(s.id.clone(), s.clone()); }
            for s in &cx.global::<NdLikesState>().songs { map.insert(s.id.clone(), s.clone()); }
            for s in &cx.global::<NdSelectedPlaylist>().tracks { map.insert(s.id.clone(), s.clone()); }
            for s in &cx.global::<NdSelectedGenre>().songs { map.insert(s.id.clone(), s.clone()); }
            map
        };

        let state = cx.global::<Controller>().state.read(cx);
        let is_playing = state.status == PlaybackStatus::Playing;
        let current_device_icon = cx
            .global::<DevicesState>()
            .devices
            .iter()
            .find(|d| d.is_current_device)
            .map(|d| device_icon(d))
            .unwrap_or(Icons::Speaker);

        let title = state
            .current_track()
            .map(|t| t.title.clone())
            .unwrap_or_else(|| "No track selected".into());
        let artist = state
            .current_track()
            .map(|t| t.artist.clone())
            .unwrap_or_default();

        let duration = state.current_track().map(|t| t.duration).unwrap_or(0);
        let album_art_url = {
            let track = state.current_track();
            let path = track.map(|t| t.path.as_str()).unwrap_or("");
            if path.starts_with("http") {
                nd_current_cover.clone().or_else(|| {
                    let song_id = nd_song_id_from_path(path)?;
                    let cover_id = nd_song_cover_map.get(song_id).cloned()?;
                    let (base, user, token, salt) = nd_creds_cover.as_ref()?;
                    Some(crate::navidrome::cover_art_url(base, user, token, salt, &cover_id, Some(300)))
                })
            } else {
                track
                    .and_then(|t| t.album_art.as_deref())
                    .filter(|s| !s.is_empty())
                    .map(|id| format!("{}{id}", crate::server::get_covers_base()))
            }
        };
        let position = state.position;
        let fill_fraction = if duration > 0 {
            (position as f32 / duration as f32).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let is_shuffling = state.shuffling;
        let is_repeat = state.repeat;
        let current_path = state
            .current_track()
            .map(|t| t.path.clone())
            .unwrap_or_default();
        let track_id = state
            .tracks
            .iter()
            .find(|t| t.path == current_path)
            .map(|t| t.id.clone())
            .unwrap_or_default();
        let is_nd_track = current_path.starts_with("http");
        let nd_song_id = if is_nd_track {
            nd_song_id_from_path(&current_path)
                .map(|s| s.to_string())
                .unwrap_or_default()
        } else {
            String::new()
        };
        let is_liked = if is_nd_track {
            nd_starred_ids.contains(&nd_song_id)
        } else {
            liked_songs.contains(&track_id)
        };
        let bluetooth_available = cx.global::<BluetoothState>().available;

        div()
            .w_full()
            .flex_shrink_0()
            .flex()
            .flex_col()
            .relative()
            .border_t_1()
            .border_color(theme.border)
            .bg(theme.app_bg)
            // Main row: [left: art+info] [center: controls+progress] [right: volume]
            .child(
                div()
                    .w_full()
                    .flex()
                    .items_center()
                    .px_4()
                    .py_3()
                    // Left — album art + track info
                    .child(
                        div()
                            .flex_1()
                            .flex()
                            .items_center()
                            .gap_x_3()
                            .child(if let Some(url) = album_art_url {
                                div()
                                    .w(px(48.0))
                                    .h(px(48.0))
                                    .rounded_lg()
                                    .flex_shrink_0()
                                    .overflow_hidden()
                                    .child(img(url).w_full().h_full().object_fit(ObjectFit::Cover))
                                    .into_any_element()
                            } else {
                                div()
                                    .w(px(48.0))
                                    .h(px(48.0))
                                    .rounded_lg()
                                    .flex_shrink_0()
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .bg(theme.library_art_bg)
                                    .text_color(theme.player_icons_text)
                                    .child(Icon::new(Icons::Music).size_4())
                                    .into_any_element()
                            })
                            .child(
                                div()
                                    .id("mini_info")
                                    .flex_1()
                                    .flex()
                                    .flex_col()
                                    .gap_y_0p5()
                                    .overflow_hidden()
                                    .child(
                                        div()
                                            .flex()
                                            .items_center()
                                            .gap_x_1p5()
                                            .overflow_hidden()
                                            .child(
                                                div()
                                                    .id("mini_title")
                                                    .flex_1()
                                                    .text_sm()
                                                    .font_weight(FontWeight(500.0))
                                                    .text_color(theme.player_title_text)
                                                    .truncate()
                                                    .cursor_pointer()
                                                    .on_click(|_, _, cx: &mut App| {
                                                        *cx.global_mut::<Page>() = Page::Player;
                                                    })
                                                    .child(title),
                                            )
                                            .child(
                                                div()
                                                    .id("mini_heart")
                                                    .flex_shrink_0()
                                                    .p_1()
                                                    .rounded_full()
                                                    .cursor_pointer()
                                                    .text_color(if is_liked {
                                                        gpui::rgb(0xFFFFFF)
                                                    } else {
                                                        theme.player_icons_text
                                                    })
                                                    .hover(|this| {
                                                        this.bg(theme.player_icons_bg_hover)
                                                    })
                                                    .on_click(move |_, _, cx: &mut App| {
                                                        cx.stop_propagation();
                                                        let rt = cx.global::<Controller>().rt();
                                                        if is_nd_track {
                                                            let sid = nd_song_id.clone();
                                                            let starred = &mut cx.global_mut::<NdStarredIds>().0;
                                                            if starred.contains(&sid) {
                                                                starred.remove(&sid);
                                                                cx.global_mut::<NdLikesState>().songs.retain(|s| s.id != sid);
                                                                if let Some((b, u, t, s)) = nd_creds_heart.clone() {
                                                                    rt.spawn(async move {
                                                                        crate::navidrome::unstar_song(&b, &u, &t, &s, &sid).await;
                                                                    });
                                                                }
                                                            } else {
                                                                starred.insert(sid.clone());
                                                                if let Some(item) = nd_song_item_map.get(&sid).cloned() {
                                                                    cx.global_mut::<NdLikesState>().songs.insert(0, item);
                                                                }
                                                                if let Some((b, u, t, s)) = nd_creds_heart.clone() {
                                                                    rt.spawn(async move {
                                                                        crate::navidrome::star_song(&b, &u, &t, &s, &sid).await;
                                                                    });
                                                                }
                                                            }
                                                        } else {
                                                            let liked = &mut cx.global_mut::<LikedSongs>().0;
                                                            if liked.contains(&track_id) {
                                                                liked.remove(&track_id);
                                                                rt.spawn(crate::client::unlike_track(track_id.clone()));
                                                            } else {
                                                                liked.insert(track_id.clone());
                                                                rt.spawn(crate::client::like_track(track_id.clone()));
                                                            }
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
                                            ),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(theme.player_artist_text)
                                            .truncate()
                                            .child(artist),
                                    ),
                            ),
                    )
                    // Center — controls stacked above progress
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .items_center()
                            .gap_y_2()
                            .flex_shrink_0()
                            // Transport controls
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_x_1()
                                    .child(
                                        div()
                                            .id("mini_shuffle")
                                            .p_2()
                                            .rounded_md()
                                            .flex()
                                            .items_center()
                                            .justify_center()
                                            .cursor_pointer()
                                            .text_color(if is_shuffling {
                                                theme.player_icons_text_active
                                            } else {
                                                theme.player_icons_text
                                            })
                                            .when(is_shuffling, |this| {
                                                this.bg(theme.player_icons_bg_active)
                                            })
                                            .hover(|this| {
                                                this.text_color(theme.player_icons_text_hover)
                                                    .bg(theme.player_icons_bg_hover)
                                            })
                                            .on_click(move |_, _, cx: &mut App| {
                                                let (state, rt) = {
                                                    let ctrl = cx.global::<Controller>();
                                                    (ctrl.state.clone(), ctrl.rt())
                                                };
                                                let new_val = !state.read(cx).shuffling;
                                                state.update(cx, |s, cx| {
                                                    s.shuffling = new_val;
                                                    cx.notify();
                                                });
                                                rt.spawn(save_shuffle(new_val));
                                            })
                                            .child(Icon::new(Icons::Shuffle).size_4()),
                                    )
                                    .child(
                                        div()
                                            .id("mini_prev")
                                            .p_2()
                                            .rounded_md()
                                            .flex()
                                            .items_center()
                                            .justify_center()
                                            .cursor_pointer()
                                            .text_color(theme.player_icons_text)
                                            .hover(|this| {
                                                this.text_color(theme.player_icons_text_hover)
                                                    .bg(theme.player_icons_bg_hover)
                                            })
                                            .on_click(|_, _, cx: &mut App| {
                                                cx.global::<Controller>().prev();
                                            })
                                            .child(Icon::new(Icons::Prev).size_4()),
                                    )
                                    .child(
                                        div()
                                            .id("mini_play_pause")
                                            .p_2p5()
                                            .rounded_full()
                                            .flex()
                                            .items_center()
                                            .justify_center()
                                            .cursor_pointer()
                                            .bg(theme.player_play_pause_bg)
                                            .hover(|this| this.bg(theme.player_play_pause_hover))
                                            .text_color(theme.player_play_pause_text)
                                            .on_click(|_, _, cx: &mut App| {
                                                play_pause(cx);
                                            })
                                            .child(if is_playing {
                                                Icon::new(Icons::Pause).size_4()
                                            } else {
                                                Icon::new(Icons::Play).size_4()
                                            }),
                                    )
                                    .child(
                                        div()
                                            .id("mini_next")
                                            .p_2()
                                            .rounded_md()
                                            .flex()
                                            .items_center()
                                            .justify_center()
                                            .cursor_pointer()
                                            .text_color(theme.player_icons_text)
                                            .hover(|this| {
                                                this.text_color(theme.player_icons_text_hover)
                                                    .bg(theme.player_icons_bg_hover)
                                            })
                                            .on_click(|_, _, cx: &mut App| {
                                                cx.global::<Controller>().next();
                                            })
                                            .child(Icon::new(Icons::Next).size_4()),
                                    )
                                    .child(
                                        div()
                                            .id("mini_repeat")
                                            .p_2()
                                            .rounded_md()
                                            .flex()
                                            .items_center()
                                            .justify_center()
                                            .cursor_pointer()
                                            .text_color(if is_repeat {
                                                theme.player_icons_text_active
                                            } else {
                                                theme.player_icons_text
                                            })
                                            .when(is_repeat, |this| {
                                                this.bg(theme.player_icons_bg_active)
                                            })
                                            .hover(|this| {
                                                this.text_color(theme.player_icons_text_hover)
                                                    .bg(theme.player_icons_bg_hover)
                                            })
                                            .on_click(move |_, _, cx: &mut App| {
                                                let (state, rt) = {
                                                    let ctrl = cx.global::<Controller>();
                                                    (ctrl.state.clone(), ctrl.rt())
                                                };
                                                let new_mode =
                                                    if state.read(cx).repeat { 0 } else { 1 };
                                                state.update(cx, |s, cx| {
                                                    s.repeat = new_mode != 0;
                                                    cx.notify();
                                                });
                                                rt.spawn(save_repeat(new_mode));
                                            })
                                            .child(Icon::new(Icons::Repeat).size_4()),
                                    ),
                            )
                            // Progress bar with elapsed / duration
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_x_2()
                                    .w(px(420.0))
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(theme.playback_position_text)
                                            .flex_shrink_0()
                                            .child(format_duration(position)),
                                    )
                                    .child(
                                        SeekBar::new(
                                            "miniplayer-seek",
                                            fill_fraction,
                                            theme.playback_slider_track,
                                            theme.playback_slider_fill,
                                            px(3.0),
                                        )
                                        .on_seek(
                                            move |frac, _window, cx: &mut App| {
                                                let seek_secs = (frac * duration as f32) as u64;
                                                cx.global::<Controller>().seek(seek_secs, duration);
                                            },
                                        ),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(theme.playback_position_text)
                                            .flex_shrink_0()
                                            .child(format_duration(duration)),
                                    ),
                            ),
                    )
                    // Right — volume control + device picker trigger
                    .child(
                        div()
                            .flex_1()
                            .flex()
                            .items_center()
                            .justify_end()
                            .gap_x_2()
                            .when(bluetooth_available, |this| {
                                this.child(
                                    div()
                                        .id("mini_bluetooth")
                                        .p_1p5()
                                        .rounded_md()
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .cursor_pointer()
                                        .text_color(theme.player_icons_text)
                                        .hover(|this| {
                                            this.bg(theme.player_icons_bg_hover)
                                                .text_color(theme.player_icons_text_hover)
                                        })
                                        .on_click(move |_, _, cx: &mut App| {
                                            fetch_and_update_bluetooth_devices(cx);
                                            let mut state = cx.global::<BluetoothState>().clone();
                                            state.picker_open = !state.picker_open;
                                            cx.set_global(state);
                                        })
                                        .child(Icon::new(Icons::Bluetooth).size_4()),
                                )
                            })
                            .child(
                                div()
                                    .id("mini_device")
                                    .p_1p5()
                                    .rounded_md()
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .cursor_pointer()
                                    .text_color(theme.player_icons_text)
                                    .hover(|this| {
                                        this.text_color(theme.player_icons_text_hover)
                                            .bg(theme.player_icons_bg_hover)
                                    })
                                    .on_click(move |_, _, cx: &mut App| {
                                        let mut state = cx.global::<DevicesState>().clone();
                                        state.picker_open = !state.picker_open;
                                        cx.set_global(state);
                                    })
                                    .child(Icon::new(current_device_icon).size_4()),
                            )
                            ,
                    ),
            )
    }
}

/// Extract the `id=` query parameter from a Navidrome stream URL.
fn nd_song_id_from_path(path: &str) -> Option<&str> {
    path.split('?')
        .nth(1)?
        .split('&')
        .find(|p| p.starts_with("id="))
        .map(|p| &p[3..])
}

