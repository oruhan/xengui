// SPDX-License-Identifier: Apache-2.0
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
/// Available `BoxSizing` choices.
pub enum BoxSizing {
    #[default]
    /// The `BorderBox` variant.
    BorderBox,
    /// The `ContentBox` variant.
    ContentBox,
}
