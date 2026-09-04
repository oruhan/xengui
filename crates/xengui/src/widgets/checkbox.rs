// SPDX-License-Identifier: Apache-2.0
use crate::{
    AnimKey, AnimLayer, AnimProperty, AnimValue, AnimationManager, Background, BorderRadius, Color,
    Constraints, Easing, ElementState, EventCtx, EventStatus, InputEvent, Interaction, Key,
    KeyState, LayoutBox, Length, MeasureContext, MeasureResult, MouseButton, PaintContext,
    RectCommand, Style, StyleBuilder, Transition, VariableIconCommand, Widget, WidgetBase,
    WidgetId,
    constants::{DEFAULT_CURSOR_ICON, DEFAULT_POINTER_CURSOR_ICON, DISABLED_WIDGET_OPACITY},
};
use std::cell::Cell;
use web_time::Duration;
use xengui_icons::material_symbols::{IconAxes, MaterialSymbolsVariable, codepoints};

type ChangeCallback = Box<dyn FnMut(bool, &mut EventCtx)>;

const CHECK_TRANSITION: Transition =
    Transition::new(Duration::from_millis(180)).easing(Easing::EaseOut);

fn lerp_color(a: Color, b: Color, t: f32) -> Color {
    let blended = AnimValue(a.to_f32_array()).lerp_premultiplied(AnimValue(b.to_f32_array()), t);
    Color::rgba_f32(blended.0[0], blended.0[1], blended.0[2], blended.0[3])
}

/// A toggleable square box with a checkmark, styled and animated the same
/// way as [`crate::Button`]. `checked` is a controlled prop - the caller
/// owns the state (e.g. via `use_state`) and updates it from `on_change`.
pub struct Checkbox {
    base: WidgetBase,
    anim_id: WidgetId,
    layout_box: LayoutBox,
    checked: bool,
    indeterminate: bool,
    display_indeterminate: Cell<bool>,
    size: f32,
    check_color: Option<Color>,
    check_codepoint: char,
    indeterminate_codepoint: char,
    icons_enabled: bool,
    check_icon_axes: Option<IconAxes>,
    on_change: Option<ChangeCallback>,
    // 0.0 (unchecked) -> 1.0 (checked), animated on every toggle and
    // driving both the fill/border color blend and the checkmark draw-in.
    check_progress: Cell<f32>,
}

impl Checkbox {
    /// Creates a value with its default configuration.
    pub fn new() -> Self {
        let mut interaction = Interaction::new();
        interaction.focusable = true;
        interaction.hover_cursor = Some(DEFAULT_POINTER_CURSOR_ICON);

        let mut checkbox = Self {
            base: WidgetBase::new(interaction),
            anim_id: WidgetId::new_unique(),
            layout_box: LayoutBox::default(),
            checked: false,
            indeterminate: false,
            display_indeterminate: Cell::new(false),
            size: 18.0,
            check_color: None,
            check_codepoint: codepoints::CHECK,
            indeterminate_codepoint: codepoints::REMOVE,
            icons_enabled: true,
            check_icon_axes: None,
            on_change: None,
            check_progress: Cell::new(0.0),
        };

        checkbox.recompute_style();
        checkbox
    }

