// SPDX-License-Identifier: Apache-2.0
use crate::{ BorderRadius, Color, Length };

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Outline {
    pub width: Length,
    pub color: Color,
    pub radius: Option<BorderRadius>,
    pub offset: Length,
}

impl Default for Outline {
    fn default() -> Self {
        Self {
            width: Length::px(0.0),
            color: Color::TRANSPARENT,
            radius: None,
            offset: Length::px(0.0),
        }
    }
}

impl Outline {
    pub fn new(
        width: impl Into<Length>,
        color: Color,
        radius: Option<BorderRadius>,
        offset: impl Into<Length>
    ) -> Self {
        Self { width: width.into(), color, radius, offset: offset.into() }
    }
}
