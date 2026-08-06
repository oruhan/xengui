// SPDX-License-Identifier: Apache-2.0
use crate::{
    AUTO_SCROLL_DEAD_ZONE_DP,
    AUTO_SCROLL_MAX_SPEED,
    AUTO_SCROLL_RANGE_DP,
    AnimKey,
    AnimLayer,
    AnimProperty,
    AnimationManager,
    Background,
    BorderRadius,
    Constraints,
    ContextMenuHandle,
    Cursor,
    DEFAULT_SCROLLBAR_THUMB_HOVER_THICKNESS,
    ElementState,
    EventCtx,
    EventStatus,
    InputEvent,
    Interaction,
    Key,
    KeyState,
    LayoutBox,
    Length,
    MOMENTUM_FRICTION,
    MOMENTUM_MIN_SPEED,
    MeasureContext,
    MeasureResult,
    ModifiersState,
    MouseButton,
    MouseScrollDelta,
    OVERSCROLL_GLOW_FADE_TRANSITION,
    OVERSCROLL_RETURN_TRANSITION,
    OVERSCROLL_RUBBER_BAND_RANGE,
    Overflow,
    Overscroll,
    PaintContext,
    Point,
    Rect,
    RectCommand,
    ResolvedScrollbar,
    SCROLL_TRANSITION,
    SCROLLBAR_ARROW_CAP_SEGMENTS,
    SCROLLBAR_ARROW_CORNER_RADIUS,
    SCROLLBAR_ARROW_PRESS_SCALE,
    SCROLLBAR_ARROW_PRESS_TRANSITION,
    SCROLLBAR_ARROW_SIZE,
    SCROLLBAR_DISABLED_OPACITY,
    SCROLLBAR_THICKNESS_TRANSITION,
    SCROLLBAR_THUMB_PADDING,
    ScrollbarGutter,
    Style,
    StyleBuilder,
    TOUCH_PAN_THRESHOLD_DP,
    TouchPanPhase,
    Triangle,
    TriangleCommand,
    Widget,
    WidgetBase,
    WidgetId,
};
use xen_animation::{ AnimValue };
use std::cell::Cell;
use web_time::Instant;

#[derive(Clone, Copy)]
struct ScrollDrag {
    vertical: bool,
    start_mouse: f32,
    start_offset: f32,
}

#[derive(Clone, Copy)]
enum ArrowDirection {
    Up,
    Down,
    Left,
    Right,
}

/// Identifies which of the four scrollbar arrow buttons is pressed, used
/// to key that button's own press-scale animation.
#[derive(Clone, Copy, PartialEq, Eq)]
enum ScrollbarArrow {
    Up = 0,
    Down = 1,
    Left = 2,
    Right = 3,
}

#[derive(Clone, Copy)]
struct AutoScrollState {
    origin: (f32, f32),
    current: (f32, f32),
    // Reflects which axes are actually scrollable, chosen once at
    // activation so it matches the native OS pan cursor for that case.
    cursor: Cursor,
}

#[derive(Clone, Copy)]
struct TouchPanState {
    origin: (f32, f32),
    last_position: (f32, f32),
    last_time: Instant,
    velocity: (f32, f32),
    // Becomes true once total movement since `origin` clears the
    // jitter-filtering threshold, after which the gesture actually scrolls.
    dragging: bool,
}

#[derive(Clone, Copy)]
struct MomentumState {
    velocity: (f32, f32),
}

#[derive(Clone, Copy)]
enum EdgeSide {
    Top,
    Right,
    Bottom,
    Left,
}

// Maps `Overscroll::Auto` onto each platform's own scroll-edge
// convention. Concrete `Overscroll` values bypass this entirely.
fn platform_default_overscroll() -> Overscroll {
    if cfg!(target_os = "ios") {
        Overscroll::Bounce
    } else if cfg!(target_os = "android") {
        Overscroll::Stretch
    } else if cfg!(target_arch = "wasm32") {
        Overscroll::Bounce
    } else {
        Overscroll::Disabled
    }
}

fn point_in_rect(point: (f32, f32), rect: (f32, f32, f32, f32)) -> bool {
    let (px, py) = point;
    let (rx, ry, rw, rh) = rect;
    px >= rx && px <= rx + rw && py >= ry && py <= ry + rh
}

// Shrinks/grows a triangle around its arrow button rect's own center,
// used for the scrollbar arrow's press-feedback scale.
fn scale_arrow_triangle(tri: Triangle, rect: Rect, scale: f32) -> Triangle {
    if (scale - 1.0).abs() < f32::EPSILON {
        return tri;
    }
    let (rx, ry, rw, rh) = rect;
    let center = (rx + rw * 0.5, ry + rh * 0.5);
    let scale_pt = |p: Point| (
        center.0 + (p.0 - center.0) * scale,
        center.1 + (p.1 - center.1) * scale,
    );
    (scale_pt(tri.0), scale_pt(tri.1), scale_pt(tri.2))
}

// Builds a solid, filled arrow with rounded corners: three straight
// corner points defining the arrow's silhouette, each corner rounded off
// with a small tangent arc, then fan-triangulated from the centroid.
fn rounded_arrow_triangles(
    rect: (f32, f32, f32, f32),
    direction: ArrowDirection,
    sf: f32
) -> Vec<Triangle> {
    let (x, y, w, h) = rect;
    let cx = x + w * 0.5;
    let cy = y + h * 0.5;
    let s = SCROLLBAR_ARROW_SIZE * sf;

    let (a, tip, b) = match direction {
        ArrowDirection::Up => ((cx - s, cy + s * 0.5), (cx, cy - s * 0.5), (cx + s, cy + s * 0.5)),
        ArrowDirection::Down =>
            ((cx - s, cy - s * 0.5), (cx, cy + s * 0.5), (cx + s, cy - s * 0.5)),
        ArrowDirection::Left =>
            ((cx + s * 0.5, cy - s), (cx - s * 0.5, cy), (cx + s * 0.5, cy + s)),
        ArrowDirection::Right =>
            ((cx - s * 0.5, cy - s), (cx + s * 0.5, cy), (cx - s * 0.5, cy + s)),
    };

    let polygon = rounded_triangle_polygon(
        a,
        tip,
        b,
        SCROLLBAR_ARROW_CORNER_RADIUS * sf,
        SCROLLBAR_ARROW_CAP_SEGMENTS
    );
    fan_triangulate(&polygon)
}

// Traces the outline of a triangle with each of its three corners rounded
// off by a tangent arc of `radius`, ready to be fan-triangulated.
fn rounded_triangle_polygon(
    p0: Point,
    p1: Point,
    p2: Point,
    radius: f32,
    segments: usize
) -> Vec<Point> {
    let mut points = Vec::new();
    push_rounded_corner(p2, p0, p1, radius, segments, &mut points);
    push_rounded_corner(p0, p1, p2, radius, segments, &mut points);
    push_rounded_corner(p1, p2, p0, radius, segments, &mut points);
    points
}

// Appends the rounded-corner arc at `corner`, tangent to the edges
// `prev -> corner` and `corner -> next`, clamped so the rounding never
// eats more than half of either adjacent edge.
fn push_rounded_corner(
    prev: Point,
    corner: Point,
    next: Point,
    radius: f32,
    segments: usize,
    out: &mut Vec<Point>
) {
    let to_prev = (prev.0 - corner.0, prev.1 - corner.1);
    let to_next = (next.0 - corner.0, next.1 - corner.1);
    let len_prev = (to_prev.0 * to_prev.0 + to_prev.1 * to_prev.1).sqrt().max(0.0001);
    let len_next = (to_next.0 * to_next.0 + to_next.1 * to_next.1).sqrt().max(0.0001);
    let dir_prev = (to_prev.0 / len_prev, to_prev.1 / len_prev);
    let dir_next = (to_next.0 / len_next, to_next.1 / len_next);

    let dot = (dir_prev.0 * dir_next.0 + dir_prev.1 * dir_next.1).clamp(-1.0, 1.0);
    let half_angle = (dot.acos() * 0.5).max(0.001);
    let tan_half = half_angle.tan().max(0.0001);
    let sin_half = half_angle.sin().max(0.0001);

    let max_t = (len_prev.min(len_next) * 0.5).max(0.01);
    let t = (radius / tan_half).min(max_t * 0.9);
    let actual_radius = t * tan_half;
    let center_dist = actual_radius / sin_half;

    let bisector = (dir_prev.0 + dir_next.0, dir_prev.1 + dir_next.1);
    let bisector_len = (bisector.0 * bisector.0 + bisector.1 * bisector.1).sqrt().max(0.0001);
    let bisector = (bisector.0 / bisector_len, bisector.1 / bisector_len);

    let center = (corner.0 + bisector.0 * center_dist, corner.1 + bisector.1 * center_dist);
    let t1 = (corner.0 + dir_prev.0 * t, corner.1 + dir_prev.1 * t);
    let t2 = (corner.0 + dir_next.0 * t, corner.1 + dir_next.1 * t);

    let start_angle = (t1.1 - center.1).atan2(t1.0 - center.0);
    let end_angle = (t2.1 - center.1).atan2(t2.0 - center.0);
    let mut delta = end_angle - start_angle;
    if delta > std::f32::consts::PI {
        delta -= std::f32::consts::TAU;
    } else if delta < -std::f32::consts::PI {
        delta += std::f32::consts::TAU;
    }

    for i in 0..=segments {
        let a = start_angle + delta * ((i as f32) / (segments as f32));
        out.push((center.0 + a.cos() * actual_radius, center.1 + a.sin() * actual_radius));
    }
}

