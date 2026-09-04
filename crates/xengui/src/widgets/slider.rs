// SPDX-License-Identifier: Apache-2.0
use crate::{
    AnimKey, AnimLayer, AnimProperty, AnimValue, AnimationManager, Background, BorderRadius, Color,
    Constraints, Easing, ElementState, EventCtx, EventStatus, InputEvent, Interaction, Key,
    KeyState, LayoutBox, Length, MeasureContext, MeasureResult, MouseButton, PaintContext,
    RectCommand, Style, StyleBuilder, Transition, Widget, WidgetBase, WidgetId,
    constants::{DEFAULT_CURSOR_ICON, DEFAULT_POINTER_CURSOR_ICON},
};
use std::cell::Cell;
use web_time::Duration;

type ChangeCallback = Box<dyn FnMut(f32, &mut EventCtx)>;

const IDLE_TRACK_HEIGHT: f32 = 4.0;
const HOVER_TRACK_HEIGHT: f32 = 6.0;
const THUMB_DIAMETER: f32 = 13.0;

const HOVER_TRANSITION: Transition =
    Transition::new(Duration::from_millis(140)).easing(Easing::EaseOut);

/// A YouTube-Music-style horizontal slider: a thin track that thickens
/// and reveals a thumb on hover/drag. Used for both seek and volume
/// controls; `value` is a controlled prop in `0.0..=1.0`.
pub struct Slider {
    base: WidgetBase,
    anim_id: WidgetId,
    layout_box: LayoutBox,

    value: f32,
    track_color: Option<Color>,
    fill_color: Option<Color>,
    thumb_color: Option<Color>,

    dragging: Cell<bool>,
    hover_progress: Cell<f32>,
    scale_factor: Cell<f32>,

    on_change: Option<ChangeCallback>,
    on_commit: Option<ChangeCallback>,
}

impl Slider {
    /// Creates a value with its default configuration.
    pub fn new() -> Self {
        let mut interaction = Interaction::new();
        interaction.focusable = true;
        interaction.hover_cursor = Some(DEFAULT_POINTER_CURSOR_ICON);

        let mut slider = Self {
            base: WidgetBase::new(interaction),
            anim_id: WidgetId::new_unique(),
            layout_box: LayoutBox::default(),

            value: 0.0,
            track_color: None,
            fill_color: None,
            thumb_color: None,

            dragging: Cell::new(false),
            hover_progress: Cell::new(0.0),
            scale_factor: Cell::new(1.0),

            on_change: None,
            on_commit: None,
        };
        slider.recompute_style();
        slider
    }

    /// Returns or updates the `value` value.
    pub fn value(mut self, value: f32) -> Self {
        self.value = value.clamp(0.0, 1.0);
        self.mark_dirty();
        self
    }

    /// Returns or updates the `track_color` value.
    pub fn track_color(mut self, color: Color) -> Self {
        self.track_color = Some(color);
        self.mark_dirty();
        self
    }

    /// Returns or updates the `fill_color` value.
    pub fn fill_color(mut self, color: Color) -> Self {
        self.fill_color = Some(color);
        self.mark_dirty();
        self
    }

    /// Returns or updates the `thumb_color` value.
    pub fn thumb_color(mut self, color: Color) -> Self {
        self.thumb_color = Some(color);
        self.mark_dirty();
        self
    }

    /// Fired continuously while the user drags or clicks the track.
    pub fn on_change(mut self, f: impl FnMut(f32, &mut EventCtx) + 'static) -> Self {
        self.on_change = Some(Box::new(f));
        self
    }

    /// Fired once the interaction ends (mouse released, or an arrow-key
    /// nudge) - the natural point to commit a seek instead of reacting to
    /// every intermediate drag frame.
    pub fn on_commit(mut self, f: impl FnMut(f32, &mut EventCtx) + 'static) -> Self {
        self.on_commit = Some(Box::new(f));
        self
    }

    fn recompute_style(&mut self) {
        self.base.recompute_style();
        self.base.interaction.hover_cursor = self
            .base
            .computed_style
            .cursor
            .or(Some(DEFAULT_POINTER_CURSOR_ICON));
    }

