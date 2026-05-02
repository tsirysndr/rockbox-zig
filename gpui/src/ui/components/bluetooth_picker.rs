use crate::controller::Controller;
use crate::state::BluetoothState;
use crate::ui::components::icons::{Icon, Icons};
use crate::ui::theme::Theme;
use gpui::prelude::FluentBuilder;
use gpui::{
    div, px, App, Context, FontWeight, InteractiveElement, IntoElement, MouseButton,
    MouseMoveEvent, ParentElement, Render, StatefulInteractiveElement, Styled, Window,
};

pub struct BluetoothPicker;

impl Render for BluetoothPicker {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let state = cx.global::<BluetoothState>().clone();

        if !state.picker_open {
            return div().into_any_element();
        }

        div()
            .id("bluetooth-picker-backdrop")
            .absolute()
            .top_0()
            .left_0()
            .w_full()
            .h_full()
            .on_mouse_down(MouseButton::Left, |_, _, cx| {
                let mut state = cx.global::<BluetoothState>().clone();
                state.picker_open = false;
                cx.set_global(state);
                cx.stop_propagation();
            })
            .on_mouse_move(|_: &MouseMoveEvent, _, cx| {
                cx.stop_propagation();
            })
            .on_scroll_wheel(|_, _, _| {})
            .child(
                div()
                    .id("bluetooth-picker-popup")
                    .absolute()
                    .bottom(px(80.0))
                    .right(px(52.0))
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
                                    .child("Bluetooth speakers"),
                            ),
                    )
                    .child(
                        div()
                            .id("bluetooth-picker-list")
                            .py_1()
                            .max_h(px(280.0))
                            .overflow_y_scroll()
                            .when(state.devices.is_empty(), |this| {
                                this.child(
                                    div()
                                        .px_4()
                                        .py_3()
                                        .text_sm()
                                        .text_color(theme.player_title_text)
                                        .child("No bluetooth devices found."),
                                )
                            })
                            .children(state.devices.iter().cloned().map(|device| {
                                let address = device.address.clone();
                                let is_connected = device.connected;
                                let name = device.name.clone();

                                div()
                                    .id(gpui::SharedString::from(format!(
                                        "bt-row-{}",
                                        device.address
                                    )))
                                    .px_4()
                                    .py_2()
                                    .flex()
                                    .items_center()
                                    .gap_x_3()
                                    .cursor_pointer()
                                    .text_color(if is_connected {
                                        gpui::rgb(0xFFFFFF)
                                    } else {
                                        theme.player_title_text
                                    })
                                    .hover(|this| this.bg(theme.player_icons_bg_hover))
                                    .on_click(move |_, _, cx: &mut App| {
                                        let rt = cx.global::<Controller>().rt();
                                        let addr = address.clone();
                                        if is_connected {
                                            rt.spawn(async move {
                                                let _ = crate::client::disconnect_bluetooth_device(
                                                    addr,
                                                )
                                                .await;
                                            });
                                        } else {
                                            rt.spawn(async move {
                                                let _ =
                                                    crate::client::connect_bluetooth_device(addr)
                                                        .await;
                                            });
                                        }
                                        let mut state = cx.global::<BluetoothState>().clone();
                                        for d in state.devices.iter_mut() {
                                            if d.address == address {
                                                d.connected = !is_connected;
                                            }
                                        }
                                        state.picker_open = false;
                                        cx.set_global(state);
                                    })
                                    .child(
                                        div()
                                            .flex_shrink_0()
                                            .child(Icon::new(Icons::Bluetooth).size_4()),
                                    )
                                    .child(div().flex_1().text_sm().truncate().child(name))
                                    .when(is_connected, |this: gpui::Stateful<gpui::Div>| {
                                        this.child(
                                            div()
                                                .flex_shrink_0()
                                                .w(px(6.0))
                                                .h(px(6.0))
                                                .rounded_full()
                                                .bg(gpui::rgb(0x39FF14)),
                                        )
                                    })
                            })),
                    ),
            )
            .into_any_element()
    }
}

/// Checks whether the connected server supports Bluetooth (by calling get_devices).
/// Re-checks automatically whenever the active server changes.
/// Updates BluetoothState::available so the UI can show/hide the bluetooth button.
pub fn check_and_set_bluetooth_available(cx: &mut App) {
    let rt = cx.global::<Controller>().rt();
    // std::sync::mpsc avoids cross-runtime waker issues — tokio side sends without await,
    // GPUI side polls with try_recv.
    let (tx, rx) = std::sync::mpsc::channel::<bool>();

    rt.spawn(async move {
        let available = crate::client::check_bluetooth_available().await;
        let _ = tx.send(available);

        let notify = crate::server::server_notify();
        loop {
            notify.notified().await;
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            let available = crate::client::check_bluetooth_available().await;
            if tx.send(available).is_err() {
                break;
            }
        }
    });

    cx.spawn(async move |cx| loop {
        while let Ok(available) = rx.try_recv() {
            if cx
                .update(|cx| {
                    let mut state = cx.global::<BluetoothState>().clone();
                    state.available = available;
                    cx.set_global(state);
                })
                .is_err()
            {
                return;
            }
        }
        cx.background_executor()
            .timer(std::time::Duration::from_millis(200))
            .await;
    })
    .detach();
}

pub fn fetch_and_update_bluetooth_devices(cx: &mut App) {
    let rt = cx.global::<Controller>().rt();
    let (tx, mut rx) = tokio::sync::mpsc::channel::<Vec<crate::state::BluetoothDevice>>(1);
    rt.spawn(async move {
        if let Ok(devices) = crate::client::fetch_bluetooth_devices().await {
            let _ = tx.send(devices).await;
        }
    });
    cx.spawn(async move |cx| {
        if let Some(devices) = rx.recv().await {
            let _ = cx.update(|cx| {
                let mut state = cx.global::<BluetoothState>().clone();
                state.devices = devices;
                cx.set_global(state);
            });
        }
    })
    .detach();
}