// Fans a convex polygon into triangles from its centroid.
fn fan_triangulate(points: &[Point]) -> Vec<Triangle> {
    if points.len() < 3 {
        return Vec::new();
    }
    let (sum_x, sum_y) = points.iter().fold((0.0, 0.0), |acc, p| (acc.0 + p.0, acc.1 + p.1));
    let n = points.len() as f32;
    let centroid = (sum_x / n, sum_y / n);

    points
        .windows(2)
        .map(|w| (centroid, w[0], w[1]))
        .chain(std::iter::once((centroid, points[points.len() - 1], points[0])))
        .collect()
}

pub struct View {
    base: WidgetBase,

    anim_id: WidgetId,
    layout_box: LayoutBox,
    children: Vec<Box<dyn Widget>>,
    scroll_offset: Cell<(f32, f32)>,
    scroll_target: Cell<(f32, f32)>,

    content_size: Cell<(f32, f32)>,
    scrollbar_drag: Cell<Option<ScrollDrag>>,
    pending_track_drag: Cell<Option<bool>>,
    scroll_step: f32,
    scrollbar_hovered: Cell<bool>,
    scrollbar_thickness_anim: Cell<f32>,
    scrollbar_right_inset: Cell<f32>,
    scrollbar_bottom_inset: Cell<f32>,
    scale_factor: Cell<f32>,
    context_menu: Option<ContextMenuHandle>,

    // Currently-pressed scrollbar arrow (if any), and its own per-arrow
    // animation identities/scale values, indexed by `ScrollbarArrow as usize`.
    pressed_arrow: Cell<Option<ScrollbarArrow>>,
    arrow_anim_ids: [WidgetId; 4],
    arrow_scale: Cell<[f32; 4]>,

    auto_scroll_enabled: bool,
    auto_scroll: Cell<Option<AutoScrollState>>,

    touch_pan: Cell<Option<TouchPanState>>,
    momentum: Cell<Option<MomentumState>>,

    // Edge-glow intensity for `Overscroll::Glow`, indexed [top, right, bottom, left].
    overscroll_glow: Cell<[f32; 4]>,
    // Edges hit since the last cascade, consumed by `animate_overscroll_glow`
    // to snap that edge's animated value back up to full intensity.
    glow_pending_hit: Cell<[bool; 4]>,
    glow_anim_ids: [WidgetId; 4],
}

impl View {
    pub fn new() -> Self {
        let mut view = Self {
            base: WidgetBase::new(Interaction::new()),

            anim_id: WidgetId::new_unique(),
            layout_box: LayoutBox::default(),
            children: Vec::new(),
            scroll_offset: Cell::new((0.0, 0.0)),
            scroll_target: Cell::new((0.0, 0.0)),
            content_size: Cell::new((0.0, 0.0)),
            scrollbar_drag: Cell::new(None),
            pending_track_drag: Cell::new(None),
            scroll_step: 96.0,
            scrollbar_hovered: Cell::new(false),
            scrollbar_thickness_anim: Cell::new(0.0),
            scrollbar_right_inset: Cell::new(0.0),
            scrollbar_bottom_inset: Cell::new(0.0),
            scale_factor: Cell::new(1.0),
            context_menu: None,

            pressed_arrow: Cell::new(None),
            arrow_anim_ids: [
                WidgetId::new_unique(),
                WidgetId::new_unique(),
                WidgetId::new_unique(),
                WidgetId::new_unique(),
            ],
            arrow_scale: Cell::new([1.0; 4]),

            auto_scroll_enabled: true,
            auto_scroll: Cell::new(None),

            touch_pan: Cell::new(None),
            momentum: Cell::new(None),

            overscroll_glow: Cell::new([0.0; 4]),
            glow_pending_hit: Cell::new([false; 4]),
            glow_anim_ids: [
                WidgetId::new_unique(),
                WidgetId::new_unique(),
                WidgetId::new_unique(),
                WidgetId::new_unique(),
            ],
        };
        view = view
            .selection_background(|theme: &crate::Theme| theme.selection)
            .selection_color(|theme: &crate::Theme| theme.selection_color)
            .caret_color(|theme: &crate::Theme| theme.caret_color)
            .selection_border_color(|theme: &crate::Theme| theme.selection_border_color)
            .selection_border_width(|theme: &crate::Theme| theme.selection_border_width)
            .selection_border_radius(|theme: &crate::Theme| theme.selection_border_radius);

        view.recompute_style();
        view
    }

    pub fn child(mut self, child: impl Widget + 'static) -> Self {
        self.children.push(Box::new(child));
        self
    }

    /// Bulk variant of `child` for dynamically built lists where each item
    /// is already a boxed trait object (e.g. produced inside a `.map()`).
    pub fn children_vec(mut self, children: Vec<Box<dyn Widget>>) -> Self {
        self.children = children;
        self
    }

    pub fn focusable(mut self, focusable: bool) -> Self {
        self.base.interaction.focusable = focusable;
        self
    }

    pub fn scroll_step(mut self, step: f32) -> Self {
        self.scroll_step = step;
        self
    }

    /// Binds a [`ContextMenuHandle`] so right-clicking this view opens the
    /// bound `ContextMenu`, without needing to wrap this view as its child.
    pub fn context_menu(mut self, handle: ContextMenuHandle) -> Self {
        self.context_menu = Some(handle);
        self
    }

    /// Enables or disables middle-click AutoScroll for this view. Enabled
    /// by default; has no effect unless the view is scrollable on at
    /// least one axis.
    pub fn auto_scroll(mut self, enabled: bool) -> Self {
        self.auto_scroll_enabled = enabled;
        self
    }

    fn recompute_style(&mut self) {
        self.base.recompute_style();
        self.base.interaction.hover_cursor = self.base.computed_style.cursor;
    }

    fn resolved_scrollbar(&self) -> ResolvedScrollbar {
        self.base.computed_style.scrollbar.unwrap_or_default().resolve()
    }

    // Overlays scrollbar_hover / scrollbar_pressed on top of the base scrollbar style.
    fn resolved_scrollbar_for_state(&self, hovered: bool, pressed: bool) -> ResolvedScrollbar {
        let base = self.resolved_scrollbar();
        let style = &self.base.computed_style;

        if pressed {
            return match style.scrollbar_pressed.as_ref().or(style.scrollbar_hover.as_ref()) {
                Some(patch) => base.patched(patch, DEFAULT_SCROLLBAR_THUMB_HOVER_THICKNESS),
                None =>
                    ResolvedScrollbar {
                        thickness: DEFAULT_SCROLLBAR_THUMB_HOVER_THICKNESS,
                        ..base
                    },
            };
        }

        if hovered {
            return match style.scrollbar_hover.as_ref() {
                Some(patch) => base.patched(patch, DEFAULT_SCROLLBAR_THUMB_HOVER_THICKNESS),
                None =>
                    ResolvedScrollbar {
                        thickness: DEFAULT_SCROLLBAR_THUMB_HOVER_THICKNESS,
                        ..base
                    },
            };
        }

        base
    }

    fn resolved_overscroll(&self) -> Overscroll {
        self.base.computed_style.overscroll.unwrap_or_default()
    }

    fn effective_overscroll(&self) -> Overscroll {
        match self.resolved_overscroll() {
            Overscroll::Auto => platform_default_overscroll(),
            other => other,
        }
    }

    // Diminishing-returns rubber-band curve: as `overshoot` grows, the
    // damped result approaches `range` asymptotically instead of linearly.
    fn rubber_band(overshoot: f32, range: f32) -> f32 {
        range * (1.0 - 1.0 / (overshoot / range + 1.0))
    }

    // Applies the current Overscroll mode to a proposed (possibly
    // out-of-bounds) offset for one axis. `allow_rubber_band` distinguishes
    // direct manipulation (drag/momentum), which may rubber-band, from
    // discrete nudges (wheel/scrollbar), which always clamp hard. Returns
    // the reactive offset and whether the bound was hit hard.
    fn react_to_bounds(&self, raw: f32, max: f32, allow_rubber_band: bool) -> (f32, bool) {
        if raw >= 0.0 && raw <= max {
            return (raw, false);
        }

        let mode = self.effective_overscroll();
        let rubber_bandable =
            allow_rubber_band && matches!(mode, Overscroll::Bounce | Overscroll::Stretch);

        if rubber_bandable {
            let range = OVERSCROLL_RUBBER_BAND_RANGE * self.scale_factor.get();
            let value = if raw < 0.0 {
                -Self::rubber_band(-raw, range)
            } else {
                max + Self::rubber_band(raw - max, range)
            };
            (value, false)
        } else {
            (raw.clamp(0.0, max), true)
        }
    }

