// SPDX-License-Identifier: Apache-2.0
use web_time::Duration;

use xen_animation::{Easing, Transition};

use crate::{Color, Cursor, Length};

/// Default font size in logical pixels
pub const DEFAULT_FONT_SIZE: Length = Length::px(15.0);
/// Default line height ratio in logical pixels
pub const DEFAULT_LINE_HEIGHT_RATIO: f32 = 1.25;
/// Default cursor icon
pub const DEFAULT_CURSOR_ICON: Cursor = Cursor::Default;
/// Default pointer cursor icon
pub const DEFAULT_POINTER_CURSOR_ICON: Cursor = Cursor::Pointer;
/// Default link color
pub const DEFAULT_LINK_COLOR: Color = Color::BLUE_300;

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
/// Scrollbar thumb thickness while neither hovered nor pressed.
pub const DEFAULT_SCROLLBAR_THUMB_THICKNESS: f32 = 5.0;
/// Scrollbar thumb thickness while hovered or pressed, unless overridden.
pub const DEFAULT_SCROLLBAR_THUMB_HOVER_THICKNESS: f32 = 7.0;

// Eased transition applied to scroll position when animating toward a
// wheel/nudge target; drag updates bypass this and snap instantly.
/// The `SCROLL_TRANSITION` constant.
pub const SCROLL_TRANSITION: Transition =
    Transition::new(web_time::Duration::from_millis(250)).easing(Easing::EaseOut);
/// The `SCROLLBAR_THICKNESS_TRANSITION` constant.
pub const SCROLLBAR_THICKNESS_TRANSITION: Transition =
    Transition::new(web_time::Duration::from_millis(160)).easing(Easing::EaseOut);
/// How long a scrollbar shown only while scrolling (see
/// `StyleBuilder::scrollbar_auto_hide`) stays visible after the last
/// scroll activity before it starts fading out.
pub const SCROLLBAR_AUTO_HIDE_LINGER: Duration = Duration::from_millis(700);
/// Fade transition applied to a scrollbar's own opacity while auto-hide is active.
pub const SCROLLBAR_OPACITY_FADE_TRANSITION: Transition =
    Transition::new(Duration::from_millis(220)).easing(Easing::EaseOut);

/// Opacity applied to a `Scroll`-mode scrollbar axis that has nothing to
/// scroll, so it stays visible but reads as disabled instead of vanishing.
pub const SCROLLBAR_DISABLED_OPACITY: f32 = 0.35;

/// Padding (px) trimmed from each side of the scrollbar thumb's
/// cross-axis thickness, so it renders thinner than its track.
pub const SCROLLBAR_THUMB_PADDING: f32 = 4.0;

/// The `SCROLLBAR_ARROW_SIZE` constant.
pub const SCROLLBAR_ARROW_SIZE: f32 = 6.0;
/// The `SCROLLBAR_ARROW_CAP_SEGMENTS` constant.
pub const SCROLLBAR_ARROW_CAP_SEGMENTS: usize = 32;
/// The `SCROLLBAR_ARROW_CORNER_RADIUS` constant.
pub const SCROLLBAR_ARROW_CORNER_RADIUS: f32 = 2.2;

/// Delay (seconds) after pressing a scrollbar arrow button before it
/// starts repeating continuously, matching native scrollbar behavior.
pub const ARROW_HOLD_INITIAL_DELAY: f32 = 0.35;
/// Interval (seconds) between repeated nudges while a scrollbar arrow
/// button is held past the initial delay.
pub const ARROW_HOLD_REPEAT_INTERVAL: f32 = 0.06;

/* ---- Scrollbar arrow press feedback ---- */
/// Transition applied to a scrollbar arrow button's press scale.
pub const SCROLLBAR_ARROW_PRESS_TRANSITION: Transition =
    Transition::new(web_time::Duration::from_millis(100)).easing(Easing::EaseOut);
/// Scale a scrollbar arrow button shrinks to while pressed.
pub const SCROLLBAR_ARROW_PRESS_SCALE: f32 = 0.85;

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
/// Extra friction multiplier applied while a fling is coasting inside
/// the rubber-band zone, so it settles in far fewer frames instead of
/// decaying at the same rate it does within bounds.
pub const MOMENTUM_OVERSCROLL_FRICTION_MULTIPLIER: f32 = 3.0;

