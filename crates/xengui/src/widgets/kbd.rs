// SPDX-License-Identifier: Apache-2.0
use crate::{
    AnimKey, AnimLayer, AnimProperty, AnimValue, AnimationManager, Background, BorderRadius, Color,
    Constraints, Cursor, Easing, Edges, ElementState, EventCtx, EventStatus, InputEvent,
    Interaction, LayoutBox, Length, MeasureContext, MeasureResult, MouseButton, PaintContext,
    RectCommand, Style, StyleBuilder, TextCommand, Transition, Widget, WidgetBase, WidgetContent,
    WidgetId, constants::DEFAULT_FONT_SIZE,
};
use smol_str::SmolStr;
use std::cell::Cell;
use web_time::Duration;

/// Vertical depth (logical px) of the 3D "well" beneath the keycap.
const KBD_DEPTH: f32 = 3.0;
const KBD_PRESS_TRANSITION: Transition =
    Transition::new(Duration::from_millis(90)).easing(Easing::EaseOut);
const KBD_HOVER_TRANSITION: Transition =
    Transition::new(Duration::from_millis(150)).easing(Easing::cubic_bezier(0.31, 0.94, 0.34, 1.0));
const KBD_HOVER_STRENGTH: f32 = 0.35;

/// Displays a single keyboard key or shortcut (e.g. "Ctrl", "⌘K"), styled
/// like a physical keycap that presses flush into its own base on click.
pub struct Kbd {
    base: WidgetBase,
    anim_id: WidgetId,
    hover_anim_id: WidgetId,
    content: SmolStr,
    layout_box: LayoutBox,
    content_size: Cell<(f32, f32)>,
    pressed: Cell<bool>,
    press_progress: Cell<f32>,
    hover_progress: Cell<f32>,
}

impl Kbd {
    /// Creates a value with its default configuration.
    pub fn new() -> Self {
        let mut interaction = Interaction::new();
        interaction.hover_cursor = Some(Cursor::Pointer);

        let mut base = WidgetBase::new(interaction);

        // A keyboard legend is code-like content. The compact padding keeps
        // the keycap proportional to short labels such as "K" and "Esc".
        base.style.padding = Some(Edges::symmetric(2.0, 1.0));
        base.style.font = Some(SmolStr::new("monospace"));
        base.style.font_size = Some(Length::px(12.0));

        let mut kbd = Self {
            base,
            anim_id: WidgetId::new_unique(),
            hover_anim_id: WidgetId::new_unique(),
            content: SmolStr::new(""),
            layout_box: LayoutBox::default(),
            content_size: Cell::new((0.0, 0.0)),
            pressed: Cell::new(false),
            press_progress: Cell::new(0.0),
            hover_progress: Cell::new(0.0),
        };
        kbd.recompute_style();
        kbd
    }

    /// Returns or updates the `label` value.
    pub fn label(mut self, content: impl Into<SmolStr>) -> Self {
        self.content = content.into();
        self.mark_dirty();
        self
    }

    fn recompute_style(&mut self) {
        self.base.recompute_style();
    }
}

impl Default for Kbd {
    fn default() -> Self {
        Self::new()
    }
}

impl StyleBuilder for Kbd {
    fn style_mut(&mut self) -> &mut Style {
        &mut self.base.style
    }

    fn mark_dirty(&mut self) {
        self.base.dirty = true;
        self.recompute_style();
    }
}

impl WidgetContent for Kbd {
    fn with_content(self, content: impl Into<SmolStr>) -> Self {
        self.label(content)
    }
}

crate::impl_common_style_builders!(base Kbd);

impl Widget for Kbd {
    crate::impl_widget_boilerplate!();

