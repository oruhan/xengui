// SPDX-License-Identifier: Apache-2.0

/// Controls whether space is reserved for a scrollbar even when it isn't
/// shown, so content doesn't shift when a scrollbar appears/disappears.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ScrollbarGutter {
    /// No space is reserved; the scrollbar overlays content when shown.
    #[default]
    Auto,
    /// Reserves scrollbar-width space on the scrollbar's own edge.
    Stable,
    /// Reserves scrollbar-width space on both edges, keeping content
    /// centered regardless of which side the scrollbar is drawn on.
    StableBothEdges,
}
