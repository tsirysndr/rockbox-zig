use crate::controller::Controller;
use crate::state::{format_duration, AppState, PlaybackStatus};
use crate::ui::components::icons::{Icon, Icons};
use crate::ui::components::Page;
use crate::ui::helpers::secs_to_slider;
use crate::ui::theme::Theme;
use gpui::{
    div, px, relative, App, Context, FontWeight, InteractiveElement, IntoElement, ParentElement,
    Render, StatefulInteractiveElement, Styled, Window,
};

pub struct MiniPlayer;

impl Render for MiniPlayer {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
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
        let position = state.position;
        let fill = secs_to_slider(position, duration) / 100.0;
        let vol_pct = (state.volume * 100.0) as u32;

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
                            .child(
                                div()
                                    .w(px(48.0))
                                    .h(px(48.0))
                                    .rounded_lg()
                                    .flex_shrink_0()
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .bg(theme.border)
                                    .text_color(theme.player_icons_text)
                                    .child(Icon::new(Icons::Music).size_4()),
                            )
                            .child(
                                div()
                                    .id("mini_info")
                                    .flex_1()
                                    .flex()
                                    .flex_col()
                                    .gap_y_0p5()
                                    .overflow_hidden()
                                    .cursor_pointer()
                                    .on_click(|_, _, cx: &mut App| {
                                        *cx.global_mut::<Page>() = Page::Player;
                                    })
                                    .child(
                                        div()
                                            .text_sm()
                                            .font_weight(FontWeight(500.0))
                                            .text_color(theme.player_title_text)
                                            .truncate()
                                            .child(title),
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
                                                let state = cx.global::<Controller>().state.clone();
                                                state.update(cx, |s: &mut AppState, _| s.prev());
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
                                                let state = cx.global::<Controller>().state.clone();
                                                state.update(cx, |s: &mut AppState, _| {
                                                    match s.status {
                                                        PlaybackStatus::Playing => s.pause(),
                                                        _ => s.play(),
                                                    }
                                                });
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
                                                let state = cx.global::<Controller>().state.clone();
                                                state.update(cx, |s: &mut AppState, _| s.next());
                                            })
                                            .child(Icon::new(Icons::Next).size_4()),
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
                                        div()
                                            .flex_1()
                                            .h(px(3.0))
                                            .rounded_full()
                                            .bg(theme.playback_slider_track)
                                            .child(
                                                div()
                                                    .h_full()
                                                    .rounded_full()
                                                    .bg(theme.playback_slider_fill)
                                                    .w(relative(fill)),
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
                                    .bg(theme.volume_slider_track)
                                    .child(
                                        div()
                                            .h_full()
                                            .rounded_full()
                                            .bg(theme.volume_slider_fill)
                                            .w(relative(state.volume)),
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
