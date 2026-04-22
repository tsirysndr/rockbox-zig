use super::navbar::NavBar;
use crate::ui::components::icons::{Icon, Icons};
use crate::ui::theme::Theme;
use gpui::{
    div, white, App, AppContext, Context, Entity, InteractiveElement, IntoElement, ParentElement,
    Render, Styled, Window, WindowControlArea,
};

#[derive(Clone)]
pub struct Titlebar {
    pub navbar: Entity<NavBar>,
}

impl Render for Titlebar {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();

        div()
            .id("titlebar")
            .h_12()
            .w_full()
            .flex()
            .items_center()
            .justify_center()
            .border_b_1()
            .border_color(theme.border)
            .bg(theme.titlebar_bg)
            .child(
                div()
                    .w_full()
                    .h_full()
                    .flex()
                    .items_center()
                    .justify_start()
                    .px_4()
                    .text_color(white())
                    .child(
                        div()
                            .id("drag_area_left")
                            .w_full()
                            .h_full()
                            .window_control_area(WindowControlArea::Drag),
                    ),
            )
            .child(
                div()
                    .w_full()
                    .h_full()
                    .flex()
                    .items_center()
                    .justify_center()
                    .px_4()
                    .py_1()
                    .child(
                        div()
                            .id("drag_area_center_left")
                            .w_full()
                            .h_full()
                            .window_control_area(WindowControlArea::Drag),
                    )
                    .child(self.navbar.clone())
                    .child(
                        div()
                            .id("drag_area_center_right")
                            .w_full()
                            .h_full()
                            .window_control_area(WindowControlArea::Drag),
                    ),
            )
            .child(
                div()
                    .w_full()
                    .h_full()
                    .flex()
                    .items_center()
                    .justify_end()
                    .px_4()
                    .child(
                        div()
                            .id("drag_area_right")
                            .w_full()
                            .h_full()
                            .window_control_area(WindowControlArea::Drag),
                    )
                    .child(
                        div()
                            .id("win_close_btn")
                            .h_8()
                            .w_8()
                            .rounded_full()
                            .flex()
                            .items_center()
                            .justify_center()
                            .hover(|this| this.bg(theme.titlebar_window_icons_bg_hover))
                            .text_color(theme.titlebar_window_icons_text)
                            .cursor_pointer()
                            .child(Icon::new(Icons::WinClose))
                            .window_control_area(WindowControlArea::Close),
                    ),
            )
    }
}

impl Titlebar {
    pub fn new(cx: &mut App) -> Titlebar {
        let navbar = cx.new(|_| NavBar::new());
        Titlebar { navbar }
    }
}
