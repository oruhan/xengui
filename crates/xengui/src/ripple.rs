// SPDX-License-Identifier: Apache-2.0
//! Material pressed-state ripple configuration and animation state.

use std::cell::Cell;

use crate::{
    Color, ElementState, InputEvent, Key, KeyState, LayoutBox, MouseButton, PaintContext,
    RippleCommand, Style,
};

/// Platforms on which the default Material ripple is enabled.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RipplePlatforms {
    /// Enable ripples on Android.
    pub android: bool,
    /// Enable ripples on Linux.
    pub linux: bool,
    /// Enable ripples on Windows.
    pub windows: bool,
    /// Enable ripples on macOS.
    pub macos: bool,
    /// Enable ripples on iOS.
    pub ios: bool,
    /// Enable ripples in WebAssembly builds.
    pub wasm: bool,
}

impl RipplePlatforms {
    /// Enables every supported target.
    pub const ALL: Self = Self {
        android: true,
        linux: true,
        windows: true,
        macos: true,
        ios: true,
        wasm: true,
    };

    /// Enables native targets and leaves WebAssembly disabled.
    pub const NATIVE: Self = Self {
        wasm: false,
        ..Self::ALL
    };

    /// Disables every target.
    pub const NONE: Self = Self {
        android: false,
        linux: false,
        windows: false,
        macos: false,
        ios: false,
        wasm: false,
    };

    /// Returns whether the platform being compiled is enabled.
    pub const fn current_enabled(self) -> bool {
        if cfg!(target_arch = "wasm32") {
            self.wasm
        } else if cfg!(target_os = "android") {
            self.android
        } else if cfg!(target_os = "linux") {
            self.linux
        } else if cfg!(target_os = "windows") {
            self.windows
        } else if cfg!(target_os = "macos") {
            self.macos
        } else if cfg!(target_os = "ios") {
            self.ios
        } else {
            false
        }
    }
}

impl Default for RipplePlatforms {
    fn default() -> Self {
        // XenGui initially opts in only where the native implementation is
        // shipped and tested. Web stays opt-in as requested.
        Self {
            android: true,
            linux: true,
            ..Self::NONE
        }
    }
}

/// Application-wide Material ripple policy.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RippleConfig {
    /// Master switch for every platform.
    pub enabled: bool,
    /// Per-platform enablement.
    pub platforms: RipplePlatforms,
    /// Multiplier applied to M3's 10% pressed-state opacity.
    pub strength: f32,
    /// Multiplier for the Android patterned-ripple animation durations.
    /// Values above `1.0` make the effect calmer and longer.
    pub duration_scale: f32,
}

impl Default for RippleConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            platforms: RipplePlatforms::default(),
            strength: 1.0,
            duration_scale: 1.0,
        }
    }
}

thread_local! {
    static CONFIG: Cell<RippleConfig> = const { Cell::new(RippleConfig {
        enabled: true,
        platforms: RipplePlatforms {
            android: true,
            linux: true,
            windows: false,
            macos: false,
            ios: false,
            wasm: false,
        },
        strength: 1.0,
        duration_scale: 1.0,
    }) };
}

/// Replaces the application-wide ripple policy.
pub fn set_ripple_config(config: RippleConfig) {
    CONFIG.with(|slot| slot.set(config));
}

/// Returns the current application-wide ripple policy.
pub fn ripple_config() -> RippleConfig {
    CONFIG.with(Cell::get)
}

// Android's patterned ripple session timings. These are intentionally much
// longer than the legacy 225/150 ms solid-circle implementation.
const ENTER_DURATION: f32 = 0.450;
const EXIT_DURATION: f32 = 0.375;
const NOISE_DURATION: f32 = 7.0;
// Bounds pathological synthetic input without imposing a practical limit on
// human clicking. Storage stays lazy, so untouched widgets pay no heap cost.
const MAX_CONCURRENT_RIPPLES: usize = 32;

/// Runtime state for one wave within a bounded ripple.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct RippleWave {
    origin: (f32, f32),
    elapsed: f32,
    released_at: Option<f32>,
}

impl RippleWave {
    fn start(&mut self, origin: (f32, f32)) {
        *self = Self {
            origin,
            elapsed: 0.0,
            released_at: None,
        };
    }

    fn release(&mut self) {
        if self.released_at.is_none() {
            self.released_at = Some(self.elapsed);
        }
    }

    fn tick(&mut self, dt: f32) {
        self.elapsed += dt.max(0.0);
    }

    fn is_finished(self, duration_scale: f32) -> bool {
        if let Some(released_at) = self.released_at {
            let fade_start = released_at.max(ENTER_DURATION * duration_scale.max(0.05));
            self.elapsed - fade_start >= EXIT_DURATION * duration_scale.max(0.05)
        } else {
            false
        }
    }

    fn press_progress(self, duration_scale: f32) -> f32 {
        (self.elapsed / (ENTER_DURATION * duration_scale.max(0.05))).clamp(0.0, 1.0)
    }

