// SPDX-License-Identifier: Apache-2.0
//! Material Symbols variable font support: axis definitions and the
//! font's own bytes, consumed by `xengui`'s `VariableIcon` widget through
//! the wgpu backend's dedicated variable-icon pipeline.
//!
//! Material Symbols icons are designed and provided by Google.

/// One variation axis tag, as encoded in the font's `fvar` table.
pub type AxisTag = [u8; 4];

pub const AXIS_FILL: AxisTag = *b"FILL";
pub const AXIS_WEIGHT: AxisTag = *b"wght";
pub const AXIS_GRADE: AxisTag = *b"GRAD";
pub const AXIS_OPTICAL_SIZE: AxisTag = *b"opsz";

/// Material Symbols' four variation axes, clamped to the ranges the font
/// itself defines. Unlike a static icon set, every value here blends
/// continuously - there's no discrete "regular/bold" step.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct IconAxes {
    fill: f32,
    weight: f32,
    grade: f32,
    optical_size: f32,
}

impl IconAxes {
    pub const FILL_RANGE: (f32, f32) = (0.0, 1.0);
    pub const WEIGHT_RANGE: (f32, f32) = (100.0, 700.0);
    pub const GRADE_RANGE: (f32, f32) = (-25.0, 200.0);
    pub const OPTICAL_SIZE_RANGE: (f32, f32) = (20.0, 48.0);

    /// (0.0, 1.0)
    pub fn fill(mut self, value: f32) -> Self {
        self.fill = value.clamp(Self::FILL_RANGE.0, Self::FILL_RANGE.1);
        self
    }

    /// (100.0, 700.0)
    pub fn weight(mut self, value: f32) -> Self {
        self.weight = value.clamp(Self::WEIGHT_RANGE.0, Self::WEIGHT_RANGE.1);
        self
    }

    /// (-25, 200)
    pub fn grade(mut self, value: f32) -> Self {
        self.grade = value.clamp(Self::GRADE_RANGE.0, Self::GRADE_RANGE.1);
        self
    }

    /// (20.0, 48.0)
    pub fn optical_size(mut self, value: f32) -> Self {
        self.optical_size = value.clamp(Self::OPTICAL_SIZE_RANGE.0, Self::OPTICAL_SIZE_RANGE.1);
        self
    }

    pub const fn fill_value(&self) -> f32 {
        self.fill
    }

    pub const fn weight_value(&self) -> f32 {
        self.weight
    }

    pub const fn grade_value(&self) -> f32 {
        self.grade
    }

    pub const fn optical_size_value(&self) -> f32 {
        self.optical_size
    }

    /// Every axis as `(tag, value)`, ready to hand to a variable-font
    /// rasterizer (e.g. `swash::Setting`).
    pub fn to_variations(self) -> [(AxisTag, f32); 4] {
        [
            (AXIS_FILL, self.fill),
            (AXIS_WEIGHT, self.weight),
            (AXIS_GRADE, self.grade),
            (AXIS_OPTICAL_SIZE, self.optical_size),
        ]
    }

    /// Stable hash of the resolved axis values, used as part of the
    /// rasterized-glyph cache key downstream - bit-identical `f32`s only,
    /// no tolerance-based comparison.
    pub fn cache_key(&self) -> u64 {
        use std::hash::{ Hash, Hasher };
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        self.fill.to_bits().hash(&mut hasher);
        self.weight.to_bits().hash(&mut hasher);
        self.grade.to_bits().hash(&mut hasher);
        self.optical_size.to_bits().hash(&mut hasher);
        hasher.finish()
    }
}

impl Default for IconAxes {
    fn default() -> Self {
        Self { fill: 0.0, weight: 400.0, grade: 0.0, optical_size: 24.0 }
    }
}

/// Material Symbols (Rounded) variable font, embedded at compile time.
pub struct MaterialSymbolsVariable;

impl MaterialSymbolsVariable {
    pub const FONT: &'static [u8] = include_bytes!("../fonts/material-symbols-rounded.woff2");
}

/// A handful of well-known codepoints, since Material Symbols glyphs live
/// in the font's Private Use Area rather than at their Unicode-visible
/// name. Not exhaustive - construct any other icon directly with its own
/// `char::from_u32(...)`.
pub mod codepoints {
    pub const HOME: char = '\u{e88a}';
    pub const SEARCH: char = '\u{e8b6}';
    pub const CLOSE: char = '\u{e5cd}';
    pub const CHECK: char = '\u{e5ca}';
    pub const ADD: char = '\u{e145}';
    pub const PLUS: char = '\u{e145}';
    pub const SETTINGS: char = '\u{e8b8}';
    pub const REMOVE: char = '\u{e15b}';
    pub const MINUS: char = '\u{e15b}';
}
