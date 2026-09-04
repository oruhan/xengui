// SPDX-License-Identifier: Apache-2.0
use crate::{Color, Length};

/// A single CSS-style drop shadow used by [`Filter::DropShadow`].
///
/// Unlike [`crate::BoxShadow`] (which shadows a widget's box), this shadows
/// the widget's *rendered content* including transparency, matching CSS
/// `filter: drop-shadow(...)` rather than `box-shadow`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DropShadow {
    /// The `offset_x` value carried by this type.
    pub offset_x: Length,
    /// The `offset_y` value carried by this type.
    pub offset_y: Length,
    /// The `blur_radius` value carried by this type.
    pub blur_radius: Length,
    /// The `color` value carried by this type.
    pub color: Color,
}

impl DropShadow {
    /// Creates a value with its default configuration.
    pub fn new(
        offset_x: impl Into<Length>,
        offset_y: impl Into<Length>,
        blur_radius: impl Into<Length>,
        color: Color,
    ) -> Self {
        Self {
            offset_x: offset_x.into(),
            offset_y: offset_y.into(),
            blur_radius: blur_radius.into(),
            color,
        }
    }
}

/// A single GPU-accelerated visual filter, applied to a widget's rendered
/// output rather than its box model.
///
/// Filters mirror CSS `filter` functions where a CSS equivalent exists.
/// Amounts follow the CSS convention: `1.0` (or `100%`) means "no change"
/// for [`Filter::Brightness`], [`Filter::Contrast`] and [`Filter::Saturate`],
/// while [`Filter::Grayscale`]/[`Filter::Invert`] use a `0.0..=1.0` blend
/// amount. [`Filter::Gamma`] has no CSS equivalent; it applies a power-law
/// curve (`color^(1/gamma)`), useful for correcting perceived brightness.
///
/// Filters are applied to raw, straight-alpha RGBA color; widgets with no
/// filter set never touch this code path at all (see
/// [`crate::StyleBuilder::filter`]).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Filter {
    /// Gaussian blur. `radius` is roughly the CSS `blur(px)` standard
    /// deviation region; larger values sample more taps and cost more.
    Blur(Length),
    /// Linear brightness multiplier. `1.0` = unchanged, `0.0` = black.
    Brightness(f32),
    /// Contrast around the mid-gray pivot. `1.0` = unchanged.
    Contrast(f32),
    /// Saturation multiplier. `0.0` = grayscale, `1.0` = unchanged, `>1.0` oversaturates.
    Saturate(f32),
    /// Blends toward grayscale (Rec. 709 luma weights). `0.0..=1.0`.
    Grayscale(f32),
    /// Rotates hue by `degrees`, using the CSS Filter Effects hue-rotate matrix.
    HueRotate(f32),
    /// Blends toward the inverted color. `0.0..=1.0`.
    Invert(f32),
    /// Multiplies alpha. `0.0..=1.0`. Distinct from `Style::opacity`-style
    /// compositing since it runs inside the filter chain, before any
    /// subsequent filter in the same chain sees the result.
    Opacity(f32),
    /// Power-law gamma correction. `1.0` = unchanged.
    Gamma(f32),
    /// CSS-style `drop-shadow`: blurs the widget's own alpha silhouette,
    /// tints it `color`, offsets it, and composites it behind the
    /// (otherwise unfiltered-by-this-step) content.
    DropShadow(DropShadow),
}

impl Filter {
    /// Whether this filter needs spatial sampling (a real render pass with
    /// neighboring texels) as opposed to a pointwise per-pixel operation
    /// that can be folded into the single combined color pass.
    pub const fn requires_blur_pass(&self) -> bool {
        matches!(self, Filter::Blur(_) | Filter::DropShadow(_))
    }
}

/// An ordered list of [`Filter`]s applied to a widget's rendered output,
/// the way CSS applies a `filter: a(...) b(...) c(...)` list left to right.
///
/// Consecutive pointwise filters (brightness, contrast, saturate,
/// grayscale, hue-rotate, invert, opacity, gamma) are automatically fused
/// into a single GPU pass by the renderer. [`Filter::Blur`] and
/// [`Filter::DropShadow`] each require their own dedicated pass(es) and
/// therefore split the chain into segments.
///
/// # Example
/// ```no_run
/// use xengui::{FilterChain, Filter};
///
/// let chain = FilterChain::new()
///     .push(Filter::Grayscale(1.0))
///     .push(Filter::Brightness(1.1))
///     .push(Filter::Blur(4.0.into()));
/// ```
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FilterChain(Vec<Filter>);

impl FilterChain {
    /// Creates a value with its default configuration.
    pub const fn new() -> Self {
        Self(Vec::new())
    }

    /// Returns or updates the `push` value.
    pub fn push(mut self, filter: Filter) -> Self {
        self.0.push(filter);
        self
    }

    /// Returns whether the `is_empty` condition is satisfied.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns or updates the `iter` value.
    pub fn iter(&self) -> std::slice::Iter<'_, Filter> {
        self.0.iter()
    }

    /// The maximum blur radius (in logical px) contributed by any
    /// [`Filter::Blur`] or [`Filter::DropShadow`] in this chain. Used by
    /// the renderer to size the offscreen texture with enough padding
    /// that a blurred edge isn't clipped.
    pub fn max_blur_radius(&self) -> f32 {
        self.0
            .iter()
            .map(|f| match f {
                Filter::Blur(r) => r.value(),
                Filter::DropShadow(d) => {
                    d.blur_radius.value() + d.offset_x.value().abs().max(d.offset_y.value().abs())
                }
                _ => 0.0,
            })
            .fold(0.0, f32::max)
    }
}

impl From<Filter> for FilterChain {
    fn from(filter: Filter) -> Self {
        FilterChain::new().push(filter)
    }
}

impl From<Vec<Filter>> for FilterChain {
    fn from(filters: Vec<Filter>) -> Self {
        Self(filters)
    }
}
