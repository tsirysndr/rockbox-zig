use crate::controller::Controller;
use crate::state::PlaybackStatus;
use crate::ui::components::Page;
use gpui::{actions, App, KeyBinding};

actions!(player, [PlayPause, Next, Prev, Shuffle, Repeat]);
actions!(pages, [CycleNext, CyclePrev, Library, Player, Queue]);

pub fn register_keybinds(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("space", PlayPause, Some("! SearchInput")),
        KeyBinding::new("cmd-right", Next, None),
        KeyBinding::new("cmd-left", Prev, None),
        KeyBinding::new("shift-s", Shuffle, None),
        KeyBinding::new("shift-r", Repeat, None),
        KeyBinding::new("cmd-tab", CycleNext, None),
        KeyBinding::new("cmd-shift-tab", CyclePrev, None),
        KeyBinding::new("cmd-1", Library, None),
        KeyBinding::new("cmd-2", Player, None),
        KeyBinding::new("cmd-3", Queue, None),
    ]);

    cx.on_action(|_: &PlayPause, cx| {
        let state = cx.global::<Controller>().state.clone();
        state.update(cx, |s, _| match s.status {
            PlaybackStatus::Playing => s.pause(),
            _ => s.play(),
        });
    });

    cx.on_action(|_: &Next, cx| {
        let state = cx.global::<Controller>().state.clone();
        state.update(cx, |s, _| s.next());
    });

    cx.on_action(|_: &Prev, cx| {
        let state = cx.global::<Controller>().state.clone();
        state.update(cx, |s, _| s.prev());
    });

    cx.on_action(|_: &Shuffle, cx| {
        let state = cx.global::<Controller>().state.clone();
        state.update(cx, |s, _| s.toggle_shuffle());
    });

    cx.on_action(|_: &Repeat, cx| {
        let state = cx.global::<Controller>().state.clone();
        state.update(cx, |s, _| s.toggle_repeat());
    });

    cx.on_action(|_: &Library, cx| *cx.global_mut::<Page>() = Page::Library);
    cx.on_action(|_: &Player, cx| *cx.global_mut::<Page>() = Page::Player);
    cx.on_action(|_: &Queue, cx| *cx.global_mut::<Page>() = Page::Queue);

    cx.on_action(|_: &CycleNext, cx| {
        *cx.global_mut::<Page>() = match *cx.global::<Page>() {
            Page::Library => Page::Player,
            Page::Player => Page::Queue,
            Page::Queue => Page::Library,
        };
    });

    cx.on_action(|_: &CyclePrev, cx| {
        *cx.global_mut::<Page>() = match *cx.global::<Page>() {
            Page::Library => Page::Queue,
            Page::Player => Page::Library,
            Page::Queue => Page::Player,
        };
    });
}
