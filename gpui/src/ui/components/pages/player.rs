use crate::controller::Controller;
use crate::state::PlaybackStatus;
use crate::ui::components::controlbar::ControlBar;
use crate::ui::components::icons::{Icon, Icons};
use crate::ui::theme::Theme;
use gpui::prelude::FluentBuilder;
use gpui::px;
use gpui::{
    div, App, Context, Entity, FontWeight, InteractiveElement, IntoElement, ParentElement, Render,
    StatefulInteractiveElement, Styled, Window,
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

        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(theme.player_bg)
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
                    // Album art placeholder
                    .child(
                        div()
                            .w(px(192.0))
                            .h(px(192.0))
                            .rounded_xl()
                            .border_2()
                            .border_color(theme.border)
                            .flex()
                            .items_center()
                            .justify_center()
                            .bg(theme.border)
                            .text_color(theme.player_icons_text)
                            .child(Icon::new(Icons::Music).size_16()),
                    )
                    // Track info
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .items_center()
                            .gap_y_1()
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
                            ),
                    )
                    // Transport controls
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
                                        let state = cx.global::<Controller>().state.clone();
                                        state.update(cx, |s: &mut crate::state::AppState, _| {
                                            s.toggle_shuffle()
                                        });
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
                                        let state = cx.global::<Controller>().state.clone();
                                        state.update(cx, |s: &mut crate::state::AppState, _| {
                                            s.prev()
                                        });
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
                                        let state = cx.global::<Controller>().state.clone();
                                        state.update(
                                            cx,
                                            |s: &mut crate::state::AppState, _| match s.status {
                                                PlaybackStatus::Playing => s.pause(),
                                                _ => s.play(),
                                            },
                                        );
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
                                        let state = cx.global::<Controller>().state.clone();
                                        state.update(cx, |s: &mut crate::state::AppState, _| {
                                            s.next()
                                        });
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
                                        let state = cx.global::<Controller>().state.clone();
                                        state.update(cx, |s: &mut crate::state::AppState, _| {
                                            s.toggle_repeat()
                                        });
                                    })
                                    .child(Icon::new(Icons::Repeat).size_4()),
                            ),
                    ),
            )
            .child(self.controlbar.clone())
    }
}
