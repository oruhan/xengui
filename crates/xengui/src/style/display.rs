// SPDX-License-Identifier: Apache-2.0
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
/// Available `Display` choices.
pub enum Display {
    #[default]
    /// The `Flex` variant.
    Flex,
    /// The `Grid` variant.
    Grid,
    /// The `Block` variant.
    Block,
    /// The `None` variant.
    None,
}
