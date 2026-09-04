// SPDX-License-Identifier: Apache-2.0
use crate::{
    AnimKey, AnimLayer, AnimProperty, AnimValue, AnimationManager, Background, BorderRadius, Color,
    Constraints, Easing, ElementState, EventCtx, EventStatus, InputEvent, Interaction, Key,
    KeyState, LayoutBox, Length, MeasureContext, MeasureResult, MouseButton, PaintContext,
    RectCommand, Style, StyleBuilder, Transition, Widget, WidgetBase, WidgetId,
    constants::{DEFAULT_CURSOR_ICON, DEFAULT_POINTER_CURSOR_ICON, DISABLED_WIDGET_OPACITY},
};
use std::cell::Cell;
use web_time::Duration;

type SelectCallback = Box<dyn FnMut(&mut EventCtx)>;

const SELECT_TRANSITION: Transition =
    Transition::new(Duration::from_millis(180)).easing(Easing::EaseOut);

fn lerp_color(a: Color, b: Color, t: f32) -> Color {
    let blended = AnimValue(a.to_f32_array()).lerp_premultiplied(AnimValue(b.to_f32_array()), t);
    Color::rgba_f32(blended.0[0], blended.0[1], blended.0[2], blended.0[3])
}

/// A single circular radio option, styled and animated like [`crate::Checkbox`]
/// but rendered as a ring with an inner dot. `selected` is a controlled
/// prop; grouping (only one selected at a time) is left to the caller,
/// typically by comparing against a shared index kept in `use_state`.
pub struct RadioButton {
    base: WidgetBase,
    anim_id: WidgetId,
    layout_box: LayoutBox,
    selected: bool,
    size: f32,
    dot_color: Option<Color>,
    on_select: Option<SelectCallback>,
    select_progress: Cell<f32>,
}

impl RadioButton {
    /// Creates a value with its default configuration.
    pub fn new() -> Self {
        let mut interaction = Interaction::new();
        interaction.focusable = true;
        interaction.hover_cursor = Some(DEFAULT_POINTER_CURSOR_ICON);

        let mut radio = Self {
            base: WidgetBase::new(interaction),
            anim_id: WidgetId::new_unique(),
            layout_box: LayoutBox::default(),
            selected: false,
            size: 18.0,
            dot_color: None,
            on_select: None,
            select_progress: Cell::new(0.0),
        };

        radio.recompute_style();
        radio
    }

    /// Returns or updates the `selected` value.
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self.select_progress.set(if selected { 1.0 } else { 0.0 });
        self.mark_dirty();
        self
    }

    /// Returns or updates the `size` value.
    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self.mark_dirty();
        self
    }

    /// Returns or updates the `dot_color` value.
    pub fn dot_color(mut self, color: Color) -> Self {
        self.dot_color = Some(color);
        self.mark_dirty();
        self
    }

    /// Registers the `on_select` callback.
    pub fn on_select(mut self, f: impl FnMut(&mut EventCtx) + 'static) -> Self {
        self.on_select = Some(Box::new(f));
        self
    }

    fn recompute_style(&mut self) {
        self.base.recompute_style();
        self.base.interaction.hover_cursor =
            self.base
                .computed_style
                .cursor
                .or(Some(if self.base.interaction.enabled {
                    DEFAULT_POINTER_CURSOR_ICON
                } else {
                    DEFAULT_CURSOR_ICON
                }));
    }

    fn select(&mut self, ctx: &mut EventCtx) {
        if self.selected {
            return;
        }
        self.selected = true;
        self.base.dirty = true;
        if let Some(cb) = self.on_select.as_mut() {
            cb(ctx);
        }
        ctx.request_redraw();
    }
}

impl Default for RadioButton {
    fn default() -> Self {
        Self::new()
    }
}

impl StyleBuilder for RadioButton {
    fn style_mut(&mut self) -> &mut Style {
        &mut self.base.style
    }

    fn mark_dirty(&mut self) {
        self.base.dirty = true;
        self.recompute_style();
    }
}

crate::impl_interaction_builders!(base RadioButton);
crate::impl_common_style_builders!(base RadioButton);
crate::impl_themed_style_builders!(base RadioButton; hover_style => hover_style, pressed_style => pressed_style, disabled_style => disabled_style, focus_style => focus_style, focused_hover_style => focused_hover_style, focused_pressed_style => focused_pressed_style);

impl Widget for RadioButton {
    crate::impl_widget_boilerplate!();

