// SPDX-License-Identifier: Apache-2.0
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
/// Available `FontWeight` choices.
pub enum FontWeight {
    /// The `Thin` variant.
    Thin,
    /// The `ExtraLight` variant.
    ExtraLight,
    /// The `Light` variant.
    Light,

    #[default]
    /// The `Regular` variant.
    Regular,

    /// The `Medium` variant.
    Medium,
    /// The `SemiBold` variant.
    SemiBold,
    /// The `Bold` variant.
    Bold,
    /// The `ExtraBold` variant.
    ExtraBold,
    /// The `Black` variant.
    Black,

    /// A numeric OpenType weight used while interpolating variable fonts.
    Variable(u16),
}

impl FontWeight {
    /// Returns or updates the `to_numeric` value.
    pub const fn to_numeric(self) -> u16 {
        match self {
            Self::Thin => 100,
            Self::ExtraLight => 200,
            Self::Light => 300,
            Self::Regular => 400,
            Self::Medium => 500,
            Self::SemiBold => 600,
            Self::Bold => 700,
            Self::ExtraBold => 800,
            Self::Black => 900,
            Self::Variable(value) => value,
        }
    }

    /// Creates a numeric OpenType weight clamped to the supported range.
    pub const fn from_numeric(value: u16) -> Self {
        Self::Variable(if value < 1 {
            1
        } else if value > 1000 {
            1000
        } else {
            value
        })
    }
}
