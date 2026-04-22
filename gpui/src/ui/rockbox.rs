use crate::ui::animations::ease_in_out_expo;
use crate::ui::components::controlbar::ControlBar;
use crate::ui::components::pages::{library::LibraryPage, player::PlayerPage, queue::QueuePage};
use crate::ui::components::titlebar::Titlebar;
use crate::ui::components::Page;
use crate::ui::global_keybinds;
use crate::ui::theme::Theme;
use gpui::prelude::FluentBuilder;
use gpui::{
    div, px, Animation, AnimationExt as _, AppContext, Context, ElementId, Entity,
    InteractiveElement, IntoElement, ParentElement, Render, Styled, Window,
};

pub struct Rockbox {
    pub titlebar: Entity<Titlebar>,
    pub player_page: Entity<PlayerPage>,
    pub library_page: Entity<LibraryPage>,
    pub queue_page: Entity<QueuePage>,
}

impl Rockbox {
    pub fn new(cx: &mut Context<Self>) -> Self {
        cx.set_global(Theme::default());
        cx.set_global(Page::Player);
        global_keybinds::register_keybinds(cx);
        let titlebar = cx.new(|cx| Titlebar::new(cx));
        let controlbar = cx.new(|_| ControlBar);
        let player_page = cx.new(|cx| PlayerPage::new(cx, controlbar));
        let library_page = cx.new(|cx| LibraryPage::new(cx));
        let queue_page = cx.new(|cx| QueuePage::new(cx));
        Rockbox {
            titlebar,
            player_page,
            library_page,
            queue_page,
        }
    }
}

impl Render for Rockbox {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let page = *cx.global::<Page>();
        let page_state = window.use_keyed_state("page_transition", cx, |_, _| page);
        let prev_page = *page_state.read(cx);
        let direction: f32 = match (prev_page, page) {
            (Page::Library, Page::Player) | (Page::Player, Page::Queue) => 1.0,
            (Page::Queue, Page::Player) | (Page::Player, Page::Library) => -1.0,
            _ => 0.0,
        };
        let page_el = match page {
            Page::Player => div().w_full().h_full().min_h_0().child(self.player_page.clone()),
            Page::Library => div().w_full().h_full().min_h_0().child(self.library_page.clone()),
            Page::Queue => div().w_full().h_full().min_h_0().child(self.queue_page.clone()),
        };
        div()
            .id("root")
            .size_full()
            .font_family("Space Grotesk")
            .relative()
            .flex()
            .flex_col()
            .bg(theme.app_bg)
            .child(self.titlebar.clone())
            .child(
                div()
                    .id("page_container")
                    .w_full()
                    .flex_1()
                    .min_h_0()
                    .overflow_hidden()
                    .map(move |this| {
                        if prev_page == page {
                            this.child(page_el).into_any_element()
                        } else {
                            let duration = std::time::Duration::from_millis(250);
                            cx.spawn({
                                let page_state = page_state.clone();
                                async move |_, cx| {
                                    cx.background_executor().timer(duration).await;
                                    let _ = page_state.update(cx, |state, _| {
                                        *state = page;
                                    });
                                }
                            })
                            .detach();
                            this.child(page_el)
                                .with_animation(
                                    ElementId::NamedInteger("page_slide".into(), page as u64),
                                    Animation::new(duration).with_easing(ease_in_out_expo()),
                                    move |this, delta| {
                                        let offset = 360.0 * direction * (1.0 - delta);
                                        this.left(px(offset)).opacity(delta)
                                    },
                                )
                                .into_any_element()
                        }
                    }),
            )
    }
}
