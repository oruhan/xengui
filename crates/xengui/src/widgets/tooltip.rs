// SPDX-License-Identifier: Apache-2.0
use crate::{
    AnimKey, AnimLayer, AnimProperty, AnimValue, AnimationManager, Background, Border,
    BorderRadius, BoxShadow, Color, Constraints, Easing, Edges, EventCtx, EventStatus, InputEvent,
    LayoutBox, Length, MeasureContext, MeasureResult, PaintContext, RectCommand, Style,
    StyleBuilder, TextCommand, Transition, Widget, WidgetBase, WidgetId,
    constants::DEFAULT_FONT_SIZE,
};
use smol_str::SmolStr;
use std::cell::Cell;
use web_time::{Duration, Instant};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
/// Available `TooltipPlacement` choices.
pub enum TooltipPlacement {
    #[default]
    /// The `Top` variant.
    Top,
    /// The `Bottom` variant.
    Bottom,
    /// The `Left` variant.
    Left,
    /// The `Right` variant.
    Right,
}

const DEFAULT_DELAY: Duration = Duration::from_millis(400);
const DEFAULT_GAP: f32 = 6.0;
const FADE_TRANSITION: Transition =
    Transition::new(Duration::from_millis(120)).easing(Easing::EaseOut);
const SCALE_TRANSITION: Transition =
    Transition::new(Duration::from_millis(140)).easing(Easing::EaseOut);
const TOOLTIP_PADDING_X: f32 = 8.0;
const TOOLTIP_PADDING_Y: f32 = 5.0;

/// Wraps exactly one child widget and shows a small floating label near it
/// once the pointer rests over the child for `delay`. The wrapped child
/// keeps receiving input normally - this widget only tracks hover via
/// bubbled `MouseMoved` events and paints the popup in `paint_top`, so it
/// always renders above everything else.
pub struct Tooltip {
    base: WidgetBase,
    anim_id: WidgetId,
    children: Vec<Box<dyn Widget>>,
    layout_box: LayoutBox,

    text: SmolStr,
    placement: TooltipPlacement,
    delay: Duration,
    gap: f32,

    background: Option<Background>,
    text_color: Option<Color>,
    padding: Option<Edges>,
    border_radius: Option<Length>,
    border: Option<Border>,
    font_size: Option<Length>,

    hover_start: Cell<Option<Instant>>,
    showing: Cell<bool>,
    opacity_anim: Cell<f32>,
    scale_progress: Cell<f32>,
    label_size: Cell<(f32, f32)>,
}

impl Tooltip {
    /// Creates a value with its default configuration.
    pub fn new(text: impl Into<SmolStr>) -> Self {
        Self {
            base: WidgetBase::new(crate::Interaction::new()),
            anim_id: WidgetId::new_unique(),
            children: Vec::new(),
            layout_box: LayoutBox::default(),

            text: text.into(),
            placement: TooltipPlacement::default(),
            delay: DEFAULT_DELAY,
            gap: DEFAULT_GAP,

            background: None,
            text_color: None,
            padding: None,
            border_radius: None,
            border: None,
            font_size: None,

            hover_start: Cell::new(None),
            showing: Cell::new(false),
            opacity_anim: Cell::new(0.0),
            scale_progress: Cell::new(1.0),
            label_size: Cell::new((0.0, 0.0)),
        }
    }

    /// Returns or updates the `key` value.
    pub fn key(mut self, key: impl Into<SmolStr>) -> Self {
        self.base.key = Some(key.into());
        self
    }

    /// Sets the anchor widget. Only the last call matters - a second
    /// child replaces the first instead of appending.
    pub fn child(mut self, child: impl Widget + 'static) -> Self {
        self.children = vec![Box::new(child)];
        self
    }

    /// Returns or updates the `placement` value.
    pub fn placement(mut self, placement: TooltipPlacement) -> Self {
        self.placement = placement;
        self
    }

    /// Returns or updates the `delay` value.
    pub fn delay(mut self, delay: Duration) -> Self {
        self.delay = delay;
        self
    }

