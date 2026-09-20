// SPDX-License-Identifier: Apache-2.0

/// Logical inline direction used by layout and keyboard navigation.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TextDirection {
    /// Inline content starts on the left and advances rightward.
    #[default]
    LeftToRight,
    /// Inline content starts on the right and advances leftward.
    RightToLeft,
}
