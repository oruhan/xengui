// SPDX-License-Identifier: Apache-2.0
use smol_str::SmolStr;

use crate::{
    AnimationManager, Background, Border, BorderRadius, BoxShadow, BoxShadowCommand, Color,
    Constraints, EventCtx, EventStatus, InputEvent, Interaction, LayoutBox, Length, MeasureContext,
    MeasureResult, Outline, PaintContext, RectCommand, Style, TransformOrigin, WidgetId,
    properties::StyleValue,
};
use std::any::Any;

/// Snapshot of a widget's text-input state, used to mirror it onto a real
/// DOM `<input>` on web targets so mobile browsers open the keyboard.
#[derive(Clone, Debug)]
pub struct NativeTextInputSnapshot {
    /// The `value` value carried by this type.
    pub value: String,
    /// The `placeholder` value carried by this type.
    pub placeholder: String,
    /// The `max_length` value carried by this type.
    pub max_length: Option<usize>,
    /// The `read_only` value carried by this type.
    pub read_only: bool,
}

/// Behavior required from `Widget` implementations.
pub trait Widget: Any {
    /// Returns or updates the `as_any` value.
    fn as_any(&self) -> &dyn Any;

    /// Returns or updates the `as_any_mut` value.
    fn as_any_mut(&mut self) -> &mut dyn Any;

