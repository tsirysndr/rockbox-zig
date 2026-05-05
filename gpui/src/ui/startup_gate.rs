use crate::controller::Controller;
use crate::state::AppState;
use crate::ui::rockbox::Rockbox;
use crate::ui::theme::Theme;
use gpui::{
    div, px, App, AppContext, Context, Entity, FontWeight, IntoElement, ParentElement, Render,
    Styled, WeakEntity, Window,
};

pub struct StartupGate {
    /// Populated once the embedded daemon has started.
    rockbox: Option<Entity<Rockbox>>,
    /// Set if rb_daemon_start() returned a negative code.
    error: Option<String>,
}

impl StartupGate {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let gate = StartupGate {
            rockbox: None,
            error: None,
        };
        // Check if gRPC is already reachable; if so skip the daemon boot.
        // Otherwise start the embedded daemon (blocks up to 30 s).
        cx.spawn(async move |this: WeakEntity<StartupGate>, cx| {
            let result = cx
                .background_executor()
                .spawn(async { crate::startup::ensure_running() })
                .await;

            let _ = this.update(cx, |this, cx| {
                if result > 0 {
                    let state = cx.new(|_| AppState::new());
                    let controller = Controller::new(state, cx);
                    cx.set_global(controller);
                    this.rockbox = Some(cx.new(Rockbox::new));
                } else {
                    this.error = Some(format!(
                        "Daemon start failed (code {result}). \
                         Check that build-headless/ is present and zig build lib has run."
                    ));
                }
                cx.notify();
            });
        })
        .detach();
        gate
    }
}

impl Render for StartupGate {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Happy path — transparent pass-through once the daemon is up.
        if let Some(rockbox) = &self.rockbox {
            return rockbox.clone().into_any_element();
        }

        let theme = *cx.global::<Theme>();

        let (title, subtitle) = if let Some(err) = &self.error {
            ("Failed to start daemon", err.as_str())
        } else {
            ("Starting Rockbox…", "Connecting to audio engine, please wait.")
        };

        div()
            .size_full()
            .flex()
            .items_center()
            .justify_center()
            .bg(theme.app_bg)
            .child(
                div()
                    .w(px(440.0))
                    .flex()
                    .flex_col()
                    .gap_y_4()
                    .p(px(36.0))
                    .rounded_xl()
                    .bg(theme.titlebar_bg)
                    .border_1()
                    .border_color(theme.border)
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
                    )
                    .when(self.error.is_some(), |this| {
                        this.child(
                            div()
                                .id("startup-quit")
                                .w(px(80.0))
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
                    }),
            )
            .into_any_element()
    }
}
