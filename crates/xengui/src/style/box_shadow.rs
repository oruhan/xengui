// SPDX-License-Identifier: Apache-2.0
use crate::{Color, Length};

/// Restricts which side(s) of the box an outset shadow is visible on.
/// `All` (the default) matches plain CSS box-shadow behavior.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ShadowDirection {
    #[default]
    /// The `All` variant.
    All,
    /// The `Top` variant.
    Top,
    /// The `Bottom` variant.
    Bottom,
    /// The `Left` variant.
    Left,
    /// The `Right` variant.
    Right,
    /// The `TopLeft` variant.
    TopLeft,
    /// The `TopRight` variant.
    TopRight,
    /// The `BottomLeft` variant.
    BottomLeft,
    /// The `BottomRight` variant.
    BottomRight,
}

/// A single CSS-style box shadow layer. Widgets accept a `Vec<BoxShadow>`
/// via `StyleBuilder::box_shadow`, painted in list order like CSS's
/// comma-separated `box-shadow` - the first shadow ends up on top.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BoxShadow {
    /// The `offset_x` value carried by this type.
    pub offset_x: Length,
    /// The `offset_y` value carried by this type.
    pub offset_y: Length,
    /// The `blur_radius` value carried by this type.
    pub blur_radius: Length,
    /// The `spread_radius` value carried by this type.
    pub spread_radius: Length,
    /// The `color` value carried by this type.
    pub color: Color,
    /// The `inset` value carried by this type.
    pub inset: bool,
    /// The `direction` value carried by this type.
    pub direction: ShadowDirection,
}

impl BoxShadow {
    /// Creates a value with its default configuration.
    pub fn new(
        offset_x: impl Into<Length>,
        offset_y: impl Into<Length>,
        blur_radius: impl Into<Length>,
        color: Color,
    ) -> Self {
        Self {
            offset_x: offset_x.into(),
            offset_y: offset_y.into(),
            blur_radius: blur_radius.into(),
            spread_radius: Length::px(0.0),
            color,
            inset: false,
            direction: ShadowDirection::All,
        }
    }

    /// Returns or updates the `spread` value.
    pub fn spread(mut self, spread: impl Into<Length>) -> Self {
        self.spread_radius = spread.into();
        self
    }

    /// Returns or updates the `inset` value.
    pub fn inset(mut self, inset: bool) -> Self {
        self.inset = inset;
        self
    }

    /// Returns or updates the `direction` value.
    pub fn direction(mut self, direction: ShadowDirection) -> Self {
        self.direction = direction;
        self
    }
}

impl From<BoxShadow> for Vec<BoxShadow> {
    fn from(value: BoxShadow) -> Self {
        vec![value]
    }
}
