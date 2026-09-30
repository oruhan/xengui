// SPDX-License-Identifier: Apache-2.0
//! Horizontal sliders with flat and expressive wavy progress tracks.

use crate::{
    Background, BorderRadius, Color, Constraints, ElementState, EventCtx, EventStatus, InputEvent,
    Interaction, Key, KeyState, LayoutBox, Length, MeasureContext, MeasureResult, MouseButton,
    PaintContext, RectCommand, Style, StyleBuilder, Widget, WidgetBase,
    constants::{DEFAULT_CURSOR_ICON, DEFAULT_POINTER_CURSOR_ICON},
    input::TouchPanPhase,
    widgets::progress_bar::draw_wave,
};
use std::cell::Cell;
use web_time::Instant;

type ChangeCallback = Box<dyn FnMut(f32, &mut EventCtx)>;

const TRACK_HEIGHT: f32 = 4.0;
const WAVE_AMPLITUDE: f32 = 3.0;
const WAVE_LENGTH: f32 = 40.0;
const HANDLE_HEIGHT: f32 = 12.0;
const HANDLE_WIDTH: f32 = 4.0;
const PRESSED_HANDLE_WIDTH: f32 = 2.0;
const TRACK_HANDLE_GAP: f32 = 4.0;
const STOP_SIZE: f32 = 4.0;
const FAST_SPATIAL_DAMPING: f32 = 0.6;
const FAST_SPATIAL_STIFFNESS: f32 = 800.0;
// My design decision (not specified by M3): one wavelength per 1.8 seconds.
const PLAYBACK_WAVE_RADIANS_PER_SECOND: f32 = std::f32::consts::TAU / 1.8;

#[cfg(any(target_os = "android", target_os = "ios"))]
const INTERACTION_TARGET_SIZE: f32 = 48.0;
#[cfg(not(any(target_os = "android", target_os = "ios")))]
const INTERACTION_TARGET_SIZE: f32 = 44.0;

/// Shape of the slider's progress-like active track.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SliderTrackShape {
    /// A straight rounded track.
    #[default]
    Flat,
    /// A wavy active track that becomes flat while the handle is pressed.
    Wavy,
}

/// A horizontal slider that also presents its value as a progress indicator.
///
/// The active track can be flat or wavy. The wavy track temporarily morphs
/// to flat while dragged, then returns to its wave when released.
pub struct Slider {
    base: WidgetBase,
    layout_box: LayoutBox,
    value: f32,
    authored_value: f32,
    track_color: Option<Color>,
    fill_color: Option<Color>,
    thumb_color: Option<Color>,
    track_shape: SliderTrackShape,
    dragging: Cell<bool>,
    wave_progress: Cell<f32>,
    wave_velocity: Cell<f32>,
    wave_phase: Cell<f32>,
    playback_reported_second: Cell<u32>,
    playback_last_tick: Cell<Option<Instant>>,
    playback_enabled: bool,
    playing: bool,
    playback_duration: f32,
    scale_factor: Cell<f32>,
    on_change: Option<ChangeCallback>,
    on_commit: Option<ChangeCallback>,
    on_playback: Option<ChangeCallback>,
}

impl Slider {
    /// Creates a flat slider with a value of zero.
    pub fn new() -> Self {
        let mut interaction = Interaction::new();
        interaction.focusable = true;
        interaction.hover_cursor = Some(DEFAULT_POINTER_CURSOR_ICON);
        interaction.ripple_overrides.enabled = Some(false);

        let mut slider = Self {
            base: WidgetBase::new(interaction),
            layout_box: LayoutBox::default(),
            value: 0.0,
            authored_value: 0.0,
            track_color: None,
            fill_color: None,
            thumb_color: None,
            track_shape: SliderTrackShape::Flat,
            dragging: Cell::new(false),
            wave_progress: Cell::new(0.0),
            wave_velocity: Cell::new(0.0),
            wave_phase: Cell::new(0.0),
            playback_reported_second: Cell::new(0),
            playback_last_tick: Cell::new(None),
            playback_enabled: false,
            playing: false,
            playback_duration: 1.0,
            scale_factor: Cell::new(1.0),
            on_change: None,
            on_commit: None,
            on_playback: None,
        };
        slider.recompute_style();
        slider
    }

