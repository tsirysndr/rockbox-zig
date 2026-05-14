use crate::ui::theme::Theme;
use gpui::{
    div, App, ClipboardItem, Context, FocusHandle, InteractiveElement, IntoElement, KeyDownEvent,
    ParentElement, Render, StatefulInteractiveElement, Styled, Subscription, Window,
};

pub struct TextInput {
    pub value: String,
    pub placeholder: String,
    pub masked: bool,
    pub focus_handle: FocusHandle,
    _focus_out_sub: Option<Subscription>,
}

impl TextInput {
    pub fn new(placeholder: impl Into<String>, cx: &mut App) -> Self {
        TextInput {
            value: String::new(),
            placeholder: placeholder.into(),
            masked: false,
            focus_handle: cx.focus_handle(),
            _focus_out_sub: None,
        }
    }

    pub fn masked(mut self) -> Self {
        self.masked = true;
        self
    }
}

impl Render for TextInput {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let is_focused = self.focus_handle.is_focused(window);

        if self._focus_out_sub.is_none() {
            let handle = self.focus_handle.clone();
            self._focus_out_sub = Some(cx.on_focus_out(&handle, window, |_, _, _, cx| {
                cx.notify();
            }));
        }

        let display = if self.value.is_empty() && !is_focused {
            self.placeholder.clone()
        } else if self.masked {
            "•".repeat(self.value.chars().count())
        } else {
            self.value.clone()
        };
        let text_color = if self.value.is_empty() && !is_focused {
            theme.library_header_text
        } else {
            theme.library_text
        };

        div()
            .id("text_input_box")
            .key_context("TextInput")
            .track_focus(&self.focus_handle)
            .on_click(cx.listener(|this, _, window, _cx| {
                window.focus(&this.focus_handle);
            }))
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, _window, cx| {
                let key = event.keystroke.key.as_str();
                let cmd = event.keystroke.modifiers.platform;
                let ctrl = event.keystroke.modifiers.control;
                if key == "backspace" {
                    this.value.pop();
                    cx.notify();
                } else if key == "escape" {
                    this.value.clear();
                    cx.notify();
                } else if (cmd || ctrl) && key == "v" {
                    if let Some(item) = cx.read_from_clipboard() {
                        if let Some(text) = item.text() {
                            this.value.push_str(&text);
                            cx.notify();
                        }
                    }
                } else if (cmd || ctrl) && key == "a" {
                    // select-all: no cursor support, just a no-op so the key isn't swallowed
                } else if (cmd || ctrl) && key == "c" && !this.masked {
                    cx.write_to_clipboard(ClipboardItem::new_string(this.value.clone()));
                } else if !cmd && !ctrl {
                    if let Some(c) = &event.keystroke.key_char {
                        this.value.push_str(c);
                        cx.notify();
                    }
                }
            }))
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
            .cursor_pointer()
            .text_sm()
            .text_color(text_color)
            .child(if is_focused {
                format!("{display}|")
            } else {
                display
            })
    }
}
