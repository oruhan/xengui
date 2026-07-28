// SPDX-License-Identifier: Apache-2.0
use crate::{
    AnimationManager,
    Background,
    Color,
    Constraints,
    ElementState,
    EventCtx,
    EventStatus,
    InputEvent,
    Interaction,
    Key,
    KeyState,
    LayoutBox,
    Length,
    MeasureContext,
    MeasureResult,
    MouseButton,
    PaintContext,
    RectCommand,
    Style,
    StyleBuilder,
    TriangleCommand,
    Widget,
    WidgetBase,
    WidgetId,
    properties::{ DEFAULT_CURSOR_ICON, DEFAULT_POINTER_CURSOR_ICON },
};

type ChangeCallback = Box<dyn FnMut(bool, &mut EventCtx)>;

/// A toggleable square box with a checkmark, styled and animated the same
/// way as [`crate::Button`]. `checked` is a controlled prop - the caller
/// owns the state (e.g. via `use_state`) and updates it from `on_change`.
pub struct Checkbox {
    base: WidgetBase,
    anim_id: WidgetId,
    layout_box: LayoutBox,
    checked: bool,
    size: f32,
    check_color: Option<Color>,
    on_change: Option<ChangeCallback>,
}

impl Checkbox {
    pub fn new() -> Self {
        let mut interaction = Interaction::new();
        interaction.focusable = true;
        interaction.hover_cursor = Some(DEFAULT_POINTER_CURSOR_ICON);

        let mut checkbox = Self {
            base: WidgetBase::new(interaction),
            anim_id: WidgetId::new_unique(),
            layout_box: LayoutBox::default(),
            checked: false,
            size: 18.0,
            check_color: None,
            on_change: None,
        };

        checkbox.recompute_style();
        checkbox
    }

    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = checked;
        self.mark_dirty();
        self
    }

    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self.mark_dirty();
        self
    }

    pub fn check_color(mut self, color: Color) -> Self {
        self.check_color = Some(color);
        self.mark_dirty();
        self
    }

    pub fn on_change(mut self, f: impl FnMut(bool, &mut EventCtx) + 'static) -> Self {
        self.on_change = Some(Box::new(f));
        self
    }

    fn recompute_style(&mut self) {
        self.base.recompute_style();
        self.base.interaction.hover_cursor = self.base.computed_style.cursor.or(
            Some(
                if self.base.interaction.enabled {
                    DEFAULT_POINTER_CURSOR_ICON
                } else {
                    DEFAULT_CURSOR_ICON
                }
            )
        );
    }

    fn toggle(&mut self, ctx: &mut EventCtx) {
        self.checked = !self.checked;
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
crate::impl_themed_style_builders!(base Checkbox; hover_style => hover_style, pressed_style => pressed_style, disabled_style => disabled_style, focus_style => focus_style, focused_hover_style => focused_hover_style);

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
        let b = self.layout_box;
        let theme = crate::current_theme();

        let radius = style.border
            .as_ref()
            .and_then(|bo| bo.radius)
            .map(|r| r.to_physical(sf))
            .unwrap_or(4.0 * sf);
        let border = style.border.as_ref();

        let (fill, border_color) = if self.checked {
            (
                style.background.clone().unwrap_or(Background::Color(theme.primary)),
                border.map(|bo| bo.color).unwrap_or(theme.primary),
            )
        } else {
            (
                style.background.clone().unwrap_or(Background::Color(Color::TRANSPARENT)),
                border.map(|bo| bo.color).unwrap_or(theme.border),
            )
        };

        ctx.draw_rect(RectCommand {
            position: (b.x, b.y),
            size: (b.width, b.height),
            background: Some(fill),
            border_radius: Some(Length::px(radius)),
            border_color: Some(border_color),
            border_width: Some(
                border.map(|bo| Length::px(bo.top.to_physical(sf))).unwrap_or(Length::px(1.5 * sf))
            ),
            clip_rect: None,
        });

        if self.checked {
            let check_color = self.check_color.unwrap_or(theme.background);
            let stroke = (b.width * 0.12).max(1.5 * sf);

            let p0 = (b.x + b.width * 0.22, b.y + b.height * 0.52);
            let p1 = (b.x + b.width * 0.42, b.y + b.height * 0.72);
            let p2 = (b.x + b.width * 0.8, b.y + b.height * 0.28);

            for (a, bnd) in [
                (p0, p1),
                (p1, p2),
            ] {
                let (dx, dy) = (bnd.0 - a.0, bnd.1 - a.1);
                let len = (dx * dx + dy * dy).sqrt().max(0.0001);
                let (nx, ny) = ((-dy / len) * stroke * 0.5, (dx / len) * stroke * 0.5);
                let q0 = (a.0 + nx, a.1 + ny);
                let q1 = (a.0 - nx, a.1 - ny);
                let q2 = (bnd.0 + nx, bnd.1 + ny);
                let q3 = (bnd.0 - nx, bnd.1 - ny);

                ctx.draw_triangle(TriangleCommand {
                    p0: q0,
                    p1: q1,
                    p2: q2,
                    color: check_color,
                    clip_rect: None,
                });
                ctx.draw_triangle(TriangleCommand {
                    p0: q1,
                    p1: q3,
                    p2: q2,
                    color: check_color,
                    clip_rect: None,
                });
            }
        }

        self.paint_outline(ctx);
    }

    fn event(&mut self, event: &InputEvent, ctx: &mut EventCtx) -> EventStatus {
        if !self.base.interaction.is_active() {
            return EventStatus::Ignored;
        }

        let is_click = match event {
            InputEvent::MouseInput {
                state: ElementState::Released,
                button: MouseButton::Left,
                ..
            } => self.base.interaction.pressed && self.base.interaction.hovered,
            InputEvent::KeyInput { event: key_event, .. } =>
                self.base.interaction.focused &&
                    !key_event.repeat &&
                    key_event.state == KeyState::Pressed &&
                    matches!(key_event.key, Key::Enter | Key::Space),
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

    fn content_eq(&self, other: &dyn Widget) -> bool {
        let Some(other) = other.as_any().downcast_ref::<Checkbox>() else {
            return false;
        };

        self.checked == other.checked &&
            self.size == other.size &&
            self.check_color == other.check_color &&
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
    }

    fn after_interaction_transfer(&mut self) {
        self.recompute_style();
    }

    fn transfer_measured_state(&mut self, old: &dyn Widget) {
        if let Some(old) = old.as_any().downcast_ref::<Checkbox>() {
            self.anim_id = old.anim_id;
        }
    }

    fn anim_id(&self) -> WidgetId {
        self.anim_id
    }
}