    /// Returns or updates the `gap` value.
    pub fn gap(mut self, gap: f32) -> Self {
        self.gap = gap;
        self
    }

    /// Returns or updates the `background` value.
    pub fn background(mut self, color: Color) -> Self {
        self.background = Some(Background::Color(color));
        self
    }

    /// Returns or updates the `text_color` value.
    pub fn text_color(mut self, color: Color) -> Self {
        self.text_color = Some(color);
        self
    }

    /// Returns or updates the `padding` value.
    pub fn padding(mut self, padding: impl Into<Edges>) -> Self {
        self.padding = Some(padding.into());
        self
    }

    /// Returns or updates the `border_radius` value.
    pub fn border_radius(mut self, radius: impl Into<Length>) -> Self {
        self.border_radius = Some(radius.into());
        self
    }

    /// Returns or updates the `border` value.
    pub fn border(mut self, border: Border) -> Self {
        self.border = Some(border);
        self
    }

    /// Returns or updates the `font_size` value.
    pub fn font_size(mut self, size: impl Into<Length>) -> Self {
        self.font_size = Some(size.into());
        self
    }

    fn effective_padding(&self) -> Edges {
        self.padding
            .unwrap_or_else(|| Edges::symmetric(TOOLTIP_PADDING_X, TOOLTIP_PADDING_Y))
    }

    fn recompute_style(&mut self) {
        self.base.computed_style = self.base.inherited_style.inherit_style(&self.base.style);
    }

    fn box_position(&self, anchor: LayoutBox, size: (f32, f32)) -> (f32, f32) {
        let (w, h) = size;
        match self.placement {
            TooltipPlacement::Top => (anchor.x + (anchor.width - w) * 0.5, anchor.y - h - self.gap),
            TooltipPlacement::Bottom => (
                anchor.x + (anchor.width - w) * 0.5,
                anchor.y + anchor.height + self.gap,
            ),
            TooltipPlacement::Left => (
                anchor.x - w - self.gap,
                anchor.y + (anchor.height - h) * 0.5,
            ),
            TooltipPlacement::Right => (
                anchor.x + anchor.width + self.gap,
                anchor.y + (anchor.height - h) * 0.5,
            ),
        }
    }
}

impl StyleBuilder for Tooltip {
    fn style_mut(&mut self) -> &mut Style {
        &mut self.base.style
    }

    fn mark_dirty(&mut self) {
        self.base.dirty = true;
        self.recompute_style();
    }
}

