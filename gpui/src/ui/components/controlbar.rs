use crate::controller::Controller;
use crate::state::format_duration;
use crate::ui::components::icons::{Icon, Icons};
use crate::ui::components::seek_bar::SeekBar;
use crate::ui::theme::Theme;
use gpui::{div, px, App, Context, IntoElement, ParentElement, Render, Styled, Window};

pub struct ControlBar;

impl Render for ControlBar {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let state = cx.global::<Controller>().state.read(cx);

        let duration = state.current_track().map(|t| t.duration).unwrap_or(0);
        let position = state.position;
        let vol_fill = crate::state::volume_fraction(state.volume);
        let fill_fraction = if duration > 0 {
            (position as f32 / duration as f32).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let vol_pct = (vol_fill * 100.0) as u32;

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
                        SeekBar::new(
                            "controlbar-seek",
                            fill_fraction,
                            theme.playback_slider_track,
                            theme.playback_slider_fill,
                            px(4.0),
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
                                    .w(px(vol_fill * 96.0)),
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
