use gpui::{
    div, px, rgb, Animation, AnimationExt as _, AnyElement, IntoElement, ParentElement,
    SharedString, Styled,
};
use std::f32::consts::PI;
use std::time::Duration;

/// Animated equalizer bars — shown on the currently playing track row.
/// Three bars in primary colors (red / blue / green) bounce at staggered
/// speeds when `is_playing` is true; static mid-height bars when paused.
pub fn equalizer_bars(id: usize, is_playing: bool) -> impl IntoElement {
    const COLOR: u32 = 0x6F00FF;
    const DURATIONS_MS: [u64; 3] = [500, 700, 600];
    const STATIC_H: [f32; 3] = [5.0, 9.0, 7.0];

    let make_bar = move |bar_idx: usize| -> AnyElement {
        let b = div().w(px(3.0)).rounded(px(1.5)).bg(rgb(COLOR));
        if is_playing {
            let anim_id: SharedString = format!("eq-{bar_idx}-{id}").into();
            b.with_animation(
                anim_id,
                Animation::new(Duration::from_millis(DURATIONS_MS[bar_idx])).repeat(),
                |this, delta: f32| {
                    let h = 3.0 + 10.0 * (delta * PI).sin();
                    this.h(px(h))
                },
            )
            .into_any_element()
        } else {
            b.h(px(STATIC_H[bar_idx])).into_any_element()
        }
    };

    div()
        .flex()
        .items_end()
        .gap(px(2.0))
        .w(px(14.0))
        .h(px(16.0))
        .child(make_bar(0))
        .child(make_bar(1))
        .child(make_bar(2))
}

pub fn ease_in_out_expo() -> impl Fn(f32) -> f32 {
    |t: f32| {
        if t == 0.0 {
            return 0.0;
        }
        if t == 1.0 {
            return 1.0;
        }
        if t < 0.5 {
            2.0_f32.powf(20.0 * t - 10.0) / 2.0
        } else {
            (2.0 - 2.0_f32.powf(-20.0 * t + 10.0)) / 2.0
        }
    }
}
