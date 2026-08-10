// SPDX-License-Identifier: Apache-2.0
use crate::{
    AnimKey,
    AnimLayer,
    AnimProperty,
    AnimValue,
    AnimationManager,
    Background,
    BorderRadius,
    Color,
    Constraints,
    Easing,
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
    Transition,
    Widget,
    WidgetBase,
    WidgetId,
    constants::{ DEFAULT_CURSOR_ICON, DEFAULT_POINTER_CURSOR_ICON },
};
use std::cell::Cell;
use web_time::Duration;

type ChangeCallback = Box<dyn FnMut(bool, &mut EventCtx)>;

const TRACK_WIDTH: f32 = 52.0;
const TRACK_HEIGHT: f32 = 32.0;

const THUMB_UNSELECTED: f32 = 16.0;
const THUMB_SELECTED: f32 = 24.0;

const THUMB_PRESSED_UNCHECKED: f32 = 28.0;
const THUMB_PRESSED_CHECKED: f32 = 28.0;

const TRACK_PADDING_LEFT: f32 = 4.0;
const TRACK_PADDING_RIGHT: f32 = 4.0;

const TOGGLE_TRANSITION: Transition = Transition::new(Duration::from_millis(200)).easing(
    Easing::EaseOut
);

const THUMB_SIZE_TRANSITION: Transition = Transition::new(Duration::from_millis(150)).easing(
    Easing::EaseInOut
);

fn lerp_color(a: Color, b: Color, t: f32) -> Color {
    let blended = AnimValue(a.to_f32_array()).lerp_premultiplied(AnimValue(b.to_f32_array()), t);
    Color::rgba_f32(blended.0[0], blended.0[1], blended.0[2], blended.0[3])
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// A Material Design 3-style toggle switch: a pill-shaped track with a
/// sliding thumb that grows and moves as it turns on, playing the same
/// controlled-prop role as [`crate::Checkbox`] for on/off settings.
pub struct Switch {
    base: WidgetBase,
    anim_id: WidgetId,
    layout_box: LayoutBox,
    checked: bool,
    size: f32,
    track_on_color: Option<Color>,
    track_off_color: Option<Color>,
    thumb_on_color: Option<Color>,
    thumb_off_color: Option<Color>,
    border_color: Option<Color>,
    progress: Cell<f32>,
    thumb_size: Cell<f32>,
    on_change: Option<ChangeCallback>,
    /*icon_on: IconSlot,
    icon_off: IconSlot,*/
}

impl Switch {
    pub fn new() -> Self {
        let mut interaction = Interaction::new();
        interaction.focusable = true;
        interaction.hover_cursor = Some(DEFAULT_POINTER_CURSOR_ICON);

        let mut switch = Self {
            base: WidgetBase::new(interaction),
            anim_id: WidgetId::new_unique(),
            layout_box: LayoutBox::default(),
            checked: false,
            size: 1.0,
            track_on_color: None,
            track_off_color: None,
            thumb_on_color: None,
            thumb_off_color: None,
            border_color: None,
            progress: Cell::new(0.0),
            thumb_size: Cell::new(THUMB_UNSELECTED),
            on_change: None,
            /*icon_on: IconSlot::default_check(),
            icon_off: IconSlot::default_minus(),*/
        };

        switch.recompute_style();
        switch
    }

    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = checked;
        self.progress.set(if checked { 1.0 } else { 0.0 });
        self.mark_dirty();
        self
    }

    /// Scales the whole switch; `1.0` matches Material's default 52x32 track.
    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self.mark_dirty();
        self
    }

    pub fn track_on_color(mut self, color: Color) -> Self {
        self.track_on_color = Some(color);
        self.mark_dirty();
        self
    }

    pub fn track_off_color(mut self, color: Color) -> Self {
        self.track_off_color = Some(color);
        self.mark_dirty();
        self
    }

    pub fn thumb_on_color(mut self, color: Color) -> Self {
        self.thumb_on_color = Some(color);
        self.mark_dirty();
        self
    }

    pub fn thumb_off_color(mut self, color: Color) -> Self {
        self.thumb_off_color = Some(color);
        self.mark_dirty();
        self
    }

    pub fn border_color(mut self, color: Color) -> Self {
        self.border_color = Some(color);
        self.mark_dirty();
        self
    }

    pub fn on_change(mut self, f: impl FnMut(bool, &mut EventCtx) + 'static) -> Self {
        self.on_change = Some(Box::new(f));
        self
    }

    /*/// Overrides the icon shown once the thumb settles into the "on"
    /// state. Defaults to xengui-icons's check icon; accepts any SVG
    /// source, including another `xengui-icons` constant.
    pub fn icon_on(mut self, svg_source: &str) -> Self {
        self.icon_on.set_svg(svg_source);
        self.mark_dirty();
        self
    }

    /// Overrides the icon shown once the thumb settles into the "off"
    /// state. Defaults to xengui-icons's minus icon.
    pub fn icon_off(mut self, svg_source: &str) -> Self {
        self.icon_off.set_svg(svg_source);
        self.mark_dirty();
        self
    }

    /// Hides both on/off thumb icons entirely.
    pub fn icons_enabled(mut self, enabled: bool) -> Self {
        self.icon_on.set_enabled(enabled);
        self.icon_off.set_enabled(enabled);
        self.mark_dirty();
        self
    }*/

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

