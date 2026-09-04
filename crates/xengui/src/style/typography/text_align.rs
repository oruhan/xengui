// SPDX-License-Identifier: Apache-2.0
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
/// Available `TextAlign` choices.
pub enum TextAlign {
    #[default]
    /// The `Start` variant.
    Start,

    /// The `Center` variant.
    Center,

    /// The `End` variant.
    End,

    /// The `Justify` variant.
    Justify,
}
