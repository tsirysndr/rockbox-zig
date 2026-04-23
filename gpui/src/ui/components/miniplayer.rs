use crate::client::{adjust_volume, save_repeat, save_shuffle};
use crate::controller::Controller;
use crate::state::{format_duration, volume_fraction, PlaybackStatus, VOLUME_MAX_DB, VOLUME_MIN_DB};
use crate::ui::components::icons::{Icon, Icons};
use crate::ui::components::{LikedSongs, Page};
use crate::ui::global_keybinds::play_pause;
use crate::ui::components::seek_bar::SeekBar;
use crate::ui::theme::Theme;
use gpui::prelude::FluentBuilder;
use gpui::{
    div, img, px, relative, App, Context, FontWeight, InteractiveElement, IntoElement, ObjectFit,
    ParentElement, Render, ScrollWheelEvent, StatefulInteractiveElement, Styled, StyledImage,
    Window,
};

pub struct MiniPlayer;

impl Render for MiniPlayer {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let liked_songs = cx.global::<LikedSongs>().0.clone();
        let state = cx.global::<Controller>().state.read(cx);
        let is_playing = state.status == PlaybackStatus::Playing;

        let title = state
            .current_track()
            .map(|t| t.title.clone())
            .unwrap_or_else(|| "No track selected".into());
        let artist = state
            .current_track()
            .map(|t| t.artist.clone())
            .unwrap_or_default();

        let duration = state.current_track().map(|t| t.duration).unwrap_or(0);
        let album_art_url = state
            .current_track()
            .and_then(|t| t.album_art.as_deref())
            .filter(|s| !s.is_empty())
            .map(|id| format!("http://localhost:6062/covers/{id}"));
        let position = state.position;
        let fill_fraction = if duration > 0 {
            (position as f32 / duration as f32).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let vol_fill = volume_fraction(state.volume);
        let vol_pct = (vol_fill * 100.0) as u32;
        let is_shuffling = state.shuffling;
        let is_repeat = state.repeat;
        let current_path = state.current_track().map(|t| t.path.clone()).unwrap_or_default();
        let track_id = state
            .tracks
            .iter()
            .find(|t| t.path == current_path)
            .map(|t| t.id.clone())
            .unwrap_or_default();
        let is_liked = liked_songs.contains(&track_id);

        div()
            .w_full()
            .flex_shrink_0()
            .flex()
            .flex_col()
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
                                                    .hover(|this| this.bg(theme.player_icons_bg_hover))
                                                    .on_click(move |_, _, cx: &mut App| {
                                                        cx.stop_propagation();
                                                        let rt = cx.global::<Controller>().rt();
                                                        let liked = &mut cx.global_mut::<LikedSongs>().0;
                                                        if liked.contains(&track_id) {
                                                            liked.remove(&track_id);
                                                            rt.spawn(crate::client::unlike_track(track_id.clone()));
                                                        } else {
                                                            liked.insert(track_id.clone());
                                                            rt.spawn(crate::client::like_track(track_id.clone()));
                                                        }
                                                    })
                                                    .child(Icon::new(if is_liked { Icons::Heart } else { Icons::HeartOutline }).size_5()),
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
                                        .on_seek(move |frac, _window, cx: &mut App| {
                                            let seek_secs = (frac * duration as f32) as u64;
                                            cx.global::<Controller>().seek(seek_secs, duration);
                                        }),
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
                    // Right — volume control
                    .child(
                        div()
                            .flex_1()
                            .flex()
                            .items_center()
                            .justify_end()
                            .gap_x_2()
                            .child(
                                div()
                                    .text_color(theme.volume_icon)
                                    .child(Icon::new(Icons::Volume1).size_4()),
                            )
                            .child(
                                div()
                                    .w_24()
                                    .h(px(4.0))
                                    .rounded_full()
                                    .cursor_pointer()
                                    .bg(theme.volume_slider_track)
                                    .on_scroll_wheel(|event: &ScrollWheelEvent, _window, cx: &mut App| {
                                        let delta = event.delta.pixel_delta(px(12.0));
                                        let steps = (-f32::from(delta.y) / 12.0).round() as i32;
                                        if steps != 0 {
                                            let (state, rt) = {
                                                let ctrl = cx.global::<Controller>();
                                                (ctrl.state.clone(), ctrl.rt())
                                            };
                                            let new_vol = {
                                                let current = state.read(cx).volume;
                                                (current + steps).clamp(VOLUME_MIN_DB, VOLUME_MAX_DB)
                                            };
                                            state.update(cx, |s, cx| {
                                                s.volume = new_vol;
                                                cx.notify();
                                            });
                                            rt.spawn(adjust_volume(steps));
                                        }
                                    })
                                    .child(
                                        div()
                                            .h_full()
                                            .rounded_full()
                                            .bg(theme.volume_slider_fill)
                                            .w(relative(vol_fill)),
                                    ),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(theme.playback_position_text)
                                    .child(format!("{vol_pct}%")),
                            ),
                    ),
            )
    }
}
