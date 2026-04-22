use crate::ui::components::icons::{Icon, Icons};
use crate::ui::theme::Theme;
use gpui::{
    div, App, Context, FocusHandle, InteractiveElement, IntoElement, KeyDownEvent, ParentElement,
    Render, StatefulInteractiveElement, Styled, Subscription, Window,
};

pub struct SearchInput {
    pub query: String,
    pub focus_handle: FocusHandle,
    _focus_out_sub: Option<Subscription>,
}

impl SearchInput {
    pub fn new(cx: &mut App) -> Self {
        SearchInput {
            query: String::new(),
            focus_handle: cx.focus_handle(),
            _focus_out_sub: None,
        }
    }
}

impl Render for SearchInput {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let is_focused = self.focus_handle.is_focused(window);

        if self._focus_out_sub.is_none() {
            let handle = self.focus_handle.clone();
            self._focus_out_sub = Some(cx.on_focus_out(&handle, window, |_, _, _, cx| {
                cx.notify();
            }));
        }

        let display = if self.query.is_empty() && !is_focused {
            "Search...".to_string()
        } else {
            self.query.clone()
        };
        let text_color = if self.query.is_empty() && !is_focused {
            theme.library_header_text
        } else {
            theme.library_text
        };

        div()
            .id("search_input_box")
            .key_context("SearchInput")
            .track_focus(&self.focus_handle)
            .on_click(cx.listener(|this, _, window, _cx| {
                window.focus(&this.focus_handle);
            }))
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, _window, cx| {
                let key = event.keystroke.key.as_str();
                if key == "backspace" {
                    this.query.pop();
                    cx.notify();
                } else if key == "escape" {
                    this.query.clear();
                    cx.notify();
                } else if !event.keystroke.modifiers.platform && !event.keystroke.modifiers.control
                {
                    if let Some(c) = &event.keystroke.key_char {
                        this.query.push_str(c);
                        cx.notify();
                    }
                }
            }))
            .mx_3()
            .mb_4()
            .px_3()
            .py_2()
            .rounded_lg()
            .bg(theme.switcher_bg)
            .border_1()
            .border_color(if is_focused {
                theme.switcher_active
            } else {
                theme.border
            })
            .flex()
            .items_center()
            .gap_x_2()
            .cursor_pointer()
            .child(
                div()
                    .text_color(if is_focused {
                        theme.library_text
                    } else {
                        theme.library_header_text
                    })
                    .child(Icon::new(Icons::Search).size_3()),
            )
            .child(
                div()
                    .flex_1()
                    .text_sm()
                    .text_color(text_color)
                    .child(if is_focused {
                        format!("{display}|")
                    } else {
                        display
                    }),
            )
    }
}
