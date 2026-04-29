use crate::client::{play_directory, play_directory_at, FileEntry};
use crate::controller::Controller;
use crate::ui::components::icons::{Icon, Icons};
use crate::ui::components::{
    FileContextMenu, FileContextMenuState, FilesBrowseState, FilesMode,
};
use crate::ui::theme::Theme;
use gpui::prelude::FluentBuilder;
use gpui::{
    div, px, uniform_list, AnyElement, App, ClickEvent, Context, FontWeight,
    InteractiveElement, IntoElement, ParentElement, Render, StatefulInteractiveElement, Styled,
    WeakEntity, Window,
};

pub struct FilesView {
    entries: Vec<FileEntry>,
    upnp_devices: Vec<FileEntry>,
    last_loaded: Option<(FilesMode, Option<String>)>,
}

impl FilesView {
    pub fn new(cx: &mut Context<Self>) -> Self {
        cx.set_global(FilesBrowseState::default());
        cx.set_global(FileContextMenuState::default());

        // Kick off a background prefetch of UPnP devices so they appear instantly
        // when the user opens "UPnP Devices". The result is stored in upnp_devices.
        let (tx, rx) = tokio::sync::oneshot::channel::<Vec<FileEntry>>();
        cx.global::<Controller>().rt().spawn(async move {
            let devices =
                crate::client::tree_get_entries(Some("upnp://".to_string()))
                    .await
                    .unwrap_or_default();
            let _ = tx.send(devices);
        });
        cx.spawn(async move |this: WeakEntity<FilesView>, cx| {
            if let Ok(devices) = rx.await {
                let _ = this.update(cx, |this, _cx| {
                    this.upnp_devices = devices;
                    // Do NOT notify or touch entries here — initial prefetch only
                    // populates the cache; entries are set in load_if_needed.
                });
            }
        })
        .detach();

        FilesView {
            entries: Vec::new(),
            upnp_devices: Vec::new(),
            last_loaded: None,
        }
    }

    fn load_if_needed(&mut self, cx: &mut Context<Self>) {
        let browse = cx.global::<FilesBrowseState>().clone();
        let key = (browse.mode.clone(), browse.current_path.clone());

        if self.last_loaded.as_ref() == Some(&key) {
            return;
        }
        // Root mode renders static tiles — no fetch needed.
        if browse.mode == FilesMode::Root {
            self.entries.clear();
            self.last_loaded = Some(key);
            return;
        }

        // UpnpDevices: show preloaded cache immediately, then re-fetch in background.
        if browse.mode == FilesMode::UpnpDevices {
            self.entries = self.upnp_devices.clone();
            self.last_loaded = Some(key.clone());
            // Re-fetch to refresh the device list (SSDP discovery is time-sensitive).
            let (tx, rx) = tokio::sync::oneshot::channel::<Vec<FileEntry>>();
            cx.global::<Controller>().rt().spawn(async move {
                let devices =
                    crate::client::tree_get_entries(Some("upnp://".to_string()))
                        .await
                        .unwrap_or_default();
                let _ = tx.send(devices);
            });
            cx.spawn(async move |this: WeakEntity<FilesView>, cx| {
                if let Ok(devices) = rx.await {
                    let _ = this.update(cx, |this, cx| {
                        this.upnp_devices = devices.clone();
                        // Only apply to displayed entries if still on the device list.
                        let current_mode = cx.global::<FilesBrowseState>().mode.clone();
                        if current_mode == FilesMode::UpnpDevices {
                            this.entries = devices;
                            cx.notify();
                        }
                    });
                }
            })
            .detach();
            return;
        }

        self.last_loaded = Some(key.clone());
        let path = browse.current_path.clone();

        let (tx, rx) = tokio::sync::oneshot::channel::<Vec<FileEntry>>();
        cx.global::<Controller>().rt().spawn(async move {
            let entries = crate::client::tree_get_entries(path).await.unwrap_or_default();
            let _ = tx.send(entries);
        });

        cx.spawn(async move |this: WeakEntity<FilesView>, cx| {
            if let Ok(entries) = rx.await {
                let _ = this.update(cx, |this, cx| {
                    // Only apply if the user is still on the same page that triggered
                    // this fetch — guards against races when navigating quickly.
                    let current_key = {
                        let browse = cx.global::<FilesBrowseState>();
                        (browse.mode.clone(), browse.current_path.clone())
                    };
                    if current_key == key {
                        this.entries = entries;
                        cx.notify();
                    }
                });
            }
        })
        .detach();
    }
}

