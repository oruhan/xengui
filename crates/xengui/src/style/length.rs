// SPDX-License-Identifier: Apache-2.0
use std::cell::Cell;

thread_local! {
    static VIEWPORT_SIZE: Cell<(f32, f32)> = const { Cell::new((0.0, 0.0)) };
}

/// Updates the viewport size used to resolve `Length::ViewportWidth` and
/// `Length::ViewportHeight` values. Called once per layout pass.
pub fn set_viewport_size(width: f32, height: f32) {
    VIEWPORT_SIZE.with(|cell| cell.set((width, height)));
}

/// Returns or updates the `viewport_size` value.
pub fn viewport_size() -> (f32, f32) {
    VIEWPORT_SIZE.with(Cell::get)
}

#[derive(Clone, Copy, Debug, PartialEq)]
/// Available `Length` choices.
pub enum Length {
    /// The `Px` variant.
    Px(f32),
    /// The `Percent` variant.
    Percent(f32),
    /// The `ViewportWidth` variant.
    ViewportWidth(f32),
    /// The `ViewportHeight` variant.
    ViewportHeight(f32),
}

impl Default for Length {
    fn default() -> Self {
        Self::Px(0.0)
    }
}

impl Length {
    /// Returns or updates the `px` value.
    pub const fn px(value: f32) -> Self {
        Self::Px(value)
    }

    /// Returns or updates the `pct` value.
    pub fn pct(value: f32) -> Self {
        Self::Percent(value)
    }

    /// Returns or updates the `vw` value.
    pub const fn vw(value: f32) -> Self {
        Self::ViewportWidth(value)
    }

    /// Returns or updates the `vh` value.
    pub const fn vh(value: f32) -> Self {
        Self::ViewportHeight(value)
    }

    /// Returns or updates the `value` value.
    pub const fn value(&self) -> f32 {
        match self {
            Self::Px(v) => *v,
            Self::Percent(v) => *v,
            Self::ViewportWidth(v) => *v,
            Self::ViewportHeight(v) => *v,
        }
    }

    /// Returns or updates the `to_physical` value.
    pub fn to_physical(self, scale_factor: f32) -> f32 {
        match self {
            Self::Px(v) => v * scale_factor,
            Self::Percent(v) => v,
            Self::ViewportWidth(v) => {
                let (vw, _) = VIEWPORT_SIZE.with(Cell::get);
                vw * (v / 100.0)
            }
            Self::ViewportHeight(v) => {
                let (_, vh) = VIEWPORT_SIZE.with(Cell::get);
                vh * (v / 100.0)
            }
        }
    }

    /// Returns or updates the `add_px` value.
    pub fn add_px(self, value: f32) -> Self {
        Self::px(self.value() + value)
    }

    /// Returns or updates the `sub_px` value.
    pub fn sub_px(self, value: f32) -> Self {
        Self::px((self.value() - value).max(0.0))
    }
}

/// Converts a numeric literal into a pixel unit.
///
/// # Example
/// ```rust
/// use xengui::px;
///
/// let width = px!(100);
/// ```
#[macro_export]
macro_rules! px {
    ($v:expr) => {
        $crate::style::Length::Px($v as f32)
    };
}

/// Converts a numeric literal into a percent unit.
///
/// # Example
/// ```rust
/// use xengui::pct;
///
/// let width = pct!(100);
/// ```
#[macro_export]
macro_rules! pct {
    ($v:expr) => {
        $crate::style::Length::Percent($v as f32)
    };
}

#[macro_export]
/// Expands the `vw` convenience syntax.
macro_rules! vw {
    ($v:expr) => {
        $crate::style::Length::ViewportWidth($v as f32)
    };
}

#[macro_export]
/// Expands the `vh` convenience syntax.
macro_rules! vh {
    ($v:expr) => {
        $crate::style::Length::ViewportHeight($v as f32)
    };
}

macro_rules! impl_length_from {
    ($($t:ty),*) => {
        $(
            impl From<$t> for Length {
                fn from(value: $t) -> Self {
                    Self::Px(value as f32)
                }
            }
        )*
    };
}

impl_length_from!(u8, u16, u32, u64, usize, i8, i16, i32, i64, isize, f32, f64);