    fn note_edge_hit(&self, side: EdgeSide, ctx: &mut EventCtx) {
        if self.effective_overscroll() != Overscroll::Glow {
            return;
        }
        let idx = match side {
            EdgeSide::Top => 0,
            EdgeSide::Right => 1,
            EdgeSide::Bottom => 2,
            EdgeSide::Left => 3,
        };
        let mut pending = self.glow_pending_hit.get();
        pending[idx] = true;
        self.glow_pending_hit.set(pending);
        ctx.request_redraw();
    }

    // Drives every edge's glow intensity through the shared
    // AnimationManager instead of a manual per-frame decay: a pending hit
    // snaps that edge straight to full intensity, then every edge is
    // (re)targeted toward 0 so the manager eases it back down on its own.
    fn animate_overscroll_glow(&mut self, anim: &mut AnimationManager) {
        let mut pending = self.glow_pending_hit.get();
        let mut values = self.overscroll_glow.get();

        for i in 0..4 {
            let key = AnimKey {
                widget: self.glow_anim_ids[i],
                layer: AnimLayer::Root,
                property: AnimProperty::Opacity,
            };

            if pending[i] {
                anim.set_target(key, AnimValue([1.0, 0.0, 0.0, 0.0]), None);
                pending[i] = false;
            }

            anim.set_target(
                key,
                AnimValue([0.0, 0.0, 0.0, 0.0]),
                Some(OVERSCROLL_GLOW_FADE_TRANSITION)
            );

            match anim.value(key) {
                Some(v) => {
                    values[i] = v.0[0];
                    self.base.dirty = true;
                }
                None => {
                    values[i] = 0.0;
                }
            }
        }

        self.glow_pending_hit.set(pending);
        self.overscroll_glow.set(values);
    }

    // Drives each scrollbar arrow's press-feedback scale through the
    // shared AnimationManager, toward a small shrink while pressed and
    // back to 1.0 once released.
    fn animate_scrollbar_arrows(&mut self, anim: &mut AnimationManager) {
        let pressed = self.pressed_arrow.get();
        let mut scales = self.arrow_scale.get();

        for (i, (anim_id, scale)) in self.arrow_anim_ids.iter().zip(scales.iter_mut()).enumerate() {
            let is_pressed = pressed.map(|p| p as usize) == Some(i);
            let target = if is_pressed { SCROLLBAR_ARROW_PRESS_SCALE } else { 1.0 };

            let key = AnimKey {
                widget: *anim_id,
                layer: AnimLayer::Root,
                property: AnimProperty::Scale,
            };
            anim.set_target(
                key,
                AnimValue([target, 0.0, 0.0, 0.0]),
                Some(SCROLLBAR_ARROW_PRESS_TRANSITION)
            );

            match anim.value(key) {
                Some(v) => {
                    *scale = v.0[0];
                    self.base.dirty = true;
                }
                None => {
                    *scale = target;
                }
            }
        }

        self.arrow_scale.set(scales);
    }

    // A new discrete gesture (wheel, scrollbar drag, a fresh touch pan, or
    // AutoScroll activation) always takes over from whatever other
    // scroll-driving gesture was previously in flight on this view.
    fn cancel_conflicting_gestures(&self) {
        self.auto_scroll.set(None);
        self.touch_pan.set(None);
        self.momentum.set(None);
    }

    fn spring_back_if_needed(&mut self, ctx: &mut EventCtx) {
        let clamped = self.clamp_offset(self.scroll_offset.get());
        if clamped != self.scroll_target.get() {
            self.scroll_target.set(clamped);
            self.base.dirty = true;
            ctx.request_redraw();
        }
    }

    fn active_scrollbar(&self) -> ResolvedScrollbar {
        let pressed = self.scrollbar_drag.get().is_some();
        let hovered = self.scrollbar_hovered.get();
        let sf = self.scale_factor.get();
        let mut sb = self.resolved_scrollbar_for_state(hovered, pressed);
        sb.min_thumb_length *= sf;
        sb.thumb_radius *= sf;
        sb.thumb_border_width *= sf;
        sb.track_border_width *= sf;
        sb.thickness = self.track_thickness();
        sb
    }

    // Derived from the largest possible thumb thickness (idle/hover/pressed)
    // plus padding on both sides, so the gutter never resizes when the
    // thumb itself animates - only the thumb's own thickness moves within it.
    fn track_thickness_logical(&self) -> f32 {
        let style = &self.base.computed_style;
        let idle = style.scrollbar.unwrap_or_default().resolve().thickness;
        let hover = style.scrollbar_hover
            .and_then(|p| p.thickness)
            .unwrap_or(DEFAULT_SCROLLBAR_THUMB_HOVER_THICKNESS);
        let pressed = style.scrollbar_pressed.and_then(|p| p.thickness).unwrap_or(hover);
        idle.max(hover).max(pressed) + SCROLLBAR_THUMB_PADDING * 2.0
    }

    fn track_thickness(&self) -> f32 {
        self.track_thickness_logical() * self.scale_factor.get()
    }

    fn target_scrollbar_thickness(&self) -> f32 {
        let pressed = self.scrollbar_drag.get().is_some();
        let hovered = self.scrollbar_hovered.get();
        self.resolved_scrollbar_for_state(hovered, pressed).thickness
    }

    fn current_scrollbar_thickness(&self) -> f32 {
        self.scrollbar_thickness_anim.get()
    }

    // Pulls the scrollbar thickness toward its hover/pressed target through
    // the shared AnimationManager, called once per frame from cascade_style.
    fn animate_scrollbar_thickness(&mut self, anim: &mut AnimationManager) {
        let target = self.target_scrollbar_thickness();
        let key = AnimKey {
            widget: self.anim_id,
            layer: AnimLayer::Root,
            property: AnimProperty::ScrollbarThickness,
        };

        anim.set_target(
            key,
            AnimValue([target, 0.0, 0.0, 0.0]),
            Some(SCROLLBAR_THICKNESS_TRANSITION)
        );

        match anim.value(key) {
            Some(v) => {
                self.scrollbar_thickness_anim.set(v.0[0]);
                self.base.dirty = true;
            }
            None => self.scrollbar_thickness_anim.set(target),
        }
    }

    // Reserves layout space for the scrollbar so content doesn't shift
    // when it appears/disappears, matching CSS's `scrollbar-gutter`.
    // Stable/StableBothEdges reserve purely from the static overflow mode
    // (Scroll/Auto), never from whether content currently overflows - this
    // keeps the reservation fixed regardless of content size or hover,
    // instead of only catching up once a later layout pass re-measures
    // the content and happens to get retriggered by something like hover.
    // Auto instead mirrors the scrollbar's actual current visibility.
    fn apply_scrollbar_gutter(&mut self) {
        let gutter = self.base.computed_style.scrollbar_gutter.unwrap_or_default();

        let sf = self.scale_factor.get();
        let thickness = self.track_thickness_logical();
        let mut padding = self.base.computed_style.padding.unwrap_or_default();

        let (shows_x, shows_y) = if gutter == ScrollbarGutter::Auto {
            self.scrollbar_visibility()
        } else {
            (self.is_scrollable_x(), self.is_scrollable_y())
        };

        self.scrollbar_right_inset.set(if shows_y { padding.right.to_physical(sf) } else { 0.0 });
        self.scrollbar_bottom_inset.set(if shows_x { padding.bottom.to_physical(sf) } else { 0.0 });

        if shows_y {
            padding.right = padding.right.add_px(thickness);
            if gutter == ScrollbarGutter::StableBothEdges {
                padding.left = padding.left.add_px(thickness);
            }
        }
        if shows_x {
            padding.bottom = padding.bottom.add_px(thickness);
            if gutter == ScrollbarGutter::StableBothEdges {
                padding.top = padding.top.add_px(thickness);
            }
        }

        self.base.computed_style.padding = Some(padding);
    }

    fn animate_scroll(&mut self, anim: &mut AnimationManager) {
        let target = self.scroll_target.get();
        let key = AnimKey {
            widget: self.anim_id,
            layer: AnimLayer::Root,
            property: AnimProperty::ScrollOffset,
        };

        let direct_manipulation =
            self.scrollbar_drag.get().is_some() ||
            self.touch_pan.get().is_some() ||
            self.momentum.get().is_some() ||
            self.auto_scroll.get().is_some();

        let offset = self.scroll_offset.get();
        let overscrolled =
            offset.0 < 0.0 ||
            offset.0 > self.max_scroll_x() ||
            offset.1 < 0.0 ||
            offset.1 > self.max_scroll_y();

        let transition = if direct_manipulation {
            None
        } else if overscrolled {
            Some(OVERSCROLL_RETURN_TRANSITION)
        } else {
            Some(SCROLL_TRANSITION)
        };

        anim.set_target(key, AnimValue([target.0, target.1, 0.0, 0.0]), transition);

        match anim.value(key) {
            Some(v) => {
                self.scroll_offset.set((v.0[0], v.0[1]));
                self.base.dirty = true;
            }
            None => self.scroll_offset.set(target),
        }
    }

