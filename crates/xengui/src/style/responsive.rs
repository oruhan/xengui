// SPDX-License-Identifier: Apache-2.0
//! Material 3 adaptive values: a single call site (`Responsive::new(...)`)
//! carries a base value plus optional per-breakpoint overrides, and plugs
//! directly into the existing `StyleBuilder` methods through `IntoThemed` -
//! no separate `.responsive_width()` API needed, e.g.:
//!
//! ```no_run
//! use xengui::{ View, StyleBuilder, Responsive, pct, px };
//!
//! let sidebar = View::new()
//!     .width(
//!         Responsive::new(pct!(100.0))
//!             .expanded(px!(280.0))
//!             .large(px!(320.0)),
//!     );
//! ```
use super::theme::IntoThemed;
use std::cell::Cell;

/// The five min-width-first adaptive breakpoints defined by Material 3.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Breakpoint {
    /// Compact windows below 600 logical pixels.
    Compact,
    /// Medium windows from 600 through 839 logical pixels.
    Medium,
    /// Expanded windows from 840 through 1199 logical pixels.
    Expanded,
    /// Large windows from 1200 through 1599 logical pixels.
    Large,
    /// Extra-large windows at or above 1600 logical pixels.
    ExtraLarge,
}

impl Breakpoint {
    /// Logical-pixel activation threshold matching Material 3 breakpoints.
    pub const fn min_width(self) -> f32 {
        match self {
            Self::Compact => 0.0,
            Self::Medium => 600.0,
            Self::Expanded => 840.0,
            Self::Large => 1200.0,
            Self::ExtraLarge => 1600.0,
        }
    }

    fn from_width(logical_width: f32) -> Self {
        [
            Self::ExtraLarge,
            Self::Large,
            Self::Expanded,
            Self::Medium,
            Self::Compact,
        ]
        .into_iter()
        .find(|bp| logical_width >= bp.min_width())
        .unwrap_or(Self::Compact)
    }
}

thread_local! {
    static CURRENT_BREAKPOINT: Cell<Breakpoint> = const { Cell::new(Breakpoint::Compact) };
}

/// Updates the breakpoint used to resolve every `Responsive<T>` value.
/// Called once per layout pass (see `LayoutEngine::layout`), the same way
/// `set_viewport_size` tracks the raw viewport size for `Length::Vw/Vh`.
pub fn set_current_breakpoint_from_width(logical_width: f32) {
    CURRENT_BREAKPOINT.with(|c| c.set(Breakpoint::from_width(logical_width)));
}

/// The breakpoint active as of the last layout pass.
pub fn current_breakpoint() -> Breakpoint {
    CURRENT_BREAKPOINT.with(Cell::get)
}

/// A value with optional per-breakpoint overrides, resolved against
/// [`current_breakpoint`] the moment it's handed to a `StyleBuilder`
/// method (via the `IntoThemed` impls below).
#[derive(Clone, Debug, PartialEq)]
pub struct Responsive<T> {
    base: T,
    medium: Option<T>,
    expanded: Option<T>,
    large: Option<T>,
    extra_large: Option<T>,
}

impl<T: Clone> Responsive<T> {
    /// Creates a value with its default configuration.
    pub fn new(base: T) -> Self {
        Self {
            base,
            medium: None,
            expanded: None,
            large: None,
            extra_large: None,
        }
    }

    /// Overrides the value for medium and wider windows.
    pub fn medium(mut self, value: T) -> Self {
        self.medium = Some(value);
        self
    }

    /// Overrides the value for expanded and wider windows.
    pub fn expanded(mut self, value: T) -> Self {
        self.expanded = Some(value);
        self
    }

    /// Overrides the value for large and wider windows.
    pub fn large(mut self, value: T) -> Self {
        self.large = Some(value);
        self
    }

    /// Overrides the value for extra-large windows.
    pub fn extra_large(mut self, value: T) -> Self {
        self.extra_large = Some(value);
        self
    }

    /// Compatibility shorthand for [`Responsive::medium`].
    pub fn sm(self, value: T) -> Self {
        self.medium(value)
    }

    /// Compatibility shorthand for [`Responsive::expanded`].
    pub fn md(self, value: T) -> Self {
        self.expanded(value)
    }

    /// Compatibility shorthand for [`Responsive::large`].
    pub fn lg(self, value: T) -> Self {
        self.large(value)
    }

    /// Compatibility shorthand for [`Responsive::extra_large`].
    pub fn xl(self, value: T) -> Self {
        self.extra_large(value)
    }

    /// Legacy alias for [`Responsive::extra_large`].
    #[deprecated(since = "0.2.9", note = "use Responsive::extra_large")]
    pub fn xl2(mut self, value: T) -> Self {
        self.extra_large = Some(value);
        self
    }

    /// Resolves to the override for the highest active breakpoint at or
    /// below the current one, falling back to `base` when none apply.
    pub fn resolve(&self) -> T {
        let bp = current_breakpoint();
        let at_or_below = |candidate: Breakpoint, value: &Option<T>| {
            (bp >= candidate).then(|| value.clone()).flatten()
        };
        at_or_below(Breakpoint::ExtraLarge, &self.extra_large)
            .or_else(|| at_or_below(Breakpoint::Large, &self.large))
            .or_else(|| at_or_below(Breakpoint::Expanded, &self.expanded))
            .or_else(|| at_or_below(Breakpoint::Medium, &self.medium))
            .unwrap_or_else(|| self.base.clone())
    }
}

/// Marker distinguishing `Responsive<T>` from `IntoThemed`'s other
/// `ValueMarker`/`FnMarker` impls, so all three can coexist per `T`.
pub struct ResponsiveMarker;

macro_rules! impl_into_themed_responsive {
    ($($t:ty),* $(,)?) => {
        $(
            impl IntoThemed<$t, ResponsiveMarker> for Responsive<$t> {
                fn resolve_themed(self) -> $t {
                    self.resolve()
                }
            }
        )*
    };
}

impl_into_themed_responsive!(
    super::Length,
    super::Color,
    super::Edges,
    super::Background,
    super::Border,
    f32
);

/// Shorthand for values that only change *whether* something shows, not
/// what it contains - e.g. collapsing a header's nav links into a
/// hamburger below [`Breakpoint::Expanded`].
pub fn responsive_bool(at: Breakpoint, when_at_or_above: bool) -> bool {
    let active = current_breakpoint() >= at;
    if when_at_or_above { active } else { !active }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn width_boundaries_match_material_breakpoints() {
        let cases = [
            (-1.0, Breakpoint::Compact),
            (0.0, Breakpoint::Compact),
            (599.99, Breakpoint::Compact),
            (600.0, Breakpoint::Medium),
            (839.99, Breakpoint::Medium),
            (840.0, Breakpoint::Expanded),
            (1199.99, Breakpoint::Expanded),
            (1200.0, Breakpoint::Large),
            (1599.99, Breakpoint::Large),
            (1600.0, Breakpoint::ExtraLarge),
        ];

        for (width, expected) in cases {
            assert_eq!(Breakpoint::from_width(width), expected);
        }
    }

    #[test]
    fn responsive_values_fall_back_to_the_nearest_lower_override() {
        let value = Responsive::new(0)
            .medium(1)
            .expanded(2)
            .large(3)
            .extra_large(4);

        for (width, expected) in [(0.0, 0), (600.0, 1), (840.0, 2), (1200.0, 3), (1600.0, 4)] {
            set_current_breakpoint_from_width(width);
            assert_eq!(value.resolve(), expected);
        }
    }
}