/* ---- AutoScroll (middle-click pan) ---- */
/// Radius (logical/DP units) around the activation point within which
/// cursor movement produces no scrolling, matching native AutoScroll.
pub const AUTO_SCROLL_DEAD_ZONE_DP: f32 = 8.0;
/// Cursor distance (logical/DP units) past the dead zone at which
/// AutoScroll reaches its maximum speed.
pub const AUTO_SCROLL_RANGE_DP: f32 = 180.0;
/// Maximum AutoScroll speed, in logical px/sec, reached at `AUTO_SCROLL_RANGE_DP`.
pub const AUTO_SCROLL_MAX_SPEED: f32 = 1900.0;
/// Exponential response rate used while AutoScroll accelerates or changes
/// direction. Larger values follow the pointer more aggressively.
pub const AUTO_SCROLL_ACCELERATION: f32 = 11.0;
/// Faster response used when returning to the dead zone, so AutoScroll
/// comes to rest promptly without snapping to zero.
pub const AUTO_SCROLL_DECELERATION: f32 = 17.0;
/// Radius of the animated AutoScroll origin marker, in logical/DP units.
pub const AUTO_SCROLL_INDICATOR_RADIUS_DP: f32 = 14.0;

/* ---- Overscroll ---- */
/// Visual travel (px) a rubber-banded drag/fling asymptotically
/// approaches no matter how far past the bounds it's pulled.
pub const OVERSCROLL_RUBBER_BAND_RANGE: f32 = 90.0;
/// Eased transition used to spring an overscrolled offset back to bounds.
pub const OVERSCROLL_RETURN_TRANSITION: Transition =
    Transition::new(web_time::Duration::from_millis(320)).easing(Easing::EaseOut);
/// Transition used to fade out an edge-glow flash for `Overscroll::Glow`,
/// driven through `xen_animation::AnimationManager` instead of a manual
/// per-frame decay.
pub const OVERSCROLL_GLOW_FADE_TRANSITION: Transition =
    Transition::new(web_time::Duration::from_millis(385)).easing(Easing::Linear);
/// Visual travel (px) a `Stretch`-mode drag/fling asymptotically
/// approaches - shorter than `OVERSCROLL_RUBBER_BAND_RANGE`, matching
/// Android's stretch overscroll, which resists further than a loose
/// rubber-band bounce.
pub const STRETCH_RUBBER_BAND_RANGE: f32 = 48.0;
/// Extra friction multiplier applied while a fling coasts inside the
/// rubber-band zone under `Overscroll::Stretch` - snappier than
/// `MOMENTUM_OVERSCROLL_FRICTION_MULTIPLIER` so the stretch settles
/// quickly instead of lingering like a bounce.
pub const STRETCH_OVERSCROLL_FRICTION_MULTIPLIER: f32 = 4.5;
/// Eased transition used to spring a `Stretch`-mode overscrolled offset
/// back to bounds - snappier than `OVERSCROLL_RETURN_TRANSITION`.
pub const STRETCH_RETURN_TRANSITION: Transition =
    Transition::new(web_time::Duration::from_millis(180)).easing(Easing::EaseOut);
/// Thickness (px) of the full-span edge-glow band for `Overscroll::Glow`,
/// drawn across the whole hit edge instead of anchored to the gesture point.
pub const OVERSCROLL_GLOW_BAND_THICKNESS: f32 = 3.0;

/* ---- Gradient ----- */
// Bounded by the rect pipeline's vertex-attribute budget (see
// rect_pipeline.rs) - WebGL2's 16-location ceiling doesn't leave room
// for more once packed alongside the existing fill/border attributes.
/// The `MAX_GRADIENT_STOPS` constant.
pub const MAX_GRADIENT_STOPS: usize = 512;

/* ----- ContextMenu ----- */
/// The `ITEM_FONT_SIZE` constant.
pub const ITEM_FONT_SIZE: Length = Length::px(13.0);

/// Opacity multiplier applied to Checkbox/Switch/RadioButton colors while
/// `Interaction::enabled` is false, so the disabled state reads visually
/// dimmed instead of looking identical to an active widget.
pub const DISABLED_WIDGET_OPACITY: f32 = 0.5;
