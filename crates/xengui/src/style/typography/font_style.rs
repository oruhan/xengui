// SPDX-License-Identifier: Apache-2.0
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
/// Available `FontStyle` choices.
pub enum FontStyle {
    #[default]
    /// The `Normal` variant.
    Normal,
    /// The `Italic` variant.
    Italic,
    /// The `Oblique` variant.
    Oblique,
}
