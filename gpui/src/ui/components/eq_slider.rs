use crate::ui::components::EqSliderDrag;
use gpui::{
    px, App, BorderStyle, Bounds, CursorStyle, DispatchPhase, Element, ElementId, GlobalElementId,
    Hitbox, HitboxBehavior, InspectorElementId, IntoElement, LayoutId, MouseButton, MouseDownEvent,
    MouseMoveEvent, MouseUpEvent, Pixels, Window,
};
use std::sync::Arc;

/// Vertical EQ band slider. `gain_fraction` is 0.0 (min gain) to 1.0 (max gain),
/// with 0.5 representing 0 dB.
pub struct EqSlider {
    id: ElementId,
    gain_fraction: f32,
    track_color: gpui::Rgba,
    positive_color: gpui::Rgba,
    negative_color: gpui::Rgba,
    on_change: Option<Box<dyn Fn(f32, &mut Window, &mut App) + 'static>>,
}

impl EqSlider {
    pub fn new(
        id: impl Into<ElementId>,
        gain_fraction: f32,
        track_color: gpui::Rgba,
        positive_color: gpui::Rgba,
        negative_color: gpui::Rgba,
    ) -> Self {
        EqSlider {
            id: id.into(),
            gain_fraction: gain_fraction.clamp(0.0, 1.0),
            track_color,
            positive_color,
            negative_color,
            on_change: None,
        }
    }

    pub fn on_change(mut self, f: impl Fn(f32, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Box::new(f));
        self
    }
}

impl IntoElement for EqSlider {
    type Element = Self;
    fn into_element(self) -> Self {
        self
    }
}

pub struct EqSliderPrepaint {
    hitbox: Hitbox,
}

impl Element for EqSlider {
    type RequestLayoutState = LayoutId;
    type PrepaintState = EqSliderPrepaint;

