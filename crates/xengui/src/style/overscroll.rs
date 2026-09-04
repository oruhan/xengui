// SPDX-License-Identifier: Apache-2.0

/// Controls what happens when a scrollable view is dragged or flung past
/// its scroll bounds.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Overscroll {
    /// Resolves to whichever behavior matches the current platform's own
    /// scrolling conventions (see `View`'s overscroll resolution logic).
    #[default]
    Auto,
    /// Scrolling stops exactly at the content bounds; no rubber-banding.
    Disabled,
    /// Dragging or flinging past the edge rubber-bands the content and
    /// springs back, matching iOS-style bounce.
    Bounce,
    /// Similar to `Bounce`, but with tighter resistance and a snappier return.
    Stretch,
    /// Content stops exactly at the bounds, like `Disabled`, but flashes a
    /// soft glow at the edge that was hit, matching Android's edge glow.
    Glow,
}
