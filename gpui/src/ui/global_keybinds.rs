use crate::controller::Controller;
use crate::state::{AppState, PlaybackStatus};
use crate::ui::components::Page;
use gpui::{actions, App, Entity, KeyBinding};

actions!(player, [PlayPause, Next, Prev, Shuffle, Repeat]);
actions!(pages, [CycleNext, CyclePrev, Library, Player, Queue]);
actions!(app, [Quit, Hide, HideOthers]);

pub fn register_keybinds(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("space", PlayPause, Some("! SearchInput && ! TextInput")),
        KeyBinding::new("cmd-right", Next, None),
        KeyBinding::new("cmd-left", Prev, None),
        KeyBinding::new("shift-s", Shuffle, None),
        KeyBinding::new("shift-r", Repeat, None),
        KeyBinding::new("cmd-tab", CycleNext, None),
        KeyBinding::new("cmd-shift-tab", CyclePrev, None),
        KeyBinding::new("cmd-1", Library, None),
        KeyBinding::new("cmd-2", Player, None),
        KeyBinding::new("cmd-3", Queue, None),
        KeyBinding::new("cmd-q", Quit, None),
        KeyBinding::new("cmd-h", Hide, None),
        KeyBinding::new("cmd-alt-h", HideOthers, None),
    ]);

    cx.on_action(|_: &PlayPause, cx| {
        play_pause(cx);
    });

    cx.on_action(|_: &Next, cx| {
        cx.global::<Controller>().next();
    });

    cx.on_action(|_: &Prev, cx| {
        cx.global::<Controller>().prev();
    });

    cx.on_action(|_: &Shuffle, cx| {
        let state = cx.global::<Controller>().state.clone();
        state.update(cx, |s, cx| {
            s.toggle_shuffle();
            cx.notify();
        });
    });

    cx.on_action(|_: &Repeat, cx| {
        let state = cx.global::<Controller>().state.clone();
        state.update(cx, |s, cx| {
            s.toggle_repeat();
            cx.notify();
        });
    });

    cx.on_action(|_: &Quit, cx| cx.quit());
    cx.on_action(|_: &Hide, cx| cx.hide());
    cx.on_action(|_: &HideOthers, cx| cx.hide_other_apps());

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

pub fn play_pause(cx: &mut App) {
    let (rt, state): (tokio::runtime::Handle, Entity<AppState>) = {
        let ctrl = cx.global::<Controller>();
        (ctrl.rt(), ctrl.state.clone())
    };
    let status = state.read(cx).status;
    let lock = std::time::Duration::from_secs(2);
    match status {
        PlaybackStatus::Playing => {
            rt.spawn(crate::client::pause());
            state.update(cx, |s, cx| {
                s.set_status_local(PlaybackStatus::Paused, lock);
                cx.notify();
            });
        }
        PlaybackStatus::Paused => {
            rt.spawn(crate::client::resume());
            state.update(cx, |s, cx| {
                s.set_status_local(PlaybackStatus::Playing, lock);
                cx.notify();
            });
        }
        PlaybackStatus::Stopped => {
            // Daemon was (re)started — audio engine is stopped, not merely paused.
            // resume_track restores the playlist from the control file and seeks
            // to the saved position; plain resume() would be a no-op here.
            rt.spawn(crate::client::resume_track());
            state.update(cx, |s, cx| {
                s.set_status_local(PlaybackStatus::Playing, lock);
                cx.notify();
            });
        }
    }
}