    fn id(&self) -> Option<ElementId> {
        Some(self.id.clone())
    }

    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, LayoutId) {
        let mut style = gpui::Style::default();
        style.size.width = px(36.0).into();
        style.size.height = px(160.0).into();
        let layout_id = window.request_layout(style, [], cx);
        (layout_id, layout_id)
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _layout: &mut LayoutId,
        window: &mut Window,
        _cx: &mut App,
    ) -> EqSliderPrepaint {
        EqSliderPrepaint {
            hitbox: window.insert_hitbox(bounds, HitboxBehavior::Normal),
        }
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _layout: &mut LayoutId,
        prepaint: &mut EqSliderPrepaint,
        window: &mut Window,
        _cx: &mut App,
    ) {
        let track_color = self.track_color;
        let positive_color = self.positive_color;
        let negative_color = self.negative_color;
        let gain_fraction = self.gain_fraction.clamp(0.0, 1.0);
        let radius = px(6.0);

        // Track background
        window.paint_quad(gpui::PaintQuad {
            bounds,
            corner_radii: gpui::Corners::all(radius),
            background: track_color.into(),
            border_widths: gpui::Edges::default(),
            border_color: gpui::transparent_black(),
            border_style: BorderStyle::default(),
        });

        let center_y = bounds.origin.y + bounds.size.height * 0.5;
        let thumb_y = bounds.origin.y + bounds.size.height * (1.0 - gain_fraction);
        let fill_color = if gain_fraction >= 0.5 {
            positive_color
        } else {
            negative_color
        };

        // Fill from center to thumb
        let (fill_top, fill_height) = if gain_fraction > 0.5 {
            (thumb_y, center_y - thumb_y)
        } else {
            (center_y, thumb_y - center_y)
        };

        if fill_height > px(1.0) {
            window.paint_quad(gpui::PaintQuad {
                bounds: Bounds {
                    origin: gpui::Point {
                        x: bounds.origin.x + px(10.0),
                        y: fill_top,
                    },
                    size: gpui::Size {
                        width: bounds.size.width - px(20.0),
                        height: fill_height,
                    },
                },
                corner_radii: gpui::Corners::all(px(2.0)),
                background: fill_color.into(),
                border_widths: gpui::Edges::default(),
                border_color: gpui::transparent_black(),
                border_style: BorderStyle::default(),
            });
        }

        // Center (0 dB) line
        window.paint_quad(gpui::PaintQuad {
            bounds: Bounds {
                origin: gpui::Point {
                    x: bounds.origin.x + px(4.0),
                    y: center_y - px(0.5),
                },
                size: gpui::Size {
                    width: bounds.size.width - px(8.0),
                    height: px(1.0),
                },
            },
            corner_radii: gpui::Corners::all(px(0.0)),
            background: gpui::rgba(0xFFFFFF40).into(),
            border_widths: gpui::Edges::default(),
            border_color: gpui::transparent_black(),
            border_style: BorderStyle::default(),
        });

        // Thumb
        window.paint_quad(gpui::PaintQuad {
            bounds: Bounds {
                origin: gpui::Point {
                    x: bounds.origin.x + px(5.0),
                    y: thumb_y - px(6.0),
                },
                size: gpui::Size {
                    width: bounds.size.width - px(10.0),
                    height: px(12.0),
                },
            },
            corner_radii: gpui::Corners::all(px(6.0)),
            background: fill_color.into(),
            border_widths: gpui::Edges::default(),
            border_color: gpui::transparent_black(),
            border_style: BorderStyle::default(),
        });

        if prepaint.hitbox.is_hovered(window) {
            window.set_cursor_style(CursorStyle::PointingHand, &prepaint.hitbox);
        }

        if let Some(on_change) = self.on_change.take() {
            let on_change: Arc<dyn Fn(f32, &mut Window, &mut App) + 'static> =
                Arc::from(on_change);
            let hitbox = prepaint.hitbox.clone();
            // Use origin_x as the unique identifier — all sliders share the same origin_y
            // (same horizontal row) but each has a distinct x position.
            let origin_x = bounds.origin.x;
            let origin_y = bounds.origin.y;
            let height = bounds.size.height;

            let fraction_at = move |pos_y: Pixels| -> f32 {
                let rel_y = f32::from(pos_y - origin_y);
                let h = f32::from(height);
                if h > 0.0 { (1.0 - rel_y / h).clamp(0.0, 1.0) } else { 0.5 }
            };

            // Mouse-down: immediate update + start drag
            {
                let on_change = on_change.clone();
                let hitbox = hitbox.clone();
                window.on_mouse_event(move |event: &MouseDownEvent, phase, window, cx| {
                    if phase == DispatchPhase::Bubble && hitbox.is_hovered(window) {
                        on_change(fraction_at(event.position.y), window, cx);
                        cx.global_mut::<EqSliderDrag>().active_origin_x = Some(origin_x);
                        cx.stop_propagation();
                    }
                });
            }

            // Mouse-move: update while this slider is the active drag target
            {
                let on_change = on_change.clone();
                window.on_mouse_event(move |event: &MouseMoveEvent, phase, window, cx| {
                    if phase == DispatchPhase::Bubble
                        && event.pressed_button == Some(MouseButton::Left)
                        && cx.global::<EqSliderDrag>().active_origin_x == Some(origin_x)
                    {
                        on_change(fraction_at(event.position.y), window, cx);
                    }
                });
            }

            // Mouse-up: end drag
            window.on_mouse_event(move |_event: &MouseUpEvent, phase, _window, cx| {
                if phase == DispatchPhase::Bubble
                    && cx.global::<EqSliderDrag>().active_origin_x == Some(origin_x)
                {
                    cx.global_mut::<EqSliderDrag>().active_origin_x = None;
                }
            });
        }
    }
}

/// Convert Rockbox EQ gain (tenths of dB, -240..=240) to slider fraction (0.0–1.0).
/// 0.0 = -24.0 dB, 0.5 = 0 dB, 1.0 = +24.0 dB.
pub fn gain_to_fraction(gain: i32) -> f32 {
    let clamped = gain.clamp(-240, 240);
    (clamped + 240) as f32 / 480.0
}

/// Convert slider fraction back to Rockbox EQ gain (tenths of dB, -240..=240).
pub fn fraction_to_gain(fraction: f32) -> i32 {
    (fraction * 480.0 - 240.0).round() as i32
}
