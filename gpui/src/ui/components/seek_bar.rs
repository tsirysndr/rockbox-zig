use gpui::{
    px, App, BorderStyle, Bounds, CursorStyle, DispatchPhase, Element, ElementId, GlobalElementId,
    Hitbox, HitboxBehavior, InspectorElementId, IntoElement, LayoutId, MouseDownEvent, Pixels,
    Window,
};

/// A clickable horizontal seek bar.
///
/// Fills left-to-right according to `fill_fraction` (0.0–1.0).
/// Calls `on_seek(fraction, window, cx)` when the user presses anywhere on the track.
pub struct SeekBar {
    id: ElementId,
    fill_fraction: f32,
    track_color: gpui::Rgba,
    fill_color: gpui::Rgba,
    height: Pixels,
    on_seek: Option<Box<dyn Fn(f32, &mut Window, &mut App) + 'static>>,
}

impl SeekBar {
    pub fn new(
        id: impl Into<ElementId>,
        fill_fraction: f32,
        track_color: gpui::Rgba,
        fill_color: gpui::Rgba,
        height: Pixels,
    ) -> Self {
        SeekBar {
            id: id.into(),
            fill_fraction,
            track_color,
            fill_color,
            height,
            on_seek: None,
        }
    }

    pub fn on_seek(mut self, f: impl Fn(f32, &mut Window, &mut App) + 'static) -> Self {
        self.on_seek = Some(Box::new(f));
        self
    }
}

impl IntoElement for SeekBar {
    type Element = Self;
    fn into_element(self) -> Self {
        self
    }
}

pub struct SeekBarPrepaint {
    hitbox: Hitbox,
}

impl Element for SeekBar {
    type RequestLayoutState = LayoutId;
    type PrepaintState = SeekBarPrepaint;

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
        style.size.width = gpui::relative(1.).into();
        style.size.height = self.height.into();
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
    ) -> SeekBarPrepaint {
        let hitbox = window.insert_hitbox(bounds, HitboxBehavior::Normal);
        SeekBarPrepaint { hitbox }
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _layout: &mut LayoutId,
        prepaint: &mut SeekBarPrepaint,
        window: &mut Window,
        _cx: &mut App,
    ) {
        let track_color = self.track_color;
        let fill_color = self.fill_color;
        let fill_fraction = self.fill_fraction.clamp(0.0, 1.0);
        let radius = px(2.0);

        // Track background
        window.paint_quad(gpui::PaintQuad {
            bounds,
            corner_radii: gpui::Corners::all(radius),
            background: track_color.into(),
            border_widths: gpui::Edges::default(),
            border_color: gpui::transparent_black(),
            border_style: BorderStyle::default(),
        });

        // Filled portion
        if fill_fraction > 0.0 {
            let fill_bounds = Bounds {
                origin: bounds.origin,
                size: gpui::Size {
                    width: bounds.size.width * fill_fraction,
                    height: bounds.size.height,
                },
            };
            window.paint_quad(gpui::PaintQuad {
                bounds: fill_bounds,
                corner_radii: gpui::Corners::all(radius),
                background: fill_color.into(),
                border_widths: gpui::Edges::default(),
                border_color: gpui::transparent_black(),
                border_style: BorderStyle::default(),
            });
        }

        // Pointer cursor on hover
        if prepaint.hitbox.is_hovered(window) {
            window.set_cursor_style(CursorStyle::PointingHand, &prepaint.hitbox);
        }

        // Seek on mouse-down
        if let Some(on_seek) = self.on_seek.take() {
            let hitbox = prepaint.hitbox.clone();
            let origin_x = bounds.origin.x;
            let width = bounds.size.width;
            window.on_mouse_event(move |event: &MouseDownEvent, phase, window, cx| {
                if phase == DispatchPhase::Bubble && hitbox.is_hovered(window) {
                    let rel_x = f32::from(event.position.x - origin_x);
                    let w = f32::from(width);
                    let fraction = if w > 0.0 {
                        (rel_x / w).clamp(0.0, 1.0)
                    } else {
                        0.0
                    };
                    on_seek(fraction, window, cx);
                    cx.stop_propagation();
                }
            });
        }
    }
}
