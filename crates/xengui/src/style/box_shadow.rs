// SPDX-License-Identifier: Apache-2.0
use crate::{ Color, Length };

/// Restricts which side(s) of the box an outset shadow is visible on.
/// `All` (the default) matches plain CSS box-shadow behavior.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ShadowDirection {
    #[default]
    All,
    Top,
    Bottom,
    Left,
    Right,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

/// A single CSS-style box shadow layer. Widgets accept a `Vec<BoxShadow>`
/// via `StyleBuilder::box_shadow`, painted in list order like CSS's
/// comma-separated `box-shadow` - the first shadow ends up on top.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BoxShadow {
    pub offset_x: Length,
    pub offset_y: Length,
    pub blur_radius: Length,
    pub spread_radius: Length,
    pub color: Color,
    pub inset: bool,
    pub direction: ShadowDirection,
}

impl BoxShadow {
    pub fn new(
        offset_x: impl Into<Length>,
        offset_y: impl Into<Length>,
        blur_radius: impl Into<Length>,
        color: Color
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

    pub fn spread(mut self, spread: impl Into<Length>) -> Self {
        self.spread_radius = spread.into();
        self
    }

    pub fn inset(mut self, inset: bool) -> Self {
        self.inset = inset;
        self
    }

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