impl Render for FilesView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.load_if_needed(cx);

        let theme = *cx.global::<Theme>();
        let browse = cx.global::<FilesBrowseState>().clone();
        let can_go_back = browse.can_go_back();

        let path_display: String = match &browse.mode {
            FilesMode::Root => "Files".to_string(),
            FilesMode::Local => browse
                .current_path
                .as_deref()
                .and_then(|p| std::path::Path::new(p).file_name())
                .and_then(|n| n.to_str())
                .unwrap_or("Music")
                .to_string(),
            FilesMode::UpnpDevices => "UPnP Devices".to_string(),
            FilesMode::UpnpBrowse => browse
                .current_path
                .as_deref()
                .and_then(|p| p.rsplit('/').next())
                .unwrap_or("UPnP")
                .to_string(),
        };

        let current_dir = browse.current_path.clone().unwrap_or_default();
        let mode = browse.mode.clone();
        let entries = self.entries.clone();

        div()
            .size_full()
            .flex()
            .flex_col()
            // ── Header ────────────────────────────────────────────────────────
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_x_2()
                    .px_4()
                    .py_3()
                    .border_b_1()
                    .border_color(theme.library_table_border)
                    .child(
                        div()
                            .id("files_back_btn")
                            .p_1p5()
                            .rounded_md()
                            .flex()
                            .items_center()
                            .justify_center()
                            .cursor_pointer()
                            .text_color(theme.library_text)
                            .opacity(if can_go_back { 1.0 } else { 0.3 })
                            .when(can_go_back, |this| {
                                this.hover(|t| t.bg(theme.library_track_bg_hover)).on_click(
                                    |_, _, cx: &mut App| {
                                        cx.global_mut::<FilesBrowseState>().go_back();
                                    },
                                )
                            })
                            .child(Icon::new(Icons::ChevronLeft).size_4()),
                    )
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight(600.0))
                            .text_color(theme.library_text)
                            .child(path_display),
                    ),
            )
            // ── Content ───────────────────────────────────────────────────────
            .child(match mode {
                FilesMode::Root => render_root(theme).into_any_element(),
                _ => render_entries(entries, current_dir, mode, theme).into_any_element(),
            })
    }
}

fn render_root(theme: Theme) -> AnyElement {
    div()
        .flex_1()
        .min_h_0()
        .child(
            div()
                .id("root_music")
                .w_full()
                .flex()
                .items_center()
                .gap_x_3()
                .px_4()
                .py_2p5()
                .cursor_pointer()
                .hover(|t| t.bg(theme.library_track_bg_hover))
                .on_click(|_, _, cx: &mut App| {
                    cx.global_mut::<FilesBrowseState>().navigate(FilesMode::Local, None);
                })
                .child(
                    div()
                        .w(px(22.0))
                        .h(px(22.0))
                        .flex_shrink_0()
                        .flex()
                        .items_center()
                        .text_color(theme.library_text)
                        .child(Icon::new(Icons::Directory).size_5()),
                )
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .text_sm()
                        .truncate()
                        .text_color(theme.library_text)
                        .child("Music"),
                ),
        )
        .child(
            div()
                .id("root_upnp")
                .w_full()
                .flex()
                .items_center()
                .gap_x_3()
                .px_4()
                .py_2p5()
                .cursor_pointer()
                .hover(|t| t.bg(theme.library_track_bg_hover))
                .on_click(|_, _, cx: &mut App| {
                    cx.global_mut::<FilesBrowseState>()
                        .navigate(FilesMode::UpnpDevices, Some("upnp://".to_string()));
                })
                .child(
                    div()
                        .w(px(22.0))
                        .h(px(22.0))
                        .flex_shrink_0()
                        .flex()
                        .items_center()
                        .text_color(theme.library_text)
                        .child(Icon::new(Icons::Upnp).size_5()),
                )
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .text_sm()
                        .truncate()
                        .text_color(theme.library_text)
                        .child("UPnP Devices"),
                ),
        )
        .into_any_element()
}

