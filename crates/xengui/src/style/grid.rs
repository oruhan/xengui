// SPDX-License-Identifier: Apache-2.0
#[derive(Clone, Copy, Debug, PartialEq)]
/// Available `GridTrack` choices.
pub enum GridTrack {
    /// The `Px` variant.
    Px(f32),
    /// The `Fr` variant.
    Fr(f32),
    /// The `Auto` variant.
    Auto,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
/// Data and behavior represented by `GridPlacement`.
pub struct GridPlacement {
    /// The `start` value carried by this type.
    pub start: i16,
    /// The `end` value carried by this type.
    pub end: i16,
}

impl GridPlacement {
    /// Returns or updates the `span` value.
    pub const fn span(start: i16, end: i16) -> Self {
        Self { start, end }
    }
}
