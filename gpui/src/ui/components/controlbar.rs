use crate::controller::Controller;
use crate::state::format_duration;
use crate::ui::components::icons::{Icon, Icons};
use crate::ui::helpers::secs_to_slider;
use crate::ui::theme::Theme;
use gpui::{div, px, Context, IntoElement, ParentElement, Render, Styled, Window};

pub struct ControlBar;

impl Render for ControlBar {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let state = cx.global::<Controller>().state.read(cx);

        let duration = state.current_track().map(|t| t.duration).unwrap_or(0);
        let position = state.position;
        let volume = state.volume;
        let fill_pct = secs_to_slider(position, duration);
        let vol_pct = (volume * 100.0) as u32;

        div()
            .w_full()
            .flex_shrink_0()
            .flex()
            .items_center()
            .gap_x_4()
            .px_6()
            .py_3()
            // Left spacer — mirrors volume width for symmetry
            .child(div().w(px(160.0)))
            // Elapsed + progress + duration
            .child(
                div()
                    .flex_1()
                    .flex()
                    .items_center()
                    .gap_x_3()
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
                            .h(px(4.0))
                            .rounded_full()
                            .bg(theme.playback_slider_track)
                            .child(
                                div()
                                    .h_full()
                                    .rounded_full()
                                    .bg(theme.playback_slider_fill)
                                    .w(px(fill_pct / 100.0 * 800.0)),
                            ),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(theme.playback_position_text)
                            .flex_shrink_0()
                            .child(format_duration(duration)),
                    ),
            )
            // Volume — fixed width to match left spacer
            .child(
                div()
                    .w(px(160.0))
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
                                    .w(px(vol_pct as f32 / 100.0 * 96.0)),
                            ),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(theme.playback_position_text)
                            .child(format!("{vol_pct}%")),
                    ),
            )
    }
}