fn render_entries(
    entries: Vec<FileEntry>,
    current_dir: String,
    mode: FilesMode,
    theme: Theme,
) -> AnyElement {
    let is_device_list = mode == FilesMode::UpnpDevices;
    uniform_list("files_list", entries.len(), move |range, _, _cx| {
        range
            .map(|idx| {
                let entry = entries[idx].clone();
                let group_name: gpui::SharedString = format!("file_row_{}", idx).into();
                let gn_icon = group_name.clone();
                let gn_play = group_name.clone();
                let gn_opts = group_name.clone();
                let path_nav = entry.path.clone();
                let path_play = entry.path.clone();
                let path_opts = entry.path.clone();
                let name_opts = entry.name.clone();
                let cur_dir_play = current_dir.clone();
                let cur_dir_opts = current_dir.clone();
                let is_dir = entry.is_dir;
                let is_upnp = entry.path.starts_with("upnp://");

                let dir_icon = if is_device_list {
                    Icons::Device
                } else {
                    Icons::Directory
                };

                div()
                    .id(("file_row", idx))
                    .group(group_name)
                    .w_full()
                    .flex()
                    .items_center()
                    .gap_x_3()
                    .px_4()
                    .py_2p5()
                    .hover(|t| t.bg(theme.library_track_bg_hover))
                    // ── Icon ─────────────────────────────────────────────────
                    .child(
                        div()
                            .w(px(22.0))
                            .h(px(22.0))
                            .flex_shrink_0()
                            .relative()
                            .child(
                                div()
                                    .absolute()
                                    .top_0()
                                    .left_0()
                                    .w_full()
                                    .h_full()
                                    .flex()
                                    .items_center()
                                    .when(!is_device_list, |this| {
                                        this.group_hover(gn_icon, |s| s.opacity(0.0))
                                    })
                                    .text_color(if is_dir {
                                        theme.library_text
                                    } else {
                                        theme.library_header_text
                                    })
                                    .child(
                                        Icon::new(if is_dir { dir_icon } else { Icons::Music })
                                            .size_5(),
                                    ),
                            )
                            .when(!is_device_list, |this| {
                                this.child(
                                    div()
                                        .id(("file_play_icon", idx))
                                        .absolute()
                                        .top_0()
                                        .left(px(-3.0))
                                        .w_full()
                                        .h_full()
                                        .flex()
                                        .items_center()
                                        .opacity(0.0)
                                        .group_hover(gn_play, |s| s.opacity(1.0))
                                        .cursor_pointer()
                                        .text_color(theme.library_text)
                                        .on_click(move |_, _, cx: &mut App| {
                                            cx.stop_propagation();
                                            let rt = cx.global::<Controller>().rt();
                                            if is_dir {
                                                rt.spawn(play_directory(path_play.clone(), false));
                                            } else {
                                                rt.spawn(play_directory_at(
                                                    cur_dir_play.clone(),
                                                    idx as i32,
                                                ));
                                            }
                                        })
                                        .child(Icon::new(Icons::Play).size_5()),
                                )
                            }),
                    )
                    // ── Name — clicking navigates into directories ────────────
                    .child(
                        div()
                            .id(("file_name", idx))
                            .flex_1()
                            .min_w_0()
                            .text_sm()
                            .truncate()
                            .text_color(theme.library_text)
                            .when(is_dir, |this| {
                                this.cursor_pointer().on_click(move |_, _, cx: &mut App| {
                                    let new_mode = if is_upnp {
                                        FilesMode::UpnpBrowse
                                    } else {
                                        FilesMode::Local
                                    };
                                    cx.global_mut::<FilesBrowseState>()
                                        .navigate(new_mode, Some(path_nav.clone()));
                                })
                            })
                            .child(entry.name.clone()),
                    )
                    // ── Context menu (not shown for device-list entries) ───────
                    .when(!is_device_list, |this| {
                        this.child(
                            div()
                                .id(("file_opts_btn", idx))
                                .w(px(28.0))
                                .flex_shrink_0()
                                .flex()
                                .items_center()
                                .justify_center()
                                .opacity(0.0)
                                .group_hover(gn_opts, |s| s.opacity(1.0))
                                .cursor_pointer()
                                .text_color(theme.library_header_text)
                                .on_click(move |event: &ClickEvent, _, cx: &mut App| {
                                    cx.stop_propagation();
                                    cx.global_mut::<FileContextMenuState>().0 =
                                        Some(FileContextMenu {
                                            pos: event.position(),
                                            path: path_opts.clone(),
                                            name: name_opts.clone(),
                                            is_dir,
                                            current_dir: cur_dir_opts.clone(),
                                            dir_idx: idx,
                                        });
                                })
                                .child(Icon::new(Icons::Options).size_4()),
                        )
                    })
            })
            .collect()
    })
    .flex_1()
    .min_h_0()
    .into_any_element()
}

pub fn menu_item(
    id: &'static str,
    label: &'static str,
    theme: Theme,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    div()
        .id(id)
        .px_4()
        .py_2()
        .text_sm()
        .cursor_pointer()
        .text_color(theme.library_text)
        .hover(|t| t.bg(theme.library_track_bg_hover))
        .on_click(on_click)
        .child(label)
}
