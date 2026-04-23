use crate::controller::Controller;
use crate::state::format_duration;
use crate::ui::components::miniplayer::MiniPlayer;
use crate::ui::theme::Theme;
use gpui::prelude::FluentBuilder;
use gpui::{
    div, px, uniform_list, App, AppContext, Entity, FontWeight, InteractiveElement, IntoElement,
    ParentElement, Render, ScrollStrategy, StatefulInteractiveElement, Styled,
    UniformListScrollHandle, Window,
};

pub struct QueuePage {
    scroll_handle: UniformListScrollHandle,
    miniplayer: Entity<MiniPlayer>,
    last_scrolled_idx: Option<usize>,
}

impl QueuePage {
    pub fn new(cx: &mut App) -> Self {
        QueuePage {
            scroll_handle: UniformListScrollHandle::new(),
            miniplayer: cx.new(|_| MiniPlayer),
            last_scrolled_idx: None,
        }
    }
}

impl Render for QueuePage {
    fn render(&mut self, _: &mut Window, cx: &mut gpui::Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let state = cx.global::<Controller>().state.read(cx);
        let n = state.queue.len();
        let current_idx = state.current_idx;
        let position_label = current_idx
            .map(|i| format!("{} / {}", i + 1, n))
            .unwrap_or_else(|| format!("{n} tracks"));
        let scroll_handle = self.scroll_handle.clone();

        // Scroll to current track whenever it changes (covers page-open and track changes).
        if current_idx != self.last_scrolled_idx {
            if let Some(idx) = current_idx {
                self.scroll_handle
                    .scroll_to_item(idx, ScrollStrategy::Center);
            }
            self.last_scrolled_idx = current_idx;
        }

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
                            .child(position_label),
                    ),
            )
            .child(
                uniform_list("queue_list", n, move |range, _window, cx| {
                    let theme = *cx.global::<Theme>();
                    let state = cx.global::<Controller>().state.read(cx);
                    let ctrl = cx.global::<Controller>();

                    range
                        .map(|pos| {
                            let track = &state.queue[pos];
                            let is_current = current_idx == Some(pos);
                            let title = track.title.clone();
                            let artist = track.artist.clone();
                            let duration = track.duration;
                            let rt = ctrl.rt();

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
                                .on_click(move |_, _, _cx: &mut App| {
                                    rt.spawn(crate::client::jump_to_queue_position(pos as i32));
                                })
                                .child(
                                    div()
                                        .w(px(32.0))
                                        .flex_shrink_0()
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
                                                .child(title),
                                        )
                                        .child(
                                            div()
                                                .text_xs()
                                                .truncate()
                                                .text_color(theme.queue_item_artist)
                                                .child(artist),
                                        ),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(theme.queue_item_artist)
                                        .child(format_duration(duration)),
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