    fn debug_name(&self) -> &'static str {
        "Widget#Kbd"
    }

    fn measure(&self, ctx: &mut MeasureContext, constraints: Constraints) -> MeasureResult {
        let scale_factor = ctx.scale_factor;
        let style = &self.base.computed_style;

        let font_size = style.font_size.unwrap_or(DEFAULT_FONT_SIZE).value();

        let result = ctx.text.measure(
            &self.content,
            style.font.as_deref(),
            font_size,
            style.font_weight.unwrap_or_default(),
            style.font_style.unwrap_or_default(),
            0.0,
            0.0,
            None,
            scale_factor,
        );

        self.content_size.set((result.width, result.height));

        let padding = style.padding.unwrap_or_default();
        let width = result.width
            + padding.left.to_physical(scale_factor)
            + padding.right.to_physical(scale_factor);
        let height = result.height
            + padding.top.to_physical(scale_factor)
            + padding.bottom.to_physical(scale_factor)
            + KBD_DEPTH * scale_factor;

        let (width, height) = constraints.constrain_size(width, height);
        MeasureResult::new(width, height)
    }

    fn paint(&self, ctx: &mut PaintContext) {
        let style = &self.base.computed_style;
        let sf = ctx.scale_factor;
        let b = self.layout_box;
        let t = self.press_progress.get();
        let theme = crate::current_theme();

        let hover = self.hover_progress.get().clamp(0.0, 1.0) * KBD_HOVER_STRENGTH;

        let depth = KBD_DEPTH * sf;
        // t=0 (idle) keeps the cap raised at the top; t=1 (pressed) sinks
        // it down flush with the well beneath it.
        let lift = depth * t;

        let (border_color, border_width) = match style.border.as_ref() {
            Some(bo) => (bo.color, bo.top.to_physical(sf)),
            None => (
                mix_color(theme.outline_variant, theme.outline, hover),
                1.0 * sf,
            ),
        };

        let radius = style
            .border
            .as_ref()
            .and_then(|bo| bo.radius)
            .map(|r| r.max_value() * sf)
            .unwrap_or(5.0 * sf);

        let well_color = Color::rgba_f32(
            border_color.r() * 0.75,
            border_color.g() * 0.75,
            border_color.b() * 0.75,
            border_color.a(),
        );

        ctx.draw_rect(RectCommand {
            position: (b.x, b.y + depth),
            size: (b.width, (b.height - depth).max(0.0)),
            background: Some(Background::Color(well_color)),
            border_radius: Some(BorderRadius::all(Length::px(radius))),
            border_width: None,
            border_color: None,
            clip_rect: None,
        });

        let cap_height = (b.height - depth).max(1.0);

        let cap_background = style
            .background
            .clone()
            .unwrap_or(Background::Color(mix_color(
                theme.surface_container,
                theme.surface_container_high,
                hover,
            )));

        ctx.draw_rect(RectCommand {
            position: (b.x, b.y + lift),
            size: (b.width, cap_height),
            background: Some(cap_background),
            border_radius: Some(BorderRadius::all(Length::px(radius))),
            border_color: Some(border_color),
            border_width: Some(Length::px(border_width)),
            clip_rect: None,
        });

        let (text_width, text_height) = self.content_size.get();
        let text_x = b.x + (b.width - text_width) * 0.5;
        let text_y = b.y + lift + (cap_height - text_height) * 0.5;

        let mut text_style = style.clone();
        text_style.font_size.get_or_insert(Length::px(12.0));
        text_style.color.get_or_insert(mix_color(
            theme.on_surface_variant,
            theme.on_surface,
            hover,
        ));

        ctx.draw_text(TextCommand {
            text: self.content.clone(),
            position: (text_x, text_y),
            style: text_style,
            max_width: None,
            clip_rect: None,
        });
    }

    fn event(&mut self, event: &InputEvent, ctx: &mut EventCtx) -> EventStatus {
        if !self.base.interaction.is_active() {
            return EventStatus::Ignored;
        }

        let status = self.base.interaction.handle(event, ctx);
        let mut handled = matches!(status, EventStatus::Handled);

        match event {
            InputEvent::MouseEntered => {
                self.base.dirty = true;
                ctx.request_redraw();
                handled = true;
            }
            InputEvent::MouseExited => {
                self.pressed.set(false);
                self.base.dirty = true;
                ctx.request_redraw();
                handled = true;
            }
            InputEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Left,
                ..
            } => {
                self.pressed.set(true);
                self.base.dirty = true;
                ctx.request_redraw();
                handled = true;
            }
            InputEvent::MouseInput {
                state: ElementState::Released,
                button: MouseButton::Left,
                ..
            } => {
                if self.pressed.get() {
                    self.pressed.set(false);
                    self.base.dirty = true;
                    ctx.request_redraw();
                }
                handled = true;
            }
            _ => {}
        }

        if handled {
            EventStatus::Handled
        } else {
            EventStatus::Ignored
        }
    }

    fn content_eq(&self, other: &dyn Widget) -> bool {
        let Some(other) = other.as_any().downcast_ref::<Kbd>() else {
            return false;
        };
        self.content == other.content && self.base.authored_styles_eq(&other.base)
    }

    fn cascade_style(&mut self, parent: &Style, anim: &mut AnimationManager) {
        self.base.inherited_style = parent.clone();
        self.recompute_style();
        if crate::animate_computed_style(self.anim_id, &mut self.base.computed_style, anim) {
            self.base.dirty = true;
        }

        let target = if self.pressed.get() { 1.0 } else { 0.0 };
        let key = AnimKey {
            widget: self.anim_id,
            layer: AnimLayer::Root,
            property: AnimProperty::Scale,
        };
        anim.set_target(
            key,
            AnimValue([target, 0.0, 0.0, 0.0]),
            Some(KBD_PRESS_TRANSITION),
        );
        match anim.value(key) {
            Some(v) => {
                self.press_progress.set(v.0[0]);
                self.base.dirty = true;
            }
            None => self.press_progress.set(target),
        }

        let hover_target = if self.base.interaction.hovered {
            1.0
        } else {
            0.0
        };
        let hover_key = AnimKey {
            widget: self.hover_anim_id,
            layer: AnimLayer::Root,
            property: AnimProperty::Opacity,
        };
        anim.set_target(
            hover_key,
            AnimValue([hover_target, 0.0, 0.0, 0.0]),
            Some(KBD_HOVER_TRANSITION),
        );
        match anim.value(hover_key) {
            Some(value) => {
                self.hover_progress.set(value.0[0]);
                self.base.dirty = true;
            }
            None => self.hover_progress.set(hover_target),
        }
    }

    fn transfer_measured_state(&mut self, old: &dyn Widget) {
        if let Some(old) = old.as_any().downcast_ref::<Kbd>() {
            self.content_size.set(old.content_size.get());
            self.pressed.set(old.pressed.get());
            self.press_progress.set(old.press_progress.get());
            self.hover_progress.set(old.hover_progress.get());
        }
    }

    fn transfer_interaction_state(&mut self, old: &dyn Widget) {
        if let (Some(new), Some(old_i)) = (self.interaction_mut(), old.interaction()) {
            new.transfer_from(old_i);
        }
        if let Some(old) = old.as_any().downcast_ref::<Kbd>() {
            self.anim_id = old.anim_id;
            self.hover_anim_id = old.hover_anim_id;
        }
    }

    fn anim_id(&self) -> WidgetId {
        self.anim_id
    }
}

fn mix_color(from: Color, to: Color, amount: f32) -> Color {
    let amount = amount.clamp(0.0, 1.0);
    Color::rgba_f32(
        from.r() + (to.r() - from.r()) * amount,
        from.g() + (to.g() - from.g()) * amount,
        from.b() + (to.b() - from.b()) * amount,
        from.a() + (to.a() - from.a()) * amount,
    )
}
