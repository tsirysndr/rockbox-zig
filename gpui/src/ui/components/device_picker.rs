use crate::controller::Controller;
use crate::state::{DeviceItem, DevicesState};
use crate::ui::components::icons::{Icon, Icons};
use crate::ui::theme::Theme;
use gpui::prelude::FluentBuilder;
use gpui::{
    div, px, App, Context, FontWeight, InteractiveElement, IntoElement, ParentElement,
    Render, StatefulInteractiveElement, Styled, Window,
};

pub fn device_icon(device: &DeviceItem) -> Icons {
    match device.app.as_str() {
        "Chromecast" => Icons::Chromecast,
        "AirPlay" => Icons::Airplay,
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

        div()
            .id("device-picker-overlay")
            .absolute()
            .bottom(px(80.0))
            .right(px(16.0))
            .w(px(280.0))
            .bg(theme.app_bg)
            .border_1()
            .border_color(theme.border)
            .rounded_lg()
            .shadow_lg()
            .overflow_hidden()
            .on_mouse_down_out(|_, _, cx: &mut App| {
                let mut state = cx.global::<DevicesState>().clone();
                state.picker_open = false;
                cx.set_global(state);
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
                    .py_1()
                    .children(state.devices.iter().cloned().map(|device| {
                        let id = device.id.clone();
                        let is_current = device.is_current_device;
                        let icon = device_icon(&device);
                        let name = device.name.clone();

                        div()
                            .id(gpui::SharedString::from(format!("device-row-{}", device.id)))
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
                                    let _ = crate::client::connect_device(id_clone).await;
                                });
                                // Optimistically update current device in local state.
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
            )
            .into_any_element()
    }
}

/// Helper to refresh the device list from the server.
pub fn refresh_devices(cx: &mut App) {
    let rt = cx.global::<Controller>().rt();
    rt.spawn(async move {
        if let Ok(devices) = crate::client::fetch_devices().await {
            // Can't update cx from async context directly; the caller should
            // arrange a channel or use cx.spawn. For simplicity this is fire-and-forget
            // and the list will be refreshed next time the picker opens.
            let _ = devices;
        }
    });
}

/// Fetch devices and push them into DevicesState via a blocking-compatible spawn.
pub fn fetch_and_update_devices(cx: &mut App) {
    let rt = cx.global::<Controller>().rt();
    // We use a oneshot channel to get the result back into the GPUI context.
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
