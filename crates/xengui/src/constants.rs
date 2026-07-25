// SPDX-License-Identifier: Apache-2.0
use std::time::Duration;

use xen_animation::{ Easing, Transition };

/// Maximum distance (in logical/DP units) between two consecutive taps or
/// clicks for them to count as part of the same multi-click sequence
/// (double-click word select, triple-click line select, etc).
pub const MULTI_CLICK_DISTANCE_DP: f32 = 18.0;

/// Maximum finger movement (in logical/DP units) allowed during a
/// long-press before it's cancelled.
pub const TOUCH_LONG_PRESS_MOVE_TOLERANCE_DP: f32 = 8.0;

/// Maximum amount of time a touch must remain pressed before it is
/// recognized as a long-press gesture.
pub const TOUCH_LONG_PRESS_DURATION: Duration = Duration::from_millis(350);

/// A second click within this time window counts as a double/triple click.
pub const MULTI_CLICK_INTERVAL: Duration = Duration::from_millis(400);

/* ---- Scrollbar ---- */
// Eased transition applied to scroll position when animating toward a
// wheel/nudge target; drag updates bypass this and snap instantly.
pub const SCROLL_TRANSITION: Transition = Transition::new(
    std::time::Duration::from_millis(250)
).easing(Easing::EaseOut);
pub const SCROLLBAR_THICKNESS_TRANSITION: Transition = Transition::new(
    std::time::Duration::from_millis(160)
).easing(Easing::EaseOut);

/// Opacity applied to a `Scroll`-mode scrollbar axis that has nothing to
/// scroll, so it stays visible but reads as disabled instead of vanishing.
pub const SCROLLBAR_DISABLED_OPACITY: f32 = 0.35;

/// Padding (px) trimmed from each side of the scrollbar thumb's
/// cross-axis thickness, so it renders thinner than its track.
pub const SCROLLBAR_THUMB_PADDING: f32 = 2.0;

pub const SCROLLBAR_ARROW_SIZE: f32 = 4.0;
pub const SCROLLBAR_ARROW_THICKNESS: f32 = 1.6;
pub const SCROLLBAR_ARROW_CAP_SEGMENTS: usize = 8;
