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

/// Runtime state for one bounded ripple.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct RippleState {
    origin: (f32, f32),
    elapsed: f32,
    released_at: Option<f32>,
    active: bool,
}

impl RippleState {
    pub(crate) fn is_active(self) -> bool {
        self.active
    }

    fn start(&mut self, origin: (f32, f32)) {
        *self = Self {
            origin,
            elapsed: 0.0,
            released_at: None,
            active: true,
        };
    }

    fn release(&mut self) {
        if self.active && self.released_at.is_none() {
            self.released_at = Some(self.elapsed);
        }
    }

    fn cancel(&mut self) {
        if self.active {
            self.released_at = Some(self.elapsed);
        }
    }

    fn tick(&mut self, dt: f32, duration_scale: f32) {
        if !self.active {
            return;
        }
        self.elapsed += dt.max(0.0);
        if let Some(released_at) = self.released_at {
            let fade_start = released_at.max(ENTER_DURATION * duration_scale.max(0.05));
            if self.elapsed - fade_start >= EXIT_DURATION * duration_scale.max(0.05) {
                self.active = false;
            }
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
    event: &InputEvent,
    center: (f32, f32),
) -> bool {
    if !configured(overrides) {
        let changed = state.active;
        state.active = false;
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
        } => state.release(),
        InputEvent::KeyInput { event, .. }
            if matches!(event.key, Key::Enter | Key::Space)
                && event.state == KeyState::Pressed
                && !event.repeat =>
        {
            state.start(center)
        }
        InputEvent::KeyInput { event, .. }
            if matches!(event.key, Key::Enter | Key::Space)
                && event.state == KeyState::Released =>
        {
            state.release()
        }
        InputEvent::PointerCancel | InputEvent::FocusLost => state.cancel(),
        InputEvent::AnimationTick { dt } => state.tick(*dt, duration_scale),
        _ => return false,
    }
    true
}

pub(crate) fn paint(
    state: RippleState,
    overrides: RippleOverrides,
    style: &Style,
    layout: LayoutBox,
    ctx: &mut PaintContext<'_>,
) {
    if !state.active || !configured(overrides) || layout.width <= 0.0 || layout.height <= 0.0 {
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
    let radius = style
        .border
        .as_ref()
        .and_then(|border| border.radius)
        .map(|radius| radius.to_physical_array(ctx.scale_factor, layout.width, layout.height))
        .unwrap_or([0.0; 4]);

    ctx.draw_ripple(RippleCommand {
        bounds: (layout.x, layout.y, layout.width, layout.height),
        origin: state.origin,
        progress: state.progress(duration_scale),
        opacity: state.opacity(duration_scale),
        noise_phase: (state.elapsed / NOISE_DURATION).fract(),
        color: base.with_alpha_f32(alpha),
        radius,
        clip_rect: Some((layout.x, layout.y, layout.width, layout.height)),
    });
}

#[cfg(test)]
mod tests {
    use super::*;

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
        ripple.release();
        ripple.tick(EXIT_DURATION + 0.001, 1.0);
        assert!(!ripple.is_active());
    }

    #[test]
    fn quick_tap_still_expands_fully_before_fading() {
        let mut ripple = RippleState::default();
        ripple.start((2.0, 3.0));
        ripple.release();
        ripple.tick(ENTER_DURATION, 1.0);
        assert!(ripple.is_active());
        assert_eq!(ripple.progress(1.0), 1.0);
        assert_eq!(ripple.opacity(1.0), 1.0);
    }
}