    // Hit area for scrollbar hover detection; uses the hover thickness so
    // a thin, unhovered bar is still easy to reach with the pointer.
    fn point_in_scrollbar(&self, point: (f32, f32)) -> bool {
        // A shown-but-inactive `Scroll` axis (no overflow) doesn't
        // intercept the pointer, so it passes through to the content.
        let (active_x, active_y) = self.scrollbar_active();
        if !active_x && !active_y {
            return false;
        }

        let b = self.layout_box;
        let t = self.resolved_scrollbar_for_state(true, false).thickness * self.scale_factor.get();
        let right_inset = self.scrollbar_right_inset.get();
        let bottom_inset = self.scrollbar_bottom_inset.get();

        if active_y && point_in_rect(point, (b.x + b.width - right_inset - t, b.y, t, b.height)) {
            return true;
        }
        if active_x && point_in_rect(point, (b.x, b.y + b.height - bottom_inset - t, b.width, t)) {
            return true;
        }
        false
    }

    fn is_scrollable_x(&self) -> bool {
        matches!(self.base.computed_style.overflow_x, Some(Overflow::Scroll | Overflow::Auto))
    }

    fn is_scrollable_y(&self) -> bool {
        matches!(self.base.computed_style.overflow_y, Some(Overflow::Scroll | Overflow::Auto))
    }

    fn clips_x(&self) -> bool {
        matches!(
            self.base.computed_style.overflow_x,
            Some(Overflow::Scroll | Overflow::Auto | Overflow::Hidden)
        )
    }

    fn clips_y(&self) -> bool {
        matches!(
            self.base.computed_style.overflow_y,
            Some(Overflow::Scroll | Overflow::Auto | Overflow::Hidden)
        )
    }

    fn max_scroll_x(&self) -> f32 {
        (self.content_size.get().0 - self.layout_box.width).max(0.0)
    }

    fn max_scroll_y(&self) -> f32 {
        (self.content_size.get().1 - self.layout_box.height).max(0.0)
    }

    fn clamp_offset(&self, offset: (f32, f32)) -> (f32, f32) {
        (offset.0.clamp(0.0, self.max_scroll_x()), offset.1.clamp(0.0, self.max_scroll_y()))
    }

    // Whether each axis's scrollbar should be painted at all. `Scroll`
    // mode is always shown (even without overflow, as a disabled track);
    // `Auto` only shows once there's real overflow to scroll.
    fn scrollbar_visibility(&self) -> (bool, bool) {
        let shown = |overflow: Option<Overflow>, max_scroll: f32| {
            match overflow {
                Some(Overflow::Scroll) => true,
                Some(Overflow::Auto) => max_scroll > 0.0,
                _ => false,
            }
        };
        (
            shown(self.base.computed_style.overflow_x, self.max_scroll_x()),
            shown(self.base.computed_style.overflow_y, self.max_scroll_y()),
        )
    }

    // Whether each axis actually has overflow to scroll; a `Scroll`-mode
    // scrollbar can be shown (see `scrollbar_visibility`) but inactive.
    fn scrollbar_active(&self) -> (bool, bool) {
        (self.max_scroll_x() > 0.0, self.max_scroll_y() > 0.0)
    }

    fn vertical_track_bounds(&self) -> Option<(f32, f32)> {
        let (has_x, has_y) = self.scrollbar_visibility();
        if !has_y {
            return None;
        }
        let b = self.layout_box;
        let t = self.active_scrollbar().thickness;
        let full_h = if has_x { b.height - t } else { b.height };
        Some((b.y + t, (full_h - 2.0 * t).max(0.0)))
    }

    fn horizontal_track_bounds(&self) -> Option<(f32, f32)> {
        let (has_x, has_y) = self.scrollbar_visibility();
        if !has_x {
            return None;
        }
        let b = self.layout_box;
        let t = self.active_scrollbar().thickness;
        let full_w = if has_y { b.width - t } else { b.width };
        Some((b.x + t, (full_w - 2.0 * t).max(0.0)))
    }

    fn vertical_thumb_rect(&self) -> Option<(f32, f32, f32, f32)> {
        // No real overflow: the track still gets painted (see
        // paint_overlay), but no thumb is drawn on top of it.
        if self.max_scroll_y() <= 0.0 {
            return None;
        }
        let (track_y, track_h) = self.vertical_track_bounds()?;
        let b = self.layout_box;
        let sb = self.active_scrollbar();
        let content_h = self.content_size.get().1.max(b.height);

        let thumb_h = ((track_h * b.height) / content_h).max(sb.min_thumb_length).min(track_h);
        let max_offset = self.max_scroll_y();
        let progress = if max_offset > 0.0 { self.scroll_offset.get().1 / max_offset } else { 0.0 };
        let thumb_y = track_y + progress * (track_h - thumb_h);

        let thumb_w = (self.current_scrollbar_thickness() * self.scale_factor.get()).min(
            sb.thickness
        );
        let right_inset = self.scrollbar_right_inset.get();
        let thumb_x = b.x + b.width - right_inset - sb.thickness + (sb.thickness - thumb_w) * 0.5;
        Some((thumb_x, thumb_y, thumb_w, thumb_h))
    }

    fn horizontal_thumb_rect(&self) -> Option<(f32, f32, f32, f32)> {
        // No real overflow: the track still gets painted (see
        // paint_overlay), but no thumb is drawn on top of it.
        if self.max_scroll_x() <= 0.0 {
            return None;
        }
        let (track_x, track_w) = self.horizontal_track_bounds()?;
        let b = self.layout_box;
        let sb = self.active_scrollbar();
        let content_w = self.content_size.get().0.max(b.width);

        let thumb_w = ((track_w * b.width) / content_w).max(sb.min_thumb_length).min(track_w);
        let max_offset = self.max_scroll_x();
        let progress = if max_offset > 0.0 { self.scroll_offset.get().0 / max_offset } else { 0.0 };
        let thumb_x = track_x + progress * (track_w - thumb_w);

        let thumb_h = (self.current_scrollbar_thickness() * self.scale_factor.get()).min(
            sb.thickness
        );
        let bottom_inset = self.scrollbar_bottom_inset.get();
        let thumb_y = b.y + b.height - bottom_inset - sb.thickness + (sb.thickness - thumb_h) * 0.5;
        Some((thumb_x, thumb_y, thumb_w, thumb_h))
    }

    fn vertical_buttons(&self) -> Option<(Rect, Rect)> {
        let (_, has_y) = self.scrollbar_visibility();
        if !has_y {
            return None;
        }
        let b = self.layout_box;
        let t = self.active_scrollbar().thickness;
        let right_inset = self.scrollbar_right_inset.get();
        let (has_x, _) = self.scrollbar_visibility();
        let bottom = if has_x { b.y + b.height - t } else { b.y + b.height };
        Some((
            (b.x + b.width - right_inset - t, b.y, t, t),
            (b.x + b.width - right_inset - t, bottom - t, t, t),
        ))
    }

    fn horizontal_buttons(&self) -> Option<(Rect, Rect)> {
        let (has_x, _) = self.scrollbar_visibility();
        if !has_x {
            return None;
        }
        let b = self.layout_box;
        let t = self.active_scrollbar().thickness;
        let bottom_inset = self.scrollbar_bottom_inset.get();
        let (_, has_y) = self.scrollbar_visibility();
        let right = if has_y { b.x + b.width - t } else { b.x + b.width };
        Some((
            (b.x, b.y + b.height - bottom_inset - t, t, t),
            (right - t, b.y + b.height - bottom_inset - t, t, t),
        ))
    }

    fn start_scroll_animation(&mut self, target: (f32, f32), ctx: &mut EventCtx) {
        self.scroll_target.set(target);
        self.base.dirty = true;
        ctx.request_redraw();
    }

    fn nudge(&mut self, dx: f32, dy: f32, ctx: &mut EventCtx) {
        let current = self.scroll_target.get();
        let next = self.clamp_offset((current.0 + dx, current.1 + dy));
        if next != current {
            self.start_scroll_animation(next, ctx);
        }
    }

    fn handle_page_key(&mut self, key: Key, modifiers: ModifiersState, ctx: &mut EventCtx) -> bool {
        if modifiers.shift {
            if !self.is_scrollable_x() {
                return false;
            }

            let dx: f32 = match key {
                Key::PageUp => -self.layout_box.width,
                Key::PageDown => self.layout_box.width,
                _ => {
                    return false;
                }
            };

            let current = self.scroll_target.get();
            let next = self.clamp_offset((current.0 + dx, current.1));
            if next == current {
                return false;
            }

            self.start_scroll_animation(next, ctx);
            return true;
        }

        if !self.is_scrollable_y() {
            return false;
        }

        let dy: f32 = match key {
            Key::PageUp => -self.layout_box.height,
            Key::PageDown => self.layout_box.height,
            _ => {
                return false;
            }
        };

        let current = self.scroll_target.get();
        let next = self.clamp_offset((current.0, current.1 + dy));
        if next == current {
            return false;
        }

        self.start_scroll_animation(next, ctx);
        true
    }

    fn handle_key(&mut self, key: Key, modifiers: ModifiersState, ctx: &mut EventCtx) -> bool {
        self.handle_page_key(key, modifiers, ctx)
    }

