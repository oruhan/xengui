// SPDX-License-Identifier: Apache-2.0
use crate::WidgetId;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
/// Available `AnimLayer` choices.
pub enum AnimLayer {
    #[default]
    /// The `Root` variant.
    Root,
    /// The `Background` variant.
    Background,
    /// The `Content` variant.
    Content,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
/// Available `AnimProperty` choices.
pub enum AnimProperty {
    /// The `BackgroundColor` variant.
    BackgroundColor,
    /// The `TextColor` variant.
    TextColor,
    /// The `BorderColor` variant.
    BorderColor,
    /// The `Opacity` variant.
    Opacity,
    /// The `Scale` variant.
    Scale,
    /// The `ContentScale` variant.
    ContentScale,
    /// The `ShadowColor` variant.
    ShadowColor,
    /// The `BorderWidth` variant.
    BorderWidth,
    /// The `BorderRadiusTL` variant.
    BorderRadiusTL,
    /// The `BorderRadiusTR` variant.
    BorderRadiusTR,
    /// The `BorderRadiusBR` variant.
    BorderRadiusBR,
    /// The `BorderRadiusBL` variant.
    BorderRadiusBL,
    /// The `Width` variant.
    Width,
    /// The `Height` variant.
    Height,
    /// The `PaddingLeft` variant.
    PaddingLeft,
    /// The `PaddingTop` variant.
    PaddingTop,
    /// The `PaddingRight` variant.
    PaddingRight,
    /// The `PaddingBottom` variant.
    PaddingBottom,
    /// The `MarginLeft` variant.
    MarginLeft,
    /// The `MarginTop` variant.
    MarginTop,
    /// The `MarginRight` variant.
    MarginRight,
    /// The `MarginBottom` variant.
    MarginBottom,
    /// The `GapX` variant.
    GapX,
    /// The `GapY` variant.
    GapY,
    /// The `ScrollOffset` variant.
    ScrollOffset,
    /// The `ScrollbarThickness` variant.
    ScrollbarThickness,
    /// The `ScrollbarThumbColor` variant.
    ScrollbarThumbColor,
    /// The `ScrollbarArrowColor` variant.
    ScrollbarArrowColor,
    /// The `ScrollbarOpacity` variant.
    ScrollbarOpacity,
}

impl AnimProperty {
    /// Whether an in-flight transition of this property changes the box
    /// model and therefore requires a real layout pass, as opposed to
    /// colors, opacity, or transform-only properties which only need a
    /// repaint on every animation frame.
    pub const fn affects_layout(self) -> bool {
        matches!(
            self,
            Self::BorderWidth
                | Self::BorderRadiusTL
                | Self::BorderRadiusTR
                | Self::BorderRadiusBR
                | Self::BorderRadiusBL
                | Self::Width
                | Self::Height
                | Self::PaddingLeft
                | Self::PaddingTop
                | Self::PaddingRight
                | Self::PaddingBottom
                | Self::MarginLeft
                | Self::MarginTop
                | Self::MarginRight
                | Self::MarginBottom
                | Self::GapX
                | Self::GapY
        )
    }
}

/// Identifies one animatable value on one widget's layer.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct AnimKey {
    /// The `widget` value carried by this type.
    pub widget: WidgetId,
    /// The `layer` value carried by this type.
    pub layer: AnimLayer,
    /// The `property` value carried by this type.
    pub property: AnimProperty,
}
