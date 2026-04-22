use super::Page;
use crate::ui::theme::Theme;
use gpui::prelude::FluentBuilder;
use gpui::{
    Animation, AnimationExt as _, Context, ElementId, FontWeight, InteractiveElement,
    IntoElement, ParentElement, Render, StatefulInteractiveElement, Styled, Window, div, px,
};

#[derive(Clone)]
pub struct NavBar;

impl NavBar {
    pub fn new() -> Self {
        NavBar {}
    }
}

impl Render for NavBar {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let page = *cx.global::<Page>();

        let active_offset = match page {
            Page::Library => 0.0,
            Page::Player => 96.0,
            Page::Queue => 192.0,
        };

        div()
            .flex()
            .w_auto()
            .h_full()
            .items_center()
            .justify_center()
            .relative()
            .child({
                let tab_state = window
                    .use_keyed_state("navbar_tab", cx, |_, _| (page, active_offset));

                let (prev_page, prev_offset) = *tab_state.read(cx);
                let duration = std::time::Duration::from_millis(200);

                div()
                    .id("active_tab_bg")
                    .absolute()
                    .top_1()
                    .bottom_1()
                    .w_24()
                    .rounded_lg()
                    .bg(theme.switcher_active)
                    .map(move |this| {
                        if prev_page == page {
                            this.left(px(active_offset)).into_any_element()
                        } else {
                            cx.spawn({
                                let tab_state = tab_state.clone();
                                async move |_, cx| {
                                    cx.background_executor().timer(duration).await;
                                    let _ = tab_state.update(cx, |state, _| {
                                        *state = (page, active_offset);
                                    });
                                }
                            })
                            .detach();

                            this.with_animation(
                                ElementId::NamedInteger("tab_move".into(), page as u64),
                                Animation::new(duration).with_easing(gpui::ease_out_quint()),
                                move |this, delta| {
                                    let x = prev_offset + (active_offset - prev_offset) * delta;
                                    this.left(px(x))
                                },
                            )
                            .into_any_element()
                        }
                    })
            })
            .child(
                div()
                    .id("library_tab")
                    .h_full()
                    .w_24()
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_sm()
                    .rounded_lg()
                    .text_color(theme.switcher_text)
                    .font_weight(FontWeight::MEDIUM)
                    .hover(|this| {
                        if page == Page::Library {
                            this
                        } else {
                            this.text_color(theme.switcher_text_hover)
                        }
                    })
                    .on_click(|_, _, cx| *cx.global_mut::<Page>() = Page::Library)
                    .when(page == Page::Library, |this| {
                        this.text_color(theme.switcher_text_active)
                    })
                    .child("Library"),
            )
            .child(
                div()
                    .id("player_tab")
                    .h_full()
                    .w_24()
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_sm()
                    .rounded_lg()
                    .text_color(theme.switcher_text)
                    .font_weight(FontWeight::MEDIUM)
                    .hover(|this| {
                        if page == Page::Player {
                            this
                        } else {
                            this.text_color(theme.switcher_text_hover)
                        }
                    })
                    .on_click(|_, _, cx| *cx.global_mut::<Page>() = Page::Player)
                    .when(page == Page::Player, |this| {
                        this.text_color(theme.switcher_text_active)
                    })
                    .child("Player"),
            )
            .child(
                div()
                    .id("queue_tab")
                    .h_full()
                    .w_24()
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_sm()
                    .rounded_lg()
                    .text_color(theme.switcher_text)
                    .font_weight(FontWeight::MEDIUM)
                    .hover(|this| {
                        if page == Page::Queue {
                            this
                        } else {
                            this.text_color(theme.switcher_text_hover)
                        }
                    })
                    .on_click(|_, _, cx| *cx.global_mut::<Page>() = Page::Queue)
                    .when(page == Page::Queue, |this| {
                        this.text_color(theme.switcher_text_active)
                    })
                    .child("Queue"),
            )
    }
}