    fn progress(self, duration_scale: f32) -> f32 {
        self.press_progress(duration_scale)
    }

    fn opacity(self, duration_scale: f32) -> f32 {
        let Some(released_at) = self.released_at else {
            return 1.0;
        };
        let scale = duration_scale.max(0.05);
        let fade_start = released_at.max(ENTER_DURATION * scale);
        (1.0 - (self.elapsed - fade_start).max(0.0) / (EXIT_DURATION * scale)).clamp(0.0, 1.0)
    }
}

/// Runtime state for all overlapping waves in one bounded ripple.
///
/// The vector allocates only after the first interaction and reuses its
/// capacity across subsequent clicks. Each wave remains a compact value; no
/// CPU-side particles or per-frame geometry are generated.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct RippleState {
    waves: Vec<RippleWave>,
}

impl RippleState {
    pub(crate) fn is_active(&self) -> bool {
        !self.waves.is_empty()
    }

    fn start(&mut self, origin: (f32, f32)) {
        if self.waves.len() == MAX_CONCURRENT_RIPPLES {
            // The oldest wave is closest to completion and least noticeable.
            self.waves.remove(0);
        }
        let mut wave = RippleWave::default();
        wave.start(origin);
        self.waves.push(wave);
    }

    fn release_latest(&mut self) {
        if let Some(wave) = self
            .waves
            .iter_mut()
            .rev()
            .find(|wave| wave.released_at.is_none())
        {
            wave.release();
        }
    }

    fn cancel_unreleased(&mut self) {
        for wave in &mut self.waves {
            if wave.released_at.is_none() {
                wave.release();
            }
        }
    }

    fn tick(&mut self, dt: f32, duration_scale: f32) {
        for wave in &mut self.waves {
            wave.tick(dt);
        }
        self.waves.retain(|wave| !wave.is_finished(duration_scale));
    }

    fn clear(&mut self) {
        self.waves.clear();
    }
}

/// Per-widget ripple overrides stored alongside interaction state.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct RippleOverrides {
    pub(crate) enabled: Option<bool>,
    pub(crate) strength: Option<f32>,
    pub(crate) color: Option<Color>,
    pub(crate) duration_scale: Option<f32>,
}

pub(crate) fn configured(overrides: RippleOverrides) -> bool {
    let config = ripple_config();
    config.enabled
        && overrides
            .enabled
            .unwrap_or_else(|| config.platforms.current_enabled())
}

pub(crate) fn handle_event(
    state: &mut RippleState,
    overrides: RippleOverrides,
    keyboard_activation: bool,
    event: &InputEvent,
    center: (f32, f32),
) -> bool {
    if !configured(overrides) {
        let changed = state.is_active();
        state.clear();
        return changed;
    }

    let duration_scale = overrides
        .duration_scale
        .unwrap_or_else(|| ripple_config().duration_scale)
        .max(0.05);

    match event {
        InputEvent::MouseInput {
            state: ElementState::Pressed,
            button: MouseButton::Left,
            position,
        } => state.start(*position),
        InputEvent::MouseInput {
            state: ElementState::Released,
            button: MouseButton::Left,
            ..
        } => state.release_latest(),
        InputEvent::KeyInput { event, .. }
            if keyboard_activation
                && matches!(event.key, Key::Enter | Key::Space)
                && event.state == KeyState::Pressed
                && !event.repeat =>
        {
            state.start(center)
        }
        InputEvent::KeyInput { event, .. }
            if keyboard_activation
                && matches!(event.key, Key::Enter | Key::Space)
                && event.state == KeyState::Released =>
        {
            state.release_latest()
        }
        // Leaving an actionable target ends its pressed state immediately,
        // but the visual feedback completes its normal enter/fade lifecycle.
        InputEvent::MouseExited => state.release_latest(),
        InputEvent::PointerCancel | InputEvent::FocusLost => state.cancel_unreleased(),
        InputEvent::AnimationTick { dt } => state.tick(*dt, duration_scale),
        _ => return false,
    }
    true
}

