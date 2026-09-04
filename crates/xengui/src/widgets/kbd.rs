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

/// Displays a single keyboard key or shortcut (e.g. "Ctrl", "⌘K"), styled
/// like a physical keycap that presses flush into its own base on click.
pub struct Kbd {
    base: WidgetBase,
    anim_id: WidgetId,
    content: SmolStr,
    layout_box: LayoutBox,
    content_size: Cell<(f32, f32)>,
    pressed: Cell<bool>,
    press_progress: Cell<f32>,
}

impl Kbd {
    /// Creates a value with its default configuration.
    pub fn new() -> Self {
        let mut interaction = Interaction::new();
        interaction.hover_cursor = Some(Cursor::Pointer);

        let mut base = WidgetBase::new(interaction);

        base.style.padding = Some(Edges::symmetric(6.0, 2.0));
        base.style.font_size = Some(Length::px(13.0));

        let mut kbd = Self {
            base,
            anim_id: WidgetId::new_unique(),
            content: SmolStr::new(""),
            layout_box: LayoutBox::default(),
            content_size: Cell::new((0.0, 0.0)),
            pressed: Cell::new(false),
            press_progress: Cell::new(0.0),
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
        let b = crate::scaled_layout_box_with_origin(
            self.layout_box,
            style.scale.unwrap_or(1.0),
            style.transform_origin.unwrap_or_default(),
            sf,
        );
        let t = self.press_progress.get();
        let theme = crate::current_theme();

        let hovered = self.base.interaction.hovered;

        let depth = KBD_DEPTH * sf;
        // t=0 (idle) keeps the cap raised at the top; t=1 (pressed) sinks
        // it down flush with the well beneath it.
        let lift = depth * t;

        let (border_color, border_width) = match style.border.as_ref() {
            Some(bo) => (bo.color, bo.top.to_physical(sf)),
            None => (
                if hovered {
                    theme.outline
                } else {
                    theme.outline_variant
                },
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
            .unwrap_or(Background::Color(if hovered {
                theme.surface_container_high
            } else {
                theme.surface_container
            }));

        ctx.draw_rect(RectCommand {
            position: (b.x, b.y + lift),
            size: (b.width, cap_height),
            background: Some(cap_background),
            border_radius: Some(BorderRadius::all(Length::px(radius))),
            border_color: Some(border_color),
            border_width: Some(Length::px(border_width)),
            clip_rect: None,
        });

        let padding = style.padding.unwrap_or_default();
        let text_x = b.x + padding.left.to_physical(sf);
        let text_y = b.y + lift + padding.top.to_physical(sf);

        let mut text_style = style.clone();
        text_style.font_size.get_or_insert(Length::px(13.0));
        text_style.color.get_or_insert(if hovered {
            theme.on_surface
        } else {
            theme.on_surface_variant
        });

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
    }

    fn transfer_measured_state(&mut self, old: &dyn Widget) {
        if let Some(old) = old.as_any().downcast_ref::<Kbd>() {
            self.content_size.set(old.content_size.get());
            self.pressed.set(old.pressed.get());
            self.press_progress.set(old.press_progress.get());
        }
    }

    fn transfer_interaction_state(&mut self, old: &dyn Widget) {
        if let (Some(new), Some(old_i)) = (self.interaction_mut(), old.interaction()) {
            new.transfer_from(old_i);
        }
        if let Some(old) = old.as_any().downcast_ref::<Kbd>() {
            self.anim_id = old.anim_id;
        }
    }

    fn anim_id(&self) -> WidgetId {
        self.anim_id
    }
}