    fn value_at(&self, local_x: f32, sf: f32) -> f32 {
        let thumb_r = THUMB_DIAMETER * 0.5 * sf;
        let usable = (self.layout_box.width - thumb_r * 2.0).max(1.0);
        ((local_x - thumb_r) / usable).clamp(0.0, 1.0)
    }

    fn set_value_from_event(&mut self, position: (f32, f32), sf: f32, ctx: &mut EventCtx) {
        let local_x = position.0 - self.layout_box.x;
        let next = self.value_at(local_x, sf);
        self.value = next;
        self.base.dirty = true;
        if let Some(cb) = self.on_change.as_mut() {
            cb(next, ctx);
        }
        ctx.request_redraw();
    }
}

impl Default for Slider {
    fn default() -> Self {
        Self::new()
    }
}

impl StyleBuilder for Slider {
    fn style_mut(&mut self) -> &mut Style {
        &mut self.base.style
    }

    fn mark_dirty(&mut self) {
        self.base.dirty = true;
        self.recompute_style();
    }
}

crate::impl_interaction_builders!(base Slider);
crate::impl_common_style_builders!(base Slider);

impl Widget for Slider {
    crate::impl_widget_boilerplate!();

    fn debug_name(&self) -> &'static str {
        "Widget#Slider"
    }

    fn measure(&self, ctx: &mut MeasureContext, constraints: Constraints) -> MeasureResult {
        let h = THUMB_DIAMETER * ctx.scale_factor;
        let w = constraints.max_width.unwrap_or(120.0 * ctx.scale_factor);
        let (w, h) = constraints.constrain_size(w, h);
        MeasureResult::new(w, h)
    }

    fn on_layout_pass(&self, ctx: &mut MeasureContext) {
        self.scale_factor.set(ctx.scale_factor);
    }

    fn paint(&self, ctx: &mut PaintContext) {
        let theme = crate::current_theme();
        let sf = ctx.scale_factor;
        let b = self.layout_box;
        let t = self.hover_progress.get();

        let track_h = (IDLE_TRACK_HEIGHT + (HOVER_TRACK_HEIGHT - IDLE_TRACK_HEIGHT) * t) * sf;
        let thumb_r = THUMB_DIAMETER * 0.5 * sf;
        let track_y = b.y + (b.height - track_h) * 0.5;
        let usable_w = (b.width - thumb_r * 2.0).max(0.0);
        let fill_w = usable_w * self.value;

        let track_color = self.track_color.unwrap_or(theme.surface_container_highest);
        let fill_color = self.fill_color.unwrap_or(theme.primary);
        let thumb_color = self.thumb_color.unwrap_or(fill_color);

        ctx.draw_rect(RectCommand {
            position: (b.x + thumb_r, track_y),
            size: (usable_w, track_h),
            background: Some(Background::Color(track_color)),
            border_radius: Some(BorderRadius::all(Length::px(track_h * 0.5))),
            border_width: None,
            border_color: None,
            clip_rect: None,
        });

        if fill_w > 0.0 {
            ctx.draw_rect(RectCommand {
                position: (b.x + thumb_r, track_y),
                size: (fill_w, track_h),
                background: Some(Background::Color(fill_color)),
                border_radius: Some(BorderRadius::all(Length::px(track_h * 0.5))),
                border_width: None,
                border_color: None,
                clip_rect: None,
            });
        }

        if t > 0.001 {
            let d = THUMB_DIAMETER * sf * t;
            let cx = b.x + thumb_r + usable_w * self.value;
            let cy = b.y + b.height * 0.5;
            let alpha_color = thumb_color.with_alpha_f32(thumb_color.a() * t);

            ctx.draw_rect(RectCommand {
                position: (cx - d * 0.5, cy - d * 0.5),
                size: (d, d),
                background: Some(Background::Color(alpha_color)),
                border_radius: Some(BorderRadius::all(Length::px(d * 0.5))),
                border_width: None,
                border_color: None,
                clip_rect: None,
            });
        }

        self.paint_focus(ctx);
    }

    fn event(&mut self, event: &InputEvent, ctx: &mut EventCtx) -> EventStatus {
        if !self.base.interaction.is_active() {
            return EventStatus::Ignored;
        }

        let sf = self.scale_factor.get();

        match event {
            InputEvent::MouseEntered => {
                self.base.interaction.hovered = true;
                if let Some(icon) = self.base.interaction.hover_cursor {
                    ctx.set_cursor_icon(icon);
                }
                self.base.dirty = true;
                ctx.request_redraw();
                return EventStatus::Handled;
            }
            InputEvent::MouseExited => {
                self.base.interaction.hovered = false;
                if self.base.interaction.hover_cursor.is_some() {
                    ctx.set_cursor_icon(DEFAULT_CURSOR_ICON);
                }
                if !self.dragging.get() {
                    self.base.dirty = true;
                    ctx.request_redraw();
                }
                return EventStatus::Handled;
            }
            InputEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Left,
                position,
            } => {
                self.dragging.set(true);
                ctx.request_focus();
                self.set_value_from_event(*position, sf, ctx);
                return EventStatus::Handled;
            }
            InputEvent::MouseMoved { position } if self.dragging.get() => {
                self.set_value_from_event(*position, sf, ctx);
                return EventStatus::Handled;
            }
            InputEvent::MouseInput {
                state: ElementState::Released,
                button: MouseButton::Left,
                position,
            } if self.dragging.get() => {
                self.dragging.set(false);
                self.set_value_from_event(*position, sf, ctx);
                if let Some(cb) = self.on_commit.as_mut() {
                    cb(self.value, ctx);
                }
                self.base.dirty = true;
                ctx.request_redraw();
                return EventStatus::Handled;
            }
            InputEvent::KeyInput {
                event: key_event, ..
            } if self.base.interaction.focused && key_event.state == KeyState::Pressed => {
                let step = 0.02;
                let next = match key_event.key {
                    Key::ArrowLeft | Key::ArrowDown => Some((self.value - step).max(0.0)),
                    Key::ArrowRight | Key::ArrowUp => Some((self.value + step).min(1.0)),
                    _ => None,
                };
                if let Some(next) = next {
                    self.value = next;
                    self.base.dirty = true;
                    if let Some(cb) = self.on_change.as_mut() {
                        cb(next, ctx);
                    }
                    if let Some(cb) = self.on_commit.as_mut() {
                        cb(next, ctx);
                    }
                    ctx.request_redraw();
                    return EventStatus::Handled;
                }
            }
            _ => {}
        }

        let status = self.base.interaction.handle(event, ctx);
        if matches!(status, EventStatus::Handled) {
            self.base.dirty = true;
            ctx.request_redraw();
        }
        status
    }

    fn content_eq(&self, other: &dyn Widget) -> bool {
        let Some(other) = other.as_any().downcast_ref::<Slider>() else {
            return false;
        };
        self.value == other.value
            && self.track_color == other.track_color
            && self.fill_color == other.fill_color
            && self.thumb_color == other.thumb_color
            && self.base.authored_styles_eq(&other.base)
    }

    fn cascade_style(&mut self, parent: &Style, anim: &mut AnimationManager) {
        self.base.inherited_style = parent.clone();
        self.recompute_style();

        let target = if self.base.interaction.hovered || self.dragging.get() {
            1.0
        } else {
            0.0
        };
        let key = AnimKey {
            widget: self.anim_id,
            layer: AnimLayer::Root,
            property: AnimProperty::Opacity,
        };
        anim.set_target(
            key,
            AnimValue([target, 0.0, 0.0, 0.0]),
            Some(HOVER_TRANSITION),
        );
        match anim.value(key) {
            Some(v) => {
                self.hover_progress.set(v.0[0]);
                self.base.dirty = true;
            }
            None => self.hover_progress.set(target),
        }
    }

    fn transfer_interaction_state(&mut self, old: &dyn Widget) {
        if let (Some(new), Some(old_i)) = (self.interaction_mut(), old.interaction()) {
            new.transfer_from(old_i);
        }
        if let Some(old) = old.as_any().downcast_ref::<Slider>() {
            self.anim_id = old.anim_id;
            self.dragging.set(old.dragging.get());
            self.hover_progress.set(old.hover_progress.get());
            self.scale_factor.set(old.scale_factor.get());
        }
    }

    fn anim_id(&self) -> WidgetId {
        self.anim_id
    }
}
