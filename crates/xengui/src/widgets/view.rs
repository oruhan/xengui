// SPDX-License-Identifier: Apache-2.0
use crate::*;
use std::cell::Cell;
use web_time::Instant;
use xen_animation::AnimValue;

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
    velocity: (f32, f32),
    activated_at: Instant,
    // Reflects which axes are actually scrollable, chosen once at
    // activation so it matches the native OS pan cursor for that case.
    cursor: Cursor,
}

fn auto_scroll_axis_speed(delta: f32, dead_zone: f32, range: f32, max_speed: f32) -> f32 {
    let magnitude = delta.abs();
    if magnitude <= dead_zone {
        return 0.0;
    }

    let t = ((magnitude - dead_zone) / range.max(f32::EPSILON)).clamp(0.0, 1.0);
    // Smoothstep gives fine control near the origin and still ramps to full
    // speed without the abrupt slope changes of a linear response.
    let eased = t * t * (3.0 - 2.0 * t);
    delta.signum() * max_speed * eased
}

fn approach_velocity(current: f32, target: f32, response: f32, dt: f32) -> f32 {
    let alpha = 1.0 - (-response * dt).exp();
    current + (target - current) * alpha
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

fn platform_default_overscroll() -> Overscroll {
    if cfg!(target_os = "ios") {
        Overscroll::Bounce
    } else if cfg!(target_os = "android") {
        Overscroll::Stretch
    } else if cfg!(target_arch = "wasm32") && crate::platform::is_touch_platform() {
        Overscroll::Bounce
    } else {
        Overscroll::Stretch
    }
}

// Touch-primary platforms show their scrollbar only while actively
// scrolling by default, fading it out afterward; desktop keeps it
// always visible unless the user opts in via `scrollbar_auto_hide`.
fn platform_default_scrollbar_auto_hide() -> bool {
    crate::platform::is_touch_platform()
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
    let scale_pt = |p: Point| {
        (
            center.0 + (p.0 - center.0) * scale,
            center.1 + (p.1 - center.1) * scale,
        )
    };
    (scale_pt(tri.0), scale_pt(tri.1), scale_pt(tri.2))
}

// Builds a filled, rounded-corner triangle pointing in `direction` -
// genuine solid triangle geometry, not a chevron outline.
fn rounded_arrow_triangles(
    rect: (f32, f32, f32, f32),
    direction: ArrowDirection,
    sf: f32,
) -> Vec<Triangle> {
    let (x, y, w, h) = rect;
    let cx = x + w * 0.5;
    let cy = y + h * 0.5;
    let s = SCROLLBAR_ARROW_SIZE * sf;

    let (p0, p1, p2) = match direction {
        ArrowDirection::Up => (
            (cx - s, cy + s * 0.55),
            (cx, cy - s * 0.75),
            (cx + s, cy + s * 0.55),
        ),
        ArrowDirection::Down => (
            (cx - s, cy - s * 0.55),
            (cx, cy + s * 0.75),
            (cx + s, cy - s * 0.55),
        ),
        ArrowDirection::Left => (
            (cx + s * 0.55, cy - s),
            (cx - s * 0.75, cy),
            (cx + s * 0.55, cy + s),
        ),
        ArrowDirection::Right => (
            (cx - s * 0.55, cy - s),
            (cx + s * 0.75, cy),
            (cx - s * 0.55, cy + s),
        ),
    };

    let outline = rounded_triangle_outline(
        p0,
        p1,
        p2,
        SCROLLBAR_ARROW_CORNER_RADIUS * sf,
        SCROLLBAR_ARROW_CAP_SEGMENTS,
    );
    fan_triangulate(&outline)
}

// Rounds each corner of a triangle by replacing it with a small arc
// tangent to both adjacent edges (inscribed-circle construction: offset
// along each edge by the radius, arc center on the angle bisector).
fn rounded_triangle_outline(
    p0: Point,
    p1: Point,
    p2: Point,
    radius: f32,
    segments: usize,
) -> Vec<Point> {
    let sub = |a: Point, b: Point| -> Point { (a.0 - b.0, a.1 - b.1) };
    let len = |v: Point| -> f32 { (v.0 * v.0 + v.1 * v.1).sqrt().max(0.0001) };
    let norm = |v: Point| -> Point {
        let l = len(v);
        (v.0 / l, v.1 / l)
    };

    let mut outline = Vec::with_capacity((segments + 1) * 3);

    for &(prev, corner, next) in &[(p2, p0, p1), (p0, p1, p2), (p1, p2, p0)] {
        let to_prev = sub(prev, corner);
        let to_next = sub(next, corner);
        let dir_prev = norm(to_prev);
        let dir_next = norm(to_next);

        let r = radius.min(len(to_prev) * 0.4).min(len(to_next) * 0.4);

        let a = (corner.0 + dir_prev.0 * r, corner.1 + dir_prev.1 * r);
        let b = (corner.0 + dir_next.0 * r, corner.1 + dir_next.1 * r);

        let bisector = norm((dir_prev.0 + dir_next.0, dir_prev.1 + dir_next.1));
        let cos_half = (dir_prev.0 * bisector.0 + dir_prev.1 * bisector.1).clamp(-1.0, 1.0);
        let sin_half = (1.0 - cos_half * cos_half).max(0.0).sqrt();
        let center_dist = if sin_half > 0.0001 { r / sin_half } else { 0.0 };
        let center = (
            corner.0 + bisector.0 * center_dist,
            corner.1 + bisector.1 * center_dist,
        );

        let start_angle = (a.1 - center.1).atan2(a.0 - center.0);
        let mut end_angle = (b.1 - center.1).atan2(b.0 - center.0);

        let mut delta = end_angle - start_angle;
        while delta <= -std::f32::consts::PI {
            delta += std::f32::consts::TAU;
        }
        while delta > std::f32::consts::PI {
            delta -= std::f32::consts::TAU;
        }
        end_angle = start_angle + delta;

        for i in 0..=segments {
            let t = (i as f32) / (segments as f32);
            let angle = start_angle + (end_angle - start_angle) * t;
            outline.push((center.0 + angle.cos() * r, center.1 + angle.sin() * r));
        }
    }

    outline
}

// Fan-triangulates a convex polygon from its centroid.
fn fan_triangulate(polygon: &[Point]) -> Vec<Triangle> {
    if polygon.len() < 3 {
        return Vec::new();
    }
    let n = polygon.len() as f32;
    let centroid = polygon
        .iter()
        .fold((0.0, 0.0), |acc, p| (acc.0 + p.0, acc.1 + p.1));
    let centroid = (centroid.0 / n, centroid.1 / n);

    (0..polygon.len())
        .map(|i| (centroid, polygon[i], polygon[(i + 1) % polygon.len()]))
        .collect()
}

/// Data and behavior represented by `View`.
pub struct View {
    base: WidgetBase,

    anim_id: WidgetId,
    layout_box: LayoutBox,
    children: Vec<Box<dyn Widget>>,
    scroll_offset: Cell<(f32, f32)>,
    scroll_target: Cell<(f32, f32)>,
    // Offset last reported to the layout engine's scroll-delta reflow
    // pass; diffed against `scroll_offset` each frame so only the moved
    // distance needs to be applied, without a full re-layout.
    last_layout_scroll: Cell<(f32, f32)>,

    content_size: Cell<(f32, f32)>,
    scrollbar_drag: Cell<Option<ScrollDrag>>,
    pending_track_drag: Cell<Option<bool>>,
    scroll_step: f32,
    scrollbar_hovered: Cell<bool>,
    scrollbar_thumb_hovered: Cell<bool>,
    scrollbar_thickness_anim: Cell<f32>,
    thumb_color_anim: Cell<Color>,
    arrow_color_anim: Cell<[Color; 4]>,
    scrollbar_right_inset: Cell<f32>,
    scrollbar_bottom_inset: Cell<f32>,
    // Opacity applied to the whole scrollbar (track/thumb/arrows) while
    // `scrollbar_auto_hide` is active - 1.0 during/just after scrolling,
    // eased back to 0.0 once idle.
    scrollbar_opacity_anim: Cell<f32>,
    scrollbar_opacity_animating: Cell<bool>,
    last_scroll_activity: Cell<Option<Instant>>,
    scale_factor: Cell<f32>,
    context_menu: Option<ContextMenuHandle>,

    // Currently-pressed scrollbar arrow (if any), and its own per-arrow
    // animation identities/scale values, indexed by `ScrollbarArrow as usize`.
    pressed_arrow: Cell<Option<ScrollbarArrow>>,
    arrow_anim_ids: [WidgetId; 4],
    arrow_scale: Cell<[f32; 4]>,
    arrow_hold_time: Cell<f32>,
    arrow_repeat_timer: Cell<f32>,
    hovered_arrow: Cell<Option<ScrollbarArrow>>,

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
    // When true, `set_content_size` snaps vertical scroll straight to the
    // bottom instead of preserving whatever offset the view already had
    pin_scroll_bottom: bool,
}

impl View {
    /// Creates a value with its default configuration.
    pub fn new() -> Self {
        let mut view = Self {
            base: WidgetBase::new(Interaction::new()),

            anim_id: WidgetId::new_unique(),
            layout_box: LayoutBox::default(),
            children: Vec::new(),
            scroll_offset: Cell::new((0.0, 0.0)),
            scroll_target: Cell::new((0.0, 0.0)),
            last_layout_scroll: Cell::new((0.0, 0.0)),

            content_size: Cell::new((0.0, 0.0)),
            scrollbar_drag: Cell::new(None),
            pending_track_drag: Cell::new(None),
            scroll_step: 96.0,
            scrollbar_hovered: Cell::new(false),
            scrollbar_thumb_hovered: Cell::new(false),
            scrollbar_thickness_anim: Cell::new(0.0),
            thumb_color_anim: Cell::new(Color::TRANSPARENT),
            arrow_color_anim: Cell::new([Color::TRANSPARENT; 4]),
            scrollbar_right_inset: Cell::new(0.0),
            scrollbar_bottom_inset: Cell::new(0.0),
            scrollbar_opacity_anim: Cell::new(1.0),
            scrollbar_opacity_animating: Cell::new(false),
            last_scroll_activity: Cell::new(None),
            scale_factor: Cell::new(1.0),
            context_menu: None,

            pressed_arrow: Cell::new(None),
            arrow_anim_ids: [
                WidgetId::new_unique(),
                WidgetId::new_unique(),
                WidgetId::new_unique(),
                WidgetId::new_unique(),
            ],
            arrow_hold_time: Cell::new(0.0),
            arrow_repeat_timer: Cell::new(0.0),
            hovered_arrow: Cell::new(None),
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
            pin_scroll_bottom: false,
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

    /// Returns or updates the `child` value.
    pub fn child(mut self, child: impl Widget + 'static) -> Self {
        self.children.push(Box::new(child));
        self
    }

    /// Returns or updates the `child_boxed` value.
    pub fn child_boxed(mut self, child: Box<dyn Widget>) -> Self {
        self.children.push(child);
        self
    }

    /// Bulk variant of `child` for dynamically built lists where each item
    /// is already a boxed trait object (e.g. produced inside a `.map()`).
    pub fn children_vec(mut self, children: Vec<Box<dyn Widget>>) -> Self {
        self.children = children;
        self
    }

    /// Returns or updates the `focusable` value.
    pub fn focusable(mut self, focusable: bool) -> Self {
        self.base.interaction.focusable = focusable;
        self
    }

    /// Returns or updates the `scroll_step` value.
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

    /// Enables or disables middle-click AutoScroll for this view.
    ///
    /// AutoScroll is enabled by default and activates only when at least
    /// one axis has real scrollable overflow. Moving away from the origin
    /// accelerates smoothly with distance; returning to the origin eases
    /// to rest. A second middle click, another pointer press, the scroll
    /// wheel, or Escape cancels the gesture.
    pub fn auto_scroll(mut self, enabled: bool) -> Self {
        self.auto_scroll_enabled = enabled;
        self
    }

    /// Snaps vertical scroll straight to the maximum offset whenever this
    /// view's content size changes, instead of preserving the previous
    /// scroll position. Used by DevTools' log view so newly appended rows
    /// are always visible without the user scrolling manually.
    pub(super) fn pin_scroll_to_bottom(mut self, pin: bool) -> Self {
        self.pin_scroll_bottom = pin;
        self
    }

    fn recompute_style(&mut self) {
        self.base.recompute_style();
        self.base.interaction.hover_cursor = self.base.computed_style.cursor;
    }

    fn resolved_scrollbar(&self) -> ResolvedScrollbar {
        self.base
            .computed_style
            .scrollbar
            .unwrap_or_default()
            .resolve()
    }

    // Overlays scrollbar_hover / scrollbar_pressed on top of the base scrollbar style.
    fn resolved_scrollbar_for_state(&self, hovered: bool, pressed: bool) -> ResolvedScrollbar {
        let base = self.resolved_scrollbar();
        let style = &self.base.computed_style;

        if pressed {
            return match style
                .scrollbar_pressed
                .as_ref()
                .or(style.scrollbar_hover.as_ref())
            {
                Some(patch) => base.patched(patch, DEFAULT_SCROLLBAR_THUMB_HOVER_THICKNESS),
                None => ResolvedScrollbar {
                    thickness: DEFAULT_SCROLLBAR_THUMB_HOVER_THICKNESS,
                    ..base
                },
            };
        }

        if hovered {
            return match style.scrollbar_hover.as_ref() {
                Some(patch) => base.patched(patch, DEFAULT_SCROLLBAR_THUMB_HOVER_THICKNESS),
                None => ResolvedScrollbar {
                    thickness: DEFAULT_SCROLLBAR_THUMB_HOVER_THICKNESS,
                    ..base
                },
            };
        }

        base
    }

    fn arrows_shown(&self) -> bool {
        self.active_scrollbar().show_arrows
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

    fn effective_auto_hide(&self) -> bool {
        self.base
            .computed_style
            .scrollbar_auto_hide
            .unwrap_or_else(platform_default_scrollbar_auto_hide)
    }

    fn note_scroll_activity(&self) {
        self.last_scroll_activity.set(Some(Instant::now()));
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
            // Stretch resists further than Bounce - a shorter travel range
            // matches Android's tighter, snappier stretch instead of iOS's
            // looser springy bounce.
            let base_range = if mode == Overscroll::Stretch {
                STRETCH_RUBBER_BAND_RANGE
            } else {
                OVERSCROLL_RUBBER_BAND_RANGE
            };
            let range = base_range * self.scale_factor.get();
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
        let axis_active = match side {
            EdgeSide::Top | EdgeSide::Bottom => self.can_scroll_y(),
            EdgeSide::Left | EdgeSide::Right => self.can_scroll_x(),
        };
        if !axis_active {
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
        let active = [
            self.can_scroll_y(),
            self.can_scroll_x(),
            self.can_scroll_y(),
            self.can_scroll_x(),
        ];

        for i in 0..4 {
            let key = AnimKey {
                widget: self.glow_anim_ids[i],
                layer: AnimLayer::Root,
                property: AnimProperty::Opacity,
            };

            if !active[i] {
                pending[i] = false;
                values[i] = 0.0;
                anim.set_target(key, AnimValue([0.0; 4]), None);
                continue;
            }

            if pending[i] {
                anim.set_target(key, AnimValue([1.0, 0.0, 0.0, 0.0]), None);
                pending[i] = false;
            }

            anim.set_target(
                key,
                AnimValue([0.0, 0.0, 0.0, 0.0]),
                Some(OVERSCROLL_GLOW_FADE_TRANSITION),
            );

            match anim.value(key) {
                Some(v) => {
                    values[i] = v.0[0];
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

        for (i, (anim_id, scale)) in self
            .arrow_anim_ids
            .iter()
            .zip(scales.iter_mut())
            .enumerate()
        {
            let is_pressed = pressed.map(|p| p as usize) == Some(i);
            let target = if is_pressed {
                SCROLLBAR_ARROW_PRESS_SCALE
            } else {
                1.0
            };

            let key = AnimKey {
                widget: *anim_id,
                layer: AnimLayer::Root,
                property: AnimProperty::Scale,
            };
            anim.set_target(
                key,
                AnimValue([target, 0.0, 0.0, 0.0]),
                Some(SCROLLBAR_ARROW_PRESS_TRANSITION),
            );

            match anim.value(key) {
                Some(v) => {
                    *scale = v.0[0];
                }
                None => {
                    *scale = target;
                }
            }
        }

        self.arrow_scale.set(scales);
    }

    fn animate_scrollbar_opacity(&mut self, anim: &mut AnimationManager) {
        if !self.effective_auto_hide() {
            self.scrollbar_opacity_anim.set(1.0);
            self.scrollbar_opacity_animating.set(false);
            return;
        }

        // Touch has no real hover concept; a synthetic MouseMoved landing on
        // the track mid-swipe can otherwise flip `scrollbar_hovered` true and
        // leave the bar stuck visible forever, since nothing on touch ever
        // sends a matching "unhover".
        let hover_counts = self.scrollbar_hovered.get() && !crate::platform::is_touch_platform();

        let active_gesture = self.scrollbar_drag.get().is_some()
            || hover_counts
            || self.touch_pan.get().is_some_and(|s| s.dragging)
            || self.momentum.get().is_some()
            || self.auto_scroll.get().is_some()
            || self.pressed_arrow.get().is_some();

        let within_linger = self
            .last_scroll_activity
            .get()
            .is_some_and(|t| Instant::now().duration_since(t) < SCROLLBAR_AUTO_HIDE_LINGER);

        let target = if active_gesture || within_linger {
            1.0
        } else {
            0.0
        };

        let key = AnimKey {
            widget: self.anim_id,
            layer: AnimLayer::Root,
            property: AnimProperty::ScrollbarOpacity,
        };
        anim.set_target(
            key,
            AnimValue([target, 0.0, 0.0, 0.0]),
            Some(SCROLLBAR_OPACITY_FADE_TRANSITION),
        );

        match anim.value(key) {
            Some(v) => {
                self.scrollbar_opacity_anim.set(v.0[0]);
                self.scrollbar_opacity_animating.set(true);
            }
            None => {
                self.scrollbar_opacity_anim.set(target);
                self.scrollbar_opacity_animating.set(false);
            }
        }
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
        let hover = style
            .scrollbar_hover
            .and_then(|p| p.thickness)
            .unwrap_or(DEFAULT_SCROLLBAR_THUMB_HOVER_THICKNESS);
        let pressed = style
            .scrollbar_pressed
            .and_then(|p| p.thickness)
            .unwrap_or(hover);
        idle.max(hover).max(pressed) + SCROLLBAR_THUMB_PADDING * 2.0
    }

    fn track_thickness(&self) -> f32 {
        self.track_thickness_logical() * self.scale_factor.get()
    }

    fn target_scrollbar_thickness(&self) -> f32 {
        let pressed = self.scrollbar_drag.get().is_some();
        let hovered = self.scrollbar_thumb_hovered.get();
        self.resolved_scrollbar_for_state(hovered, pressed)
            .thickness
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
            Some(SCROLLBAR_THICKNESS_TRANSITION),
        );

        match anim.value(key) {
            Some(v) => {
                self.scrollbar_thickness_anim.set(v.0[0]);
            }
            None => self.scrollbar_thickness_anim.set(target),
        }
    }

    // Smoothly blends the thumb and arrow fill colors toward whatever
    // idle/hover/pressed ScrollbarStyle currently applies, instead of
    // snapping instantly.
    fn animate_scrollbar_colors(&mut self, anim: &mut AnimationManager) {
        let pressed = self.scrollbar_drag.get().is_some();
        let hovered = self.scrollbar_hovered.get();
        let target = self.resolved_scrollbar_for_state(hovered, pressed);

        let thumb_key = AnimKey {
            widget: self.anim_id,
            layer: AnimLayer::Root,
            property: AnimProperty::ScrollbarThumbColor,
        };
        anim.set_color_target(
            thumb_key,
            AnimValue(target.thumb_color.to_f32_array()),
            Some(SCROLLBAR_THICKNESS_TRANSITION),
        );
        let thumb_color = match anim.value(thumb_key) {
            Some(v) => Color::rgba_f32(v.0[0], v.0[1], v.0[2], v.0[3]),
            None => target.thumb_color,
        };
        self.thumb_color_anim.set(thumb_color);

        let arrow_key = AnimKey {
            widget: self.anim_id,
            layer: AnimLayer::Root,
            property: AnimProperty::ScrollbarArrowColor,
        };
        anim.set_color_target(
            arrow_key,
            AnimValue(target.arrow_color.to_f32_array()),
            Some(SCROLLBAR_THICKNESS_TRANSITION),
        );
        let arrow_color = match anim.value(arrow_key) {
            Some(v) => Color::rgba_f32(v.0[0], v.0[1], v.0[2], v.0[3]),
            None => target.arrow_color,
        };
        self.arrow_color_anim.set([arrow_color; 4]);
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
        let gutter = self
            .base
            .computed_style
            .scrollbar_gutter
            .unwrap_or_default();

        let sf = self.scale_factor.get();
        let thickness = self.track_thickness_logical();
        let mut padding = self.base.computed_style.padding.unwrap_or_default();

        let (shows_x, shows_y) = if gutter == ScrollbarGutter::Auto {
            self.scrollbar_visibility()
        } else {
            (self.is_scrollable_x(), self.is_scrollable_y())
        };

        self.scrollbar_right_inset.set(if shows_y {
            padding.right.to_physical(sf)
        } else {
            0.0
        });
        self.scrollbar_bottom_inset.set(if shows_x {
            padding.bottom.to_physical(sf)
        } else {
            0.0
        });

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

        let direct_manipulation = self.scrollbar_drag.get().is_some()
            || self.touch_pan.get().is_some()
            || self.momentum.get().is_some()
            || self.auto_scroll.get().is_some();

        let offset = self.scroll_offset.get();
        let overscrolled = offset.0 < 0.0
            || offset.0 > self.max_scroll_x()
            || offset.1 < 0.0
            || offset.1 > self.max_scroll_y();

        let transition = if direct_manipulation {
            None
        } else if overscrolled {
            Some(if self.effective_overscroll() == Overscroll::Stretch {
                STRETCH_RETURN_TRANSITION
            } else {
                OVERSCROLL_RETURN_TRANSITION
            })
        } else {
            Some(SCROLL_TRANSITION)
        };

        anim.set_target(key, AnimValue([target.0, target.1, 0.0, 0.0]), transition);

        match anim.value(key) {
            // Repositioning happens via the layout engine's scroll-delta
            // reflow pass, not the paint-cache dirty flag - marking dirty
            // here would force a full taffy re-layout every scroll frame.
            Some(v) => self.scroll_offset.set((v.0[0], v.0[1])),
            None => self.scroll_offset.set(target),
        }
    }

    // Hit area for scrollbar hover detection; uses the hover thickness so
    // a thin, unhovered bar is still easy to reach with the pointer.
    fn point_in_scrollbar(&self, point: (f32, f32)) -> bool {
        let (active_x, active_y) = self.scrollbar_active();
        if !active_x && !active_y {
            return false;
        }

        let b = self.layout_box;
        let t = self.track_thickness();
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

    // Track hover may still affect authored scrollbar colors, but thumb
    // thickness follows only the thumb's own hit area. This mirrors HTML
    // scrollbar behavior and avoids expanding the thumb over empty track.
    fn point_in_scrollbar_thumb(&self, point: (f32, f32)) -> bool {
        self.vertical_thumb_hit_rect()
            .is_some_and(|rect| point_in_rect(point, rect))
            || self
                .horizontal_thumb_hit_rect()
                .is_some_and(|rect| point_in_rect(point, rect))
    }

    fn is_scrollable_x(&self) -> bool {
        matches!(
            self.base.computed_style.overflow_x,
            Some(Overflow::Scroll | Overflow::Auto)
        )
    }

    fn is_scrollable_y(&self) -> bool {
        matches!(
            self.base.computed_style.overflow_y,
            Some(Overflow::Scroll | Overflow::Auto)
        )
    }

    // `overflow: auto/scroll` creates a potential scroll container, but
    // input may only move an axis when content actually overflows it.
    // Keeping both concepts preserves disabled `overflow: scroll` tracks
    // and stable gutters without making an empty container rubber-band.
    fn can_scroll_x(&self) -> bool {
        self.is_scrollable_x() && self.max_scroll_x() > 0.0
    }

    fn can_scroll_y(&self) -> bool {
        self.is_scrollable_y() && self.max_scroll_y() > 0.0
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
        (
            if self.is_scrollable_x() {
                offset.0.clamp(0.0, self.max_scroll_x())
            } else {
                0.0
            },
            if self.is_scrollable_y() {
                offset.1.clamp(0.0, self.max_scroll_y())
            } else {
                0.0
            },
        )
    }

    // Whether each axis's scrollbar should be painted at all. `Scroll`
    // mode is always shown (even without overflow, as a disabled track);
    // `Auto` only shows once there's real overflow to scroll.
    fn scrollbar_visibility(&self) -> (bool, bool) {
        let shown = |overflow: Option<Overflow>, max_scroll: f32| match overflow {
            Some(Overflow::Scroll) => true,
            Some(Overflow::Auto) => max_scroll > 0.0,
            _ => false,
        };
        (
            shown(self.base.computed_style.overflow_x, self.max_scroll_x()),
            shown(self.base.computed_style.overflow_y, self.max_scroll_y()),
        )
    }

    // Whether each axis actually has overflow to scroll; a `Scroll`-mode
    // scrollbar can be shown (see `scrollbar_visibility`) but inactive.
    fn scrollbar_active(&self) -> (bool, bool) {
        (self.can_scroll_x(), self.can_scroll_y())
    }

    fn vertical_track_bounds(&self) -> Option<(f32, f32)> {
        let (has_x, has_y) = self.scrollbar_visibility();
        if !has_y {
            return None;
        }
        let b = self.layout_box;
        let t = self.active_scrollbar().thickness;
        let full_h = if has_x { b.height - t } else { b.height };
        // Always reserve the same end padding whether or not arrow buttons
        // are actually drawn, so a touch scrollbar (no arrows) still keeps
        // its thumb clear of the track's rounded ends instead of running
        // flush edge-to-edge.
        Some((b.y + t, (full_h - 2.0 * t).max(0.0)))
    }

    fn horizontal_track_bounds(&self) -> Option<(f32, f32)> {
        let (has_x, _) = self.scrollbar_visibility();
        if !has_x {
            return None;
        }
        let b = self.layout_box;
        let t = self.active_scrollbar().thickness;
        let (_, has_y) = self.scrollbar_visibility();
        let full_w = if has_y { b.width - t } else { b.width };
        Some((b.x + t, (full_w - 2.0 * t).max(0.0)))
    }

    fn vertical_thumb_rect(&self) -> Option<(f32, f32, f32, f32)> {
        if self.max_scroll_y() <= 0.0 {
            return None;
        }
        let (track_y, track_h) = self.vertical_track_bounds()?;
        let b = self.layout_box;
        let sb = self.active_scrollbar();
        let content_h = self.content_size.get().1.max(b.height);

        let thumb_h = ((track_h * b.height) / content_h)
            .max(sb.min_thumb_length)
            .min(track_h);
        let max_offset = self.max_scroll_y();
        // Clamped before computing progress so the thumb pins to the track's
        // edge during overscroll instead of sliding past it, while the
        // content itself keeps rubber-banding beyond its real scroll range.
        let clamped_offset = self.scroll_offset.get().1.clamp(0.0, max_offset);
        let progress = if max_offset > 0.0 {
            clamped_offset / max_offset
        } else {
            0.0
        };
        let thumb_y = track_y + progress * (track_h - thumb_h);

        let thumb_w =
            (self.current_scrollbar_thickness() * self.scale_factor.get()).min(sb.thickness);
        let right_inset = self.scrollbar_right_inset.get();
        let thumb_x = b.x + b.width - right_inset - sb.thickness + (sb.thickness - thumb_w) * 0.5;
        Some((thumb_x, thumb_y, thumb_w, thumb_h))
    }

    fn vertical_thumb_hit_rect(&self) -> Option<(f32, f32, f32, f32)> {
        if self.max_scroll_y() <= 0.0 {
            return None;
        }
        let (track_y, track_h) = self.vertical_track_bounds()?;
        let b = self.layout_box;
        let sb = self.active_scrollbar();
        let content_h = self.content_size.get().1.max(b.height);

        let thumb_h = ((track_h * b.height) / content_h)
            .max(sb.min_thumb_length)
            .min(track_h);
        let max_offset = self.max_scroll_y();
        let clamped_offset = self.scroll_offset.get().1.clamp(0.0, max_offset);
        let progress = if max_offset > 0.0 {
            clamped_offset / max_offset
        } else {
            0.0
        };
        let thumb_y = track_y + progress * (track_h - thumb_h);

        let right_inset = self.scrollbar_right_inset.get();
        let thumb_x = b.x + b.width - right_inset - sb.thickness;
        Some((thumb_x, thumb_y, sb.thickness, thumb_h))
    }

    fn horizontal_thumb_rect(&self) -> Option<(f32, f32, f32, f32)> {
        if self.max_scroll_x() <= 0.0 {
            return None;
        }
        let (track_x, track_w) = self.horizontal_track_bounds()?;
        let b = self.layout_box;
        let sb = self.active_scrollbar();
        let content_w = self.content_size.get().0.max(b.width);

        let thumb_w = ((track_w * b.width) / content_w)
            .max(sb.min_thumb_length)
            .min(track_w);
        let max_offset = self.max_scroll_x();
        let clamped_offset = self.scroll_offset.get().0.clamp(0.0, max_offset);
        let progress = if max_offset > 0.0 {
            clamped_offset / max_offset
        } else {
            0.0
        };
        let thumb_x = track_x + progress * (track_w - thumb_w);

        let thumb_h =
            (self.current_scrollbar_thickness() * self.scale_factor.get()).min(sb.thickness);
        let bottom_inset = self.scrollbar_bottom_inset.get();
        let thumb_y = b.y + b.height - bottom_inset - sb.thickness + (sb.thickness - thumb_h) * 0.5;
        Some((thumb_x, thumb_y, thumb_w, thumb_h))
    }

    fn horizontal_thumb_hit_rect(&self) -> Option<(f32, f32, f32, f32)> {
        if self.max_scroll_x() <= 0.0 {
            return None;
        }
        let (track_x, track_w) = self.horizontal_track_bounds()?;
        let b = self.layout_box;
        let sb = self.active_scrollbar();
        let content_w = self.content_size.get().0.max(b.width);

        let thumb_w = ((track_w * b.width) / content_w)
            .max(sb.min_thumb_length)
            .min(track_w);
        let max_offset = self.max_scroll_x();
        let clamped_offset = self.scroll_offset.get().0.clamp(0.0, max_offset);
        let progress = if max_offset > 0.0 {
            clamped_offset / max_offset
        } else {
            0.0
        };
        let thumb_x = track_x + progress * (track_w - thumb_w);

        let bottom_inset = self.scrollbar_bottom_inset.get();
        let thumb_y = b.y + b.height - bottom_inset - sb.thickness;
        Some((thumb_x, thumb_y, thumb_w, sb.thickness))
    }

    fn vertical_buttons(&self) -> Option<(Rect, Rect)> {
        if !self.arrows_shown() {
            return None;
        }
        let (_, has_y) = self.scrollbar_visibility();
        if !has_y {
            return None;
        }
        let b = self.layout_box;
        let t = self.active_scrollbar().thickness;
        let right_inset = self.scrollbar_right_inset.get();
        let (has_x, _) = self.scrollbar_visibility();
        let bottom = if has_x {
            b.y + b.height - t
        } else {
            b.y + b.height
        };
        Some((
            (b.x + b.width - right_inset - t, b.y, t, t),
            (b.x + b.width - right_inset - t, bottom - t, t, t),
        ))
    }

    fn horizontal_buttons(&self) -> Option<(Rect, Rect)> {
        if !self.arrows_shown() {
            return None;
        }
        let (has_x, _) = self.scrollbar_visibility();
        if !has_x {
            return None;
        }
        let b = self.layout_box;
        let t = self.active_scrollbar().thickness;
        let bottom_inset = self.scrollbar_bottom_inset.get();
        let (_, has_y) = self.scrollbar_visibility();
        let right = if has_y {
            b.x + b.width - t
        } else {
            b.x + b.width
        };
        Some((
            (b.x, b.y + b.height - bottom_inset - t, t, t),
            (right - t, b.y + b.height - bottom_inset - t, t, t),
        ))
    }

    fn arrow_at(&self, position: (f32, f32)) -> Option<ScrollbarArrow> {
        if let Some((up, down)) = self.vertical_buttons() {
            if point_in_rect(position, up) {
                return Some(ScrollbarArrow::Up);
            }
            if point_in_rect(position, down) {
                return Some(ScrollbarArrow::Down);
            }
        }
        if let Some((left, right)) = self.horizontal_buttons() {
            if point_in_rect(position, left) {
                return Some(ScrollbarArrow::Left);
            }
            if point_in_rect(position, right) {
                return Some(ScrollbarArrow::Right);
            }
        }
        None
    }

    fn start_scroll_animation(&mut self, target: (f32, f32), ctx: &mut EventCtx) {
        self.scroll_target.set(target);
        self.base.dirty = true;
        self.note_scroll_activity();
        ctx.request_redraw();
    }

    // scroll_step is authored in logical px; every other Length in this
    // widget's layout goes through to_physical(scale_factor), so wheel/
    // arrow-button scroll distance must too, or it drifts relative to
    // content whenever the platform's own scale_factor differs.
    fn scroll_step_physical(&self) -> f32 {
        self.scroll_step * self.scale_factor.get()
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
            if !self.can_scroll_x() {
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

        if !self.can_scroll_y() {
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
        scroll_step: f32,
    ) -> bool {
        if !self.hit_test(position) || (!self.can_scroll_x() && !self.can_scroll_y()) {
            return false;
        }

        let scroll_step = scroll_step * self.scale_factor.get();

        let (raw_dx, raw_dy) = match delta {
            MouseScrollDelta::LineDelta(x, y) => (-x * scroll_step, -y * scroll_step),
            MouseScrollDelta::PixelDelta(x, y) => (-x as f32, -y as f32),
        };

        let (raw_dx, raw_dy) = if modifiers.shift {
            (raw_dy, raw_dx)
        } else {
            (raw_dx, raw_dy)
        };

        // Match browser routing: a vertical wheel delta remains vertical
        // and bubbles past an X-only scroller. Shift swaps the axes above,
        // providing the conventional horizontal-wheel gesture.
        let dx = if self.can_scroll_x() { raw_dx } else { 0.0 };
        let dy = if self.can_scroll_y() { raw_dy } else { 0.0 };

        if dx == 0.0 && dy == 0.0 {
            return false;
        }

        self.cancel_conflicting_gestures();

        let current = self.scroll_target.get();
        // Wheel input is a discrete nudge. Bounce/stretch is reserved for
        // direct touch manipulation and momentum; wheel scrolling clamps.
        let (next_x, hit_x) = self.react_to_bounds(current.0 + dx, self.max_scroll_x(), false);
        let (next_y, hit_y) = self.react_to_bounds(current.1 + dy, self.max_scroll_y(), false);

        if hit_x {
            self.note_edge_hit(
                if dx < 0.0 {
                    EdgeSide::Left
                } else {
                    EdgeSide::Right
                },
                ctx,
            );
        }
        if hit_y {
            self.note_edge_hit(
                if dy < 0.0 {
                    EdgeSide::Top
                } else {
                    EdgeSide::Bottom
                },
                ctx,
            );
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
        ctx: &mut EventCtx,
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
                        if target.1 > 0.0 {
                            self.pressed_arrow.set(Some(ScrollbarArrow::Up));
                            self.arrow_hold_time.set(0.0);
                            self.arrow_repeat_timer.set(0.0);
                            self.nudge(0.0, -self.scroll_step_physical(), ctx);
                        }
                        ctx.request_redraw();
                        return true;
                    }
                    if point_in_rect(position, down) {
                        if target.1 < self.max_scroll_y() {
                            self.pressed_arrow.set(Some(ScrollbarArrow::Down));
                            self.arrow_hold_time.set(0.0);
                            self.arrow_repeat_timer.set(0.0);
                            self.nudge(0.0, self.scroll_step_physical(), ctx);
                        }
                        ctx.request_redraw();
                        return true;
                    }
                }
                if active_x && let Some((left, right)) = self.horizontal_buttons() {
                    if point_in_rect(position, left) {
                        if target.0 > 0.0 {
                            self.pressed_arrow.set(Some(ScrollbarArrow::Left));
                            self.arrow_hold_time.set(0.0);
                            self.arrow_repeat_timer.set(0.0);
                            self.nudge(-self.scroll_step_physical(), 0.0, ctx);
                        }
                        ctx.request_redraw();
                        return true;
                    }
                    if point_in_rect(position, right) {
                        if target.0 < self.max_scroll_x() {
                            self.pressed_arrow.set(Some(ScrollbarArrow::Right));
                            self.arrow_hold_time.set(0.0);
                            self.arrow_repeat_timer.set(0.0);
                            self.nudge(self.scroll_step_physical(), 0.0, ctx);
                        }
                        ctx.request_redraw();
                        return true;
                    }
                }

                if active_y
                    && let Some(thumb) = self.vertical_thumb_hit_rect()
                    && point_in_rect(position, thumb)
                {
                    self.scrollbar_drag.set(Some(ScrollDrag {
                        vertical: true,
                        start_mouse: position.1,
                        start_offset: self.scroll_offset.get().1,
                    }));
                    return true;
                }
                if active_x
                    && let Some(thumb) = self.horizontal_thumb_hit_rect()
                    && point_in_rect(position, thumb)
                {
                    self.scrollbar_drag.set(Some(ScrollDrag {
                        vertical: false,
                        start_mouse: position.0,
                        start_offset: self.scroll_offset.get().0,
                    }));
                    return true;
                }

                // Clicking an empty stretch of track jumps the thumb straight to that
                // point instead of requiring a drag - this "click-to-jump" behavior is
                // desktop-only. Touch scrollbars only respond to dragging the thumb
                // itself; a track tap doing nothing is the native mobile convention
                // (Android/iOS never jump-scroll from a track tap).
                if !crate::platform::is_touch_platform() {
                    if active_y && let Some((track_y, track_h)) = self.vertical_track_bounds() {
                        let t = self.active_scrollbar().thickness;
                        let b = self.layout_box;
                        let right_inset = self.scrollbar_right_inset.get();
                        if point_in_rect(
                            position,
                            (b.x + b.width - right_inset - t, track_y, t, track_h),
                        ) {
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
                        if point_in_rect(
                            position,
                            (track_x, b.y + b.height - bottom_inset - t, track_w, t),
                        ) {
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
                }

                false
            }
            ElementState::Released => {
                self.pending_track_drag.set(None);
                let had_pressed_arrow = self.pressed_arrow.take().is_some();

                if self.scrollbar_drag.get().is_some() {
                    self.scrollbar_drag.set(None);
                    self.note_scroll_activity();
                    ctx.request_redraw();
                    true
                } else if had_pressed_arrow {
                    self.note_scroll_activity();
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
        let thumb_h = ((track_h * b.height) / content_h)
            .max(sb.min_thumb_length)
            .min(track_h);
        let travel = (track_h - thumb_h).max(1.0);
        let progress = ((mouse_y - track_y - thumb_h * 0.5) / travel).clamp(0.0, 1.0);
        Some(progress * self.max_scroll_y())
    }

    fn horizontal_track_offset_for(&self, mouse_x: f32) -> Option<f32> {
        let (track_x, track_w) = self.horizontal_track_bounds()?;
        let b = self.layout_box;
        let sb = self.active_scrollbar();
        let content_w = self.content_size.get().0.max(b.width);
        let thumb_w = ((track_w * b.width) / content_w)
            .max(sb.min_thumb_length)
            .min(track_w);
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

        let mouse_pos = if drag.vertical {
            position.1
        } else {
            position.0
        };
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
        ctx: &mut EventCtx,
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
                let scrollable = self.is_scrollable_x() || self.is_scrollable_y();
                if self.auto_scroll_enabled
                    && scrollable
                    && self.hit_test(*position)
                    && (active_x || active_y)
                {
                    self.cancel_conflicting_gestures();
                    let cursor = self.autoscroll_cursor();
                    self.auto_scroll.set(Some(AutoScrollState {
                        origin: *position,
                        current: *position,
                        velocity: (0.0, 0.0),
                        activated_at: Instant::now(),
                        cursor,
                    }));
                    ctx.set_cursor_icon(cursor);
                    // Escape only reaches a widget on the focused path -
                    // claiming focus here is what lets it cancel AutoScroll.
                    ctx.request_focus();
                    ctx.request_redraw();
                    return Some(EventStatus::Handled);
                }

                None
            }

            InputEvent::MouseInput {
                state: ElementState::Pressed,
                ..
            } if self.auto_scroll.get().is_some() => {
                self.auto_scroll.set(None);
                ctx.set_cursor_icon(Cursor::Default);
                ctx.request_redraw();
                Some(EventStatus::Handled)
            }

            InputEvent::KeyInput {
                event: key_event, ..
            } if self.auto_scroll.get().is_some()
                && key_event.key == Key::Escape
                && key_event.state == KeyState::Pressed =>
            {
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
        let Some(mut state) = self.auto_scroll.get() else {
            return;
        };
        // A stalled tab or debugger pause must not turn into a giant jump.
        let dt = dt.clamp(0.0, 1.0 / 20.0);
        let sf = self.scale_factor.get();
        let dead_zone = AUTO_SCROLL_DEAD_ZONE_DP * sf;
        let range = AUTO_SCROLL_RANGE_DP * sf;
        let max_speed = AUTO_SCROLL_MAX_SPEED * sf;

        let dx = state.current.0 - state.origin.0;
        let dy = state.current.1 - state.origin.1;

        let mut target_vx = if self.can_scroll_x() {
            auto_scroll_axis_speed(dx, dead_zone, range, max_speed)
        } else {
            0.0
        };
        let mut target_vy = if self.can_scroll_y() {
            auto_scroll_axis_speed(dy, dead_zone, range, max_speed)
        } else {
            0.0
        };

        // Cap diagonal motion to the same total maximum as a single axis,
        // avoiding an unintended sqrt(2) speed boost in two-axis views.
        let target_length = target_vx.hypot(target_vy);
        if target_length > max_speed {
            let scale = max_speed / target_length;
            target_vx *= scale;
            target_vy *= scale;
        }

        let response_x = if target_vx == 0.0 {
            AUTO_SCROLL_DECELERATION
        } else {
            AUTO_SCROLL_ACCELERATION
        };
        let response_y = if target_vy == 0.0 {
            AUTO_SCROLL_DECELERATION
        } else {
            AUTO_SCROLL_ACCELERATION
        };
        state.velocity.0 = approach_velocity(state.velocity.0, target_vx, response_x, dt);
        state.velocity.1 = approach_velocity(state.velocity.1, target_vy, response_y, dt);

        let stop_threshold = 0.5 * sf;
        if target_vx == 0.0 && state.velocity.0.abs() < stop_threshold {
            state.velocity.0 = 0.0;
        }
        if target_vy == 0.0 && state.velocity.1.abs() < stop_threshold {
            state.velocity.1 = 0.0;
        }

        self.auto_scroll.set(Some(state));

        let current = self.scroll_offset.get();
        let (next_x, hit_x) = self.react_to_bounds(
            current.0 + state.velocity.0 * dt,
            self.max_scroll_x(),
            false,
        );
        let (next_y, hit_y) = self.react_to_bounds(
            current.1 + state.velocity.1 * dt,
            self.max_scroll_y(),
            false,
        );

        if hit_x {
            self.note_edge_hit(
                if state.velocity.0 < 0.0 {
                    EdgeSide::Left
                } else {
                    EdgeSide::Right
                },
                ctx,
            );
        }
        if hit_y {
            self.note_edge_hit(
                if state.velocity.1 < 0.0 {
                    EdgeSide::Top
                } else {
                    EdgeSide::Bottom
                },
                ctx,
            );
        }

        let next = (next_x, next_y);
        if next != current {
            self.scroll_offset.set(next);
            self.scroll_target.set(next);
            self.note_scroll_activity();
        }
        // Keeps velocity easing and the origin marker animation alive even
        // while stationary in the dead zone or pressed against a bound.
        ctx.request_redraw();
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
                let (active_x, active_y) = self.scrollbar_active();
                if !active_x && !active_y {
                    return None;
                }
                self.cancel_conflicting_gestures();
                self.touch_pan.set(Some(TouchPanState {
                    origin: *position,
                    last_position: *position,
                    last_time: Instant::now(),
                    velocity: (0.0, 0.0),
                    dragging: false,
                }));
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
                        state.last_position = *position;
                        state.last_time = now;
                        self.touch_pan.set(Some(state));
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

                let current = self.scroll_offset.get();
                // A disabled axis must never move or rubber-band, even
                // when the drag has a component along it - otherwise a
                // mostly-vertical drag also nudges/bounces the page
                // horizontally on a page that only scrolls on Y (or the
                // reverse).
                let (next_x, hit_x) = if self.can_scroll_x() {
                    self.react_to_bounds(current.0 - dx, self.max_scroll_x(), true)
                } else {
                    (current.0, false)
                };
                let (next_y, hit_y) = if self.can_scroll_y() {
                    self.react_to_bounds(current.1 - dy, self.max_scroll_y(), true)
                } else {
                    (current.1, false)
                };

                if hit_x {
                    self.note_edge_hit(
                        if dx > 0.0 {
                            EdgeSide::Left
                        } else {
                            EdgeSide::Right
                        },
                        ctx,
                    );
                }
                if hit_y {
                    self.note_edge_hit(
                        if dy > 0.0 {
                            EdgeSide::Top
                        } else {
                            EdgeSide::Bottom
                        },
                        ctx,
                    );
                }

                let next = (next_x, next_y);
                if next != current {
                    self.scroll_offset.set(next);
                    self.scroll_target.set(next);
                    self.note_scroll_activity();
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
        let out_of_bounds = current.0 < 0.0
            || current.0 > self.max_scroll_x()
            || current.1 < 0.0
            || current.1 > self.max_scroll_y();

        if out_of_bounds {
            self.spring_back_if_needed(ctx);
            return;
        }

        let sf = self.scale_factor.get();
        let vx = if self.can_scroll_x() {
            state.velocity.0
        } else {
            0.0
        };
        let vy = if self.can_scroll_y() {
            state.velocity.1
        } else {
            0.0
        };

        if vx.abs() < MOMENTUM_MIN_SPEED * sf && vy.abs() < MOMENTUM_MIN_SPEED * sf {
            return;
        }

        self.momentum
            .set(Some(MomentumState { velocity: (vx, vy) }));
        self.base.dirty = true;
        ctx.request_redraw();
    }

    // Repeats the same step-sized nudge the initial click used, at a
    // fixed interval, for as long as the button stays held past the
    // initial delay - matches native scrollbar arrow repeat behavior
    // instead of a continuous pixel-based scroll.
    fn tick_arrow_hold(&mut self, dt: f32, ctx: &mut EventCtx) {
        let Some(arrow) = self.pressed_arrow.get() else {
            return;
        };

        let elapsed = self.arrow_hold_time.get() + dt;
        self.arrow_hold_time.set(elapsed);
        if elapsed < ARROW_HOLD_INITIAL_DELAY {
            return;
        }

        let mut repeat_timer = self.arrow_repeat_timer.get() + dt;

        while repeat_timer >= ARROW_HOLD_REPEAT_INTERVAL {
            repeat_timer -= ARROW_HOLD_REPEAT_INTERVAL;

            let step = self.scroll_step_physical();
            let (dx, dy) = match arrow {
                ScrollbarArrow::Up => (0.0, -step),
                ScrollbarArrow::Down => (0.0, step),
                ScrollbarArrow::Left => (-step, 0.0),
                ScrollbarArrow::Right => (step, 0.0),
            };

            let before = self.scroll_target.get();
            self.nudge(dx, dy, ctx);

            if self.scroll_target.get() == before {
                // Bound reached - stop repeating so the press-scale
                // animation releases too.
                self.pressed_arrow.set(None);
                self.arrow_hold_time.set(0.0);
                repeat_timer = 0.0;
                break;
            }
        }

        self.arrow_repeat_timer.set(repeat_timer);
    }

    fn tick_momentum(&mut self, dt: f32, ctx: &mut EventCtx) {
        let Some(mut state) = self.momentum.get() else {
            return;
        };

        let current = self.scroll_offset.get();
        let can_scroll_x = self.can_scroll_x();
        let can_scroll_y = self.can_scroll_y();
        if !can_scroll_x {
            state.velocity.0 = 0.0;
        }
        if !can_scroll_y {
            state.velocity.1 = 0.0;
        }
        let raw_x = if can_scroll_x {
            current.0 - state.velocity.0 * dt
        } else {
            0.0
        };
        let raw_y = if can_scroll_y {
            current.1 - state.velocity.1 * dt
        } else {
            0.0
        };

        let max_x = self.max_scroll_x();
        let max_y = self.max_scroll_y();

        // Eases into the rubber-band zone the same way an active drag does,
        // instead of hard-clamping the fling dead at the boundary and only
        // then playing a disconnected bounce-back once it settles.
        let (next_x, hit_x) = if can_scroll_x {
            self.react_to_bounds(raw_x, max_x, true)
        } else {
            (0.0, false)
        };
        let (next_y, hit_y) = if can_scroll_y {
            self.react_to_bounds(raw_y, max_y, true)
        } else {
            (0.0, false)
        };

        if hit_x {
            self.note_edge_hit(
                if state.velocity.0 > 0.0 {
                    EdgeSide::Left
                } else {
                    EdgeSide::Right
                },
                ctx,
            );
        }
        if hit_y {
            self.note_edge_hit(
                if state.velocity.1 > 0.0 {
                    EdgeSide::Top
                } else {
                    EdgeSide::Bottom
                },
                ctx,
            );
        }

        self.scroll_offset.set((next_x, next_y));
        self.scroll_target.set((next_x, next_y));
        self.note_scroll_activity();
        ctx.request_redraw();

        let out_of_bounds = next_x < 0.0 || next_x > max_x || next_y < 0.0 || next_y > max_y;

        // Rubber-band range never hits (hit_x/hit_y) while allow_rubber_band
        // is true, so without this the tick loop keeps coasting at the same
        // rate inside and outside the bounds - many extra frames of
        // reflow_scroll/paint on every overscroll fling.
        let friction = if out_of_bounds {
            let multiplier = if self.effective_overscroll() == Overscroll::Stretch {
                STRETCH_OVERSCROLL_FRICTION_MULTIPLIER
            } else {
                MOMENTUM_OVERSCROLL_FRICTION_MULTIPLIER
            };
            MOMENTUM_FRICTION * multiplier
        } else {
            MOMENTUM_FRICTION
        };
        let decay = (-friction * dt).exp();
        state.velocity.0 *= if hit_x { 0.0 } else { decay };
        state.velocity.1 *= if hit_y { 0.0 } else { decay };

        let sf = self.scale_factor.get();
        let min_speed = MOMENTUM_MIN_SPEED * sf;
        let settled = state.velocity.0.abs() < min_speed && state.velocity.1.abs() < min_speed;

        if settled {
            self.momentum.set(None);
            if out_of_bounds {
                self.spring_back_if_needed(ctx);
            } else if (hit_x || hit_y)
                && matches!(
                    self.effective_overscroll(),
                    Overscroll::Bounce | Overscroll::Stretch
                )
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
            offset.0 = if offset.0 <= 0.0 {
                -nudge
            } else {
                offset.0 + nudge
            };
        }
        if hit_y {
            offset.1 = if offset.1 <= 0.0 {
                -nudge
            } else {
                offset.1 + nudge
            };
        }

        self.scroll_offset.set(offset);
        self.spring_back_if_needed(ctx);
    }

    fn paint_auto_scroll_indicator(&self, ctx: &mut PaintContext) {
        let Some(state) = self.auto_scroll.get() else {
            return;
        };

        let sf = self.scale_factor.get();
        let radius = AUTO_SCROLL_INDICATOR_RADIUS_DP * sf;
        let pulse = (state.activated_at.elapsed().as_secs_f32() * 5.0).sin() * 0.5 + 0.5;
        let pulse_radius = radius + (2.0 + pulse * 2.0) * sf;
        let theme = crate::current_theme();
        let origin = state.origin;

        ctx.draw_rect(RectCommand {
            position: (origin.0 - pulse_radius, origin.1 - pulse_radius),
            size: (pulse_radius * 2.0, pulse_radius * 2.0),
            background: Some(Background::Color(Color::TRANSPARENT)),
            border_radius: Some(BorderRadius::all(Length::px(pulse_radius))),
            border_width: Some(Length::px(sf)),
            border_color: Some(theme.primary.with_alpha_f32(0.18 + pulse * 0.16)),
            clip_rect: None,
        });
        ctx.draw_rect(RectCommand {
            position: (origin.0 - radius, origin.1 - radius),
            size: (radius * 2.0, radius * 2.0),
            background: Some(Background::Color(theme.surface.with_alpha_f32(0.94))),
            border_radius: Some(BorderRadius::all(Length::px(radius))),
            border_width: Some(Length::px(1.5 * sf)),
            border_color: Some(theme.primary.with_alpha_f32(0.9)),
            clip_rect: None,
        });

        let mut direction = (
            if self.can_scroll_x() {
                state.current.0 - origin.0
            } else {
                0.0
            },
            if self.can_scroll_y() {
                state.current.1 - origin.1
            } else {
                0.0
            },
        );
        let length = direction.0.hypot(direction.1);
        let dead_zone = AUTO_SCROLL_DEAD_ZONE_DP * sf;
        if length > dead_zone {
            direction.0 /= length;
            direction.1 /= length;
            let perpendicular = (-direction.1, direction.0);
            let tip_distance = radius - 3.0 * sf;
            let base_distance = tip_distance - 7.0 * sf;
            let half_width = 3.5 * sf;
            ctx.draw_triangle(TriangleCommand {
                p0: (
                    origin.0 + direction.0 * tip_distance,
                    origin.1 + direction.1 * tip_distance,
                ),
                p1: (
                    origin.0 + direction.0 * base_distance + perpendicular.0 * half_width,
                    origin.1 + direction.1 * base_distance + perpendicular.1 * half_width,
                ),
                p2: (
                    origin.0 + direction.0 * base_distance - perpendicular.0 * half_width,
                    origin.1 + direction.1 * base_distance - perpendicular.1 * half_width,
                ),
                color: theme.primary,
                clip_rect: None,
            });
        } else {
            let dot_radius = 2.5 * sf;
            ctx.draw_rect(RectCommand {
                position: (origin.0 - dot_radius, origin.1 - dot_radius),
                size: (dot_radius * 2.0, dot_radius * 2.0),
                background: Some(Background::Color(theme.primary)),
                border_radius: Some(BorderRadius::all(Length::px(dot_radius))),
                border_width: None,
                border_color: None,
                clip_rect: None,
            });
        }
    }

    // Renders a thin, full-span band along whichever edge(s) recently hit
    // their scroll bound under `Overscroll::Glow`, instead of a radial
    // highlight anchored to the drag/scroll point - matches the classic
    // Android edge-glow silhouette, sized down for a lighter, modern look.
    fn paint_overscroll_glow(&self, ctx: &mut PaintContext) {
        let glow = self.overscroll_glow.get();
        if glow.iter().all(|v| *v <= 0.0) {
            return;
        }

        let b = self.layout_box;
        let sf = self.scale_factor.get();
        let color = crate::current_theme().primary;
        let thickness = OVERSCROLL_GLOW_BAND_THICKNESS * sf;

        let mut draw_band = |alpha: f32, position: (f32, f32), size: (f32, f32), angle_deg: f32| {
            if alpha <= 0.0 {
                return;
            }
            let stops = vec![
                GradientStop::new(color.with_alpha_f32(color.a() * alpha * 0.55), 0.0),
                GradientStop::new(color.with_alpha_f32(0.0), 1.0),
            ];
            ctx.draw_rect(RectCommand {
                position,
                size,
                background: Some(Background::LinearGradient(LinearGradient::new(
                    angle_deg, stops,
                ))),
                border_radius: None,
                border_width: None,
                border_color: None,
                clip_rect: Some((b.x, b.y, b.width, b.height)),
            });
        };

        if self.can_scroll_y() {
            draw_band(glow[0], (b.x, b.y), (b.width, thickness), 90.0);
            draw_band(
                glow[2],
                (b.x, b.y + b.height - thickness),
                (b.width, thickness),
                270.0,
            );
        }
        if self.can_scroll_x() {
            draw_band(
                glow[1],
                (b.x + b.width - thickness, b.y),
                (thickness, b.height),
                180.0,
            );
            draw_band(glow[3], (b.x, b.y), (thickness, b.height), 0.0);
        }
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
crate::impl_themed_style_builders!(base View; hover_style => hover_style, pressed_style => pressed_style, disabled_style => disabled_style, focus_style => focus_style, focus_within_style => focus_within_style, focused_hover_style => focused_hover_style, focused_pressed_style => focused_pressed_style);

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

    fn take_scroll_delta(&self) -> (f32, f32) {
        let current = self.scroll_offset.get();
        let previous = self.last_layout_scroll.replace(current);
        (current.0 - previous.0, current.1 - previous.1)
    }

    fn set_content_size(&mut self, size: (f32, f32)) {
        self.content_size.set(size);

        if self.pin_scroll_bottom {
            let bottom = self.clamp_offset((self.scroll_offset.get().0, self.max_scroll_y()));
            self.scroll_offset.set(bottom);
            self.scroll_target.set(bottom);
            return;
        }

        // Re-clamp in case the scrollable range shrank (e.g. children were
        // removed, or the viewport was resized).
        self.scroll_offset
            .set(self.clamp_offset(self.scroll_offset.get()));
        self.scroll_target
            .set(self.clamp_offset(self.scroll_target.get()));
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
        let fade = self.scrollbar_opacity_anim.get();

        if fade <= 0.001 {
            self.paint_overscroll_glow(ctx);
            self.paint_auto_scroll_indicator(ctx);
            return;
        }

        let sb = self.active_scrollbar();
        let b = self.layout_box;
        let t = sb.thickness;
        let (active_x, active_y) = self.scrollbar_active();
        let (shows_x, shows_y) = self.scrollbar_visibility();

        let thumb_border_width =
            (sb.thumb_border_width > 0.0).then(|| Length::px(sb.thumb_border_width));
        let track_border_width =
            (sb.track_border_width > 0.0).then(|| Length::px(sb.track_border_width));

        if shows_y {
            let dim = (if active_y {
                1.0
            } else {
                SCROLLBAR_DISABLED_OPACITY
            }) * fade;

            if sb.track_color.a() > 0.0 || track_border_width.is_some() {
                ctx.draw_rect(RectCommand {
                    position: (b.x + b.width - self.scrollbar_right_inset.get() - t, b.y),
                    size: (t, b.height),
                    background: Some(Background::Color(
                        sb.track_color.with_alpha_f32(sb.track_color.a() * dim),
                    )),
                    border_radius: None,
                    border_width: track_border_width,
                    border_color: Some(
                        sb.track_border_color
                            .with_alpha_f32(sb.track_border_color.a() * dim),
                    ),
                    clip_rect: None,
                });
            }

            if let Some((x, y, w, h)) = self.vertical_thumb_rect() {
                let thumb_color = self.thumb_color_anim.get();
                ctx.draw_rect(RectCommand {
                    position: (x, y),
                    size: (w, h),
                    background: Some(Background::Color(
                        thumb_color.with_alpha_f32(thumb_color.a() * dim),
                    )),
                    border_radius: Some(BorderRadius::all(Length::px(sb.thumb_radius))),
                    border_width: thumb_border_width,
                    border_color: Some(
                        sb.thumb_border_color
                            .with_alpha_f32(sb.thumb_border_color.a() * dim),
                    ),
                    clip_rect: None,
                });
            }
        }

        if shows_x {
            let dim = (if active_x {
                1.0
            } else {
                SCROLLBAR_DISABLED_OPACITY
            }) * fade;

            if sb.track_color.a() > 0.0 || track_border_width.is_some() {
                ctx.draw_rect(RectCommand {
                    position: (b.x, b.y + b.height - self.scrollbar_bottom_inset.get() - t),
                    size: (b.width, t),
                    background: Some(Background::Color(
                        sb.track_color.with_alpha_f32(sb.track_color.a() * dim),
                    )),
                    border_radius: None,
                    border_width: track_border_width,
                    border_color: Some(
                        sb.track_border_color
                            .with_alpha_f32(sb.track_border_color.a() * dim),
                    ),
                    clip_rect: None,
                });
            }

            if let Some((x, y, w, h)) = self.horizontal_thumb_rect() {
                let thumb_color = self.thumb_color_anim.get();
                ctx.draw_rect(RectCommand {
                    position: (x, y),
                    size: (w, h),
                    background: Some(Background::Color(
                        thumb_color.with_alpha_f32(thumb_color.a() * dim),
                    )),
                    border_radius: Some(BorderRadius::all(Length::px(sb.thumb_radius))),
                    border_width: thumb_border_width,
                    border_color: Some(
                        sb.thumb_border_color
                            .with_alpha_f32(sb.thumb_border_color.a() * dim),
                    ),
                    clip_rect: None,
                });
            }
        }

        if let Some((up, down)) = self.vertical_buttons() {
            let target = self.scroll_target.get();
            let axis_dim = (if active_y {
                1.0
            } else {
                SCROLLBAR_DISABLED_OPACITY
            }) * fade;
            let scales = self.arrow_scale.get();
            let arrow_colors = self.arrow_color_anim.get();

            for (rect, dir, edge_disabled, scale, color_idx) in [
                (
                    up,
                    ArrowDirection::Up,
                    target.1 <= 0.0,
                    scales[ScrollbarArrow::Up as usize],
                    ScrollbarArrow::Up as usize,
                ),
                (
                    down,
                    ArrowDirection::Down,
                    target.1 >= self.max_scroll_y(),
                    scales[ScrollbarArrow::Down as usize],
                    ScrollbarArrow::Down as usize,
                ),
            ] {
                let dim = axis_dim * (if edge_disabled { 0.35 } else { 1.0 });
                let arrow_color = arrow_colors[color_idx];
                let color = arrow_color.with_alpha_f32(arrow_color.a() * dim);

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
            let axis_dim = (if active_x {
                1.0
            } else {
                SCROLLBAR_DISABLED_OPACITY
            }) * fade;
            let scales = self.arrow_scale.get();
            let arrow_colors = self.arrow_color_anim.get();

            for (rect, dir, edge_disabled, scale, color_idx) in [
                (
                    left,
                    ArrowDirection::Left,
                    target.0 <= 0.0,
                    scales[ScrollbarArrow::Left as usize],
                    ScrollbarArrow::Left as usize,
                ),
                (
                    right,
                    ArrowDirection::Right,
                    target.0 >= self.max_scroll_x(),
                    scales[ScrollbarArrow::Right as usize],
                    ScrollbarArrow::Right as usize,
                ),
            ] {
                let dim = axis_dim * (if edge_disabled { 0.35 } else { 1.0 });
                let arrow_color = arrow_colors[color_idx];
                let color = arrow_color.with_alpha_f32(arrow_color.a() * dim);

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
        self.paint_auto_scroll_indicator(ctx);
    }

    fn event(&mut self, event: &InputEvent, ctx: &mut EventCtx) -> EventStatus {
        if let Some(status) = self.handle_auto_scroll(event, ctx) {
            return status;
        }
        if let Some(status) = self.handle_touch_pan(event, ctx) {
            return status;
        }

        if let InputEvent::FocusWithinChanged(within) = event {
            self.base.interaction.focus_within = *within;
            self.base.dirty = true;
            self.recompute_style();
            ctx.request_redraw();
            return EventStatus::Handled;
        }

        if let InputEvent::AnimationTick { dt } = event {
            if self.momentum.get().is_some() {
                self.tick_momentum(*dt, ctx);
            }
            if self.pressed_arrow.get().is_some() {
                self.tick_arrow_hold(*dt, ctx);
            }

            // The fade itself only advances inside cascade_style, which
            // only runs on an actual repaint - without this, it never
            // gets scheduled and the bar stays stuck visible.
            let fading = self.scrollbar_opacity_animating.get();
            let within_linger = self.effective_auto_hide()
                && self
                    .last_scroll_activity
                    .get()
                    .is_some_and(|t| Instant::now().duration_since(t) < SCROLLBAR_AUTO_HIDE_LINGER);
            if fading || within_linger {
                ctx.request_redraw();
            }
        }

        if let InputEvent::MouseInput {
            state: ElementState::Pressed,
            button: MouseButton::Right,
            position,
        } = event
            && let Some(handle) = &self.context_menu
            && self.hit_test(*position)
        {
            handle.open_at(*position);
            ctx.request_redraw();
            return EventStatus::Handled;
        }

        if let InputEvent::MouseMoved { position } = event {
            if let Some(vertical) = self.pending_track_drag.take()
                && self.scrollbar_drag.get().is_none()
            {
                let current = self.scroll_offset.get();
                let start_offset = if vertical { current.1 } else { current.0 };
                let start_mouse = if vertical { position.1 } else { position.0 };
                self.scrollbar_drag.set(Some(ScrollDrag {
                    vertical,
                    start_mouse,
                    start_offset,
                }));
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

            let thumb_hovered = self.point_in_scrollbar_thumb(*position);
            if thumb_hovered != self.scrollbar_thumb_hovered.get() {
                self.scrollbar_thumb_hovered.set(thumb_hovered);
                self.base.dirty = true;
                ctx.request_redraw();
            }

            let now_hovered_arrow = self.arrow_at(*position);
            if now_hovered_arrow != self.hovered_arrow.get() {
                self.hovered_arrow.set(now_hovered_arrow);
                self.base.dirty = true;
                ctx.request_redraw();
            }
        }

        if matches!(event, InputEvent::MouseExited) {
            if self.scrollbar_hovered.get() {
                self.scrollbar_hovered.set(false);
                self.base.dirty = true;
                ctx.request_redraw();
            }
            if self.scrollbar_thumb_hovered.get() {
                self.scrollbar_thumb_hovered.set(false);
                self.base.dirty = true;
                ctx.request_redraw();
            }
            if self.hovered_arrow.get().is_some() {
                self.hovered_arrow.set(None);
                self.base.dirty = true;
                ctx.request_redraw();
            }
        }

        if let InputEvent::MouseWheel {
            delta,
            position,
            modifiers,
        } = event
            && self.handle_wheel(*delta, *position, *modifiers, ctx, self.scroll_step)
        {
            return EventStatus::Handled;
        }

        if let InputEvent::KeyInput {
            event: key_event,
            modifiers,
        } = event
            && key_event.state == KeyState::Pressed
            && self.handle_key(key_event.key, *modifiers, ctx)
        {
            return EventStatus::Handled;
        }

        if let InputEvent::MouseInput {
            state,
            button,
            position,
        } = event
            && self.handle_scrollbar_mouse(*state, *button, *position, ctx)
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

            if self.base.computed_style != before_style
                || self.base.interaction.focus_visible != before_focus_visible
            {
                self.base.dirty = true;
                ctx.request_redraw();
            }
        }

        status
    }

    fn wants_animation_frame(&self) -> bool {
        self.momentum.get().is_some()
            || self.auto_scroll.get().is_some()
            || self.scrollbar_opacity_animating.get()
            || (self.effective_auto_hide()
                && self
                    .last_scroll_activity
                    .get()
                    .is_some_and(|t| Instant::now().duration_since(t) < SCROLLBAR_AUTO_HIDE_LINGER))
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

        self.base.authored_styles_eq(&other.base)
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
        self.animate_scrollbar_colors(anim);
        self.animate_scrollbar_arrows(anim);
        self.animate_scrollbar_opacity(anim);
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
            self.last_layout_scroll.set(old.last_layout_scroll.get());
            self.content_size.set(old.content_size.get());
            self.scrollbar_hovered.set(old.scrollbar_hovered.get());
            self.scrollbar_thumb_hovered
                .set(old.scrollbar_thumb_hovered.get());
            self.scrollbar_thickness_anim
                .set(old.scrollbar_thickness_anim.get());
            self.thumb_color_anim.set(old.thumb_color_anim.get());
            self.arrow_color_anim.set(old.arrow_color_anim.get());
            self.scrollbar_right_inset
                .set(old.scrollbar_right_inset.get());
            self.scrollbar_bottom_inset
                .set(old.scrollbar_bottom_inset.get());
            self.scale_factor.set(old.scale_factor.get());
            self.pressed_arrow.set(old.pressed_arrow.get());
            self.arrow_scale.set(old.arrow_scale.get());
            self.arrow_hold_time.set(old.arrow_hold_time.get());
            self.arrow_repeat_timer.set(old.arrow_repeat_timer.get());
            self.hovered_arrow.set(old.hovered_arrow.get());
            self.auto_scroll.set(old.auto_scroll.get());
            self.touch_pan.set(old.touch_pan.get());
            self.momentum.set(old.momentum.get());
            self.overscroll_glow.set(old.overscroll_glow.get());
            self.glow_pending_hit.set(old.glow_pending_hit.get());
            self.scrollbar_opacity_anim
                .set(old.scrollbar_opacity_anim.get());
            self.scrollbar_opacity_animating
                .set(old.scrollbar_opacity_animating.get());
            self.last_scroll_activity
                .set(old.last_scroll_activity.get());
        }
    }

    fn anim_id(&self) -> WidgetId {
        self.anim_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sized_view(view: View, viewport: (f32, f32), content: (f32, f32)) -> View {
        let mut view = view;
        view.layout_box = LayoutBox {
            x: 0.0,
            y: 0.0,
            width: viewport.0,
            height: viewport.1,
        };
        view.content_size.set(content);
        view
    }

    #[test]
    fn content_overflow_without_scroll_mode_cannot_scroll_or_overscroll() {
        let mut view = sized_view(View::new(), (100.0, 100.0), (300.0, 300.0));
        let mut ctx = EventCtx::new();

        let handled = view.handle_wheel(
            MouseScrollDelta::LineDelta(0.0, -1.0),
            (50.0, 50.0),
            ModifiersState::default(),
            &mut ctx,
            96.0,
        );

        assert!(!handled);
        assert_eq!(view.scroll_offset.get(), (0.0, 0.0));
        assert_eq!(view.scroll_target.get(), (0.0, 0.0));
        assert_eq!(view.scrollbar_active(), (false, false));
        assert_eq!(view.glow_pending_hit.get(), [false; 4]);
    }

    #[test]
    fn empty_scroll_container_does_not_rubber_band() {
        let mut view = sized_view(
            View::new()
                .overflow_y(Overflow::Auto)
                .overscroll(Overscroll::Stretch),
            (100.0, 100.0),
            (100.0, 100.0),
        );
        let mut ctx = EventCtx::new();

        let handled = view.handle_wheel(
            MouseScrollDelta::LineDelta(0.0, -1.0),
            (50.0, 50.0),
            ModifiersState::default(),
            &mut ctx,
            96.0,
        );

        assert!(!handled);
        assert_eq!(view.scroll_offset.get(), (0.0, 0.0));
        assert_eq!(view.scroll_target.get(), (0.0, 0.0));
    }

    #[test]
    fn vertical_wheel_bubbles_past_horizontal_only_scroller() {
        let mut view = sized_view(
            View::new().overflow_x(Overflow::Auto),
            (100.0, 100.0),
            (300.0, 100.0),
        );
        let mut ctx = EventCtx::new();

        assert!(!view.handle_wheel(
            MouseScrollDelta::LineDelta(0.0, -1.0),
            (50.0, 50.0),
            ModifiersState::default(),
            &mut ctx,
            96.0,
        ));
        assert_eq!(view.scroll_target.get(), (0.0, 0.0));

        assert!(view.handle_wheel(
            MouseScrollDelta::LineDelta(0.0, -1.0),
            (50.0, 50.0),
            ModifiersState {
                shift: true,
                ..Default::default()
            },
            &mut ctx,
            96.0,
        ));
        assert_eq!(view.scroll_target.get(), (96.0, 0.0));
    }

    #[test]
    fn wheel_clamps_at_edge_without_bounce() {
        let mut view = sized_view(
            View::new()
                .overflow_y(Overflow::Auto)
                .overscroll(Overscroll::Bounce),
            (100.0, 100.0),
            (100.0, 300.0),
        );
        view.scroll_offset.set((0.0, 200.0));
        view.scroll_target.set((0.0, 200.0));
        let mut ctx = EventCtx::new();

        let handled = view.handle_wheel(
            MouseScrollDelta::LineDelta(0.0, -1.0),
            (50.0, 50.0),
            ModifiersState::default(),
            &mut ctx,
            96.0,
        );

        assert!(!handled);
        assert_eq!(view.scroll_offset.get(), (0.0, 200.0));
        assert_eq!(view.scroll_target.get(), (0.0, 200.0));
    }

    #[test]
    fn track_hover_does_not_expand_thumb() {
        let mut view = sized_view(
            View::new().overflow_y(Overflow::Auto),
            (100.0, 100.0),
            (100.0, 400.0),
        );
        let idle_thickness = view.resolved_scrollbar_for_state(false, false).thickness;
        let hover_thickness = view.resolved_scrollbar_for_state(true, false).thickness;
        let thumb = view.vertical_thumb_hit_rect().expect("vertical thumb");
        let x = thumb.0 + thumb.2 * 0.5;
        let mut ctx = EventCtx::new();

        view.event(
            &InputEvent::MouseMoved {
                position: (x, 90.0),
            },
            &mut ctx,
        );
        assert!(view.scrollbar_hovered.get());
        assert!(!view.scrollbar_thumb_hovered.get());
        assert_eq!(view.target_scrollbar_thickness(), idle_thickness);

        let thumb_center = (x, thumb.1 + thumb.3 * 0.5);
        view.event(
            &InputEvent::MouseMoved {
                position: thumb_center,
            },
            &mut ctx,
        );
        assert!(view.scrollbar_thumb_hovered.get());
        assert_eq!(view.target_scrollbar_thickness(), hover_thickness);
    }

    #[test]
    fn auto_scroll_speed_curve_has_dead_zone_and_cap() {
        assert_eq!(auto_scroll_axis_speed(8.0, 8.0, 180.0, 1900.0), 0.0);
        let near = auto_scroll_axis_speed(32.0, 8.0, 180.0, 1900.0);
        let far = auto_scroll_axis_speed(120.0, 8.0, 180.0, 1900.0);
        assert!(near > 0.0);
        assert!(far > near);
        assert_eq!(auto_scroll_axis_speed(400.0, 8.0, 180.0, 1900.0), 1900.0);
        assert_eq!(auto_scroll_axis_speed(-400.0, 8.0, 180.0, 1900.0), -1900.0);
    }

    #[test]
    fn auto_scroll_accelerates_instead_of_jumping_to_target_speed() {
        let mut view = sized_view(
            View::new().overflow_y(Overflow::Auto),
            (100.0, 100.0),
            (100.0, 1000.0),
        );
        view.auto_scroll.set(Some(AutoScrollState {
            origin: (50.0, 50.0),
            current: (50.0, 180.0),
            velocity: (0.0, 0.0),
            activated_at: Instant::now(),
            cursor: Cursor::NsResize,
        }));
        let mut ctx = EventCtx::new();

        view.tick_auto_scroll(1.0 / 60.0, &mut ctx);
        let first = view
            .auto_scroll
            .get()
            .expect("active AutoScroll")
            .velocity
            .1;
        assert!(first > 0.0);
        assert!(first < AUTO_SCROLL_MAX_SPEED);
        assert!(view.scroll_offset.get().1 > 0.0);
        assert!(ctx.redraw_requested());

        view.tick_auto_scroll(1.0 / 60.0, &mut ctx);
        let second = view
            .auto_scroll
            .get()
            .expect("active AutoScroll")
            .velocity
            .1;
        assert!(second > first);
    }

    #[test]
    fn auto_scroll_origin_indicator_is_always_visible_while_active() {
        let view = sized_view(
            View::new().overflow_y(Overflow::Auto),
            (100.0, 100.0),
            (100.0, 400.0),
        );
        view.auto_scroll.set(Some(AutoScrollState {
            origin: (50.0, 50.0),
            current: (50.0, 50.0),
            velocity: (0.0, 0.0),
            activated_at: Instant::now(),
            cursor: Cursor::NsResize,
        }));
        let mut commands = Vec::new();
        let mut paint = PaintContext::new(&mut commands, 1.0);

        view.paint_auto_scroll_indicator(&mut paint);

        assert_eq!(commands.len(), 3);
    }
}