    /// Returns or updates the `debug_name` value.
    fn debug_name(&self) -> &'static str {
        "Widget"
    }

    /// Returns or updates the `get_key` value.
    fn get_key(&self) -> Option<&SmolStr> {
        None
    }

    /// Returns whether the `is_dirty` condition is satisfied.
    fn is_dirty(&self) -> bool;

    /// Updates the `set_dirty` value.
    fn set_dirty(&mut self, dirty: bool);

    /// Whether this widget's dirtiness stems from something that would
    /// change its taffy layout node, as opposed to a purely visual
    /// repaint. Defaults to mirroring `is_dirty()` for widgets that don't
    /// track this separately; widgets backed by `WidgetBase` track it
    /// precisely instead (see `WidgetBase::layout_dirty`).
    fn is_layout_dirty(&self) -> bool {
        self.is_dirty()
    }

    /// Clears the layout-dirty flag once a full layout pass has run.
    /// No-op by default; only meaningful alongside a real `is_layout_dirty` override.
    fn set_layout_dirty(&mut self, _value: bool) {}

    /// Returns or updates the `style` value.
    fn style(&self) -> &Style;

    /// Returns or updates the `style_mut` value.
    fn style_mut(&mut self) -> &mut Style;

    /// Returns or updates the `computed_style` value.
    fn computed_style(&self) -> &Style {
        self.style()
    }

    /// Called once, right after this widget is inserted into the tree.
    fn on_mount(&mut self) {}

    /// Called once, right before this widget is permanently removed from the tree.
    fn on_unmount(&mut self) {}

    /// Returns or updates the `children` value.
    fn children(&self) -> &[Box<dyn Widget>] {
        &[]
    }

    /// Returns or updates the `children_mut` value.
    fn children_mut(&mut self) -> Option<&mut Vec<Box<dyn Widget>>> {
        None
    }

    /// Additional scroll translation applied to this widget's children
    /// during layout, in logical pixels.
    fn scroll_offset(&self) -> (f32, f32) {
        (0.0, 0.0)
    }

    /// How far this widget's own `scroll_offset()` moved since the layout
    /// engine last asked, resetting its bookkeeping to the current value.
    /// Only `View` overrides this; every other widget has nothing to
    /// report and the default is a no-op.
    fn take_scroll_delta(&self) -> (f32, f32) {
        (0.0, 0.0)
    }

    /// Reports this widget's total content size after layout, which may
    /// exceed `layout_box()` when children overflow it.
    fn set_content_size(&mut self, _size: (f32, f32)) {}

    /// The rectangle this widget clips its children's painted output to,
    /// in absolute screen coordinates. `None` means no clipping.
    fn clip_children(&self) -> Option<(f32, f32, f32, f32)> {
        None
    }

    /// Returns or updates the `measure` value.
    fn measure(&self, ctx: &mut MeasureContext, constraints: Constraints) -> MeasureResult;

    /// Runs once per full layout pass for every widget, leaf or not,
    /// purely so a widget can cache intrinsic measurements it needs later
    /// for painting (e.g. a popup's natural width) without affecting its
    /// own flex/grid size, which `measure` alone controls.
    fn on_layout_pass(&self, _ctx: &mut MeasureContext) {}

    /// Returns or updates the `layout` value.
    fn layout(&mut self, rect: LayoutBox);

    /// Returns or updates the `layout_box` value.
    fn layout_box(&self) -> &LayoutBox;

    /// Returns or updates the `paint` value.
    fn paint(&self, ctx: &mut PaintContext);

    /// Marks this widget's entire subtree as rendered by the top layer
    /// instead of inline - see [`crate::widgets::Portal`].
    fn is_portal(&self) -> bool {
        false
    }

    /// Painted after every descendant, on top of them, and never cached -
    /// used for overlays like a scrollbar thumb that depend on live state.
    fn paint_overlay(&self, _ctx: &mut PaintContext) {}

    /// Returns or updates the `paint_box` value.
    fn paint_box(&self, ctx: &mut PaintContext) {
        let style = self.computed_style();
        let sf = ctx.scale_factor;
        // Always routed through scaled_layout_box_with_origin (even at
        // scale 1.0) so every widget's box position is rounded through
        // the same single site, instead of drifting in/out of subpixel
        // alignment depending on whether a scale happens to be set.
        let layout = scaled_layout_box_with_origin(
            *self.layout_box(),
            style.scale.unwrap_or(1.0),
            style.transform_origin.unwrap_or_default(),
            sf,
        );
        let radius = style
            .border
            .as_ref()
            .and_then(|b| b.radius)
            .map(|r| r.to_physical_array(sf, layout.width, layout.height))
            .unwrap_or([0.0; 4]);
        // `radius[0]` (top-left) still stands in for "does this box have
        // any rounding at all" in the non-uniform-border branch below.
        let has_radius = radius.iter().any(|r| *r > 0.0);

        if let Some(shadows) = &style.box_shadow {
            for shadow in shadows.iter().rev().filter(|s| !s.inset) {
                self.paint_shadow_layer(ctx, layout, radius, shadow, sf);
            }
        }

        if style.background.is_some() || style.border.is_some() {
            let border = style.border.as_ref();

            if border.is_some_and(|b| !b.is_uniform()) {
                if style.background.is_some() {
                    ctx.draw_rect(RectCommand {
                        position: (layout.x, layout.y),
                        size: (layout.width, layout.height),
                        background: style.background.clone(),
                        border_radius: has_radius.then_some(border_radius_from_physical(radius)),
                        border_color: None,
                        border_width: None,
                        clip_rect: None,
                    });
                }
                self.paint_edge_borders(ctx, layout, border.unwrap(), sf);
            } else {
                ctx.draw_rect(RectCommand {
                    position: (layout.x, layout.y),
                    size: (layout.width, layout.height),
                    background: style.background.clone(),
                    border_radius: has_radius.then_some(border_radius_from_physical(radius)),
                    border_color: border.map(|b| b.color),
                    border_width: border.map(|b| Length::px(b.top.to_physical(sf))),
                    clip_rect: None,
                });
            }
        }

        if let Some(shadows) = &style.box_shadow {
            for shadow in shadows.iter().rev().filter(|s| s.inset) {
                self.paint_shadow_layer(ctx, layout, radius, shadow, sf);
            }
        }
    }

    /// Returns or updates the `paint_edge_borders` value.
    fn paint_edge_borders(&self, ctx: &mut PaintContext, layout: LayoutBox, b: &Border, sf: f32) {
        let top = b.top.to_physical(sf);
        let right = b.right.to_physical(sf);
        let bottom = b.bottom.to_physical(sf);
        let left = b.left.to_physical(sf);

        let mut edge = |x: f32, y: f32, w: f32, h: f32| {
            ctx.draw_rect(RectCommand {
                position: (x, y),
                size: (w, h),
                background: Some(Background::Color(b.color)),
                border_radius: None,
                border_width: None,
                border_color: None,
                clip_rect: None,
            });
        };

        if top > 0.0 {
            edge(layout.x, layout.y, layout.width, top);
        }
        if bottom > 0.0 {
            edge(
                layout.x,
                layout.y + layout.height - bottom,
                layout.width,
                bottom,
            );
        }
        if left > 0.0 {
            edge(layout.x, layout.y, left, layout.height);
        }
        if right > 0.0 {
            edge(
                layout.x + layout.width - right,
                layout.y,
                right,
                layout.height,
            );
        }
    }

    /// Returns or updates the `paint_shadow_layer` value.
    fn paint_shadow_layer(
        &self,
        ctx: &mut PaintContext,
        layout: LayoutBox,
        radius: [f32; 4],
        shadow: &BoxShadow,
        sf: f32,
    ) {
        let ox = shadow.offset_x.to_physical(sf);
        let oy = shadow.offset_y.to_physical(sf);
        let blur = shadow.blur_radius.to_physical(sf).max(0.0);
        let spread = shadow.spread_radius.to_physical(sf);

        let cx = layout.x + layout.width * 0.5;
        let cy = layout.y + layout.height * 0.5;

        let grow = |r: f32, by: f32| (r + by).max(0.0);
        let shrink = |r: f32, by: f32| (r - by).max(0.0);

        let (shadow_position, shadow_size, shadow_radius) = if shadow.inset {
            let half_w = (layout.width * 0.5 - spread).max(0.0);
            let half_h = (layout.height * 0.5 - spread).max(0.0);
            (
                (cx + ox - half_w, cy + oy - half_h),
                (half_w * 2.0, half_h * 2.0),
                radius.map(|r| shrink(r, spread)),
            )
        } else {
            let half_w = layout.width * 0.5 + spread;
            let half_h = layout.height * 0.5 + spread;
            (
                (cx + ox - half_w, cy + oy - half_h),
                (half_w * 2.0, half_h * 2.0),
                radius.map(|r| grow(r, spread)),
            )
        };

        ctx.draw_box_shadow(BoxShadowCommand {
            shadow_position,
            shadow_size,
            shadow_radius,
            blur,
            color: shadow.color,
            inset: shadow.inset,
            box_position: (layout.x, layout.y),
            box_size: (layout.width, layout.height),
            box_radius: radius.iter().copied().fold(0.0_f32, f32::max),
            clip_rect: None,
            direction: shadow.direction,
        });
    }

    /// Returns or updates the `paint_outline` value.
    fn paint_outline(&self, ctx: &mut PaintContext) {
        if self
            .interaction()
            .is_some_and(|i| i.focused && i.focus_visible)
        {
            return;
        }

        let style = self.computed_style();
        let outline = match &style.outline {
            StyleValue::None | StyleValue::Default => {
                return;
            }
            StyleValue::Value(outline) => outline,
        };

        let sf = ctx.scale_factor;
        let layout = self.layout_box();
        let offset = outline.offset.to_physical(sf);
        let radius = outline
            .radius
            .or_else(|| style.border.as_ref().and_then(|b| b.radius))
            .map(|r| {
                r.to_physical_array(
                    sf,
                    layout.width + offset * 2.0,
                    layout.height + offset * 2.0,
                )
            })
            .unwrap_or([0.0; 4]);

        ctx.draw_rect(RectCommand {
            position: (layout.x - offset, layout.y - offset),
            size: (layout.width + offset * 2.0, layout.height + offset * 2.0),
            background: None,
            border_radius: radius
                .iter()
                .any(|r| *r > 0.0)
                .then_some(border_radius_from_physical(radius)),
            border_color: Some(outline.color),
            border_width: Some(Length::px(outline.width.to_physical(sf))),
            clip_rect: None,
        });
    }

    /// Returns or updates the `paint_focus` value.
    fn paint_focus(&self, ctx: &mut PaintContext) {
        let Some(interaction) = self.interaction() else {
            return;
        };
        if !interaction.focused || !interaction.focus_visible {
            return;
        }

        let style = self.computed_style();
        let layout = self.layout_box();

        let outline = match &style.outline {
            StyleValue::None => {
                return;
            }
            StyleValue::Value(outline) => *outline,
            StyleValue::Default => Outline {
                width: Length::px(2.5),
                color: Color::BLUE_500,
                radius: style.border.as_ref().and_then(|b| b.radius).map(|r| {
                    BorderRadius::only(
                        r.top_left.add_px(4.0),
                        r.top_right.add_px(4.0),
                        r.bottom_right.add_px(4.0),
                        r.bottom_left.add_px(4.0),
                    )
                }),
                offset: Length::px(4.0),
            },
        };

        let sf = ctx.scale_factor;
        let offset = outline.offset.to_physical(sf);
        let radius = outline
            .radius
            .or_else(|| style.border.as_ref().and_then(|b| b.radius))
            .map(|r| {
                r.to_physical_array(
                    sf,
                    layout.width + offset * 2.0,
                    layout.height + offset * 2.0,
                )
            })
            .unwrap_or([0.0; 4]);

        ctx.draw_rect(RectCommand {
            position: (layout.x - offset, layout.y - offset),
            size: (layout.width + offset * 2.0, layout.height + offset * 2.0),
            background: None,
            border_radius: radius
                .iter()
                .any(|r| *r > 0.0)
                .then_some(border_radius_from_physical(radius)),
            border_width: Some(Length::px(outline.width.to_physical(sf))),
            border_color: Some(outline.color),
            clip_rect: None,
        });
    }

    /// Painted after every other widget's own content and after all
    /// deferred text has been flushed - use this instead of
    /// `paint_overlay` for a popup that must render above everything
    /// else, including other widgets' text (which is otherwise batched
    /// and flushed in its own pass after every widget's rects).
    fn paint_top(&self, _ctx: &mut PaintContext) {}

    /// Returns or updates the `hit_test` value.
    fn hit_test(&self, point: (f32, f32)) -> bool {
        let b = self.layout_box();

        if point.0 < b.x || point.0 > b.x + b.width || point.1 < b.y || point.1 > b.y + b.height {
            return false;
        }

        let Some(border) = &self.style().border else {
            return true;
        };

        let radius = border.radius.map(|r| r.max_value()).unwrap_or(0.0);

        if radius <= 0.0 {
            return true;
        }

        let r = radius.min(b.width * 0.5).min(b.height * 0.5);

        let local_x = point.0 - b.x;
        let local_y = point.1 - b.y;

        if local_x >= r && local_x <= b.width - r {
            return true;
        }

        if local_y >= r && local_y <= b.height - r {
            return true;
        }

        let cx = if local_x < r { r } else { b.width - r };

        let cy = if local_y < r { r } else { b.height - r };

        let dx = local_x - cx;
        let dy = local_y - cy;

        dx * dx + dy * dy <= r * r
    }

    /// When true at `point`, hit-testing stops at this widget instead of
    /// descending into its children.
    fn blocks_children_hit_test(&self, _point: (f32, f32)) -> bool {
        false
    }

    /// Returns or updates the `interaction` value.
    fn interaction(&self) -> Option<&Interaction> {
        None
    }

    /// Returns or updates the `interaction_mut` value.
    fn interaction_mut(&mut self) -> Option<&mut Interaction> {
        None
    }

    /// Returns or updates the `transfer_interaction_state` value.
    fn transfer_interaction_state(&mut self, old: &dyn Widget) {
        if let (Some(new), Some(old)) = (self.interaction_mut(), old.interaction()) {
            new.transfer_from(old);
        }
    }

    /// Called during reconciliation for every widget matched against its
    /// predecessor, with mutable access to that predecessor - lets a
    /// composite widget reconcile its freshly re-rendered content against
    /// whatever the predecessor already had committed, so a descendant's
    /// interaction/hook state survives a parent prop update. Every other
    /// widget can ignore this; the default does nothing.
    fn transfer_composite_children(&mut self, _old: &mut dyn Widget) {}

    /// Returns or updates the `event` value.
    fn event(&mut self, event: &InputEvent, ctx: &mut EventCtx) -> EventStatus {
        let status = match self.interaction_mut() {
            Some(interaction) if interaction.is_active() => interaction.handle(event, ctx),
            _ => EventStatus::Ignored,
        };

        if matches!(status, EventStatus::Handled) {
            self.set_dirty(true);
        }

        status
    }

    /// Returns or updates the `content_eq` value.
    fn content_eq(&self, _other: &dyn Widget) -> bool {
        false
    }

    /// Returns or updates the `cascade_style` value.
    fn cascade_style(&mut self, parent: &Style, anim: &mut AnimationManager) {
        if let Some(children) = self.children_mut() {
            for child in children.iter_mut() {
                child.cascade_style(parent, anim);
            }
        }
    }

    /// Returns or updates the `after_interaction_transfer` value.
    fn after_interaction_transfer(&mut self) {}

    /// Returns or updates the `transfer_measured_state` value.
    fn transfer_measured_state(&mut self, _old: &dyn Widget) {}

    /// Returns or updates the `blink_interval` value.
    fn blink_interval(&self) -> Option<web_time::Duration> {
        None
    }

    /// Whether this widget needs a continuous per-frame animation callback
    /// (`InputEvent::AnimationTick`) right now, independent of focus.
    fn wants_animation_frame(&self) -> bool {
        false
    }

    /// HTML-style selectable text content for mouse selection / Ctrl+C.
    /// `None` opts the widget out entirely (e.g. Button, TextBox).
    fn selectable_text(&self) -> Option<&str> {
        None
    }

    /// Returns or updates the `text_selection` value.
    fn text_selection(&self) -> Option<(usize, usize)> {
        None
    }

    /// Updates the `set_text_selection` value.
    fn set_text_selection(&mut self, _range: Option<(usize, usize)>) {}

    // Called by the global Escape handler; clears the selection and also
    // stops any drag-in-progress so a still-held mouse button can't
    // immediately re-create the selection on the next move.
    /// Performs the `cancel_text_selection` operation.
    fn cancel_text_selection(&mut self) {
        self.set_text_selection(None);
    }

    /// Cancels any in-progress AutoScroll (middle-click pan) gesture on
    /// this widget. Default no-op; only scrollable containers like
    /// `View` override it. Called globally before dispatching a
    /// non-middle mouse press, since a press that lands on an
    /// interactive descendant (e.g. a `Button`) is consumed there and
    /// never bubbles up to the scrollable ancestor's own event handler.
    fn cancel_auto_scroll(&mut self, _ctx: &mut EventCtx) {}

    /// Nearest character index to an absolute screen point, used by
    /// cross-widget drag selection to know where a widget's own
    /// selection should start or end.
    fn text_index_at(&self, _point: (f32, f32)) -> usize {
        0
    }

    /// Returns or updates the `select_all_text` value.
    fn select_all_text(&mut self) {}

    /// Stable per-instance animation key namespace. Assign once in the
    /// constructor via `WidgetId::new_unique()` and preserve it across
    /// reconciliation so in-flight transitions aren't reset.
    fn anim_id(&self) -> WidgetId {
        WidgetId::default()
    }

    /// GPU filter chain applied to this widget's own rendered subtree.
    /// `None` (the default) keeps this widget on the fast, unfiltered
    /// paint path.
    fn filter(&self) -> Option<&crate::FilterChain> {
        self.computed_style().filter.as_ref()
    }

    /// GPU filter chain applied to the already-painted frame content
    /// behind this widget's own bounds, before this widget paints its own
    /// background/children on top. `None` (the default) skips the
    /// backdrop-capture pass entirely.
    fn backdrop_filter(&self) -> Option<&crate::FilterChain> {
        self.computed_style().backdrop_filter.as_ref()
    }

    /// `Some(...)` marks this widget as backed by a real DOM `<input>` on
    /// web targets. `None` (the default) means it has no native counterpart.
    fn native_text_input(&self) -> Option<NativeTextInputSnapshot> {
        None
    }

    /// Applies a value typed into the native DOM `<input>` back onto this
    /// widget's own state.
    fn set_native_text_value(&mut self, _value: &str, _ctx: &mut EventCtx) {}

    /// Syncs this widget's native DOM input (web only). Widgets exposing
    /// `native_text_input()` should override this to keep the hidden
    /// `<input>`'s value/placeholder/read-only state in sync, so mobile
    /// keyboards get correct context.
    #[cfg(target_arch = "wasm32")]
    fn sync_native_input(&self, _input: &web_sys::HtmlInputElement) {}
}