impl Widget for Tooltip {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn debug_name(&self) -> &'static str {
        "Widget#Tooltip"
    }

    fn get_key(&self) -> Option<&SmolStr> {
        self.base.key.as_ref()
    }

    fn is_dirty(&self) -> bool {
        self.base.dirty
    }

    fn set_dirty(&mut self, dirty: bool) {
        self.base.dirty = dirty;
    }

    fn style(&self) -> &Style {
        &self.base.style
    }

    fn style_mut(&mut self) -> &mut Style {
        &mut self.base.style
    }

    fn computed_style(&self) -> &Style {
        &self.base.computed_style
    }

    fn children(&self) -> &[Box<dyn Widget>] {
        &self.children
    }

    fn children_mut(&mut self) -> Option<&mut Vec<Box<dyn Widget>>> {
        Some(&mut self.children)
    }

    fn measure(&self, _ctx: &mut MeasureContext, _constraints: Constraints) -> MeasureResult {
        MeasureResult::new(0.0, 0.0)
    }

    fn on_layout_pass(&self, ctx: &mut MeasureContext) {
        let sf = ctx.scale_factor;
        let style = &self.base.computed_style;
        // Logical font size; TextMeasurer converts to physical internally.
        let font_size = self.font_size.unwrap_or(DEFAULT_FONT_SIZE).value();

        let result = ctx.text.measure(
            &self.text,
            style.font.as_deref(),
            font_size,
            style.font_weight.unwrap_or_default(),
            style.font_style.unwrap_or_default(),
            0.0,
            0.0,
            None,
            sf,
        );

        let padding = self.effective_padding();
        let w = result.width + padding.left.to_physical(sf) + padding.right.to_physical(sf);
        let h = result.height + padding.top.to_physical(sf) + padding.bottom.to_physical(sf);
        self.label_size.set((w, h));
    }

    fn layout(&mut self, rect: LayoutBox) {
        self.layout_box = rect;
    }

    fn layout_box(&self) -> &LayoutBox {
        &self.layout_box
    }

    fn paint(&self, _ctx: &mut PaintContext) {}

    fn paint_top(&self, ctx: &mut PaintContext) {
        let opacity = self.opacity_anim.get();
        if opacity <= 0.001 {
            return;
        }

        let theme = crate::current_theme();
        let sf = ctx.scale_factor;
        let size = self.label_size.get();
        let (x, y) = self.box_position(self.layout_box, size);
        let scale = self.scale_progress.get();

        let raw_box = LayoutBox {
            x,
            y,
            width: size.0,
            height: size.1,
        };
        let popup_box = crate::scaled_layout_box(raw_box, scale);

        let bg = self
            .background
            .clone()
            .unwrap_or(Background::Color(theme.surface_container_highest));
        let bg_color = bg.representative_color();
        let radius = self
            .border_radius
            .unwrap_or(Length::px(8.0))
            .to_physical(sf)
            * scale;
        let border = self
            .border
            .unwrap_or_else(|| Border::all(1.0, theme.outline_variant));

        if let Some(shadows) = &self.base.computed_style.box_shadow {
            for shadow in shadows.iter().rev().filter(|s: &&BoxShadow| !s.inset) {
                let mut faded = *shadow;
                faded.color = faded.color.with_alpha_f32(faded.color.a() * opacity);
                self.paint_shadow_layer(ctx, popup_box, [radius; 4], &faded, sf);
            }
        }

        ctx.draw_rect(RectCommand {
            position: (popup_box.x, popup_box.y),
            size: (popup_box.width, popup_box.height),
            background: Some(Background::Color(
                bg_color.with_alpha_f32(bg_color.a() * opacity),
            )),
            border_radius: Some(BorderRadius::all(Length::px(radius))),
            border_width: Some(Length::px(border.top.to_physical(sf) * scale)),
            border_color: Some(border.color.with_alpha_f32(border.color.a() * opacity)),
            clip_rect: None,
        });

        let padding = self.effective_padding();
        let text_color = self
            .text_color
            .unwrap_or(theme.on_surface)
            .with_alpha_f32(opacity);

        let mut text_style = self.base.computed_style.clone();
        text_style
            .font_size
            .get_or_insert(self.font_size.unwrap_or(DEFAULT_FONT_SIZE));
        text_style.color = Some(text_color);

        ctx.draw_text(TextCommand {
            text: self.text.clone(),
            position: (
                popup_box.x + padding.left.to_physical(sf) * scale,
                popup_box.y + padding.top.to_physical(sf) * scale,
            ),
            style: text_style,
            max_width: None,
            clip_rect: None,
        });
    }

    fn event(&mut self, event: &InputEvent, ctx: &mut EventCtx) -> EventStatus {
        match event {
            InputEvent::MouseMoved { position } => {
                // hit_test already covers the floating popup's own bounds
                // while showing, so this doubles as the closing path when
                // the pointer leaves without a matching MouseExited ever
                // reaching this widget.
                if self.hit_test(*position) {
                    if self.hover_start.get().is_none() {
                        self.hover_start.set(Some(Instant::now()));
                    }
                } else {
                    self.hover_start.set(None);
                    if self.showing.get() {
                        self.showing.set(false);
                        self.base.dirty = true;
                        ctx.request_redraw();
                    }
                }
            }
            // Guaranteed to fire whenever the pointer leaves this widget's
            // whole subtree (anchor, and the popup itself while shown - see
            // `hit_test`) - unlike MouseMoved, which stops reaching this
            // widget the instant the cursor moves onto unrelated UI.
            InputEvent::MouseExited => {
                self.hover_start.set(None);
                if self.showing.get() {
                    self.showing.set(false);
                    self.base.dirty = true;
                    ctx.request_redraw();
                }
            }
            InputEvent::AnimationTick { .. } => {
                if !self.showing.get()
                    && let Some(start) = self.hover_start.get()
                    && Instant::now().duration_since(start) >= self.delay
                {
                    self.showing.set(true);
                    self.base.dirty = true;
                    ctx.request_redraw();
                }
            }
            _ => {}
        }
        EventStatus::Ignored
    }

    fn hit_test(&self, point: (f32, f32)) -> bool {
        if self.layout_box.contains_rounded(point, 0.0) {
            return true;
        }
        // While shown, the floating popup counts as part of this widget too,
        // so hovering it keeps the tooltip open instead of closing it the
        // moment the cursor leaves the anchor.
        if self.showing.get() {
            let size = self.label_size.get();
            let (x, y) = self.box_position(self.layout_box, size);
            return point.0 >= x && point.0 <= x + size.0 && point.1 >= y && point.1 <= y + size.1;
        }
        false
    }

    fn content_eq(&self, other: &dyn Widget) -> bool {
        let Some(other) = other.as_any().downcast_ref::<Tooltip>() else {
            return false;
        };

        self.text == other.text
            && self.placement == other.placement
            && self.delay == other.delay
            && self.gap == other.gap
            && self.background == other.background
            && self.text_color == other.text_color
            && self.padding == other.padding
            && self.border_radius == other.border_radius
            && self.font_size == other.font_size
            && self.border == other.border
            && self.base.authored_styles_eq(&other.base)
    }

    fn cascade_style(&mut self, parent: &Style, anim: &mut AnimationManager) {
        self.base.inherited_style = parent.clone();
        self.recompute_style();

        let target = if self.showing.get() { 1.0 } else { 0.0 };
        let key = AnimKey {
            widget: self.anim_id,
            layer: AnimLayer::Root,
            property: AnimProperty::Opacity,
        };
        let transition = self
            .base
            .computed_style
            .transition_overrides
            .opacity
            .or(self.base.computed_style.transition)
            .unwrap_or(FADE_TRANSITION);
        anim.set_target(key, AnimValue([target, 0.0, 0.0, 0.0]), Some(transition));
        self.opacity_anim
            .set(anim.value(key).map_or(target, |v| v.0[0]));

        let scale_target = if self.showing.get() { 1.0 } else { 0.92 };
        let scale_key = AnimKey {
            widget: self.anim_id,
            layer: AnimLayer::Root,
            property: AnimProperty::Scale,
        };
        anim.set_target(
            scale_key,
            AnimValue([scale_target, 0.0, 0.0, 0.0]),
            Some(SCALE_TRANSITION),
        );
        self.scale_progress
            .set(anim.value(scale_key).map_or(scale_target, |v| v.0[0]));

        for child in self.children.iter_mut() {
            child.cascade_style(&self.base.computed_style, anim);
        }
    }

    fn wants_animation_frame(&self) -> bool {
        self.hover_start.get().is_some() && !self.showing.get()
    }

    fn transfer_interaction_state(&mut self, old: &dyn Widget) {
        if let (Some(new), Some(old_i)) = (self.interaction_mut(), old.interaction()) {
            new.transfer_from(old_i);
        }
        if let Some(old) = old.as_any().downcast_ref::<Tooltip>() {
            self.anim_id = old.anim_id;
        }
    }

    fn transfer_measured_state(&mut self, old: &dyn Widget) {
        if let Some(old) = old.as_any().downcast_ref::<Tooltip>() {
            self.hover_start.set(old.hover_start.get());
            self.showing.set(old.showing.get());
            self.opacity_anim.set(old.opacity_anim.get());
            self.scale_progress.set(old.scale_progress.get());
            self.label_size.set(old.label_size.get());
        }
    }

    fn anim_id(&self) -> WidgetId {
        self.anim_id
    }
}