    /// Sets the controlled value in `0.0..=1.0`.
    pub fn value(mut self, value: f32) -> Self {
        self.value = if value.is_finite() {
            value.clamp(0.0, 1.0)
        } else {
            0.0
        };
        self.authored_value = self.value;
        self.mark_dirty();
        self
    }

    /// Overrides the inactive track color.
    pub fn track_color(mut self, color: Color) -> Self {
        self.track_color = Some(color);
        self.mark_dirty();
        self
    }

    /// Overrides the progress-like active track color.
    pub fn fill_color(mut self, color: Color) -> Self {
        self.fill_color = Some(color);
        self.mark_dirty();
        self
    }

    /// Overrides the handle color.
    pub fn thumb_color(mut self, color: Color) -> Self {
        self.thumb_color = Some(color);
        self.mark_dirty();
        self
    }

    /// Selects a flat or expressive wavy active track.
    pub fn track_shape(mut self, shape: SliderTrackShape) -> Self {
        self.track_shape = shape;
        self.wave_progress
            .set((shape == SliderTrackShape::Wavy) as u8 as f32);
        self.mark_dirty();
        self
    }

    /// Enables automatic playback progression over `duration_seconds`.
    ///
    /// Playback pauses while the handle is being dragged. Value updates are
    /// reported through [`Self::on_change`], allowing a controlled slider and
    /// an adjacent elapsed-time label to stay synchronized.
    pub fn playback(mut self, playing: bool, duration_seconds: f32) -> Self {
        self.playback_enabled = true;
        self.playing = playing;
        self.playback_duration = duration_seconds.max(0.001);
        if !playing {
            self.wave_progress.set(0.0);
        }
        self.playback_reported_second
            .set((self.authored_value * self.playback_duration).floor() as u32);
        self
    }

    /// Fires continuously while the track or handle changes the value.
    pub fn on_change(mut self, f: impl FnMut(f32, &mut EventCtx) + 'static) -> Self {
        self.on_change = Some(Box::new(f));
        self
    }

    /// Fires after pointer release or one keyboard adjustment.
    pub fn on_commit(mut self, f: impl FnMut(f32, &mut EventCtx) + 'static) -> Self {
        self.on_commit = Some(Box::new(f));
        self
    }

