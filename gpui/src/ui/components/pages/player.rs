use crate::client::{save_repeat, save_shuffle};
use crate::controller::Controller;
use crate::state::PlaybackStatus;
use crate::ui::components::controlbar::ControlBar;
use crate::ui::components::icons::{Icon, Icons};
use crate::ui::components::LikedSongs;
use crate::ui::global_keybinds::play_pause;
use crate::ui::theme::Theme;
use gpui::prelude::FluentBuilder;
use gpui::px;
use gpui::{
    div, img, rgba, App, Context, Entity, FontWeight, InteractiveElement, IntoElement, ObjectFit,
    ParentElement, Render, StatefulInteractiveElement, Styled, StyledImage, Window,
};

pub struct PlayerPage {
    pub controlbar: Entity<ControlBar>,
}

impl PlayerPage {
    pub fn new(_cx: &mut App, controlbar: Entity<ControlBar>) -> Self {
        PlayerPage { controlbar }
    }
}

impl Render for PlayerPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let liked_songs = cx.global::<LikedSongs>().0.clone();
        let state = cx.global::<Controller>().state.read(cx);
        let is_playing = state.status == PlaybackStatus::Playing;
        let is_shuffling = state.shuffling;
        let is_repeat = state.repeat;

        let title = state
            .current_track()
            .map(|t| t.title.clone())
            .unwrap_or_else(|| "No track selected".into());
        let artist = state
            .current_track()
            .map(|t| t.artist.clone())
            .unwrap_or_default();
        let album = state
            .current_track()
            .map(|t| t.album.clone())
            .unwrap_or_default();
        let album_art_url = state
            .current_track()
            .and_then(|t| t.album_art.as_deref())
            .filter(|s| !s.is_empty())
            .map(|id| format!("http://localhost:6062/covers/{id}"));
        let bg_art_url = album_art_url.clone();
        let queue_total = state.queue.len();
        let queue_pos = state.current_idx.map(|i| i + 1);
        let current_path = state.current_track().map(|t| t.path.clone()).unwrap_or_default();
        let track_id = state
            .tracks
            .iter()
            .find(|t| t.path == current_path)
            .map(|t| t.id.clone())
            .unwrap_or_default();
        let is_liked = liked_songs.contains(&track_id);

        div()
            .size_full()
            .flex()
            .flex_col()
            .relative()
            .bg(theme.player_bg)
            // Full-screen album art background (low opacity)
            .when_some(bg_art_url, |this, url| {
                this.child(
                    div()
                        .absolute()
                        .top_0()
                        .left_0()
                        .w_full()
                        .h_full()
                        .opacity(0.4)
                        .overflow_hidden()
                        .child(img(url).w_full().h_full().object_fit(ObjectFit::Cover)),
                )
                // Dark scrim so text stays readable
                .child(
                    div()
                        .absolute()
                        .top_0()
                        .left_0()
                        .w_full()
                        .h_full()
                        .bg(rgba(0x0F1117BF)),
                )
            })
            .child(
                div()
                    .flex_1()
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .gap_y_6()
                    .px_16()
                    .pt_8()
                    .child(if let Some(url) = album_art_url {
                        div()
                            .w(px(360.0))
                            .h(px(360.0))
                            .rounded_xl()
                            .overflow_hidden()
                            .flex_shrink_0()
                            .child(img(url).w_full().h_full().object_fit(ObjectFit::Cover))
                            .into_any_element()
                    } else {
                        div()
                            .w(px(360.0))
                            .h(px(360.0))
                            .rounded_xl()
                            .border_2()
                            .border_color(theme.border)
                            .flex()
                            .items_center()
                            .justify_center()
                            .bg(theme.library_art_bg)
                            .text_color(theme.player_icons_text)
                            .child(Icon::new(Icons::Music).size_16())
                            .into_any_element()
                    })
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .items_center()
                            .gap_y_1()
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_x_3()
                                    .child(
                                        div()
                                            .text_2xl()
                                            .font_weight(FontWeight(600.0))
                                            .text_color(theme.player_title_text)
                                            .max_w_96()
                                            .truncate()
                                            .child(title),
                                    )
                                    .child(
                                        div()
                                            .id("player_heart")
                                            .flex_shrink_0()
                                            .p_1()
                                            .mt_2()
                                            .rounded_full()
                                            .cursor_pointer()
                                            .text_color(if is_liked {
                                                gpui::rgb(0xFFFFFF)
                                            } else {
                                                theme.player_icons_text
                                            })
                                            .hover(|this| this.bg(theme.player_icons_bg_hover))
                                            .on_click(move |_, _, cx: &mut App| {
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
                                            .child(Icon::new(if is_liked { Icons::Heart } else { Icons::HeartOutline }).size_7()),
                                    ),
                            )
                            .child(
                                div()
                                    .text_base()
                                    .text_color(theme.player_artist_text)
                                    .max_w_96()
                                    .truncate()
                                    .child(artist),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(theme.player_icons_text)
                                    .max_w_96()
                                    .truncate()
                                    .child(album),
                            )
                            .child(div().text_xs().text_color(theme.player_icons_text).child(
                                if let Some(pos) = queue_pos {
                                    format!("{pos} / {queue_total}")
                                } else {
                                    String::new()
                                },
                            )),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_x_4()
                            .child(
                                div()
                                    .id("shuffle_btn")
                                    .p_3()
                                    .rounded_md()
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .cursor_pointer()
                                    .text_color(theme.player_icons_text)
                                    .when(is_shuffling, |this| {
                                        this.text_color(theme.player_icons_text_active)
                                            .bg(theme.player_icons_bg_active)
                                    })
                                    .hover(|this| {
                                        this.bg(theme.player_icons_bg_hover)
                                            .text_color(theme.player_icons_text_hover)
                                    })
                                    .on_click(|_, _, cx: &mut App| {
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
                                    .id("prev_btn")
                                    .p_3()
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
                                    .on_click(|_, _, cx: &mut App| {
                                        cx.global::<Controller>().prev();
                                    })
                                    .child(Icon::new(Icons::Prev).size_4()),
                            )
                            .child(
                                div()
                                    .id("play_pause_btn")
                                    .p_5()
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
                                        Icon::new(Icons::Pause).size_5()
                                    } else {
                                        Icon::new(Icons::Play).size_5()
                                    }),
                            )
                            .child(
                                div()
                                    .id("next_btn")
                                    .p_3()
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
                                    .on_click(|_, _, cx: &mut App| {
                                        cx.global::<Controller>().next();
                                    })
                                    .child(Icon::new(Icons::Next).size_4()),
                            )
                            .child(
                                div()
                                    .id("repeat_btn")
                                    .p_3()
                                    .rounded_md()
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .cursor_pointer()
                                    .text_color(theme.player_icons_text)
                                    .when(is_repeat, |this| {
                                        this.text_color(theme.player_icons_text_active)
                                            .bg(theme.player_icons_bg_active)
                                    })
                                    .hover(|this| {
                                        this.bg(theme.player_icons_bg_hover)
                                            .text_color(theme.player_icons_text_hover)
                                    })
                                    .on_click(|_, _, cx: &mut App| {
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
                    ),
            )
            .child(self.controlbar.clone())
    }
}
