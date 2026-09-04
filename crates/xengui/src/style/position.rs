// SPDX-License-Identifier: Apache-2.0
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
/// Available `Position` choices.
pub enum Position {
    #[default]
    /// The `Static` variant.
    Static,
    /// The `Relative` variant.
    Relative,
    /// The `Absolute` variant.
    Absolute,
    /// The `Fixed` variant.
    Fixed,
    /// The `Sticky` variant.
    Sticky,
}
