use crate::MAX_GRADIENT_STOPS;

// SPDX-License-Identifier: Apache-2.0
use super::Color;

#[derive(Clone, Copy, Debug, PartialEq)]
/// Data and behavior represented by `GradientStop`.
pub struct GradientStop {
    /// The `color` value carried by this type.
    pub color: Color,
    /// The `position` value carried by this type.
    pub position: f32,
}

impl GradientStop {
    /// Creates a value with its default configuration.
    pub fn new(color: Color, position: f32) -> Self {
        Self {
            color,
            position: position.clamp(0.0, 1.0),
        }
    }
}

impl From<(Color, f32)> for GradientStop {
    fn from((color, position): (Color, f32)) -> Self {
        Self::new(color, position)
    }
}

#[derive(Clone, Debug, PartialEq)]
/// Data and behavior represented by `LinearGradient`.
pub struct LinearGradient {
    /// The `angle_deg` value carried by this type.
    pub angle_deg: f32,
    /// The `stops` value carried by this type.
    pub stops: Vec<GradientStop>,
}

impl LinearGradient {
    /// Creates a value with its default configuration.
    pub fn new(angle_deg: f32, stops: impl Into<Vec<GradientStop>>) -> Self {
        let mut stops: Vec<GradientStop> = stops.into();
        if stops.len() > MAX_GRADIENT_STOPS {
            log::warn!(
                "LinearGradient: {} stops given, only the first {MAX_GRADIENT_STOPS} are used",
                stops.len()
            );
            stops.truncate(MAX_GRADIENT_STOPS);
        }
        Self { angle_deg, stops }
    }
}

#[derive(Clone, Debug, PartialEq)]
/// Data and behavior represented by `RadialGradient`.
pub struct RadialGradient {
    /// The `stops` value carried by this type.
    pub stops: Vec<GradientStop>,
}

impl RadialGradient {
    /// Creates a value with its default configuration.
    pub fn new(stops: impl Into<Vec<GradientStop>>) -> Self {
        let mut stops: Vec<GradientStop> = stops.into();
        if stops.len() > MAX_GRADIENT_STOPS {
            log::warn!(
                "RadialGradient: {} stops given, only the first {MAX_GRADIENT_STOPS} are used",
                stops.len()
            );
            stops.truncate(MAX_GRADIENT_STOPS);
        }
        Self { stops }
    }
}

#[derive(Clone, Debug, PartialEq)]
/// Available `Background` choices.
pub enum Background {
    /// The `Color` variant.
    Color(Color),
    /// The `LinearGradient` variant.
    LinearGradient(LinearGradient),
    /// The `RadialGradient` variant.
    RadialGradient(RadialGradient),
}

impl From<Color> for Background {
    fn from(color: Color) -> Self {
        Self::Color(color)
    }
}

impl From<LinearGradient> for Background {
    fn from(gradient: LinearGradient) -> Self {
        Self::LinearGradient(gradient)
    }
}

impl From<RadialGradient> for Background {
    fn from(gradient: RadialGradient) -> Self {
        Self::RadialGradient(gradient)
    }
}

impl Background {
    // Single-color stand-in for call sites that only need one color
    // (fading, non-gradient fallback paths, etc). Uses the first stop.
    /// Returns or updates the `representative_color` value.
    pub fn representative_color(&self) -> Color {
        match self {
            Self::Color(c) => *c,
            Self::LinearGradient(g) => g
                .stops
                .first()
                .map(|s| s.color)
                .unwrap_or(Color::TRANSPARENT),
            Self::RadialGradient(g) => g
                .stops
                .first()
                .map(|s| s.color)
                .unwrap_or(Color::TRANSPARENT),
        }
    }
}
