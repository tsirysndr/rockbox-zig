use crate::controller::Controller;
use crate::startup::StartupError;
use crate::state::AppState;
use crate::ui::rockbox::Rockbox;
use crate::ui::theme::Theme;
use gpui::prelude::FluentBuilder;
use gpui::{
    actions, div, px, App, AppContext, ClipboardItem, Context, Entity, FontWeight,
    InteractiveElement, IntoElement, ParentElement, Render, StatefulInteractiveElement, Styled,
    WeakEntity, Window,
};

actions!(startup, [Retry, Quit]);

/// Which copy button (if any) is currently showing "Copied!".
#[derive(Clone, Copy, PartialEq, Eq)]
enum CopiedState {
    None,
    Main,
    Start,
}

pub struct StartupGate {
    error: Option<StartupError>,
    /// Populated once the startup check passes (either initially or on retry).
    rockbox: Option<Entity<Rockbox>>,
    /// True while a background retry check is in flight.
    checking: bool,
    copied: CopiedState,
}

impl StartupGate {
    pub fn new(cx: &mut Context<Self>) -> Self {
        match crate::startup::check() {
            None => Self::boot(cx),
            Some(error) => StartupGate {
                error: Some(error),
                rockbox: None,
                checking: false,
                copied: CopiedState::None,
            },
        }
    }

    fn boot(cx: &mut Context<Self>) -> Self {
        let state = cx.new(|_| AppState::new());
        let controller = Controller::new(state, cx);
        cx.set_global(controller);
        StartupGate {
            error: None,
            rockbox: Some(cx.new(Rockbox::new)),
            checking: false,
            copied: CopiedState::None,
        }
    }

