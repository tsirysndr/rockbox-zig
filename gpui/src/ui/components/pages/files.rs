use crate::client::{play_directory, play_directory_at, FileEntry};
use crate::controller::Controller;
use crate::ui::components::icons::{Icon, Icons};
use crate::ui::components::{FileContextMenu, FileContextMenuState, FilesBrowseState};
use crate::ui::theme::Theme;
use gpui::prelude::FluentBuilder;
use gpui::{
    div, px, uniform_list, App, ClickEvent, Context, FontWeight, InteractiveElement, IntoElement,
    ParentElement, Render, StatefulInteractiveElement, Styled, WeakEntity, Window,
};

pub struct FilesView {
    entries: Vec<FileEntry>,
    last_requested_path: Option<Option<String>>,
}

impl FilesView {
    pub fn new(cx: &mut App) -> Self {
        cx.set_global(FilesBrowseState::default());
        cx.set_global(FileContextMenuState::default());
        FilesView {
            entries: Vec::new(),
            last_requested_path: None,
        }
    }

    fn load_if_needed(&mut self, cx: &mut Context<Self>) {
        let current_path = cx.global::<FilesBrowseState>().current_path.clone();
        let needs_load = self
            .last_requested_path
            .as_ref()
            .map(|p| p != &current_path)
            .unwrap_or(true);
        if !needs_load {
            return;
        }
        self.last_requested_path = Some(current_path.clone());

        // Run the gRPC fetch on the Tokio runtime (requires a Tokio reactor).
        let (tx, rx) = tokio::sync::oneshot::channel::<Vec<FileEntry>>();
        cx.global::<Controller>().rt().spawn(async move {
            let entries = crate::client::tree_get_entries(current_path)
                .await
                .unwrap_or_default();
            let _ = tx.send(entries);
        });

        // Await the oneshot in GPUI's executor (no Tokio reactor needed for the
        // oneshot receiver itself), then push the result back into our entity.
        cx.spawn(async move |this: WeakEntity<FilesView>, cx| {
            if let Ok(entries) = rx.await {
                let _ = this.update(cx, |this, cx| {
                    this.entries = entries;
                    cx.notify();
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
        let entries = self.entries.clone();
        let can_go_back = !browse.path_history.is_empty();

        let path_display: String = browse
            .current_path
            .as_deref()
            .and_then(|p| std::path::Path::new(p).file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("Files")
            .to_string();

        let current_dir = browse.current_path.clone().unwrap_or_default();

        div()
            .size_full()
            .flex()
            .flex_col()
            // ── Header: back button + current path ────────────────────────────
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
                                        let state = cx.global_mut::<FilesBrowseState>();
                                        if let Some(prev) = state.path_history.pop() {
                                            state.current_path = prev;
                                        }
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
            // ── File list ─────────────────────────────────────────────────────
            .child(
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

                            div()
                                .id(("file_row", idx))
                                .group(group_name)
                                .w_full()
                                .flex()
                                .items_center()
                                .gap_x_3()
                                .px_4()
                                .py_2p5()
                                .cursor_pointer()
                                .hover(|t| t.bg(theme.library_track_bg_hover))
                                // Directory: click row to navigate in
                                .when(is_dir, |this| {
                                    this.on_click(move |_, _, cx: &mut App| {
                                        let state = cx.global_mut::<FilesBrowseState>();
                                        state.path_history.push(state.current_path.clone());
                                        state.current_path = Some(path_nav.clone());
                                    })
                                })
                                // ── Icon column (dir/music icon → play on hover) ──
                                .child(
                                    div()
                                        .w(px(22.0))
                                        .h(px(22.0))
                                        .flex_shrink_0()
                                        .relative()
                                        // Default icon (hidden on hover)
                                        .child(
                                            div()
                                                .absolute()
                                                .top_0()
                                                .left_0()
                                                .w_full()
                                                .h_full()
                                                .flex()
                                                .items_center()
                                                .group_hover(gn_icon, |s| s.opacity(0.0))
                                                .text_color(if is_dir {
                                                    theme.library_text
                                                } else {
                                                    theme.library_header_text
                                                })
                                                .child(
                                                    Icon::new(if is_dir {
                                                        Icons::Directory
                                                    } else {
                                                        Icons::Music
                                                    })
                                                    .size_5(),
                                                ),
                                        )
                                        // Play icon (visible on hover)
                                        .child(
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
                                                        rt.spawn(play_directory(
                                                            path_play.clone(),
                                                            false,
                                                        ));
                                                    } else {
                                                        rt.spawn(play_directory_at(
                                                            cur_dir_play.clone(),
                                                            idx as i32,
                                                        ));
                                                    }
                                                })
                                                .child(Icon::new(Icons::Play).size_5()),
                                        ),
                                )
                                // ── Name ─────────────────────────────────────
                                .child(
                                    div()
                                        .flex_1()
                                        .min_w_0()
                                        .text_sm()
                                        .truncate()
                                        .text_color(theme.library_text)
                                        .child(entry.name.clone()),
                                )
                                // ── Context menu button (⋮) ──────────────────
                                .child(
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
                        .collect()
                })
                .flex_1()
                .min_h_0(),
            )
    }
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
