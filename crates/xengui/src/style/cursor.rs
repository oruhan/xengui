// SPDX-License-Identifier: Apache-2.0

/// Platform-agnostic pointer cursor kind. Backends (e.g. xenframe) map this
/// onto their own native cursor type.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Cursor {
    #[default]
    /// The `Default` variant.
    Default,
    /// The `ContextMenu` variant.
    ContextMenu,
    /// The `Help` variant.
    Help,
    /// The `Pointer` variant.
    Pointer,
    /// The `Progress` variant.
    Progress,
    /// The `Wait` variant.
    Wait,
    /// The `Cell` variant.
    Cell,
    /// The `Crosshair` variant.
    Crosshair,
    /// The `Text` variant.
    Text,
    /// The `VerticalText` variant.
    VerticalText,
    /// The `Alias` variant.
    Alias,
    /// The `Copy` variant.
    Copy,
    /// The `Move` variant.
    Move,
    /// The `NoDrop` variant.
    NoDrop,
    /// The `NotAllowed` variant.
    NotAllowed,
    /// The `Grab` variant.
    Grab,
    /// The `Grabbing` variant.
    Grabbing,
    /// The `AllScroll` variant.
    AllScroll,
    /// The `ZoomIn` variant.
    ZoomIn,
    /// The `ZoomOut` variant.
    ZoomOut,
    /// The `EResize` variant.
    EResize,
    /// The `NResize` variant.
    NResize,
    /// The `NeResize` variant.
    NeResize,
    /// The `NwResize` variant.
    NwResize,
    /// The `SResize` variant.
    SResize,
    /// The `SeResize` variant.
    SeResize,
    /// The `SwResize` variant.
    SwResize,
    /// The `WResize` variant.
    WResize,
    /// The `EwResize` variant.
    EwResize,
    /// The `NsResize` variant.
    NsResize,
    /// The `NeswResize` variant.
    NeswResize,
    /// The `NwseResize` variant.
    NwseResize,
    /// The `ColResize` variant.
    ColResize,
    /// The `RowResize` variant.
    RowResize,
}