    fn retry(&mut self, cx: &mut Context<Self>) {
        if self.checking {
            return;
        }
        self.checking = true;
        cx.notify();

        cx.spawn(async move |this: WeakEntity<StartupGate>, cx| {
            let result = cx
                .background_executor()
                .spawn(async { crate::startup::check() })
                .await;

            let _ = this.update(cx, |this, cx| {
                this.checking = false;
                match result {
                    None => {
                        let state = cx.new(|_| AppState::new());
                        let controller = Controller::new(state, cx);
                        cx.set_global(controller);
                        this.rockbox = Some(cx.new(Rockbox::new));
                        this.error = None;
                    }
                    Some(e) => {
                        this.error = Some(e);
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn copy_command(&mut self, text: &str, which: CopiedState, cx: &mut Context<Self>) {
        cx.write_to_clipboard(ClipboardItem::new_string(text.to_string()));
        self.copied = which;
        cx.notify();
        cx.spawn(async move |this: WeakEntity<StartupGate>, cx| {
            cx.background_executor()
                .timer(std::time::Duration::from_millis(1500))
                .await;
            let _ = this.update(cx, |this, cx| {
                // Only reset if this button is still the one showing "Copied!"
                if this.copied == which {
                    this.copied = CopiedState::None;
                    cx.notify();
                }
            });
        })
        .detach();
    }

    /// Renders a code block row: truncated monospace text + a per-button copy indicator.
    fn code_block(
        &self,
        id: &'static str,
        text: &'static str,
        which: CopiedState,
        theme: Theme,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let is_copied = self.copied == which;
        div()
            .w_full()
            .px(px(14.0))
            .py(px(10.0))
            .rounded_lg()
            .bg(theme.app_bg)
            .border_1()
            .border_color(theme.border)
            .flex()
            .items_center()
            .gap_x_3()
            // Text shrinks to make room for the button
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .truncate()
                    .text_sm()
                    .font_family("monospace")
                    .text_color(gpui::rgb(0xFFFFFF))
                    .child(text),
            )
            // Copy button — fixed width so it never gets squeezed
            .child(
                div()
                    .id(id)
                    .flex_shrink_0()
                    .px(px(10.0))
                    .py(px(4.0))
                    .rounded_md()
                    .text_xs()
                    .font_weight(FontWeight(500.0))
                    .cursor_pointer()
                    .text_color(if is_copied {
                        gpui::rgb(0x6F00FF)
                    } else {
                        theme.library_header_text
                    })
                    .bg(theme.titlebar_bg)
                    .border_1()
                    .border_color(theme.border)
                    .hover(|this| this.border_color(gpui::rgb(0x6F00FF)))
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.copy_command(text, which, cx);
                    }))
                    .child(if is_copied { "Copied!" } else { "Copy" }),
            )
    }
}

impl Render for StartupGate {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Happy path — render the full app transparently.
        if let Some(rockbox) = &self.rockbox {
            return rockbox.clone().into_any_element();
        }

        let theme = *cx.global::<Theme>();
        let checking = self.checking;

        let (title, subtitle, code) = match self.error {
            Some(StartupError::NotInstalled) | None => (
                "rockboxd is not installed",
                "Install the Rockbox CLI, then start the daemon:",
                "curl -fsSL https://raw.githubusercontent.com/tsirysndr/rockbox-zig/HEAD/install.sh | bash",
            ),
            Some(StartupError::NotRunning) => (
                "rockboxd is not running",
                "Start the Rockbox daemon by running:",
                "rockboxd",
            ),
        };

        let start_hint = matches!(self.error, Some(StartupError::NotInstalled));

        let main_code_block =
            self.code_block("copy-main-cmd", code, CopiedState::Main, theme, cx);
        let start_code_block =
            self.code_block("copy-start-cmd", "rockboxd", CopiedState::Start, theme, cx);

        div()
            .size_full()
            .flex()
            .items_center()
            .justify_center()
            .bg(theme.app_bg)
            .child(
                div()
                    .w(px(520.0))
                    .flex()
                    .flex_col()
                    .gap_y_5()
                    .p(px(36.0))
                    .rounded_xl()
                    .bg(theme.titlebar_bg)
                    .border_1()
                    .border_color(theme.border)
                    // Warning header
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_x_3()
                            .child(
                                div()
                                    .w(px(36.0))
                                    .h(px(36.0))
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .rounded_full()
                                    .bg(gpui::rgba(0x6F00FF20))
                                    .text_color(gpui::rgb(0x6F00FF))
                                    .text_xl()
                                    .font_weight(FontWeight::BOLD)
                                    .child("⚠"),
                            )
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_y_0p5()
                                    .child(
                                        div()
                                            .text_base()
                                            .font_weight(FontWeight::BOLD)
                                            .text_color(theme.library_text)
                                            .child(title),
                                    )
                                    .child(
                                        div()
                                            .text_sm()
                                            .text_color(theme.library_header_text)
                                            .child(subtitle),
                                    ),
                            ),
                    )
                    // Main command code block
                    .child(main_code_block)
                    // "Then start:" — NotInstalled only
                    .when(start_hint, |this| {
                        this.child(
                            div()
                                .flex()
                                .flex_col()
                                .gap_y_2()
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(theme.library_header_text)
                                        .child("Then start the daemon:"),
                                )
                                .child(start_code_block),
                        )
                    })
                    // Action buttons
                    .child(
                        div()
                            .flex()
                            .justify_end()
                            .gap_x_3()
                            .child(
                                div()
                                    .id("startup-quit")
                                    .px_5()
                                    .py_2()
                                    .rounded_lg()
                                    .text_sm()
                                    .font_weight(FontWeight(500.0))
                                    .text_color(theme.library_header_text)
                                    .bg(theme.app_bg)
                                    .border_1()
                                    .border_color(theme.border)
                                    .cursor_pointer()
                                    .hover(|this| this.bg(theme.border))
                                    .on_click(|_, _, cx: &mut App| cx.quit())
                                    .child("Quit"),
                            )
                            .child(
                                div()
                                    .id("startup-retry")
                                    .px_5()
                                    .py_2()
                                    .rounded_lg()
                                    .text_sm()
                                    .font_weight(FontWeight(500.0))
                                    .text_color(gpui::rgb(0xFFFFFF))
                                    .bg(gpui::rgb(0x6F00FF))
                                    .cursor_pointer()
                                    .hover(|this| this.bg(gpui::rgb(0x5A00D6)))
                                    .when(checking, |this| {
                                        this.opacity(0.6).cursor_default()
                                    })
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.retry(cx);
                                    }))
                                    .child(if checking { "Checking…" } else { "Retry" }),
                            ),
                    ),
            )
            .into_any_element()
    }
}
