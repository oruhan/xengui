// SPDX-License-Identifier: Apache-2.0

/// The point around which transforms (rotate, scale, skew) are applied.
/// Corresponds to CSS `transform-origin`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TransformOrigin {
    pub x: TransformOriginAxis,
    pub y: TransformOriginAxis,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TransformOriginAxis {
    /// Absolute pixel offset from the widget's top-left corner.
    Px(f32),
    /// Percentage of the widget's width (x-axis) or height (y-axis).
    Percent(f32),
}

impl TransformOrigin {
    /// Center of the widget on both axes — CSS default (`50% 50%`).
    pub const CENTER: Self = Self {
        x: TransformOriginAxis::Percent(50.0),
        y: TransformOriginAxis::Percent(50.0),
    };

    pub const TOP_LEFT: Self = Self {
        x: TransformOriginAxis::Percent(0.0),
        y: TransformOriginAxis::Percent(0.0),
    };

    pub const TOP_RIGHT: Self = Self {
        x: TransformOriginAxis::Percent(100.0),
        y: TransformOriginAxis::Percent(0.0),
    };

    pub const BOTTOM_LEFT: Self = Self {
        x: TransformOriginAxis::Percent(0.0),
        y: TransformOriginAxis::Percent(100.0),
    };

    pub const BOTTOM_RIGHT: Self = Self {
        x: TransformOriginAxis::Percent(100.0),
        y: TransformOriginAxis::Percent(100.0),
    };

    pub fn new(x: TransformOriginAxis, y: TransformOriginAxis) -> Self {
        Self { x, y }
    }

    pub fn percent(x: f32, y: f32) -> Self {
        Self {
            x: TransformOriginAxis::Percent(x),
            y: TransformOriginAxis::Percent(y),
        }
    }

    pub fn px(x: f32, y: f32) -> Self {
        Self {
            x: TransformOriginAxis::Px(x),
            y: TransformOriginAxis::Px(y),
        }
    }

    /// Resolves the origin to physical pixel offsets relative to the
    /// widget's top-left corner, given the widget's rendered size.
    pub fn resolve(&self, widget_width: f32, widget_height: f32) -> (f32, f32) {
        let x = match self.x {
            TransformOriginAxis::Px(v) => v,
            TransformOriginAxis::Percent(p) => widget_width * (p / 100.0),
        };
        let y = match self.y {
            TransformOriginAxis::Px(v) => v,
            TransformOriginAxis::Percent(p) => widget_height * (p / 100.0),
        };
        (x, y)
    }
}

impl Default for TransformOrigin {
    fn default() -> Self {
        Self::CENTER
    }
}
