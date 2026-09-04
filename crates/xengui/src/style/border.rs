// SPDX-License-Identifier: Apache-2.0
use crate::{BorderRadius, Color, Length};

#[derive(Clone, Copy, Debug, PartialEq)]
/// Data and behavior represented by `Border`.
pub struct Border {
    /// The `top` value carried by this type.
    pub top: Length,
    /// The `right` value carried by this type.
    pub right: Length,
    /// The `bottom` value carried by this type.
    pub bottom: Length,
    /// The `left` value carried by this type.
    pub left: Length,
    /// The `color` value carried by this type.
    pub color: Color,
    /// The `radius` value carried by this type.
    pub radius: Option<BorderRadius>,
}

impl Default for Border {
    fn default() -> Self {
        Self {
            top: Length::px(0.0),
            right: Length::px(0.0),
            bottom: Length::px(0.0),
            left: Length::px(0.0),
            color: Color::TRANSPARENT,
            radius: None,
        }
    }
}

impl Border {
    /// Creates a value with its default configuration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns or updates the `all` value.
    pub fn all(width: impl Into<Length>, color: Color) -> Self {
        let width = width.into();
        Self {
            top: width,
            right: width,
            bottom: width,
            left: width,
            color,
            radius: None,
        }
    }

    /// Returns or updates the `sides` value.
    pub fn sides(
        top: impl Into<Length>,
        right: impl Into<Length>,
        bottom: impl Into<Length>,
        left: impl Into<Length>,
        color: Color,
    ) -> Self {
        Self {
            top: top.into(),
            right: right.into(),
            bottom: bottom.into(),
            left: left.into(),
            color,
            radius: None,
        }
    }

    /// Returns or updates the `top` value.
    pub fn top(width: impl Into<Length>, color: Color) -> Self {
        Self {
            top: width.into(),
            color,
            ..Self::default()
        }
    }

    /// Returns or updates the `right` value.
    pub fn right(width: impl Into<Length>, color: Color) -> Self {
        Self {
            right: width.into(),
            color,
            ..Self::default()
        }
    }

    /// Returns or updates the `bottom` value.
    pub fn bottom(width: impl Into<Length>, color: Color) -> Self {
        Self {
            bottom: width.into(),
            color,
            ..Self::default()
        }
    }

    /// Returns or updates the `left` value.
    pub fn left(width: impl Into<Length>, color: Color) -> Self {
        Self {
            left: width.into(),
            color,
            ..Self::default()
        }
    }

    /// Returns or updates the `horizontal` value.
    pub fn horizontal(width: impl Into<Length>, color: Color) -> Self {
        let width = width.into();
        Self {
            left: width,
            right: width,
            color,
            ..Self::default()
        }
    }

    /// Returns or updates the `vertical` value.
    pub fn vertical(width: impl Into<Length>, color: Color) -> Self {
        let width = width.into();
        Self {
            top: width,
            bottom: width,
            color,
            ..Self::default()
        }
    }

    /// Sets this border's corner radii. Accepts a single `Length`/`f32`
    /// (applied to all four corners) or an explicit [`BorderRadius`] for
    /// per-corner control, e.g. `border.radius(BorderRadius::top(8.0))`.
    pub fn radius(mut self, radius: impl Into<BorderRadius>) -> Self {
        self.radius = Some(radius.into());
        self
    }

    /// Returns or updates the `width` value.
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        let width = width.into();
        self.top = width;
        self.right = width;
        self.bottom = width;
        self.left = width;
        self
    }

    /// Returns or updates the `sides_width` value.
    pub fn sides_width(
        mut self,
        top: impl Into<Length>,
        right: impl Into<Length>,
        bottom: impl Into<Length>,
        left: impl Into<Length>,
    ) -> Self {
        self.top = top.into();
        self.right = right.into();
        self.bottom = bottom.into();
        self.left = left.into();
        self
    }

    /// Returns or updates the `top_width` value.
    pub fn top_width(mut self, width: impl Into<Length>) -> Self {
        self.top = width.into();
        self
    }

    /// Returns or updates the `right_width` value.
    pub fn right_width(mut self, width: impl Into<Length>) -> Self {
        self.right = width.into();
        self
    }

    /// Returns or updates the `bottom_width` value.
    pub fn bottom_width(mut self, width: impl Into<Length>) -> Self {
        self.bottom = width.into();
        self
    }

    /// Returns or updates the `left_width` value.
    pub fn left_width(mut self, width: impl Into<Length>) -> Self {
        self.left = width.into();
        self
    }

    /// Returns or updates the `horizontal_width` value.
    pub fn horizontal_width(mut self, width: impl Into<Length>) -> Self {
        let width = width.into();
        self.left = width;
        self.right = width;
        self
    }

    /// Returns or updates the `vertical_width` value.
    pub fn vertical_width(mut self, width: impl Into<Length>) -> Self {
        let width = width.into();
        self.top = width;
        self.bottom = width;
        self
    }

    /// Returns or updates the `color` value.
    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    /// Returns whether the `is_uniform` condition is satisfied.
    pub fn is_uniform(&self) -> bool {
        self.top == self.right && self.right == self.bottom && self.bottom == self.left
    }
}
