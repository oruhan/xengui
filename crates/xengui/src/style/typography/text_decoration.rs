// SPDX-License-Identifier: Apache-2.0
use crate::{Color, Length};

#[derive(Clone, Copy, Debug, PartialEq, Default)]
/// Data and behavior represented by `TextDecoration`.
pub struct TextDecoration {
    underline: bool,
    strike: bool,
    overline: bool,
    color: Option<Color>,
    width: Option<Length>,
}

impl TextDecoration {
    /// The `NONE` constant.
    pub const NONE: TextDecoration = TextDecoration {
        underline: false,
        strike: false,
        overline: false,
        color: None,
        width: None,
    };

    /// The `UNDERLINE` constant.
    pub const UNDERLINE: TextDecoration = TextDecoration {
        underline: true,
        strike: false,
        overline: false,
        color: None,
        width: None,
    };

    /// The `STRIKETHROUGH` constant.
    pub const STRIKETHROUGH: TextDecoration = TextDecoration {
        underline: false,
        strike: true,
        overline: false,
        color: None,
        width: None,
    };

    /// The `OVERLINE` constant.
    pub const OVERLINE: TextDecoration = TextDecoration {
        underline: false,
        strike: false,
        overline: true,
        color: None,
        width: None,
    };

    /// Returns or updates the `underline` value.
    pub fn underline(&self) -> bool {
        self.underline
    }

    /// Returns or updates the `strike` value.
    pub fn strike(&self) -> bool {
        self.strike
    }

    /// Returns or updates the `overline` value.
    pub fn overline(&self) -> bool {
        self.overline
    }

    /// Returns or updates the `color` value.
    pub fn color(&self) -> Option<Color> {
        self.color
    }

    /// Returns or updates the `width` value.
    pub fn width(&self) -> Option<Length> {
        self.width
    }

    /// Updates the `with_underline` value.
    pub fn with_underline(mut self, value: bool) -> Self {
        self.underline = value;
        self
    }

    /// Updates the `with_strike` value.
    pub fn with_strike(mut self, value: bool) -> Self {
        self.strike = value;
        self
    }

    /// Updates the `with_overline` value.
    pub fn with_overline(mut self, value: bool) -> Self {
        self.overline = value;
        self
    }

    /// Updates the `with_color` value.
    pub fn with_color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    /// Updates the `with_width` value.
    pub fn with_width(mut self, width: impl Into<Length>) -> Self {
        self.width = Some(width.into());
        self
    }
}