    fn handle_wheel(
        &mut self,
        delta: MouseScrollDelta,
        position: (f32, f32),
        modifiers: ModifiersState,
        ctx: &mut EventCtx,
        scroll_step: f32
    ) -> bool {
        if !self.hit_test(position) || (!self.is_scrollable_x() && !self.is_scrollable_y()) {
            return false;
        }

        let (raw_dx, raw_dy) = match delta {
            MouseScrollDelta::LineDelta(x, y) => (-x * scroll_step, -y * scroll_step),
            MouseScrollDelta::PixelDelta(x, y) => (-x as f32, -y as f32),
        };

        let (raw_dx, raw_dy) = if modifiers.shift { (raw_dy, raw_dx) } else { (raw_dx, raw_dy) };

        let (dx, dy) = if self.is_scrollable_y() {
            (raw_dx, raw_dy)
        } else {
            (raw_dx + raw_dy, 0.0)
        };
        let dx = if self.is_scrollable_x() { dx } else { 0.0 };
        let dy = if self.is_scrollable_y() { dy } else { 0.0 };

        if dx == 0.0 && dy == 0.0 {
            return false;
        }

        self.cancel_conflicting_gestures();

        let current = self.scroll_target.get();
        let (next_x, hit_x) = self.react_to_bounds(current.0 + dx, self.max_scroll_x(), false);
        let (next_y, hit_y) = self.react_to_bounds(current.1 + dy, self.max_scroll_y(), false);
        if hit_x {
            self.note_edge_hit(if dx < 0.0 { EdgeSide::Left } else { EdgeSide::Right }, ctx);
        }
        if hit_y {
            self.note_edge_hit(if dy < 0.0 { EdgeSide::Top } else { EdgeSide::Bottom }, ctx);
        }

        let next = (next_x, next_y);

        if next == current {
            return false;
        }

        self.start_scroll_animation(next, ctx);
        true
    }

    fn handle_scrollbar_mouse(
        &mut self,
        state: ElementState,
        button: MouseButton,
        position: (f32, f32),
        ctx: &mut EventCtx
    ) -> bool {
        if button != MouseButton::Left {
            return false;
        }

        let (active_x, active_y) = self.scrollbar_active();

        match state {
            ElementState::Pressed => {
                self.cancel_conflicting_gestures();
                let target = self.scroll_target.get();

                if active_y && let Some((up, down)) = self.vertical_buttons() {
                    if point_in_rect(position, up) {
                        self.pressed_arrow.set(Some(ScrollbarArrow::Up));
                        if target.1 > 0.0 {
                            self.nudge(0.0, -self.scroll_step, ctx);
                        }
                        ctx.request_redraw();
                        return true;
                    }
                    if point_in_rect(position, down) {
                        self.pressed_arrow.set(Some(ScrollbarArrow::Down));
                        if target.1 < self.max_scroll_y() {
                            self.nudge(0.0, self.scroll_step, ctx);
                        }
                        ctx.request_redraw();
                        return true;
                    }
                }
                if active_x && let Some((left, right)) = self.horizontal_buttons() {
                    if point_in_rect(position, left) {
                        self.pressed_arrow.set(Some(ScrollbarArrow::Left));
                        if target.0 > 0.0 {
                            self.nudge(-self.scroll_step, 0.0, ctx);
                        }
                        ctx.request_redraw();
                        return true;
                    }
                    if point_in_rect(position, right) {
                        self.pressed_arrow.set(Some(ScrollbarArrow::Right));
                        if target.0 < self.max_scroll_x() {
                            self.nudge(self.scroll_step, 0.0, ctx);
                        }
                        ctx.request_redraw();
                        return true;
                    }
                }
                if
                    active_y &&
                    let Some(thumb) = self.vertical_thumb_rect() &&
                    point_in_rect(position, thumb)
                {
                    self.scrollbar_drag.set(
                        Some(ScrollDrag {
                            vertical: true,
                            start_mouse: position.1,
                            start_offset: self.scroll_offset.get().1,
                        })
                    );
                    return true;
                }
                if
                    active_x &&
                    let Some(thumb) = self.horizontal_thumb_rect() &&
                    point_in_rect(position, thumb)
                {
                    self.scrollbar_drag.set(
                        Some(ScrollDrag {
                            vertical: false,
                            start_mouse: position.0,
                            start_offset: self.scroll_offset.get().0,
                        })
                    );
                    return true;
                }

                // Clicking an empty stretch of track jumps the thumb straight
                // to that point instead of requiring a drag or repeated nudges.
                if active_y && let Some((track_y, track_h)) = self.vertical_track_bounds() {
                    let t = self.active_scrollbar().thickness;
                    let b = self.layout_box;
                    let right_inset = self.scrollbar_right_inset.get();
                    if
                        point_in_rect(position, (
                            b.x + b.width - right_inset - t,
                            track_y,
                            t,
                            track_h,
                        ))
                    {
                        self.pending_track_drag.set(Some(true));
                        if let Some(target_y) = self.vertical_track_offset_for(position.1) {
                            let next = self.clamp_offset((target.0, target_y));
                            if next != target {
                                self.start_scroll_animation(next, ctx);
                            }
                        }
                        return true;
                    }
                }
                if active_x && let Some((track_x, track_w)) = self.horizontal_track_bounds() {
                    let t = self.active_scrollbar().thickness;
                    let b = self.layout_box;
                    let bottom_inset = self.scrollbar_bottom_inset.get();
                    if
                        point_in_rect(position, (
                            track_x,
                            b.y + b.height - bottom_inset - t,
                            track_w,
                            t,
                        ))
                    {
                        self.pending_track_drag.set(Some(false));
                        if let Some(target_x) = self.horizontal_track_offset_for(position.0) {
                            let next = self.clamp_offset((target_x, target.1));
                            if next != target {
                                self.start_scroll_animation(next, ctx);
                            }
                        }
                        return true;
                    }
                }

                false
            }
            ElementState::Released => {
                self.pending_track_drag.set(None);
                let had_pressed_arrow = self.pressed_arrow.take().is_some();

                if self.scrollbar_drag.get().is_some() {
                    self.scrollbar_drag.set(None);
                    ctx.request_redraw();
                    true
                } else if had_pressed_arrow {
                    ctx.request_redraw();
                    true
                } else {
                    false
                }
            }
        }
    }

    fn vertical_track_offset_for(&self, mouse_y: f32) -> Option<f32> {
        let (track_y, track_h) = self.vertical_track_bounds()?;
        let b = self.layout_box;
        let sb = self.active_scrollbar();
        let content_h = self.content_size.get().1.max(b.height);
        let thumb_h = ((track_h * b.height) / content_h).max(sb.min_thumb_length).min(track_h);
        let travel = (track_h - thumb_h).max(1.0);
        let progress = ((mouse_y - track_y - thumb_h * 0.5) / travel).clamp(0.0, 1.0);
        Some(progress * self.max_scroll_y())
    }

    fn horizontal_track_offset_for(&self, mouse_x: f32) -> Option<f32> {
        let (track_x, track_w) = self.horizontal_track_bounds()?;
        let b = self.layout_box;
        let sb = self.active_scrollbar();
        let content_w = self.content_size.get().0.max(b.width);
        let thumb_w = ((track_w * b.width) / content_w).max(sb.min_thumb_length).min(track_w);
        let travel = (track_w - thumb_w).max(1.0);
        let progress = ((mouse_x - track_x - thumb_w * 0.5) / travel).clamp(0.0, 1.0);
        Some(progress * self.max_scroll_x())
    }

    fn handle_scrollbar_drag(&mut self, position: (f32, f32), ctx: &mut EventCtx) -> bool {
        let Some(drag) = self.scrollbar_drag.get() else {
            return false;
        };

        let sb = self.active_scrollbar();

        let (track_len, content_len, viewport_len, max_offset) = if drag.vertical {
            let (_, track) = self.vertical_track_bounds().unwrap_or((0.0, 0.0));
            (
                track,
                self.content_size.get().1.max(self.layout_box.height),
                self.layout_box.height,
                self.max_scroll_y(),
            )
        } else {
            let (_, track) = self.horizontal_track_bounds().unwrap_or((0.0, 0.0));
            (
                track,
                self.content_size.get().0.max(self.layout_box.width),
                self.layout_box.width,
                self.max_scroll_x(),
            )
        };

        let thumb_len = ((track_len * viewport_len) / content_len)
            .max(sb.min_thumb_length)
            .min(track_len);
        let travel = (track_len - thumb_len).max(1.0);

        let mouse_pos = if drag.vertical { position.1 } else { position.0 };
        let delta_offset = (mouse_pos - drag.start_mouse) * (max_offset / travel);

        let current = self.scroll_offset.get();
        let next = if drag.vertical {
            self.clamp_offset((current.0, drag.start_offset + delta_offset))
        } else {
            self.clamp_offset((drag.start_offset + delta_offset, current.1))
        };

        if next != current {
            // Thumb drag tracks the cursor 1:1 - no easing here, only wheel
            // and button nudges go through the animated path.
            self.scroll_offset.set(next);
            self.scroll_target.set(next);
            self.base.dirty = true;
            ctx.request_redraw();
        }

        true
    }

