// SPDX-License-Identifier: Apache-2.0
use crate::{BorderRadius, Color, Length};

#[derive(Clone, Copy, Debug, PartialEq)]
/// Data and behavior represented by `Outline`.
pub struct Outline {
    /// The `width` value carried by this type.
    pub width: Length,
    /// The `color` value carried by this type.
    pub color: Color,
    /// The `radius` value carried by this type.
    pub radius: Option<BorderRadius>,
    /// The `offset` value carried by this type.
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
    /// Creates a value with its default configuration.
    pub fn new(
        width: impl Into<Length>,
        color: Color,
        radius: Option<BorderRadius>,
        offset: impl Into<Length>,
    ) -> Self {
        Self {
            width: width.into(),
            color,
            radius,
            offset: offset.into(),
        }
    }
}