    fn debug_name(&self) -> &'static str {
        "Widget#RadioButton"
    }

    fn measure(&self, ctx: &mut MeasureContext, constraints: Constraints) -> MeasureResult {
        let px = self.size * ctx.scale_factor;
        let (w, h) = constraints.constrain_size(px, px);
        MeasureResult::new(w, h)
    }

    fn paint(&self, ctx: &mut PaintContext) {
        let style = &self.base.computed_style;
        let sf = ctx.scale_factor;
        let b = crate::scaled_layout_box_with_origin(
            self.layout_box,
            style.scale.unwrap_or(1.0),
            style.transform_origin.unwrap_or_default(),
            sf,
        );
        let theme = crate::current_theme();

        let t = self.select_progress.get();
        let dim = if self.base.interaction.enabled {
            1.0
        } else {
            DISABLED_WIDGET_OPACITY
        };

        let border = style.border.as_ref();
        let unselected_border = border
            .map(|bo| bo.color)
            .unwrap_or(theme.on_surface_variant);
        let selected_border = border.map(|bo| bo.color).unwrap_or(theme.primary);
        let ring_color_base = lerp_color(unselected_border, selected_border, t);
        let ring_color = ring_color_base.with_alpha_f32(ring_color_base.a() * dim);

        let fill_base = style
            .background
            .clone()
            .unwrap_or(Background::Color(Color::TRANSPARENT))
            .representative_color();
        let fill = fill_base.with_alpha_f32(fill_base.a() * dim);

        ctx.draw_rect(RectCommand {
            position: (b.x, b.y),
            size: (b.width, b.height),
            background: Some(Background::Color(fill)),
            border_radius: Some(BorderRadius::all(Length::px(b.width * 0.5))),
            border_color: Some(ring_color),
            border_width: Some(
                border
                    .map(|bo| Length::px(bo.top.to_physical(sf)))
                    .unwrap_or(Length::px(2.0 * sf)),
            ),
            clip_rect: None,
        });

        if t > 0.001 {
            let dot_color_base = self.dot_color.unwrap_or(selected_border);
            let dot_d = b.width * 0.5 * t;
            let cx = b.x + b.width * 0.5;
            let cy = b.y + b.height * 0.5;

            ctx.draw_rect(RectCommand {
                position: (cx - dot_d * 0.5, cy - dot_d * 0.5),
                size: (dot_d, dot_d),
                background: Some(Background::Color(
                    dot_color_base.with_alpha_f32(dot_color_base.a() * t * dim),
                )),
                border_radius: Some(BorderRadius::all(Length::px(dot_d * 0.5))),
                border_width: None,
                border_color: None,
                clip_rect: None,
            });
        }

        self.paint_focus(ctx);
    }

    fn hit_test(&self, point: (f32, f32)) -> bool {
        self.layout_box
            .contains_rounded(point, self.layout_box.width * 0.5)
    }

    fn event(&mut self, event: &InputEvent, ctx: &mut EventCtx) -> EventStatus {
        if !self.base.interaction.is_active() {
            return EventStatus::Ignored;
        }

        if let InputEvent::AnimationTick { .. } = event {
            if let Some(id) = &self.base.id {
                for action in crate::dom::take_actions(id) {
                    match action {
                        crate::dom::DomAction::Click | crate::dom::DomAction::SetChecked(true) => {
                            self.select(ctx)
                        }
                        crate::dom::DomAction::SetChecked(false) => {
                            self.selected = false;
                            self.base.dirty = true;
                            ctx.request_redraw();
                        }
                        _ => {}
                    }
                }
            }
            return EventStatus::Handled;
        }

        let is_click = match event {
            InputEvent::MouseInput {
                state: ElementState::Released,
                button: MouseButton::Left,
                ..
            } => self.base.interaction.pressed && self.base.interaction.hovered,
            InputEvent::KeyInput {
                event: key_event, ..
            } => {
                self.base.interaction.focused
                    && !key_event.repeat
                    && key_event.state == KeyState::Pressed
                    && matches!(key_event.key, Key::Enter | Key::Space)
            }
            _ => false,
        };

        let before_style = self.base.computed_style.clone();
        let before_focus_visible = self.base.interaction.focus_visible;

        let status = self.base.interaction.handle(event, ctx);

        if is_click {
            self.select(ctx);
        }

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

    fn content_eq(&self, other: &dyn Widget) -> bool {
        let Some(other) = other.as_any().downcast_ref::<RadioButton>() else {
            return false;
        };

        self.selected == other.selected
            && self.size == other.size
            && self.dot_color == other.dot_color
            && self.base.authored_styles_eq(&other.base)
    }

    fn cascade_style(&mut self, parent: &Style, anim: &mut AnimationManager) {
        self.base.inherited_style = parent.clone();
        self.recompute_style();
        if crate::animate_computed_style(self.anim_id, &mut self.base.computed_style, anim) {
            self.base.dirty = true;
        }

        let target = if self.selected { 1.0 } else { 0.0 };
        let key = AnimKey {
            widget: self.anim_id,
            layer: AnimLayer::Content,
            property: AnimProperty::Opacity,
        };
        anim.set_target(
            key,
            AnimValue([target, 0.0, 0.0, 0.0]),
            Some(SELECT_TRANSITION),
        );
        match anim.value(key) {
            Some(v) => {
                self.select_progress.set(v.0[0]);
                self.base.dirty = true;
            }
            None => self.select_progress.set(target),
        }
    }

    fn after_interaction_transfer(&mut self) {
        self.recompute_style();
    }

    fn transfer_measured_state(&mut self, old: &dyn Widget) {
        if let Some(old) = old.as_any().downcast_ref::<RadioButton>() {
            self.select_progress.set(old.select_progress.get());
        }
    }

    fn transfer_interaction_state(&mut self, old: &dyn Widget) {
        if let (Some(new), Some(old_i)) = (self.interaction_mut(), old.interaction()) {
            new.transfer_from(old_i);
        }
        if let Some(old) = old.as_any().downcast_ref::<RadioButton>() {
            self.anim_id = old.anim_id;
        }
    }

    fn wants_animation_frame(&self) -> bool {
        self.base.interaction.enabled
            && self.base.id.as_deref().is_some_and(crate::dom::has_pending)
    }

    fn anim_id(&self) -> WidgetId {
        self.anim_id
    }
}
