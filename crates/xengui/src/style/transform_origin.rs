// SPDX-License-Identifier: Apache-2.0

/// The point around which transforms (rotate, scale, skew) are applied.
/// Corresponds to CSS `transform-origin`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TransformOrigin {
    /// The `x` value carried by this type.
    pub x: TransformOriginAxis,
    /// The `y` value carried by this type.
    pub y: TransformOriginAxis,
}

#[derive(Clone, Copy, Debug, PartialEq)]
/// Available `TransformOriginAxis` choices.
pub enum TransformOriginAxis {
    /// The `Px` variant.
    Px(f32),
    /// The `Percent` variant.
    Percent(f32),
}

impl TransformOrigin {
    /// The `CENTER` constant.
    pub const CENTER: Self = Self {
        x: TransformOriginAxis::Percent(50.0),
        y: TransformOriginAxis::Percent(50.0),
    };

    /// The `TOP` constant.
    pub const TOP: Self = Self {
        x: TransformOriginAxis::Percent(50.0),
        y: TransformOriginAxis::Percent(0.0),
    };

    /// The `TOP_RIGHT` constant.
    pub const TOP_RIGHT: Self = Self {
        x: TransformOriginAxis::Percent(100.0),
        y: TransformOriginAxis::Percent(0.0),
    };

    /// The `RIGHT` constant.
    pub const RIGHT: Self = Self {
        x: TransformOriginAxis::Percent(100.0),
        y: TransformOriginAxis::Percent(50.0),
    };

    /// The `BOTTOM_RIGHT` constant.
    pub const BOTTOM_RIGHT: Self = Self {
        x: TransformOriginAxis::Percent(100.0),
        y: TransformOriginAxis::Percent(100.0),
    };

    /// The `BOTTOM` constant.
    pub const BOTTOM: Self = Self {
        x: TransformOriginAxis::Percent(50.0),
        y: TransformOriginAxis::Percent(100.0),
    };

    /// The `BOTTOM_LEFT` constant.
    pub const BOTTOM_LEFT: Self = Self {
        x: TransformOriginAxis::Percent(0.0),
        y: TransformOriginAxis::Percent(100.0),
    };

    /// The `LEFT` constant.
    pub const LEFT: Self = Self {
        x: TransformOriginAxis::Percent(0.0),
        y: TransformOriginAxis::Percent(50.0),
    };

    /// The `TOP_LEFT` constant.
    pub const TOP_LEFT: Self = Self {
        x: TransformOriginAxis::Percent(0.0),
        y: TransformOriginAxis::Percent(0.0),
    };

    /// Returns or updates the `percent` value.
    pub fn percent(x: f32, y: f32) -> Self {
        Self {
            x: TransformOriginAxis::Percent(x),
            y: TransformOriginAxis::Percent(y),
        }
    }

    /// Returns or updates the `px` value.
    pub fn px(x: f32, y: f32) -> Self {
        Self {
            x: TransformOriginAxis::Px(x),
            y: TransformOriginAxis::Px(y),
        }
    }

    /// Resolves to physical pixel offsets relative to the widget's top-left corner.
    pub fn resolve(&self, widget_width: f32, widget_height: f32, scale_factor: f32) -> (f32, f32) {
        let x = match self.x {
            TransformOriginAxis::Px(v) => v * scale_factor,
            TransformOriginAxis::Percent(p) => widget_width * (p / 100.0),
        };
        let y = match self.y {
            TransformOriginAxis::Px(v) => v * scale_factor,
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
