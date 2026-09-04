// SPDX-License-Identifier: Apache-2.0

use crate::{FontStyle, FontWeight, MeasureResult};

/// Provides text measurement services to the layout system.
///
/// Widgets use this trait to measure text without depending on a specific
/// rendering backend.
///
/// Typical implementations wrap platform APIs such as DirectWrite,
/// CoreText, FreeType, Skia, or any future text renderer.
pub trait TextMeasurer {
    /// Measures the visual size of a string.
    ///
    /// `font_size`, `letter_spacing` and `line_height` are logical
    /// (pre-scale) pixels; the implementation converts them to physical
    /// pixels internally using `scale_factor`. `max_width` is already
    /// physical, since it comes from the layout engine's own constraints.
    ///
    /// The returned size is expressed in physical pixels.
    #[allow(clippy::too_many_arguments)]
    fn measure(
        &mut self,
        text: &str,
        font: Option<&str>,
        font_size: f32,
        font_weight: FontWeight,
        font_style: FontStyle,
        letter_spacing: f32,
        line_height: f32,
        max_width: Option<f32>,
        scale_factor: f32,
    ) -> MeasureResult;

    /// Returns the horizontal advance of every character boundary.
    ///
    /// The returned vector always contains `text.chars().count() + 1`
    /// elements, where the first value is always `0.0`.
    ///
    /// `font_size`, `letter_spacing` and `line_height` are logical pixels;
    /// see [`Self::measure`]. The returned offsets are physical pixels.
    ///
    /// This method is primarily used for:
    /// - caret positioning
    /// - mouse hit testing
    /// - text selection
    /// - horizontal scrolling
    /// - double-click word selection
    #[allow(clippy::too_many_arguments)]
    fn character_offsets(
        &mut self,
        text: &str,
        font: Option<&str>,
        font_size: f32,
        font_weight: FontWeight,
        font_style: FontStyle,
        letter_spacing: f32,
        line_height: f32,
        scale_factor: f32,
    ) -> Vec<f32>;

    /// Returns the font ascent in physical pixels. `font_size` is logical.
    fn ascent(
        &mut self,
        font: Option<&str>,
        font_size: f32,
        font_weight: FontWeight,
        font_style: FontStyle,
        scale_factor: f32,
    ) -> f32;

    /// Returns the font descent in physical pixels. `font_size` is logical.
    fn descent(
        &mut self,
        font: Option<&str>,
        font_size: f32,
        font_weight: FontWeight,
        font_style: FontStyle,
        scale_factor: f32,
    ) -> f32;

    /// Returns the recommended line height in physical pixels. `font_size` is logical.
    fn line_height(
        &mut self,
        font: Option<&str>,
        font_size: f32,
        font_weight: FontWeight,
        font_style: FontStyle,
        scale_factor: f32,
    ) -> f32;
}