    fn autoscroll_cursor(&self) -> Cursor {
        let (active_x, active_y) = self.scrollbar_active();
        match (active_x, active_y) {
            (true, true) => Cursor::AllScroll,
            (true, false) => Cursor::EwResize,
            (false, true) => Cursor::NsResize,
            (false, false) => Cursor::AllScroll,
        }
    }

    fn handle_auto_scroll(
        &mut self,
        event: &InputEvent,
        ctx: &mut EventCtx
    ) -> Option<EventStatus> {
        match event {
            InputEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Middle,
                position,
            } => {
                if self.auto_scroll.get().is_some() {
                    self.auto_scroll.set(None);
                    ctx.set_cursor_icon(Cursor::Default);
                    ctx.request_redraw();
                    return Some(EventStatus::Handled);
                }

                // Only real overflow (not just an overflow:scroll/auto
                // *mode* with nothing to scroll) should activate AutoScroll.
                let (active_x, active_y) = self.scrollbar_active();
                if self.auto_scroll_enabled && self.hit_test(*position) && (active_x || active_y) {
                    self.cancel_conflicting_gestures();
                    let cursor = self.autoscroll_cursor();
                    self.auto_scroll.set(
                        Some(AutoScrollState { origin: *position, current: *position, cursor })
                    );
                    ctx.set_cursor_icon(cursor);
                    // Escape only reaches a widget on the focused path -
                    // claiming focus here is what lets it cancel AutoScroll.
                    ctx.request_focus();
                    ctx.request_redraw();
                    return Some(EventStatus::Handled);
                }

                None
            }

            InputEvent::MouseInput { state: ElementState::Pressed, .. } if
                self.auto_scroll.get().is_some()
            => {
                self.auto_scroll.set(None);
                ctx.set_cursor_icon(Cursor::Default);
                ctx.request_redraw();
                Some(EventStatus::Handled)
            }

            InputEvent::KeyInput { event: key_event, .. } if
                self.auto_scroll.get().is_some() &&
                key_event.key == Key::Escape &&
                key_event.state == KeyState::Pressed
            => {
                self.auto_scroll.set(None);
                ctx.set_cursor_icon(Cursor::Default);
                ctx.request_redraw();
                Some(EventStatus::Handled)
            }

            InputEvent::MouseMoved { position } if self.auto_scroll.get().is_some() => {
                if let Some(mut state) = self.auto_scroll.get() {
                    state.current = *position;
                    ctx.set_cursor_icon(state.cursor);
                    self.auto_scroll.set(Some(state));
                }
                Some(EventStatus::Handled)
            }

            InputEvent::AnimationTick { dt } if self.auto_scroll.get().is_some() => {
                self.tick_auto_scroll(*dt, ctx);
                Some(EventStatus::Handled)
            }

            _ => None,
        }
    }

    fn tick_auto_scroll(&mut self, dt: f32, ctx: &mut EventCtx) {
        let Some(state) = self.auto_scroll.get() else {
            return;
        };
        let sf = self.scale_factor.get();
        let dead_zone = AUTO_SCROLL_DEAD_ZONE_DP * sf;
        let range = AUTO_SCROLL_RANGE_DP * sf;
        let max_speed = AUTO_SCROLL_MAX_SPEED * sf;

        // Linear ramp from the dead zone to max_speed, matching the
        // middle-click autoscroll curve used by Chromium/WebView-based
        // browsers (speed scales directly with distance past the
        // activation radius, no easing).
        let speed_along = |delta: f32| -> f32 {
            let mag = delta.abs();
            if mag <= dead_zone {
                return 0.0;
            }
            let t = ((mag - dead_zone) / range).min(1.0);
            delta.signum() * max_speed * t
        };

        let dx = state.current.0 - state.origin.0;
        let dy = state.current.1 - state.origin.1;

        let vx = if self.is_scrollable_x() { speed_along(dx) } else { 0.0 };
        let vy = if self.is_scrollable_y() { speed_along(dy) } else { 0.0 };

        if vx == 0.0 && vy == 0.0 {
            return;
        }

        let current = self.scroll_offset.get();
        let (next_x, hit_x) = self.react_to_bounds(current.0 + vx * dt, self.max_scroll_x(), false);
        let (next_y, hit_y) = self.react_to_bounds(current.1 + vy * dt, self.max_scroll_y(), false);
        if hit_x {
            self.note_edge_hit(if vx < 0.0 { EdgeSide::Left } else { EdgeSide::Right }, ctx);
        }
        if hit_y {
            self.note_edge_hit(if vy < 0.0 { EdgeSide::Top } else { EdgeSide::Bottom }, ctx);
        }

        let next = (next_x, next_y);
        if next != current {
            self.scroll_offset.set(next);
            self.scroll_target.set(next);
            self.base.dirty = true;
            ctx.request_redraw();
        }
    }

    // Claims the dedicated TouchPan gesture (by returning Handled on
    // Start) for whichever scrollable view is nearest the touched leaf,
    // so nested scrollables scroll the innermost one first.
    fn handle_touch_pan(&mut self, event: &InputEvent, ctx: &mut EventCtx) -> Option<EventStatus> {
        let InputEvent::TouchPan { phase, position } = event else {
            return None;
        };

        match phase {
            TouchPanPhase::Start => {
                if !(self.is_scrollable_x() || self.is_scrollable_y()) {
                    return None;
                }
                self.cancel_conflicting_gestures();
                self.touch_pan.set(
                    Some(TouchPanState {
                        origin: *position,
                        last_position: *position,
                        last_time: Instant::now(),
                        velocity: (0.0, 0.0),
                        dragging: false,
                    })
                );
                Some(EventStatus::Handled)
            }

            TouchPanPhase::Move => {
                let mut state = self.touch_pan.get()?;

                let now = Instant::now();
                let dt = now
                    .duration_since(state.last_time)
                    .as_secs_f32()
                    .max(1.0 / 240.0);

                if !state.dragging {
                    let threshold = TOUCH_PAN_THRESHOLD_DP * self.scale_factor.get();
                    let moved =
                        (position.0 - state.origin.0).abs() + (position.1 - state.origin.1).abs();
                    if moved < threshold {
                        return Some(EventStatus::Handled);
                    }
                    state.dragging = true;
                }

                let dx = position.0 - state.last_position.0;
                let dy = position.1 - state.last_position.1;

                let inst_vx = dx / dt;
                let inst_vy = dy / dt;
                state.velocity = (
                    state.velocity.0 * 0.8 + inst_vx * 0.2,
                    state.velocity.1 * 0.8 + inst_vy * 0.2,
                );
                state.last_position = *position;
                state.last_time = now;
                self.touch_pan.set(Some(state));

                // Content follows the finger: dragging down/right reveals
                // earlier content, so the offset moves the opposite way.
                let current = self.scroll_offset.get();
                let (next_x, hit_x) = self.react_to_bounds(
                    current.0 - dx,
                    self.max_scroll_x(),
                    true
                );
                let (next_y, hit_y) = self.react_to_bounds(
                    current.1 - dy,
                    self.max_scroll_y(),
                    true
                );
                if hit_x {
                    self.note_edge_hit(
                        if dx > 0.0 {
                            EdgeSide::Left
                        } else {
                            EdgeSide::Right
                        },
                        ctx
                    );
                }
                if hit_y {
                    self.note_edge_hit(
                        if dy > 0.0 {
                            EdgeSide::Top
                        } else {
                            EdgeSide::Bottom
                        },
                        ctx
                    );
                }

                let next = (next_x, next_y);
                if next != current {
                    self.scroll_offset.set(next);
                    self.scroll_target.set(next);
                    self.base.dirty = true;
                    ctx.request_redraw();
                }

                Some(EventStatus::Handled)
            }

            TouchPanPhase::End => {
                let state = self.touch_pan.take()?;
                self.end_touch_pan(state, ctx);
                Some(EventStatus::Handled)
            }

            TouchPanPhase::Cancel => {
                self.touch_pan.take()?;
                self.spring_back_if_needed(ctx);
                Some(EventStatus::Handled)
            }
        }
    }

    fn end_touch_pan(&mut self, state: TouchPanState, ctx: &mut EventCtx) {
        if !state.dragging {
            return;
        }

        let current = self.scroll_offset.get();
        let out_of_bounds =
            current.0 < 0.0 ||
            current.0 > self.max_scroll_x() ||
            current.1 < 0.0 ||
            current.1 > self.max_scroll_y();

        if out_of_bounds {
            self.spring_back_if_needed(ctx);
            return;
        }

        let sf = self.scale_factor.get();
        let vx = if self.is_scrollable_x() { state.velocity.0 } else { 0.0 };
        let vy = if self.is_scrollable_y() { state.velocity.1 } else { 0.0 };

        if vx.abs() < MOMENTUM_MIN_SPEED * sf && vy.abs() < MOMENTUM_MIN_SPEED * sf {
            return;
        }

        self.momentum.set(Some(MomentumState { velocity: (vx, vy) }));
        self.base.dirty = true;
        ctx.request_redraw();
    }

    fn tick_momentum(&mut self, dt: f32, ctx: &mut EventCtx) {
        let Some(mut state) = self.momentum.get() else {
            return;
        };

        let current = self.scroll_offset.get();
        let raw_x = current.0 - state.velocity.0 * dt;
        let raw_y = current.1 - state.velocity.1 * dt;

        let max_x = self.max_scroll_x();
        let max_y = self.max_scroll_y();

        // Momentum always integrates against the hard bounds - any
        // rubber-bounce is a separate, explicit hand-off (below) instead
        // of something momentum has to keep reconciling every tick.
        let clamped_x = raw_x.clamp(0.0, max_x);
        let clamped_y = raw_y.clamp(0.0, max_y);
        let hit_x = clamped_x != raw_x;
        let hit_y = clamped_y != raw_y;

        if hit_x {
            self.note_edge_hit(
                if state.velocity.0 > 0.0 {
                    EdgeSide::Left
                } else {
                    EdgeSide::Right
                },
                ctx
            );
        }
        if hit_y {
            self.note_edge_hit(
                if state.velocity.1 > 0.0 {
                    EdgeSide::Top
                } else {
                    EdgeSide::Bottom
                },
                ctx
            );
        }

        self.scroll_offset.set((clamped_x, clamped_y));
        self.scroll_target.set((clamped_x, clamped_y));
        self.base.dirty = true;
        ctx.request_redraw();

        let decay = (-MOMENTUM_FRICTION * dt).exp();
        state.velocity.0 *= if hit_x { 0.0 } else { decay };
        state.velocity.1 *= if hit_y { 0.0 } else { decay };

        let sf = self.scale_factor.get();
        let min_speed = MOMENTUM_MIN_SPEED * sf;
        let settled = state.velocity.0.abs() < min_speed && state.velocity.1.abs() < min_speed;

        if settled {
            self.momentum.set(None);
            if
                (hit_x || hit_y) &&
                matches!(self.effective_overscroll(), Overscroll::Bounce | Overscroll::Stretch)
            {
                self.play_bounce_impact(hit_x, hit_y, ctx);
            }
        } else {
            self.momentum.set(Some(state));
        }
    }

    // A brief rubber-band-and-return played once a fling comes to rest
    // against the bound, giving Bounce/Stretch a little give-and-spring-back
    // on impact without momentum having to track overscroll every tick.
    fn play_bounce_impact(&mut self, hit_x: bool, hit_y: bool, ctx: &mut EventCtx) {
        let sf = self.scale_factor.get();
        let nudge_scale = if self.effective_overscroll() == Overscroll::Stretch {
            0.12
        } else {
            0.22
        };
        let nudge = OVERSCROLL_RUBBER_BAND_RANGE * nudge_scale * sf;
        let mut offset = self.scroll_offset.get();

        if hit_x {
            offset.0 = if offset.0 <= 0.0 { -nudge } else { offset.0 + nudge };
        }
        if hit_y {
            offset.1 = if offset.1 <= 0.0 { -nudge } else { offset.1 + nudge };
        }

        self.scroll_offset.set(offset);
        self.spring_back_if_needed(ctx);
    }

    // Renders a soft translucent band at whichever edges recently hit
    // their scroll bound under `Overscroll::Glow` (a flat fade instead of
    // a true radial gradient, since the paint primitives here don't have
    // a gradient shader).
    fn paint_overscroll_glow(&self, ctx: &mut PaintContext) {
        let glow = self.overscroll_glow.get();
        if glow.iter().all(|v| *v <= 0.0) {
            return;
        }

        let b = self.layout_box;
        let sf = self.scale_factor.get();
        let band = 48.0 * sf;
        let color = crate::current_theme().primary;

        let mut draw_band = |alpha: f32, position: (f32, f32), size: (f32, f32)| {
            if alpha <= 0.0 {
                return;
            }
            ctx.draw_rect(RectCommand {
                position,
                size,
                background: Some(Background::Color(color.with_alpha_f32(color.a() * alpha * 0.35))),
                border_radius: None,
                border_width: None,
                border_color: None,
                clip_rect: None,
            });
        };

        draw_band(glow[0], (b.x, b.y), (b.width, band));
        draw_band(glow[1], (b.x + b.width - band, b.y), (band, b.height));
        draw_band(glow[2], (b.x, b.y + b.height - band), (b.width, band));
        draw_band(glow[3], (b.x, b.y), (band, b.height));
    }
}