pub(crate) fn paint(
    state: &RippleState,
    overrides: RippleOverrides,
    style: &Style,
    layout: LayoutBox,
    radius: [f32; 4],
    ctx: &mut PaintContext<'_>,
) {
    if !state.is_active() || !configured(overrides) || layout.width <= 0.0 || layout.height <= 0.0 {
        return;
    }

    let config = ripple_config();
    let strength = overrides.strength.unwrap_or(config.strength).max(0.0);
    let base = overrides
        .color
        .or(style.color)
        .unwrap_or_else(|| crate::current_theme().on_surface);
    let alpha = (base.a() * 0.10 * strength).clamp(0.0, 1.0);
    if alpha <= 0.0 {
        return;
    }

    let duration_scale = overrides
        .duration_scale
        .unwrap_or(config.duration_scale)
        .max(0.05);
    for wave in &state.waves {
        ctx.draw_ripple(RippleCommand {
            bounds: (layout.x, layout.y, layout.width, layout.height),
            origin: wave.origin,
            progress: wave.progress(duration_scale),
            opacity: wave.opacity(duration_scale),
            noise_phase: (wave.elapsed / NOISE_DURATION).fract(),
            color: base.with_alpha_f32(alpha),
            radius,
            clip_rect: Some((layout.x, layout.y, layout.width, layout.height)),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Checkbox, RadioButton, Switch, Widget};

    #[test]
    fn wasm_is_opt_in_and_linux_android_are_defaults() {
        let defaults = RipplePlatforms::default();
        assert!(defaults.android);
        assert!(defaults.linux);
        assert!(!defaults.wasm);
    }

    #[test]
    fn released_ripple_finishes_after_fade() {
        let mut ripple = RippleState::default();
        ripple.start((2.0, 3.0));
        ripple.tick(ENTER_DURATION, 1.0);
        ripple.release_latest();
        ripple.tick(EXIT_DURATION + 0.001, 1.0);
        assert!(!ripple.is_active());
    }

    #[test]
    fn quick_tap_still_expands_fully_before_fading() {
        let mut ripple = RippleState::default();
        ripple.start((2.0, 3.0));
        ripple.release_latest();
        ripple.tick(ENTER_DURATION, 1.0);
        assert!(ripple.is_active());
        assert_eq!(ripple.waves[0].progress(1.0), 1.0);
        assert_eq!(ripple.waves[0].opacity(1.0), 1.0);
    }

    #[test]
    fn repeated_clicks_keep_previous_waves_alive() {
        let mut ripple = RippleState::default();
        ripple.start((2.0, 3.0));
        ripple.release_latest();
        ripple.tick(0.1, 1.0);
        ripple.start((8.0, 9.0));

        assert_eq!(ripple.waves.len(), 2);
        assert_eq!(ripple.waves[0].origin, (2.0, 3.0));
        assert_eq!(ripple.waves[1].origin, (8.0, 9.0));
        assert!(ripple.waves[0].elapsed > ripple.waves[1].elapsed);
    }

    #[test]
    fn mouse_exit_releases_without_cutting_off_the_wave() {
        let mut ripple = RippleState::default();
        ripple.start((2.0, 3.0));

        assert!(handle_event(
            &mut ripple,
            RippleOverrides {
                enabled: Some(true),
                ..RippleOverrides::default()
            },
            true,
            &InputEvent::MouseExited,
            (0.0, 0.0),
        ));
        assert!(ripple.is_active());
        assert!(ripple.waves[0].released_at.is_some());

        ripple.tick(ENTER_DURATION + EXIT_DURATION + 0.001, 1.0);
        assert!(!ripple.is_active());
    }

    #[test]
    fn text_input_can_disable_keyboard_activation_ripples() {
        let mut ripple = RippleState::default();
        let event = InputEvent::KeyInput {
            event: crate::KeyboardEvent {
                key: Key::Space,
                state: KeyState::Pressed,
                repeat: false,
            },
            modifiers: crate::ModifiersState::default(),
        };

        assert!(!handle_event(
            &mut ripple,
            RippleOverrides {
                enabled: Some(true),
                ..RippleOverrides::default()
            },
            false,
            &event,
            (0.0, 0.0),
        ));
        assert!(!ripple.is_active());
    }

    #[test]
    fn wave_storage_is_lazy_and_reused() {
        let mut ripple = RippleState::default();
        assert_eq!(ripple.waves.capacity(), 0);

        ripple.start((2.0, 3.0));
        let allocated_capacity = ripple.waves.capacity();
        ripple.release_latest();
        ripple.tick(ENTER_DURATION + EXIT_DURATION + 0.001, 1.0);
        assert!(!ripple.is_active());

        ripple.start((8.0, 9.0));
        assert_eq!(ripple.waves.capacity(), allocated_capacity);
    }

    #[test]
    fn synthetic_input_is_bounded_to_a_small_active_set() {
        let mut ripple = RippleState::default();
        for index in 0..MAX_CONCURRENT_RIPPLES + 5 {
            ripple.start((index as f32, 0.0));
            ripple.release_latest();
        }

        assert_eq!(ripple.waves.len(), MAX_CONCURRENT_RIPPLES);
        assert_eq!(ripple.waves[0].origin.0, 5.0);
    }

    #[test]
    fn custom_controls_expose_their_painted_shape_to_ripple_clipping() {
        let checkbox_bounds = LayoutBox {
            width: 18.0,
            height: 18.0,
            ..LayoutBox::default()
        };
        assert_eq!(
            Checkbox::new().ripple_radius(1.0, checkbox_bounds),
            [4.0; 4]
        );

        let switch_bounds = LayoutBox {
            width: 52.0,
            height: 32.0,
            ..LayoutBox::default()
        };
        assert_eq!(Switch::new().ripple_radius(1.0, switch_bounds), [16.0; 4]);

        let radio_bounds = LayoutBox {
            width: 20.0,
            height: 20.0,
            ..LayoutBox::default()
        };
        assert_eq!(
            RadioButton::new().ripple_radius(1.0, radio_bounds),
            [10.0; 4]
        );
    }
}
