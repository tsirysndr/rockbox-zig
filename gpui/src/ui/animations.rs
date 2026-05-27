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

/// A single skeleton "pill" (fully-rounded ends) that pulses between dim and bright.
/// Use for text-like placeholders (title, artist, duration rows).
pub fn skeleton_pill(id: impl Into<SharedString>, w: f32, h: f32, color: u32) -> AnyElement {
    div()
        .w(px(w))
        .h(px(h))
        .rounded(px(h / 2.0))
        .bg(rgb(color))
        .with_animation(
            id.into(),
            Animation::new(Duration::from_millis(1200)).repeat(),
            move |this, t: f32| {
                let alpha = 0.25 + 0.35 * (t * PI).sin();
                this.opacity(alpha)
            },
        )
        .into_any_element()
}

/// A skeleton rectangle with a fixed corner radius (e.g. `rounded_lg` = 8px).
/// Use for square/rectangular placeholders like album art thumbnails.
pub fn skeleton_rect(
    id: impl Into<SharedString>,
    w: f32,
    h: f32,
    corner_px: f32,
    color: u32,
) -> AnyElement {
    div()
        .w(px(w))
        .h(px(h))
        .rounded(px(corner_px))
        .bg(rgb(color))
        .with_animation(
            id.into(),
            Animation::new(Duration::from_millis(1200)).repeat(),
            move |this, t: f32| {
                let alpha = 0.25 + 0.35 * (t * PI).sin();
                this.opacity(alpha)
            },
        )
        .into_any_element()
}

/// A skeleton circle (fully-rounded square). Use for artist avatar placeholders.
pub fn skeleton_circle(id: impl Into<SharedString>, size: f32, color: u32) -> AnyElement {
    skeleton_pill(id, size, size, color)
}

/// Full skeleton loading layout for a list of N track rows.
pub fn skeleton_track_list(count: usize, color: u32) -> impl IntoElement {
    div()
        .w_full()
        .flex()
        .flex_col()
        .children((0..count).map(move |i| {
            div()
                .w_full()
                .flex()
                .items_center()
                .gap_x_4()
                .px_6()
                .py_3()
                .child(skeleton_pill(format!("sk_num_{i}"), 20.0, 12.0, color))
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .flex()
                        .flex_col()
                        .gap_y_1()
                        .child(skeleton_pill(
                            format!("sk_title_{i}"),
                            200.0 - (i % 3) as f32 * 30.0,
                            12.0,
                            color,
                        ))
                        .child(skeleton_pill(
                            format!("sk_artist_{i}"),
                            120.0 - (i % 2) as f32 * 20.0,
                            10.0,
                            color,
                        )),
                )
                .child(skeleton_pill(format!("sk_dur_{i}"), 36.0, 12.0, color))
        }))
}

/// Skeleton grid of album cards — art is a rounded_lg square (8 px corners), not a circle.
pub fn skeleton_album_grid(cols: usize, rows: usize, color: u32) -> impl IntoElement {
    div()
        .w_full()
        .p_6()
        .flex()
        .flex_col()
        .gap_y_8()
        .children((0..rows).map(move |row| {
            div()
                .w_full()
                .flex()
                .items_start()
                .gap_x_4()
                .children((0..cols).map(move |col| {
                    let idx = row * cols + col;
                    div()
                        .w(px(150.0))
                        .flex_shrink_0()
                        .flex()
                        .flex_col()
                        .gap_y_2()
                        // rounded_lg square (matches actual album card)
                        .child(skeleton_rect(
                            format!("sk_alb_art_{idx}"),
                            150.0,
                            150.0,
                            8.0,
                            color,
                        ))
                        .child(skeleton_pill(
                            format!("sk_alb_title_{idx}"),
                            110.0 - (idx % 3) as f32 * 10.0,
                            12.0,
                            color,
                        ))
                        .child(skeleton_pill(
                            format!("sk_alb_artist_{idx}"),
                            80.0 - (idx % 2) as f32 * 10.0,
                            10.0,
                            color,
                        ))
                }))
        }))
}

/// Skeleton grid of artist cards — avatar is a full circle (matches actual artist card).
pub fn skeleton_artist_grid(cols: usize, rows: usize, color: u32) -> impl IntoElement {
    div()
        .w_full()
        .p_6()
        .flex()
        .flex_col()
        .gap_y_8()
        .children((0..rows).map(move |row| {
            div()
                .w_full()
                .flex()
                .items_start()
                .gap_x_4()
                .children((0..cols).map(move |col| {
                    let idx = row * cols + col;
                    div()
                        .w(px(120.0))
                        .flex_shrink_0()
                        .flex()
                        .flex_col()
                        .items_center()
                        .gap_y_2()
                        // full circle (matches actual artist avatar)
                        .child(skeleton_circle(format!("sk_art_av_{idx}"), 120.0, color))
                        .child(skeleton_pill(
                            format!("sk_art_name_{idx}"),
                            80.0 - (idx % 3) as f32 * 8.0,
                            12.0,
                            color,
                        ))
                        .child(skeleton_pill(
                            format!("sk_art_count_{idx}"),
                            55.0 - (idx % 2) as f32 * 8.0,
                            10.0,
                            color,
                        ))
                }))
        }))
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
