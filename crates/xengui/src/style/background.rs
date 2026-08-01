use crate::MAX_GRADIENT_STOPS;

// SPDX-License-Identifier: Apache-2.0
use super::Color;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GradientStop {
    pub color: Color,
    pub position: f32,
}

impl GradientStop {
    pub fn new(color: Color, position: f32) -> Self {
        Self { color, position: position.clamp(0.0, 1.0) }
    }
}

impl From<(Color, f32)> for GradientStop {
    fn from((color, position): (Color, f32)) -> Self {
        Self::new(color, position)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct LinearGradient {
    pub angle_deg: f32,
    pub stops: Vec<GradientStop>,
}

impl LinearGradient {
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
pub struct RadialGradient {
    pub stops: Vec<GradientStop>,
}

impl RadialGradient {
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
pub enum Background {
    Color(Color),
    LinearGradient(LinearGradient),
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
    pub fn representative_color(&self) -> Color {
        match self {
            Self::Color(c) => *c,
            Self::LinearGradient(g) =>
                g.stops
                    .first()
                    .map(|s| s.color)
                    .unwrap_or(Color::TRANSPARENT),
            Self::RadialGradient(g) =>
                g.stops
                    .first()
                    .map(|s| s.color)
                    .unwrap_or(Color::TRANSPARENT),
        }
    }
}
