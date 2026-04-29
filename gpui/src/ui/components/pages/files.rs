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
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// How long the UPnP device list (upnp://) stays fresh. SSDP is time-sensitive.
const UPNP_DEVICE_TTL: Duration = Duration::from_secs(30);
/// How long a browsed UPnP content folder stays fresh. Content is stable.
const UPNP_CONTENT_TTL: Duration = Duration::from_secs(300);

struct CacheEntry {
    entries: Vec<FileEntry>,
    fetched_at: Instant,
}

pub struct FilesView {
    entries: Vec<FileEntry>,
    last_loaded: Option<(FilesMode, Option<String>)>,
    loading: bool,
    upnp_cache: HashMap<String, CacheEntry>,
}

impl FilesView {
    pub fn new(cx: &mut Context<Self>) -> Self {
        cx.set_global(FilesBrowseState::default());
        cx.set_global(FileContextMenuState::default());

        // Prefetch UPnP device list into the cache so the first open is instant.
        let (tx, rx) = tokio::sync::oneshot::channel::<Vec<FileEntry>>();
        cx.global::<Controller>().rt().spawn(async move {
            let devices = crate::client::tree_get_entries(Some("upnp://".to_string()))
                .await
                .unwrap_or_default();
            let _ = tx.send(devices);
        });
        cx.spawn(async move |this: WeakEntity<FilesView>, cx| {
            if let Ok(devices) = rx.await {
                let _ = this.update(cx, |this, _cx| {
                    this.upnp_cache.insert(
                        "upnp://".to_string(),
                        CacheEntry { entries: devices, fetched_at: Instant::now() },
                    );
                });
            }
        })
        .detach();

        FilesView {
            entries: Vec::new(),
            last_loaded: None,
            loading: false,
            upnp_cache: HashMap::new(),
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

        // Determine if this is a UPnP path and its cache key.
        let cache_key: Option<String> = match browse.mode {
            FilesMode::UpnpDevices => Some("upnp://".to_string()),
            FilesMode::UpnpBrowse => browse.current_path.clone(),
            _ => None,
        };
        let ttl = match browse.mode {
            FilesMode::UpnpDevices => UPNP_DEVICE_TTL,
            _ => UPNP_CONTENT_TTL,
        };

        if let Some(ref ck) = cache_key {
            let cached_entries = self.upnp_cache.get(ck).filter(|c| !c.entries.is_empty());
            if let Some(cached) = cached_entries {
                let fresh = cached.fetched_at.elapsed() < ttl;
                // Show cached entries immediately — no spinner.
                self.entries = cached.entries.clone();
                self.loading = false;
                self.last_loaded = Some(key.clone());
                cx.notify();

                if fresh {
                    // Cache is warm — nothing more to do.
                    return;
                }
                // Cache is stale: silent background refresh (stale-while-revalidate).
                self.spawn_fetch(cx, browse.current_path.clone(), key, Some(ck.clone()));
                return;
            }
            // Cache miss (or empty result from a previous failed fetch): show
            // loading spinner and fetch.
            self.entries.clear();
            self.loading = true;
            self.last_loaded = Some(key.clone());
            cx.notify();
            self.spawn_fetch(cx, browse.current_path.clone(), key, Some(ck.clone()));
            return;
        }

        // Local filesystem — no caching, plain fetch with spinner.
        self.entries.clear();
        self.loading = true;
        self.last_loaded = Some(key.clone());
        cx.notify();
        self.spawn_fetch(cx, browse.current_path.clone(), key, None);
    }

    fn spawn_fetch(
        &self,
        cx: &mut Context<Self>,
        path: Option<String>,
        key: (FilesMode, Option<String>),
        cache_key: Option<String>,
    ) {
        let (tx, rx) = tokio::sync::oneshot::channel::<Vec<FileEntry>>();
        cx.global::<Controller>().rt().spawn(async move {
            let entries = crate::client::tree_get_entries(path).await.unwrap_or_default();
            let _ = tx.send(entries);
        });
        cx.spawn(async move |this: WeakEntity<FilesView>, cx| {
            if let Ok(entries) = rx.await {
                let _ = this.update(cx, |this, cx| {
                    // Always update the cache.
                    if let Some(ref ck) = cache_key {
                        this.upnp_cache.insert(
                            ck.clone(),
                            CacheEntry { entries: entries.clone(), fetched_at: Instant::now() },
                        );
                    }
                    // Only update the displayed entries if the user is still
                    // on the page that triggered this fetch.
                    let current_key = {
                        let browse = cx.global::<FilesBrowseState>();
                        (browse.mode.clone(), browse.current_path.clone())
                    };
                    if current_key == key {
                        this.entries = entries;
                        this.loading = false;
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
        let loading = self.loading;

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
                _ if loading => render_loading(theme).into_any_element(),
                _ => render_entries(entries, current_dir, mode, theme).into_any_element(),
            })
    }
}

fn render_loading(theme: Theme) -> AnyElement {
    div()
        .flex_1()
        .min_h_0()
        .flex()
        .items_center()
        .justify_center()
        .child(
            div()
                .text_sm()
                .text_color(theme.library_header_text)
                .child("Loading…"),
        )
        .into_any_element()
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