    /// Fires on whole-second playback boundaries.
    ///
    /// This callback is separate from pointer-driven [`Self::on_change`] so
    /// media seek controls can paint scrubbing locally and commit external
    /// state only once on release, avoiding reconciliation jitter mid-drag.
    pub fn on_playback(mut self, f: impl FnMut(f32, &mut EventCtx) + 'static) -> Self {
        self.on_playback = Some(Box::new(f));
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

    fn usable_geometry(&self, sf: f32) -> (f32, f32) {
        let half_handle = HANDLE_WIDTH * 0.5 * sf;
        (
            half_handle,
            (self.layout_box.width - half_handle * 2.0).max(1.0),
        )
    }

    fn value_at(&self, local_x: f32, sf: f32) -> f32 {
        let (start, width) = self.usable_geometry(sf);
        ((local_x - start) / width).clamp(0.0, 1.0)
    }

    fn set_value_from_event(&mut self, position: (f32, f32), sf: f32, ctx: &mut EventCtx) {
        let next = self.value_at(position.0 - self.layout_box.x, sf);
        self.value = next;
        self.base.dirty = true;
        if let Some(callback) = self.on_change.as_mut() {
            callback(next, ctx);
        }
        ctx.request_redraw();
    }

    fn set_keyboard_value(&mut self, next: f32, ctx: &mut EventCtx) {
        self.value = next.clamp(0.0, 1.0);
        self.base.dirty = true;
        if let Some(callback) = self.on_change.as_mut() {
            callback(self.value, ctx);
        }
        if let Some(callback) = self.on_commit.as_mut() {
            callback(self.value, ctx);
        }
        ctx.request_redraw();
    }

    fn tick_wave_morph(&self, dt: f32) -> bool {
        let target = self.wave_target();
        let mut position = self.wave_progress.get();
        let mut velocity = self.wave_velocity.get();
        if (position - target).abs() < 0.001 && velocity.abs() < 0.001 {
            self.wave_progress.set(target);
            self.wave_velocity.set(0.0);
            return false;
        }

        let dt = dt.clamp(0.0, 1.0 / 30.0);
        let damping = 2.0 * FAST_SPATIAL_DAMPING * FAST_SPATIAL_STIFFNESS.sqrt();
        let acceleration = -FAST_SPATIAL_STIFFNESS * (position - target) - damping * velocity;
        velocity += acceleration * dt;
        position += velocity * dt;
        self.wave_progress.set(position);
        self.wave_velocity.set(velocity);
        true
    }

    fn wave_target(&self) -> f32 {
        if self.track_shape == SliderTrackShape::Wavy
            && !self.dragging.get()
            && (!self.playback_enabled || self.playing)
        {
            1.0
        } else {
            0.0
        }
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
    fn semantics(&self) -> Option<crate::Semantics> {
        let mut semantics = crate::Semantics::new(crate::SemanticRole::Slider)
            .value(self.value.to_string())
            .action(crate::SemanticAction::Focus);
        semantics.label = self.base.accessible_label.as_ref().map(ToString::to_string);
        semantics.disabled = !self.base.interaction.enabled;
        if !semantics.disabled {
            semantics = semantics
                .action(crate::SemanticAction::SetValue)
                .action(crate::SemanticAction::Increment)
                .action(crate::SemanticAction::Decrement);
        }
        Some(semantics)
    }

    crate::impl_widget_boilerplate!();

    fn debug_name(&self) -> &'static str {
        "Widget#Slider"
    }

    fn measure(&self, ctx: &mut MeasureContext, constraints: Constraints) -> MeasureResult {
        let height = INTERACTION_TARGET_SIZE * ctx.scale_factor;
        let width = constraints.max_width.unwrap_or(120.0 * ctx.scale_factor);
        let (width, height) = constraints.constrain_size(width, height);
        MeasureResult::new(width, height)
    }

    fn on_layout_pass(&self, ctx: &mut MeasureContext) {
        self.scale_factor.set(ctx.scale_factor);
    }

    fn paint(&self, ctx: &mut PaintContext) {
        let theme = crate::current_theme();
        let sf = ctx.scale_factor;
        let b = self.layout_box;
        let center_y = b.y + b.height * 0.5;
        let (track_offset, usable_width) = self.usable_geometry(sf);
        let track_start = b.x + track_offset;
        let handle_x = track_start + usable_width * self.value;
        let gap = TRACK_HANDLE_GAP * sf;
        let thickness = TRACK_HEIGHT * sf;
        let track_color = self.track_color.unwrap_or(theme.secondary_container);
        let fill_color = self.fill_color.unwrap_or(theme.primary);
        let thumb_color = self.thumb_color.unwrap_or(fill_color);
        let active_end = (handle_x - gap).max(track_start);
        let inactive_start = (handle_x + gap).min(track_start + usable_width);

        if inactive_start < track_start + usable_width {
            let width = track_start + usable_width - inactive_start;
            ctx.draw_rect(RectCommand {
                position: (inactive_start, center_y - thickness * 0.5),
                size: (width, thickness),
                background: Some(Background::Color(track_color)),
                border_radius: Some(BorderRadius::all(Length::px(thickness * 0.5))),
                border_width: None,
                border_color: None,
                clip_rect: None,
            });
        }

        if active_end > track_start {
            if self.track_shape == SliderTrackShape::Wavy {
                let wave_progress = self.wave_progress.get().clamp(0.0, 1.0);
                draw_wave(
                    ctx,
                    track_start,
                    active_end,
                    center_y,
                    WAVE_AMPLITUDE * sf * wave_progress,
                    WAVE_LENGTH * sf,
                    self.wave_phase.get(),
                    thickness,
                    fill_color,
                );
            } else {
                ctx.draw_rect(RectCommand {
                    position: (track_start, center_y - thickness * 0.5),
                    size: (active_end - track_start, thickness),
                    background: Some(Background::Color(fill_color)),
                    border_radius: Some(BorderRadius::all(Length::px(thickness * 0.5))),
                    border_width: None,
                    border_color: None,
                    clip_rect: None,
                });
            }
        }

        let stop = STOP_SIZE * sf;
        ctx.draw_rect(RectCommand {
            position: (track_start + usable_width - stop, center_y - stop * 0.5),
            size: (stop, stop),
            background: Some(Background::Color(fill_color)),
            border_radius: Some(BorderRadius::all(Length::px(stop * 0.5))),
            border_width: None,
            border_color: None,
            clip_rect: None,
        });

        let handle_width = if self.dragging.get() {
            PRESSED_HANDLE_WIDTH
        } else {
            HANDLE_WIDTH
        } * sf;
        let handle_height = HANDLE_HEIGHT * sf;
        ctx.draw_rect(RectCommand {
            position: (
                handle_x - handle_width * 0.5,
                center_y - handle_height * 0.5,
            ),
            size: (handle_width, handle_height),
            background: Some(Background::Color(thumb_color)),
            border_radius: Some(BorderRadius::all(Length::px(handle_width * 0.5))),
            border_width: None,
            border_color: None,
            clip_rect: None,
        });

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
                self.base.dirty = true;
                ctx.request_redraw();
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
            InputEvent::TouchPan {
                phase: TouchPanPhase::Start,
                ..
            } if self.dragging.get() => {
                // Claim the gesture before a scrollable ancestor can. Later
                // touch-pan events are routed straight back to this slider.
                ctx.suppress_text_drag();
                return EventStatus::Handled;
            }
            InputEvent::TouchPan {
                phase: TouchPanPhase::Move,
                position,
            } => {
                // The host cancels mouse-shaped activation once touch movement
                // crosses its pan threshold. A slider owns that pan, so keep
                // scrubbing instead of handing it to the page scroller.
                self.dragging.set(true);
                self.set_value_from_event(*position, sf, ctx);
                return EventStatus::Handled;
            }
            InputEvent::TouchPan {
                phase: TouchPanPhase::End,
                position,
            } if self.dragging.get() => {
                self.dragging.set(false);
                self.set_value_from_event(*position, sf, ctx);
                if let Some(callback) = self.on_commit.as_mut() {
                    callback(self.value, ctx);
                }
                self.base.dirty = true;
                ctx.request_redraw();
                return EventStatus::Handled;
            }
            InputEvent::TouchPan {
                phase: TouchPanPhase::Cancel,
                ..
            } => {
                self.dragging.set(false);
                self.base.dirty = true;
                ctx.request_redraw();
                return EventStatus::Handled;
            }
            InputEvent::AnimationTick { dt } => {
                let morphing = self.tick_wave_morph(*dt);
                if morphing {
                    self.base.dirty = true;
                }
                if self.playing && !self.dragging.get() && self.value < 1.0 {
                    let now = Instant::now();
                    let playback_dt = self
                        .playback_last_tick
                        .replace(Some(now))
                        .map_or(0.0, |last| now.duration_since(last).as_secs_f32());
                    let next = (self.value + playback_dt / self.playback_duration).min(1.0);
                    self.value = next;
                    self.wave_phase.set(
                        (self.wave_phase.get() + playback_dt * PLAYBACK_WAVE_RADIANS_PER_SECOND)
                            .rem_euclid(std::f32::consts::TAU),
                    );
                    let elapsed_second = (next * self.playback_duration).floor() as u32;
                    if elapsed_second != self.playback_reported_second.get() || next >= 1.0 {
                        self.playback_reported_second.set(elapsed_second);
                        if let Some(callback) = self.on_playback.as_mut() {
                            callback(next, ctx);
                        } else if let Some(callback) = self.on_change.as_mut() {
                            callback(next, ctx);
                        }
                    }
                    self.base.dirty = true;
                } else {
                    self.playback_last_tick.set(None);
                }
                if morphing || (self.playing && self.value < 1.0) {
                    ctx.request_redraw();
                }
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
                if let Some(callback) = self.on_commit.as_mut() {
                    callback(self.value, ctx);
                }
                self.base.dirty = true;
                ctx.request_redraw();
                return EventStatus::Handled;
            }
            InputEvent::KeyInput {
                event: key_event, ..
            } if self.base.interaction.focused && key_event.state == KeyState::Pressed => {
                let next = match key_event.key {
                    Key::ArrowLeft | Key::ArrowDown => Some(self.value - 0.02),
                    Key::ArrowRight | Key::ArrowUp => Some(self.value + 0.02),
                    Key::Home => Some(0.0),
                    Key::End => Some(1.0),
                    _ => None,
                };
                if let Some(next) = next {
                    self.set_keyboard_value(next, ctx);
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
        let Some(other) = other.as_any().downcast_ref::<Self>() else {
            return false;
        };
        self.value == other.value
            && self.track_color == other.track_color
            && self.fill_color == other.fill_color
            && self.thumb_color == other.thumb_color
            && self.track_shape == other.track_shape
            && self.playback_enabled == other.playback_enabled
            && self.playing == other.playing
            && self.playback_duration == other.playback_duration
            && self.base.authored_styles_eq(&other.base)
    }

    fn cascade_style(&mut self, parent: &Style, _anim: &mut crate::AnimationManager) {
        self.base.inherited_style = parent.clone();
        self.recompute_style();
    }

    fn transfer_interaction_state(&mut self, old: &dyn Widget) {
        if let (Some(new), Some(old_interaction)) = (self.interaction_mut(), old.interaction()) {
            new.transfer_from(old_interaction);
        }
        if let Some(old) = old.as_any().downcast_ref::<Self>() {
            let external_value_unchanged =
                (self.authored_value - old.authored_value).abs() < f32::EPSILON;
            // While captured, the widget's pointer-derived value is the
            // freshest source of truth. A controlled parent can be one
            // reconciliation behind at high pointer rates; retaining the
            // live value prevents one-frame jumps (often all the way to an
            // endpoint) until release commits the final external value.
            if old.dragging.get() || external_value_unchanged {
                self.value = old.value;
                self.playback_reported_second
                    .set(old.playback_reported_second.get());
            }
            self.dragging.set(old.dragging.get());
            self.wave_progress.set(old.wave_progress.get());
            self.wave_velocity.set(old.wave_velocity.get());
            self.wave_phase.set(old.wave_phase.get());
            self.playback_last_tick.set(old.playback_last_tick.get());
            self.scale_factor.set(old.scale_factor.get());
        }
    }

    fn wants_animation_frame(&self) -> bool {
        let target = self.wave_target();
        self.playing
            || (self.wave_progress.get() - target).abs() >= 0.001
            || self.wave_velocity.get().abs() >= 0.001
    }
}

#[cfg(test)]
mod tests {
    use super::{Slider, SliderTrackShape};
    use crate::{
        ElementState, EventCtx, EventStatus, InputEvent, LayoutBox, MouseButton, TouchPanPhase,
        Widget,
    };

    #[test]
    fn clamps_non_finite_and_out_of_range_values() {
        assert_eq!(Slider::new().value(-1.0).value, 0.0);
        assert_eq!(Slider::new().value(2.0).value, 1.0);
        assert_eq!(Slider::new().value(f32::NAN).value, 0.0);
    }

    #[test]
    fn exposes_both_track_shapes() {
        assert_eq!(
            Slider::new()
                .track_shape(SliderTrackShape::Wavy)
                .track_shape,
            SliderTrackShape::Wavy
        );
    }

    #[test]
    fn playback_pause_flattens_only_playback_waves() {
        let normal = Slider::new().track_shape(SliderTrackShape::Wavy);
        let paused = Slider::new()
            .track_shape(SliderTrackShape::Wavy)
            .playback(false, 120.0);
        let playing = Slider::new()
            .track_shape(SliderTrackShape::Wavy)
            .playback(true, 120.0);

        assert_eq!(normal.wave_target(), 1.0);
        assert_eq!(paused.wave_target(), 0.0);
        assert_eq!(playing.wave_target(), 1.0);
    }

    #[test]
    fn touch_scrubbing_claims_pan_instead_of_scrolling_an_ancestor() {
        let mut slider = Slider::new();
        slider.layout(LayoutBox {
            x: 0.0,
            y: 0.0,
            width: 200.0,
            height: 48.0,
        });
        let mut ctx = EventCtx::new();

        assert_eq!(
            slider.event(
                &InputEvent::MouseInput {
                    state: ElementState::Pressed,
                    button: MouseButton::Left,
                    position: (20.0, 24.0),
                },
                &mut ctx,
            ),
            EventStatus::Handled
        );
        assert_eq!(
            slider.event(
                &InputEvent::TouchPan {
                    phase: TouchPanPhase::Start,
                    position: (20.0, 24.0),
                },
                &mut ctx,
            ),
            EventStatus::Handled
        );
        assert_eq!(
            slider.event(
                &InputEvent::TouchPan {
                    phase: TouchPanPhase::Move,
                    position: (160.0, 24.0),
                },
                &mut ctx,
            ),
            EventStatus::Handled
        );
        assert!(slider.value > 0.7);

        slider.event(
            &InputEvent::TouchPan {
                phase: TouchPanPhase::End,
                position: (160.0, 24.0),
            },
            &mut ctx,
        );
        assert!(!slider.dragging.get());
    }
}
