use crate::controller::Controller;
use crate::state::{DeviceItem, DevicesState};
use crate::ui::components::icons::{Icon, Icons};
use crate::ui::theme::Theme;
use gpui::prelude::FluentBuilder;
use gpui::{
    div, px, App, Context, FontWeight, InteractiveElement, IntoElement, MouseButton,
    ParentElement, Render, StatefulInteractiveElement, Styled, Window, MouseMoveEvent,
};

pub fn device_icon(device: &DeviceItem) -> Icons {
    match device.service.as_str() {
        "chromecast" => Icons::Chromecast,
        "airplay" => Icons::Airplay,
        "snapcast" => Icons::Speaker,
        _ => {
            if device.is_current_device {
                Icons::Device
            } else {
                Icons::Speaker
            }
        }
    }
}

pub struct DevicePicker;

impl Render for DevicePicker {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let state = cx.global::<DevicesState>().clone();

        if !state.picker_open {
            return div().into_any_element();
        }

        // Full-screen transparent backdrop: swallows scroll events and
        // closes the picker when the user clicks outside the popup.
        div()
            .id("device-picker-backdrop")
            .absolute()
            .top_0()
            .left_0()
            .w_full()
            .h_full()
            .on_mouse_down(MouseButton::Left, |_, _, cx| {
                let mut state = cx.global::<DevicesState>().clone();
                state.picker_open = false;
                cx.set_global(state);
                cx.stop_propagation();
            })
            .on_mouse_move(|_: &MouseMoveEvent, _, cx| {
                cx.stop_propagation();
            })
            .on_scroll_wheel(|_, _, _| {})
            .child(
                // Popup panel — stop propagation so clicks inside don't hit the backdrop.
                div()
                    .id("device-picker-popup")
                    .absolute()
                    .bottom(px(80.0))
                    .right(px(16.0))
                    .w(px(280.0))
                    .bg(theme.app_bg)
                    .border_1()
                    .border_color(theme.border)
                    .rounded_lg()
                    .shadow_lg()
                    .on_mouse_down(MouseButton::Left, |_, _, cx| {
                        cx.stop_propagation();
                    })
                    .on_scroll_wheel(|_, _, cx| {
                        cx.stop_propagation();
                    })
                    .child(
                        div()
                            .px_4()
                            .py_3()
                            .border_b_1()
                            .border_color(theme.border)
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(FontWeight(600.0))
                                    .text_color(theme.player_title_text)
                                    .child("Output device"),
                            ),
                    )
                    .child(
                        div()
                            .id("device-picker-list")
                            .py_1()
                            .max_h(px(280.0))
                            .overflow_y_scroll()
                            .children(state.devices.iter().cloned().map(|device| {
                                let id = device.id.clone();
                                let is_current = device.is_current_device;
                                let icon = device_icon(&device);
                                let name = device.name.clone();

                                div()
                                    .id(gpui::SharedString::from(format!(
                                        "device-row-{}",
                                        device.id
                                    )))
                                    .px_4()
                                    .py_2()
                                    .flex()
                                    .items_center()
                                    .gap_x_3()
                                    .cursor_pointer()
                                    .text_color(if is_current {
                                        theme.player_icons_text_active
                                    } else {
                                        theme.player_title_text
                                    })
                                    .when(is_current, |this: gpui::Stateful<gpui::Div>| {
                                        this.bg(theme.player_icons_bg_active)
                                    })
                                    .hover(|this| this.bg(theme.player_icons_bg_hover))
                                    .on_click(move |_, _, cx: &mut App| {
                                        let rt = cx.global::<Controller>().rt();
                                        let id_clone = id.clone();
                                        rt.spawn(async move {
                                            let _ =
                                                crate::client::connect_device(id_clone).await;
                                        });
                                        let mut state = cx.global::<DevicesState>().clone();
                                        for d in state.devices.iter_mut() {
                                            d.is_current_device = d.id == id;
                                        }
                                        state.picker_open = false;
                                        cx.set_global(state);
                                    })
                                    .child(
                                        div()
                                            .flex_shrink_0()
                                            .child(Icon::new(icon).size_4()),
                                    )
                                    .child(
                                        div()
                                            .flex_1()
                                            .text_sm()
                                            .truncate()
                                            .child(name),
                                    )
                                    .when(is_current, |this: gpui::Stateful<gpui::Div>| {
                                        this.child(
                                            div()
                                                .flex_shrink_0()
                                                .child(Icon::new(Icons::Device).size_3()),
                                        )
                                    })
                            })),
                    ),
            )
            .into_any_element()
    }
}

/// Fetch devices and push them into DevicesState via a blocking-compatible spawn.
pub fn fetch_and_update_devices(cx: &mut App) {
    let rt = cx.global::<Controller>().rt();
    let (tx, mut rx) = tokio::sync::mpsc::channel::<Vec<DeviceItem>>(1);
    rt.spawn(async move {
        if let Ok(devices) = crate::client::fetch_devices().await {
            let _ = tx.send(devices).await;
        }
    });
    cx.spawn(async move |cx| {
        if let Some(devices) = rx.recv().await {
            let _ = cx.update(|cx| {
                let mut state = cx.global::<DevicesState>().clone();
                state.devices = devices;
                cx.set_global(state);
            });
        }
    })
    .detach();
}