impl Default for Switch {
    fn default() -> Self {
        Self::new()
    }
}

impl StyleBuilder for Switch {
    fn style_mut(&mut self) -> &mut Style {
        &mut self.base.style
    }

    fn mark_dirty(&mut self) {
        self.base.dirty = true;
        self.recompute_style();
    }
}

crate::impl_interaction_builders!(base Switch);
crate::impl_common_style_builders!(base Switch);

impl Widget for Switch {
    crate::impl_widget_boilerplate!();

    fn debug_name(&self) -> &'static str {
        "Widget#Switch"
    }

    fn measure(&self, ctx: &mut MeasureContext, constraints: Constraints) -> MeasureResult {
        let w = TRACK_WIDTH * self.size * ctx.scale_factor;
        let h = TRACK_HEIGHT * self.size * ctx.scale_factor;
        let (w, h) = constraints.constrain_size(w, h);
        MeasureResult::new(w, h)
    }

    fn paint(&self, ctx: &mut PaintContext) {
        let sf = ctx.scale_factor * self.size;
        let b = self.layout_box;
        let theme = crate::current_theme();
        let t = self.progress.get();

        let track_off = self.track_off_color.unwrap_or(theme.surface_container_high);
        let track_on = self.track_on_color.unwrap_or(theme.primary);
        let thumb_off = self.thumb_off_color.unwrap_or(
            self.border_color.unwrap_or(theme.on_surface_variant)
        );
        let thumb_on = self.thumb_on_color.unwrap_or(theme.on_primary);
        let border_color = self.border_color.unwrap_or(theme.outline);

        let track_color = lerp_color(track_off, track_on, t);
        let thumb_color = lerp_color(thumb_off, thumb_on, t);

        ctx.draw_rect(RectCommand {
            position: (b.x, b.y),
            size: (b.width, b.height),
            background: Some(Background::Color(track_color)),
            border_radius: Some(BorderRadius::all(Length::px(b.height * 0.5))),
            border_width: (t < 0.999).then(|| Length::px(2.0 * sf * (1.0 - t))),
            border_color: (t < 0.999).then_some(border_color),
            clip_rect: None,
        });

        // Animated thumb diameter, driven from cascade_style instead of
        // snapping instantly between idle/pressed sizes.
        let thumb_d = self.thumb_size.get() * sf;
        let pad_left = TRACK_PADDING_LEFT * sf;
        let pad_right = TRACK_PADDING_RIGHT * sf;

        let min_cx = b.x + pad_left + thumb_d * 0.5;
        let max_cx = b.x + b.width - pad_right - thumb_d * 0.5;
        let cx = lerp(min_cx, max_cx, t);
        let cy = b.y + b.height * 0.5;

        ctx.draw_rect(RectCommand {
            position: (cx - thumb_d * 0.5, cy - thumb_d * 0.5),
            size: (thumb_d, thumb_d),
            background: Some(Background::Color(thumb_color)),
            border_radius: Some(BorderRadius::all(Length::px(thumb_d * 0.5))),
            border_width: None,
            border_color: None,
            clip_rect: None,
        });

        let icon_size = thumb_d * 0.64;
        let icon_rect = (cx - icon_size * 0.5, cy - icon_size * 0.5, icon_size, icon_size);

        /*if t > 0.6 {
            // Icon fades in once the thumb has mostly grown to its "on"
            // size, matching Material 3's icon-switch timing.
            let mark_alpha = ((t - 0.6) / 0.4).clamp(0.0, 1.0);
            self.icon_on.paint(ctx, icon_rect, track_on, mark_alpha);
        } else if t < 0.4 {
            // Icon fades in as the thumb shrinks back to its "off" size.
            let mark_alpha = ((0.4 - t) / 0.4).clamp(0.0, 1.0);
            self.icon_off.paint(ctx, icon_rect, track_off, mark_alpha);
        }*/

        self.paint_outline(ctx);
    }

    fn hit_test(&self, point: (f32, f32)) -> bool {
        self.layout_box.contains_rounded(point, self.layout_box.height * 0.5)
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
        let before_pressed = self.base.interaction.pressed;

        let status = self.base.interaction.handle(event, ctx);

        if is_click {
            self.toggle(ctx);
        }

        if
            matches!(status, EventStatus::Handled) ||
            before_pressed != self.base.interaction.pressed
        {
            self.recompute_style();

            if
                self.base.computed_style != before_style ||
                self.base.interaction.focus_visible != before_focus_visible ||
                before_pressed != self.base.interaction.pressed
            {
                self.base.dirty = true;
                ctx.request_redraw();
            }
        }

        status
    }

    fn content_eq(&self, other: &dyn Widget) -> bool {
        let Some(other) = other.as_any().downcast_ref::<Switch>() else {
            return false;
        };

        self.checked == other.checked &&
            self.size == other.size &&
            self.track_on_color == other.track_on_color &&
            self.track_off_color == other.track_off_color &&
            self.thumb_on_color == other.thumb_on_color &&
            self.thumb_off_color == other.thumb_off_color &&
            self.border_color == other.border_color &&
            /*self.icon_on == other.icon_on &&
            self.icon_off == other.icon_off &&*/
            self.base.style == other.base.style
    }

    fn cascade_style(&mut self, parent: &Style, anim: &mut AnimationManager) {
        self.base.inherited_style = parent.clone();
        self.recompute_style();

        let target = if self.checked { 1.0 } else { 0.0 };
        let key = AnimKey {
            widget: self.anim_id,
            layer: AnimLayer::Content,
            property: AnimProperty::Opacity,
        };
        anim.set_target(key, AnimValue([target, 0.0, 0.0, 0.0]), Some(TOGGLE_TRANSITION));
        match anim.value(key) {
            Some(v) => {
                self.progress.set(v.0[0]);
                self.base.dirty = true;
            }
            None => self.progress.set(target),
        }

        // Thumb size target depends on both checked progress and pressed
        // state, so "pressed while off" and "pressed while on" grow to
        // different sizes instead of sharing one fixed pressed diameter.
        let t = self.progress.get();
        let pressed = self.base.interaction.pressed;
        let idle_thumb = lerp(THUMB_UNSELECTED, THUMB_SELECTED, t);
        let pressed_thumb = lerp(THUMB_PRESSED_UNCHECKED, THUMB_PRESSED_CHECKED, t);
        let thumb_target = if pressed { pressed_thumb } else { idle_thumb };

        let thumb_key = AnimKey {
            widget: self.anim_id,
            layer: AnimLayer::Root,
            property: AnimProperty::ContentScale,
        };
        anim.set_target(
            thumb_key,
            AnimValue([thumb_target, 0.0, 0.0, 0.0]),
            Some(THUMB_SIZE_TRANSITION)
        );
        match anim.value(thumb_key) {
            Some(v) => {
                self.thumb_size.set(v.0[0]);
                self.base.dirty = true;
            }
            None => self.thumb_size.set(thumb_target),
        }
    }

    fn after_interaction_transfer(&mut self) {
        self.recompute_style();
    }

    fn transfer_measured_state(&mut self, old: &dyn Widget) {
        if let Some(old) = old.as_any().downcast_ref::<Switch>() {
            self.progress.set(old.progress.get());
            self.thumb_size.set(old.thumb_size.get());
        }
    }

    fn transfer_interaction_state(&mut self, old: &dyn Widget) {
        if let (Some(new), Some(old_i)) = (self.interaction_mut(), old.interaction()) {
            new.transfer_from(old_i);
        }
        if let Some(old) = old.as_any().downcast_ref::<Switch>() {
            self.anim_id = old.anim_id;
        }
    }

    fn anim_id(&self) -> WidgetId {
        self.anim_id
    }
}