impl Default for View {
    fn default() -> Self {
        Self::new()
    }
}

impl StyleBuilder for View {
    fn style_mut(&mut self) -> &mut Style {
        &mut self.base.style
    }

    fn mark_dirty(&mut self) {
        self.base.dirty = true;
        self.recompute_style();
    }
}

crate::impl_interaction_builders!(base View);
crate::impl_common_style_builders!(base View);
crate::impl_themed_style_builders!(base View; hover_style => hover_style, pressed_style => pressed_style, disabled_style => disabled_style, focus_style => focus_style, focused_hover_style => focused_hover_style, focused_pressed_style => focused_pressed_style);

impl Widget for View {
    crate::impl_widget_boilerplate!();

    fn debug_name(&self) -> &'static str {
        "Widget#View"
    }

    fn children(&self) -> &[Box<dyn Widget>] {
        &self.children
    }

    fn children_mut(&mut self) -> Option<&mut Vec<Box<dyn Widget>>> {
        Some(&mut self.children)
    }

    fn scroll_offset(&self) -> (f32, f32) {
        self.scroll_offset.get()
    }

    fn set_content_size(&mut self, size: (f32, f32)) {
        self.content_size.set(size);
        // Re-clamp in case the scrollable range shrank (e.g. children were
        // removed, or the viewport was resized).
        self.scroll_offset.set(self.clamp_offset(self.scroll_offset.get()));
        self.scroll_target.set(self.clamp_offset(self.scroll_target.get()));
    }

    fn clip_children(&self) -> Option<(f32, f32, f32, f32)> {
        if self.clips_x() || self.clips_y() {
            let b = &self.layout_box;
            Some((b.x, b.y, b.width, b.height))
        } else {
            None
        }
    }

    fn measure(&self, _ctx: &mut MeasureContext, _constraints: Constraints) -> MeasureResult {
        MeasureResult::new(0.0, 0.0)
    }

    fn on_layout_pass(&self, ctx: &mut MeasureContext) {
        self.scale_factor.set(ctx.scale_factor);
    }

    fn paint(&self, ctx: &mut PaintContext) {
        self.paint_box(ctx);
        self.paint_outline(ctx);
    }

    fn paint_overlay(&self, ctx: &mut PaintContext) {
        let sb = self.active_scrollbar();
        let b = self.layout_box;
        let t = sb.thickness;
        let (active_x, active_y) = self.scrollbar_active();
        let (shows_x, shows_y) = self.scrollbar_visibility();

        let thumb_border_width = (sb.thumb_border_width > 0.0).then(||
            Length::px(sb.thumb_border_width)
        );
        let track_border_width = (sb.track_border_width > 0.0).then(||
            Length::px(sb.track_border_width)
        );

        if shows_y {
            let dim = if active_y { 1.0 } else { SCROLLBAR_DISABLED_OPACITY };

            if sb.track_color.a() > 0.0 || track_border_width.is_some() {
                ctx.draw_rect(RectCommand {
                    position: (b.x + b.width - self.scrollbar_right_inset.get() - t, b.y),
                    size: (t, b.height),
                    background: Some(
                        Background::Color(sb.track_color.with_alpha_f32(sb.track_color.a() * dim))
                    ),
                    border_radius: None,
                    border_width: track_border_width,
                    border_color: Some(
                        sb.track_border_color.with_alpha_f32(sb.track_border_color.a() * dim)
                    ),
                    clip_rect: None,
                });
            }

            if let Some((x, y, w, h)) = self.vertical_thumb_rect() {
                ctx.draw_rect(RectCommand {
                    position: (x, y),
                    size: (w, h),
                    background: Some(
                        Background::Color(sb.thumb_color.with_alpha_f32(sb.thumb_color.a() * dim))
                    ),
                    border_radius: Some(BorderRadius::all(Length::px(sb.thumb_radius))),
                    border_width: thumb_border_width,
                    border_color: Some(
                        sb.thumb_border_color.with_alpha_f32(sb.thumb_border_color.a() * dim)
                    ),
                    clip_rect: None,
                });
            }
        }

        if shows_x {
            let dim = if active_x { 1.0 } else { SCROLLBAR_DISABLED_OPACITY };

            if sb.track_color.a() > 0.0 || track_border_width.is_some() {
                ctx.draw_rect(RectCommand {
                    position: (b.x, b.y + b.height - self.scrollbar_bottom_inset.get() - t),
                    size: (b.width, t),
                    background: Some(
                        Background::Color(sb.track_color.with_alpha_f32(sb.track_color.a() * dim))
                    ),
                    border_radius: None,
                    border_width: track_border_width,
                    border_color: Some(
                        sb.track_border_color.with_alpha_f32(sb.track_border_color.a() * dim)
                    ),
                    clip_rect: None,
                });
            }

            if let Some((x, y, w, h)) = self.horizontal_thumb_rect() {
                ctx.draw_rect(RectCommand {
                    position: (x, y),
                    size: (w, h),
                    background: Some(
                        Background::Color(sb.thumb_color.with_alpha_f32(sb.thumb_color.a() * dim))
                    ),
                    border_radius: Some(BorderRadius::all(Length::px(sb.thumb_radius))),
                    border_width: thumb_border_width,
                    border_color: Some(
                        sb.thumb_border_color.with_alpha_f32(sb.thumb_border_color.a() * dim)
                    ),
                    clip_rect: None,
                });
            }
        }

        if let Some((up, down)) = self.vertical_buttons() {
            let target = self.scroll_target.get();
            let axis_dim = if active_y { 1.0 } else { SCROLLBAR_DISABLED_OPACITY };
            let scales = self.arrow_scale.get();

            for (rect, dir, edge_disabled, scale) in [
                (up, ArrowDirection::Up, target.1 <= 0.0, scales[ScrollbarArrow::Up as usize]),
                (
                    down,
                    ArrowDirection::Down,
                    target.1 >= self.max_scroll_y(),
                    scales[ScrollbarArrow::Down as usize],
                ),
            ] {
                let dim = axis_dim * (if edge_disabled { 0.35 } else { 1.0 });
                let color = sb.arrow_color.with_alpha_f32(sb.arrow_color.a() * dim);

                for (p0, p1, p2) in rounded_arrow_triangles(rect, dir, ctx.scale_factor) {
                    let (a, b, c) = scale_arrow_triangle((p0, p1, p2), rect, scale);
                    ctx.draw_triangle(TriangleCommand {
                        p0: a,
                        p1: b,
                        p2: c,
                        color,
                        clip_rect: None,
                    });
                }
            }
        }

        if let Some((left, right)) = self.horizontal_buttons() {
            let target = self.scroll_target.get();
            let axis_dim = if active_x { 1.0 } else { SCROLLBAR_DISABLED_OPACITY };
            let scales = self.arrow_scale.get();

            for (rect, dir, edge_disabled, scale) in [
                (
                    left,
                    ArrowDirection::Left,
                    target.0 <= 0.0,
                    scales[ScrollbarArrow::Left as usize],
                ),
                (
                    right,
                    ArrowDirection::Right,
                    target.0 >= self.max_scroll_x(),
                    scales[ScrollbarArrow::Right as usize],
                ),
            ] {
                let dim = axis_dim * (if edge_disabled { 0.35 } else { 1.0 });
                let color = sb.arrow_color.with_alpha_f32(sb.arrow_color.a() * dim);

                for (p0, p1, p2) in rounded_arrow_triangles(rect, dir, ctx.scale_factor) {
                    let (a, b, c) = scale_arrow_triangle((p0, p1, p2), rect, scale);
                    ctx.draw_triangle(TriangleCommand {
                        p0: a,
                        p1: b,
                        p2: c,
                        color,
                        clip_rect: None,
                    });
                }
            }
        }

        self.paint_overscroll_glow(ctx);
    }

    fn event(&mut self, event: &InputEvent, ctx: &mut EventCtx) -> EventStatus {
        if let Some(status) = self.handle_auto_scroll(event, ctx) {
            return status;
        }
        if let Some(status) = self.handle_touch_pan(event, ctx) {
            return status;
        }

        if let InputEvent::AnimationTick { dt } = event && self.momentum.get().is_some() {
            self.tick_momentum(*dt, ctx);
        }

        if
            let InputEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Right,
                position,
            } = event &&
            let Some(handle) = &self.context_menu &&
            self.hit_test(*position)
        {
            handle.open_at(*position);
            ctx.request_redraw();
            return EventStatus::Handled;
        }

        if let InputEvent::MouseMoved { position } = event {
            if
                let Some(vertical) = self.pending_track_drag.take() &&
                self.scrollbar_drag.get().is_none()
            {
                let current = self.scroll_offset.get();
                let start_offset = if vertical { current.1 } else { current.0 };
                let start_mouse = if vertical { position.1 } else { position.0 };
                self.scrollbar_drag.set(Some(ScrollDrag { vertical, start_mouse, start_offset }));
            }

            if self.scrollbar_drag.get().is_some() && self.handle_scrollbar_drag(*position, ctx) {
                return EventStatus::Handled;
            }

            let now_hovered = self.point_in_scrollbar(*position);
            if now_hovered != self.scrollbar_hovered.get() {
                self.scrollbar_hovered.set(now_hovered);
                self.base.dirty = true;
                ctx.request_redraw();
            }
        }

        if matches!(event, InputEvent::MouseExited) && self.scrollbar_hovered.get() {
            self.scrollbar_hovered.set(false);
            self.base.dirty = true;
            ctx.request_redraw();
        }

        if
            let InputEvent::MouseWheel { delta, position, modifiers } = event &&
            self.handle_wheel(*delta, *position, *modifiers, ctx, self.scroll_step)
        {
            return EventStatus::Handled;
        }

        if
            let InputEvent::KeyInput { event: key_event, modifiers } = event &&
            key_event.state == KeyState::Pressed &&
            self.handle_key(key_event.key, *modifiers, ctx)
        {
            return EventStatus::Handled;
        }

        if
            let InputEvent::MouseInput { state, button, position } = event &&
            self.handle_scrollbar_mouse(*state, *button, *position, ctx)
        {
            return EventStatus::Handled;
        }

        if !self.base.interaction.is_active() {
            return EventStatus::Ignored;
        }

        let before_style = self.base.computed_style.clone();
        let before_focus_visible = self.base.interaction.focus_visible;

        let status = self.base.interaction.handle(event, ctx);

        if matches!(status, EventStatus::Handled) {
            self.recompute_style();

            if
                self.base.computed_style != before_style ||
                self.base.interaction.focus_visible != before_focus_visible
            {
                self.base.dirty = true;
                ctx.request_redraw();
            }
        }

        status
    }

    fn wants_animation_frame(&self) -> bool {
        self.momentum.get().is_some() || self.auto_scroll.get().is_some()
    }

    fn cancel_auto_scroll(&mut self, ctx: &mut EventCtx) {
        if self.auto_scroll.get().is_some() {
            self.auto_scroll.set(None);
            ctx.set_cursor_icon(Cursor::Default);
            ctx.request_redraw();
        }
    }

    fn content_eq(&self, other: &dyn Widget) -> bool {
        let Some(other) = other.as_any().downcast_ref::<View>() else {
            return false;
        };

        self.base.style == other.base.style &&
            self.base.hover_style == other.base.hover_style &&
            self.base.pressed_style == other.base.pressed_style &&
            self.base.disabled_style == other.base.disabled_style &&
            self.base.focus_style == other.base.focus_style &&
            self.base.focused_hover_style == other.base.focused_hover_style
    }

    fn cascade_style(&mut self, parent: &Style, anim: &mut AnimationManager) {
        self.base.inherited_style = parent.clone();
        self.recompute_style();

        if crate::animate_computed_style(self.anim_id, &mut self.base.computed_style, anim) {
            self.base.dirty = true;
        }

        self.apply_scrollbar_gutter();
        self.animate_scroll(anim);
        self.animate_scrollbar_thickness(anim);
        self.animate_scrollbar_arrows(anim);
        self.animate_overscroll_glow(anim);

        for child in self.children.iter_mut() {
            child.cascade_style(&self.base.computed_style, anim);
        }
    }

    fn after_interaction_transfer(&mut self) {
        self.recompute_style();
    }

    fn transfer_interaction_state(&mut self, old: &dyn Widget) {
        if let (Some(new), Some(old_i)) = (self.interaction_mut(), old.interaction()) {
            new.transfer_from(old_i);
        }
        if let Some(old) = old.as_any().downcast_ref::<View>() {
            self.anim_id = old.anim_id;
            self.arrow_anim_ids = old.arrow_anim_ids;
            self.glow_anim_ids = old.glow_anim_ids;
        }
    }

    fn transfer_measured_state(&mut self, old: &dyn Widget) {
        if let Some(old) = old.as_any().downcast_ref::<View>() {
            self.scroll_offset.set(old.scroll_offset.get());
            self.scroll_target.set(old.scroll_target.get());
            self.content_size.set(old.content_size.get());
            self.scrollbar_hovered.set(old.scrollbar_hovered.get());
            self.scrollbar_thickness_anim.set(old.scrollbar_thickness_anim.get());
            self.scrollbar_right_inset.set(old.scrollbar_right_inset.get());
            self.scrollbar_bottom_inset.set(old.scrollbar_bottom_inset.get());
            self.scale_factor.set(old.scale_factor.get());
            self.pressed_arrow.set(old.pressed_arrow.get());
            self.arrow_scale.set(old.arrow_scale.get());
            self.auto_scroll.set(old.auto_scroll.get());
            self.touch_pan.set(old.touch_pan.get());
            self.momentum.set(old.momentum.get());
            self.overscroll_glow.set(old.overscroll_glow.get());
            self.glow_pending_hit.set(old.glow_pending_hit.get());
        }
    }

    fn anim_id(&self) -> WidgetId {
        self.anim_id
    }
}
