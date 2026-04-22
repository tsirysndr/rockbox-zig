use crate::controller::Controller;
use crate::state::format_duration;
use crate::ui::components::miniplayer::MiniPlayer;
use crate::ui::theme::Theme;
use gpui::prelude::FluentBuilder;
use gpui::{
    App, AppContext, Entity, FontWeight, InteractiveElement, IntoElement, ParentElement, Render,
    StatefulInteractiveElement, Styled, UniformListScrollHandle, Window, div, px, uniform_list,
};

pub struct LibraryPage {
    scroll_handle: UniformListScrollHandle,
    miniplayer: Entity<MiniPlayer>,
}

impl LibraryPage {
    pub fn new(cx: &mut App) -> Self {
        LibraryPage {
            scroll_handle: UniformListScrollHandle::new(),
            miniplayer: cx.new(|_| MiniPlayer),
        }
    }
}

impl Render for LibraryPage {
    fn render(&mut self, _: &mut Window, cx: &mut gpui::Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let n = cx.global::<Controller>().state.read(cx).tracks.len();
        let scroll_handle = self.scroll_handle.clone();

        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(theme.library_bg)
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
                uniform_list("library_tracks", n, move |range, _window, cx| {
                    let theme = *cx.global::<Theme>();
                    let state = cx.global::<Controller>().state.read(cx);
                    let current_idx = state.current_idx;

                    range
                        .map(|idx| {
                            let track = &state.tracks[idx];
                            let is_current = current_idx == Some(idx);

                            div()
                                .id(("track_row", idx))
                                .w_full()
                                .flex()
                                .items_center()
                                .px_6()
                                .py_3()
                                .border_b_1()
                                .border_color(theme.library_track_border)
                                .cursor_pointer()
                                .hover(|this| this.bg(theme.library_track_bg_hover))
                                .when(is_current, |this| {
                                    this.bg(theme.library_track_bg_active)
                                        .border_l_2()
                                        .border_b_2()
                                        .border_color(theme.switcher_active)
                                })
                                .on_click(move |_, _, cx: &mut App| {
                                    let state = cx.global::<Controller>().state.clone();
                                    state.update(cx, |s: &mut crate::state::AppState, _| s.play_track(idx));
                                })
                                .child(
                                    div()
                                        .w(px(32.0))
                                        .text_sm()
                                        .text_color(theme.library_header_text)
                                        .child(track.track_number.to_string()),
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
                                        .child(track.title.clone()),
                                )
                                .child(
                                    div()
                                        .w_48()
                                        .text_sm()
                                        .truncate()
                                        .text_color(theme.library_header_text)
                                        .child(track.artist.clone()),
                                )
                                .child(
                                    div()
                                        .w_48()
                                        .text_sm()
                                        .truncate()
                                        .text_color(theme.library_header_text)
                                        .child(track.album.clone()),
                                )
                                .child(
                                    div()
                                        .w_16()
                                        .text_sm()
                                        .text_color(theme.library_header_text)
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