    /// Returns or updates the `checked` value.
    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = checked;
        self.mark_dirty();
        self
    }

    /// Shows a dash instead of a checkmark for a "some but not all"
    /// tri-state selection.
    pub fn indeterminate(mut self, value: bool) -> Self {
        self.indeterminate = value;
        self.mark_dirty();
        self
    }

    /// Returns or updates the `size` value.
    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self.mark_dirty();
        self
    }

    /// Returns or updates the `check_color` value.
    pub fn check_color(mut self, color: Color) -> Self {
        self.check_color = Some(color);
        self.mark_dirty();
        self
    }

    /// Overrides the codepoint drawn while checked. Defaults to
    /// xengui-icons's check icon.
    pub fn check_icon(mut self, codepoint: char) -> Self {
        self.check_codepoint = codepoint;
        self.mark_dirty();
        self
    }

    /// Overrides the codepoint drawn while indeterminate. Defaults to
    /// xengui-icons's minus/remove icon.
    pub fn indeterminate_icon(mut self, codepoint: char) -> Self {
        self.indeterminate_codepoint = codepoint;
        self.mark_dirty();
        self
    }

    /// Hides the check/indeterminate icon entirely, leaving only the box.
    pub fn icons_enabled(mut self, enabled: bool) -> Self {
        self.icons_enabled = enabled;
        self.mark_dirty();
        self
    }

    /// Overrides the variable-font axes (weight/fill/grade/opsz) used to
    /// render the check/indeterminate icon.
    pub fn icon_axes(mut self, axes: IconAxes) -> Self {
        self.check_icon_axes = Some(axes);
        self.mark_dirty();
        self
    }

    /// Registers the `on_change` callback.
    pub fn on_change(mut self, f: impl FnMut(bool, &mut EventCtx) + 'static) -> Self {
        self.on_change = Some(Box::new(f));
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

    fn toggle(&mut self, ctx: &mut EventCtx) {
        // Coming from indeterminate, a click lands directly on unchecked -
        // toggling `checked` here (which was already false) would flash a
        // fully-checked frame before external state's own update lands.
        self.checked = if self.indeterminate {
            false
        } else {
            !self.checked
        };
        self.indeterminate = false;
        self.base.dirty = true;
        if let Some(cb) = self.on_change.as_mut() {
            cb(self.checked, ctx);
        }
        ctx.request_redraw();
    }
}

impl Default for Checkbox {
    fn default() -> Self {
        Self::new()
    }
}

impl StyleBuilder for Checkbox {
    fn style_mut(&mut self) -> &mut Style {
        &mut self.base.style
    }

    fn mark_dirty(&mut self) {
        self.base.dirty = true;
        self.recompute_style();
    }
}

crate::impl_interaction_builders!(base Checkbox);
crate::impl_common_style_builders!(base Checkbox);
crate::impl_themed_style_builders!(base Checkbox; hover_style => hover_style, pressed_style => pressed_style, disabled_style => disabled_style, focus_style => focus_style, focused_hover_style => focused_hover_style, focused_pressed_style => focused_pressed_style);

impl Widget for Checkbox {
    crate::impl_widget_boilerplate!();

    fn debug_name(&self) -> &'static str {
        "Widget#Checkbox"
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

        let t = self.check_progress.get();
        let dim = if self.base.interaction.enabled {
            1.0
        } else {
            DISABLED_WIDGET_OPACITY
        };

        let radius = style
            .border
            .as_ref()
            .and_then(|bo| bo.radius)
            .map(|r| Length::px(r.max_value()).to_physical(sf))
            .unwrap_or(4.0 * sf);

        let border = style.border.as_ref();

        let unchecked_fill = style
            .background
            .clone()
            .unwrap_or(Background::Color(Color::TRANSPARENT))
            .representative_color();
        let checked_fill = style
            .background
            .clone()
            .unwrap_or(Background::Color(theme.primary))
            .representative_color();
        let unchecked_border = border
            .map(|bo| bo.color)
            .unwrap_or(theme.on_surface_variant);
        let checked_border = border.map(|bo| bo.color).unwrap_or(theme.primary);

        let fill_base = lerp_color(unchecked_fill, checked_fill, t);
        let border_color_base = lerp_color(unchecked_border, checked_border, t);
        let fill = fill_base.with_alpha_f32(fill_base.a() * dim);
        let border_color = border_color_base.with_alpha_f32(border_color_base.a() * dim);

        ctx.draw_rect(RectCommand {
            position: (b.x, b.y),
            size: (b.width, b.height),
            background: Some(Background::Color(fill)),
            border_radius: Some(BorderRadius::all(Length::px(radius))),
            border_color: Some(border_color),
            border_width: Some(
                border
                    .map(|bo| Length::px(bo.top.to_physical(sf)))
                    .unwrap_or(Length::px(2.0 * sf)),
            ),
            clip_rect: None,
        });

