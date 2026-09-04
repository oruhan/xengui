// SPDX-License-Identifier: Apache-2.0
//! Tailwind-style responsive values: a single call site (`Responsive::new(...)`)
//! carries a base value plus optional per-breakpoint overrides, and plugs
//! directly into the existing `StyleBuilder` methods through `IntoThemed` -
//! no separate `.responsive_width()` API needed, e.g.:
//!
//! ```no_run
//! use xengui::{ View, StyleBuilder, Responsive, pct, px };
//!
//! let sidebar = View::new()
//!     .width(Responsive::new(pct!(100.0)).md(px!(280.0)).lg(px!(320.0)));
//! ```
use super::theme::IntoThemed;
use std::cell::Cell;

/// Tailwind-parity breakpoints, activated min-width-first like CSS media
/// queries (`@media (min-width: ...)`) - a value set at `Md` also applies
/// at `Lg`/`Xl`/`Xl2` unless overridden there.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Breakpoint {
    /// The `Base` variant.
    Base,
    /// The `Sm` variant.
    Sm,
    /// The `Md` variant.
    Md,
    /// The `Lg` variant.
    Lg,
    /// The `Xl` variant.
    Xl,
    /// The `Xl2` variant.
    Xl2,
}

impl Breakpoint {
    /// Logical-px activation threshold, matching Tailwind's own defaults.
    pub const fn min_width(self) -> f32 {
        match self {
            Self::Base => 0.0,
            Self::Sm => 640.0,
            Self::Md => 768.0,
            Self::Lg => 1024.0,
            Self::Xl => 1280.0,
            Self::Xl2 => 1536.0,
        }
    }

    fn from_width(logical_width: f32) -> Self {
        [
            Self::Xl2,
            Self::Xl,
            Self::Lg,
            Self::Md,
            Self::Sm,
            Self::Base,
        ]
        .into_iter()
        .find(|bp| logical_width >= bp.min_width())
        .unwrap_or(Self::Base)
    }
}

thread_local! {
    static CURRENT_BREAKPOINT: Cell<Breakpoint> = const { Cell::new(Breakpoint::Base) };
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
    sm: Option<T>,
    md: Option<T>,
    lg: Option<T>,
    xl: Option<T>,
    xl2: Option<T>,
}

impl<T: Clone> Responsive<T> {
    /// Creates a value with its default configuration.
    pub fn new(base: T) -> Self {
        Self {
            base,
            sm: None,
            md: None,
            lg: None,
            xl: None,
            xl2: None,
        }
    }

    /// Returns or updates the `sm` value.
    pub fn sm(mut self, value: T) -> Self {
        self.sm = Some(value);
        self
    }

    /// Returns or updates the `md` value.
    pub fn md(mut self, value: T) -> Self {
        self.md = Some(value);
        self
    }

    /// Returns or updates the `lg` value.
    pub fn lg(mut self, value: T) -> Self {
        self.lg = Some(value);
        self
    }

    /// Returns or updates the `xl` value.
    pub fn xl(mut self, value: T) -> Self {
        self.xl = Some(value);
        self
    }

    /// Returns or updates the `xl2` value.
    pub fn xl2(mut self, value: T) -> Self {
        self.xl2 = Some(value);
        self
    }

    /// Resolves to the override for the highest active breakpoint at or
    /// below the current one, falling back to `base` when none apply.
    pub fn resolve(&self) -> T {
        let bp = current_breakpoint();
        let at_or_below = |candidate: Breakpoint, value: &Option<T>| {
            (bp >= candidate).then(|| value.clone()).flatten()
        };
        at_or_below(Breakpoint::Xl2, &self.xl2)
            .or_else(|| at_or_below(Breakpoint::Xl, &self.xl))
            .or_else(|| at_or_below(Breakpoint::Lg, &self.lg))
            .or_else(|| at_or_below(Breakpoint::Md, &self.md))
            .or_else(|| at_or_below(Breakpoint::Sm, &self.sm))
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
/// hamburger below `Md`.
pub fn responsive_bool(at: Breakpoint, when_at_or_above: bool) -> bool {
    let active = current_breakpoint() >= at;
    if when_at_or_above { active } else { !active }
}
