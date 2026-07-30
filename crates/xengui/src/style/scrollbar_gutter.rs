// SPDX-License-Identifier: Apache-2.0

/// Controls whether space is reserved for a scrollbar even when it isn't
/// shown, so content doesn't shift when a scrollbar appears/disappears.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ScrollbarGutter {
    /// Reserves scrollbar-width space on the scrollbar's own edge whenever
    /// that axis is scrollable. The reserved amount is fixed to the
    /// resting thickness, so it's already there before the scrollbar is
    /// hovered and never grows or shrinks afterward.
    #[default]
    Auto,
    /// Reserves scrollbar-width space on the scrollbar's own edge.
    Stable,
    /// Reserves scrollbar-width space on both edges, keeping content
    /// centered regardless of which side the scrollbar is drawn on.
    StableBothEdges,
}