        if self.icons_enabled && t > 0.001 {
            let icon_color = self.check_color.unwrap_or(theme.on_primary);
            let icon_size = b.width * 0.76 + 2.5 * sf;
            // Snapped to the pixel grid so the icon's own rounding inside
            // the pipeline can't drift relative to the (also rounded) box.
            let icon_x = (b.x + (b.width - icon_size) * 0.5).round();
            let icon_y = (b.y + (b.height - icon_size) * 0.5).round();

            let codepoint = if self.display_indeterminate.get() {
                self.indeterminate_codepoint
            } else {
                self.check_codepoint
            };

            let axes = self
                .check_icon_axes
                .unwrap_or_else(|| IconAxes::default().fill(1.0).weight(600.0));

            ctx.draw_variable_icon(VariableIconCommand {
                position: (icon_x, icon_y),
                size: (icon_size, icon_size),
                codepoint,
                font: MaterialSymbolsVariable::FONT,
                axes,
                color: icon_color.with_alpha_f32(icon_color.a() * t * dim),
                clip_rect: None,
            });
        }

        self.paint_outline(ctx);
    }

    fn event(&mut self, event: &InputEvent, ctx: &mut EventCtx) -> EventStatus {
        if !self.base.interaction.is_active() {
            return EventStatus::Ignored;
        }

        if let InputEvent::AnimationTick { .. } = event {
            if let Some(id) = &self.base.id {
                for action in crate::dom::take_actions(id) {
                    match action {
                        crate::dom::DomAction::Click => self.toggle(ctx),
                        crate::dom::DomAction::SetChecked(value) => {
                            self.checked = value;
                            self.indeterminate = false;
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
            self.toggle(ctx);
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
        let Some(other) = other.as_any().downcast_ref::<Checkbox>() else {
            return false;
        };

        self.checked == other.checked
            && self.indeterminate == other.indeterminate
            && self.size == other.size
            && self.check_color == other.check_color
            && self.check_codepoint == other.check_codepoint
            && self.indeterminate_codepoint == other.indeterminate_codepoint
            && self.icons_enabled == other.icons_enabled
            && self.check_icon_axes == other.check_icon_axes
            && self.base.authored_styles_eq(&other.base)
    }

    fn cascade_style(&mut self, parent: &Style, anim: &mut AnimationManager) {
        self.base.inherited_style = parent.clone();
        self.recompute_style();
        if crate::animate_computed_style(self.anim_id, &mut self.base.computed_style, anim) {
            self.base.dirty = true;
        }

        // Drives the fill/border blend and checkmark draw-in toward the
        // current `checked` state every frame.
        let target = if self.checked || self.indeterminate {
            1.0
        } else {
            0.0
        };
        let key = AnimKey {
            widget: self.anim_id,
            layer: AnimLayer::Content,
            property: AnimProperty::Opacity,
        };
        anim.set_target(
            key,
            AnimValue([target, 0.0, 0.0, 0.0]),
            Some(CHECK_TRANSITION),
        );
        match anim.value(key) {
            Some(v) => {
                self.check_progress.set(v.0[0]);
                self.base.dirty = true;
            }
            None => {
                if (self.check_progress.get() - target).abs() > f32::EPSILON {
                    self.base.dirty = true;
                }
                self.check_progress.set(target);
            }
        }

        // Keeps showing the indeterminate glyph through the whole fade-out when
        // leaving that state, instead of switching to the checkmark glyph the
        // instant `indeterminate` clears while the fill is still animating away.
        if self.indeterminate {
            self.display_indeterminate.set(true);
        } else if self.check_progress.get() <= 0.001 {
            self.display_indeterminate.set(false);
        }
    }

    fn after_interaction_transfer(&mut self) {
        self.recompute_style();
    }

    fn transfer_measured_state(&mut self, old: &dyn Widget) {
        if let Some(old) = old.as_any().downcast_ref::<Checkbox>() {
            self.check_progress.set(old.check_progress.get());
            self.display_indeterminate
                .set(old.display_indeterminate.get());
        }
    }

    fn transfer_interaction_state(&mut self, old: &dyn Widget) {
        if let (Some(new), Some(old_i)) = (self.interaction_mut(), old.interaction()) {
            new.transfer_from(old_i);
        }
        if let Some(old) = old.as_any().downcast_ref::<Checkbox>() {
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
