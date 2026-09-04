// SPDX-License-Identifier: Apache-2.0
//! Runtime touch-vs-pointer platform detection, kept separate from the
//! compile target: native iOS/Android are always touch, but wasm32 runs
//! on both desktop and mobile browsers, so that case can't be decided at
//! compile time and is instead set once at startup by the host platform
//! layer (see `xenframe::web::detect_touch_platform`).
use std::cell::Cell;

thread_local! {
    static IS_TOUCH: Cell<bool> = const { Cell::new(false) };
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
