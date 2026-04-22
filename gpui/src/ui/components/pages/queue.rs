use crate::controller::Controller;
use crate::state::format_duration;
use crate::ui::components::miniplayer::MiniPlayer;
use crate::ui::theme::Theme;
use gpui::prelude::FluentBuilder;
use gpui::{
    App, AppContext, Entity, FontWeight, InteractiveElement, IntoElement, ParentElement, Render,
    StatefulInteractiveElement, Styled, UniformListScrollHandle, Window, div, px, uniform_list,
};

pub struct QueuePage {
    scroll_handle: UniformListScrollHandle,
    miniplayer: Entity<MiniPlayer>,
}

impl QueuePage {
    pub fn new(cx: &mut App) -> Self {
        QueuePage {
            scroll_handle: UniformListScrollHandle::new(),
            miniplayer: cx.new(|_| MiniPlayer),
        }
    }
}

impl Render for QueuePage {
    fn render(&mut self, _: &mut Window, cx: &mut gpui::Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let state = cx.global::<Controller>().state.read(cx);
        let n = state.queue.len();
        let scroll_handle = self.scroll_handle.clone();

        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(theme.queue_bg)
            .child(
                div()
                    .w_full()
                    .flex_shrink_0()
                    .flex()
                    .items_center()
                    .px_6()
                    .py_4()
                    .border_b_1()
                    .border_color(theme.border)
                    .child(
                        div()
                            .text_base()
                            .font_weight(FontWeight(600.0))
                            .text_color(theme.queue_heading_text)
                            .child("Queue"),
                    )
                    .child(
                        div()
                            .ml_auto()
                            .text_sm()
                            .text_color(theme.queue_item_artist)
                            .child(format!("{n} tracks")),
                    ),
            )
            .child(
                uniform_list("queue_list", n, move |range, _window, cx| {
                    let theme = *cx.global::<Theme>();
                    let state = cx.global::<Controller>().state.read(cx);
                    let current_idx = state.current_idx;

                    range
                        .map(|pos| {
                            let track_idx = state.queue[pos];
                            let track = &state.tracks[track_idx];
                            let is_current = current_idx == Some(track_idx);

                            div()
                                .id(("queue_row", pos))
                                .w_full()
                                .flex()
                                .items_center()
                                .px_6()
                                .py_3()
                                .gap_x_4()
                                .cursor_pointer()
                                .hover(|this| this.bg(theme.queue_item_bg_hover))
                                .when(is_current, |this| {
                                    this.bg(theme.queue_item_bg_current)
                                        .border_l_2()
                                        .border_b_2()
                                        .border_color(theme.switcher_active)
                                })
                                .on_click(move |_, _, cx: &mut App| {
                                    let state = cx.global::<Controller>().state.clone();
                                    state.update(cx, |s: &mut crate::state::AppState, _| s.play_track(track_idx));
                                })
                                .child(
                                    div()
                                        .w(px(20.0))
                                        .text_xs()
                                        .text_color(theme.queue_item_artist)
                                        .child(format!("{}", pos + 1)),
                                )
                                .child(
                                    div()
                                        .flex_1()
                                        .flex()
                                        .flex_col()
                                        .gap_y_0p5()
                                        .child(
                                            div()
                                                .text_sm()
                                                .truncate()
                                                .text_color(if is_current {
                                                    theme.queue_item_title_current
                                                } else {
                                                    theme.queue_item_title
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
                                                .text_xs()
                                                .truncate()
                                                .text_color(theme.queue_item_artist)
                                                .child(track.artist.clone()),
                                        ),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(theme.queue_item_artist)
                                        .child(format_duration(track.duration)),
                                )
                        })
                        .collect()
                })
                .flex_1()
                .w_full()
                .track_scroll(scroll_handle),
            )
            .child(self.miniplayer.clone())
    }
}
