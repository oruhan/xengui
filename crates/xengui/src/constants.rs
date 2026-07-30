// SPDX-License-Identifier: Apache-2.0
use web_time::Duration;

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
/// Scrollbar thickness while neither hovered nor pressed.
pub const DEFAULT_SCROLLBAR_THICKNESS: f32 = 15.0;
/// Scrollbar thickness while hovered or pressed, unless overridden.
pub const DEFAULT_SCROLLBAR_HOVER_THICKNESS: f32 = 15.0;

// Eased transition applied to scroll position when animating toward a
// wheel/nudge target; drag updates bypass this and snap instantly.
pub const SCROLL_TRANSITION: Transition = Transition::new(
    web_time::Duration::from_millis(250)
).easing(Easing::EaseOut);
pub const SCROLLBAR_THICKNESS_TRANSITION: Transition = Transition::new(
    web_time::Duration::from_millis(160)
).easing(Easing::EaseOut);

/// Opacity applied to a `Scroll`-mode scrollbar axis that has nothing to
/// scroll, so it stays visible but reads as disabled instead of vanishing.
pub const SCROLLBAR_DISABLED_OPACITY: f32 = 0.35;

/// Padding (px) trimmed from each side of the scrollbar thumb's
/// cross-axis thickness, so it renders thinner than its track.
pub const SCROLLBAR_THUMB_PADDING: f32 = 4.0;

pub const SCROLLBAR_ARROW_SIZE: f32 = 6.0;
pub const SCROLLBAR_ARROW_CAP_SEGMENTS: usize = 24;
pub const SCROLLBAR_ARROW_CORNER_RADIUS: f32 = 1.8;

/* ---- Touch pan & momentum scrolling ---- */
/// Minimum finger movement (logical/DP units) since a touch-pan gesture
/// began before it actually starts translating content, filtering out
/// jitter on an otherwise stationary tap.
pub const TOUCH_PAN_THRESHOLD_DP: f32 = 6.0;
/// Exponential velocity decay rate (per second) applied to momentum
/// scrolling after a touch pan ends; higher values stop sooner.
pub const MOMENTUM_FRICTION: f32 = 4.2;
/// Momentum stops ticking once its speed drops below this (px/sec).
pub const MOMENTUM_MIN_SPEED: f32 = 4.0;

/* ---- AutoScroll (middle-click pan) ---- */
/// Radius (logical/DP units) around the activation point within which
/// cursor movement produces no scrolling, matching native AutoScroll.
pub const AUTO_SCROLL_DEAD_ZONE_DP: f32 = 12.0;
/// Cursor distance (logical/DP units) past the dead zone at which
/// AutoScroll reaches its maximum speed.
pub const AUTO_SCROLL_RANGE_DP: f32 = 160.0;
/// Maximum AutoScroll speed, in logical px/sec, reached at `AUTO_SCROLL_RANGE_DP`.
pub const AUTO_SCROLL_MAX_SPEED: f32 = 1400.0;

/* ---- Overscroll ---- */
/// Visual travel (px) a rubber-banded drag/fling asymptotically
/// approaches no matter how far past the bounds it's pulled.
pub const OVERSCROLL_RUBBER_BAND_RANGE: f32 = 90.0;
/// Eased transition used to spring an overscrolled offset back to bounds.
pub const OVERSCROLL_RETURN_TRANSITION: Transition = Transition::new(
    web_time::Duration::from_millis(320)
).easing(Easing::EaseOut);
/// Edge-glow fade-out rate (alpha per second) for `Overscroll::Glow`.
pub const OVERSCROLL_GLOW_DECAY_PER_SEC: f32 = 2.6;
