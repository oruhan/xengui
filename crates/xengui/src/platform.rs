// SPDX-License-Identifier: Apache-2.0
//! Runtime touch-vs-pointer platform detection, kept separate from the
//! compile target: native iOS/Android are always touch, but wasm32 runs
//! on both desktop and mobile browsers, so that case can't be decided at
//! compile time and is instead set once at startup by the host platform
//! layer (see `xenframe::web::detect_touch_platform`).
use std::cell::Cell;

thread_local! {
    static IS_TOUCH: Cell<bool> = const { Cell::new(false) };
    static SAFE_AREA_INSETS: Cell<SafeAreaInsets> = const { Cell::new(SafeAreaInsets::ZERO) };
}

/// Logical-pixel insets occupied by platform system UI or display cutouts.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SafeAreaInsets {
    /// Inset from the top edge.
    pub top: f32,
    /// Inset from the right edge.
    pub right: f32,
    /// Inset from the bottom edge.
    pub bottom: f32,
    /// Inset from the left edge.
    pub left: f32,
}

impl SafeAreaInsets {
    /// No platform inset on any edge.
    pub const ZERO: Self = Self {
        top: 0.0,
        right: 0.0,
        bottom: 0.0,
        left: 0.0,
    };
}

/// Records whether the current device is touch-primary. No-op on native
/// iOS/Android, where `is_touch_platform` already returns `true`
/// unconditionally; meant to be called once, on startup, by a wasm32 host.
pub fn set_is_touch_platform(value: bool) {
    IS_TOUCH.with(|cell| cell.set(value));
}

/// Returns whether the `is_touch_platform` condition is satisfied.
pub fn is_touch_platform() -> bool {
    if cfg!(any(target_os = "ios", target_os = "android")) {
        return true;
    }
    if cfg!(target_arch = "wasm32") {
        return IS_TOUCH.with(Cell::get);
    }
    false
}

/// Returns the latest safe-area insets supplied by the platform host.
pub fn safe_area_insets() -> SafeAreaInsets {
    SAFE_AREA_INSETS.with(Cell::get)
}

/// Updates platform safe-area insets and reports whether they changed.
pub fn set_safe_area_insets(insets: SafeAreaInsets) -> bool {
    SAFE_AREA_INSETS.with(|current| {
        if current.get() == insets {
            false
        } else {
            current.set(insets);
            true
        }
    })
}
