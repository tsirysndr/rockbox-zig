use crate::controller::Controller;
use crate::state::{format_duration, DevicesState};
use crate::ui::components::device_picker::{device_icon, fetch_and_update_devices};
use crate::ui::components::icons::{Icon, Icons};
use crate::ui::components::seek_bar::SeekBar;
use crate::ui::theme::Theme;
use gpui::{
    div, px, App, Context, InteractiveElement, IntoElement, ParentElement, Render,
    StatefulInteractiveElement, Styled, Window,
};

pub struct ControlBar;

impl Render for ControlBar {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let state = cx.global::<Controller>().state.read(cx);

        let duration = state.current_track().map(|t| t.duration).unwrap_or(0);
        let position = state.position;
        let fill_fraction = if duration > 0 {
            (position as f32 / duration as f32).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let current_device_icon = cx
            .global::<DevicesState>()
            .devices
            .iter()
            .find(|d| d.is_current_device)
            .map(|d| device_icon(d))
            .unwrap_or(Icons::Speaker);

        div()
            .w_full()
            .flex_shrink_0()
            .flex()
            .items_center()
            .gap_x_4()
            .px_6()
            .py_3()
            // Left spacer — mirrors device button width for symmetry
            .child(div().w(px(160.0)))
            // Elapsed + progress + duration — center
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
            // Device picker — right
            .child(
                div()
                    .w(px(160.0))
                    .flex()
                    .items_center()
                    .justify_end()
                    .child(
                        div()
                            .id("controlbar-device-btn")
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
                                fetch_and_update_devices(cx);
                                let mut state = cx.global::<DevicesState>().clone();
                                state.picker_open = !state.picker_open;
                                cx.set_global(state);
                            })
                            .child(Icon::new(current_device_icon).size_4()),
                    ),
            )
    }
}