/// Shrinks or grows `rect` around its own center by `scale`, so an
/// animated scale transform can be painted without touching layout.
pub fn scaled_layout_box(rect: LayoutBox, scale: f32) -> LayoutBox {
    let cx = rect.x + rect.width * 0.5;
    let cy = rect.y + rect.height * 0.5;
    let w = rect.width * scale;
    let h = rect.height * scale;
    LayoutBox {
        x: (cx - w * 0.5).round(),
        y: (cy - h * 0.5).round(),
        width: w,
        height: h,
    }
}

/// Like `scaled_layout_box`, but scales around an explicit pivot instead
/// of always the widget's own center - lets `Style::transform_origin`
/// (e.g. a corner or an off-center point) actually take effect. A point
/// `p` transforms as `pivot + (p - pivot) * scale`; applied to the box's
/// own top-left corner this reduces to `rect.pos + origin * (1 - scale)`.
pub fn scaled_layout_box_with_origin(
    rect: LayoutBox,
    scale: f32,
    origin: TransformOrigin,
    scale_factor: f32,
) -> LayoutBox {
    let (ox, oy) = origin.resolve(rect.width, rect.height, scale_factor);
    LayoutBox {
        x: (rect.x + ox * (1.0 - scale)).round(),
        y: (rect.y + oy * (1.0 - scale)).round(),
        width: rect.width * scale,
        height: rect.height * scale,
    }
}

/// Wraps four already-physical-px corner radii back into a [`BorderRadius`]
/// so `RectCommand::border_radius` (which stores physical values, unlike
/// `Style`'s logical `BorderRadius`) has a single documented type.
pub(crate) fn border_radius_from_physical(radii: [f32; 4]) -> BorderRadius {
    BorderRadius::only(
        Length::px(radii[0]),
        Length::px(radii[1]),
        Length::px(radii[2]),
        Length::px(radii[3]),
    )
}
