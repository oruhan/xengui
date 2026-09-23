// SPDX-License-Identifier: Apache-2.0
use crate::{
    BoxShadowCommand, Color, ImageCommand, RectCommand, RippleCommand, StrokeCommand, SystemTheme,
    TextCommand, TextMeasurer, TriangleCommand, VariableIconCommand,
};
use std::fmt;

/// Optional rendering operations a backend may expose.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(missing_docs)]
pub enum BackendFeature {
    Ripple,
    Filter,
    BackdropFilter,
}

/// Features negotiated once a backend is created.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[allow(missing_docs)]
pub struct BackendCapabilities {
    pub ripple: bool,
    pub filter: bool,
    pub backdrop_filter: bool,
}

impl BackendCapabilities {
    /// A backend implementing every optional operation.
    pub const ALL: Self = Self {
        ripple: true,
        filter: true,
        backdrop_filter: true,
    };

    /// Returns whether `feature` was negotiated.
    pub const fn supports(self, feature: BackendFeature) -> bool {
        match feature {
            BackendFeature::Ripple => self.ripple,
            BackendFeature::Filter => self.filter,
            BackendFeature::BackdropFilter => self.backdrop_filter,
        }
    }
}

/// Application-selected behavior when an authored effect is unsupported.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[allow(missing_docs)]
pub enum UnsupportedFeaturePolicy {
    #[default]
    Strict,
    Fallback,
    Disabled,
}

/// Structured, non-fatal information emitted by a backend.
#[derive(Clone, Debug, PartialEq, Eq)]
#[allow(missing_docs)]
pub enum BackendDiagnostic {
    FeatureFallback { feature: BackendFeature },
    FeatureDisabled { feature: BackendFeature },
}

/// Failures produced while executing backend commands.
#[derive(Clone, Debug, PartialEq, Eq)]
#[allow(missing_docs)]
pub enum BackendError {
    UnsupportedFeature { feature: BackendFeature },
    Font(crate::FontError),
    Internal(String),
}

impl fmt::Display for BackendError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedFeature { feature } => {
                write!(f, "render feature is unsupported: {feature:?}")
            }
            Self::Font(error) => write!(f, "font rendering failed: {error}"),
            Self::Internal(message) => write!(f, "backend failure: {message}"),
        }
    }
}

impl std::error::Error for BackendError {}

/// Abstracts the GPU backend so xengui's core (layout, widgets,
/// reconciler, `FrameRenderer`) never depends on a concrete graphics API.
/// Implemented by `xengui-wgpu`; any other host (e.g. a Bevy render node)
/// can implement it too.
pub trait RenderBackend {
    /// Capabilities negotiated when the backend was initialized.
    fn capabilities(&self) -> BackendCapabilities;

    /// Policy for authored operations absent from [`Self::capabilities`].
    fn unsupported_feature_policy(&self) -> UnsupportedFeaturePolicy;

    /// Receives structured, non-fatal fallback/disabled notices.
    fn report_diagnostic(&mut self, diagnostic: BackendDiagnostic);
    /// Returns or updates the `text_measurer` value.
    fn text_measurer(&mut self) -> &mut dyn TextMeasurer;

    /// Prepares a new frame. Returning `false` skips the frame entirely
    /// (e.g. a native swapchain temporarily unavailable).
    fn begin_frame(&mut self, background: Color, width: u32, height: u32) -> bool;

    /// Returns or updates the `draw_rects` value.
    fn draw_rects(&mut self, cmds: &[RectCommand]);
    /// Draws procedural patterned ripple commands. Backends without a
    /// dedicated implementation may ignore them.
    fn draw_ripples(&mut self, cmds: &[RippleCommand]) -> Result<(), BackendError>;
    /// Returns or updates the `draw_triangles` value.
    fn draw_triangles(&mut self, cmds: &[TriangleCommand]);
    /// Returns or updates the `draw_images` value.
    fn draw_images(&mut self, cmds: &[ImageCommand]);
    /// Returns or updates the `draw_box_shadows` value.
    fn draw_box_shadows(&mut self, cmds: &[BoxShadowCommand]);
    /// Returns or updates the `draw_strokes` value.
    fn draw_strokes(&mut self, cmds: &[StrokeCommand]);
    /// Returns or updates the `draw_variable_icons` value.
    fn draw_variable_icons(&mut self, cmds: &[VariableIconCommand]) -> Result<(), BackendError>;
    /// Returns or updates the `draw_text` value.
    fn draw_text(&mut self, theme: SystemTheme, scale_factor: f32, cmd: &TextCommand);

    /// Rasterizes `cmd.commands` at their natural metrics and applies the
    /// visual scale only while compositing the resulting texture. Backends
    /// should keep text/SVG rasterization independent from `cmd.scale`.
    fn draw_composited(&mut self, cmd: &crate::CompositedCommand) -> Result<(), BackendError>;

    /// Renders `cmds` in isolation, runs `chain` over the result, and
    /// composites the filtered output at `bounds`. Backends without
    /// filter support may implement this as a no-op fallback that paints
    /// `cmds` directly (unfiltered) - correctness over a hard failure.
    fn draw_filtered(
        &mut self,
        cmds: &[crate::DrawCommand],
        chain: &crate::FilterChain,
        bounds: (f32, f32, f32, f32),
        clip_rect: Option<(f32, f32, f32, f32)>,
    ) -> Result<(), BackendError>;

    /// Captures whatever has already been painted within `bounds` at this
    /// point in the frame, runs `chain` over that live snapshot, and
    /// composites the blurred result back in place - matches CSS
    /// `backdrop-filter`. Backends that can't read back the frame in
    /// progress may implement this as a no-op; the widget's own
    /// background/content still paints normally afterward, it just won't
    /// show a blurred backdrop underneath.
    fn draw_backdrop_filtered(
        &mut self,
        _chain: &crate::FilterChain,
        _bounds: (f32, f32, f32, f32),
        _clip_rect: Option<(f32, f32, f32, f32)>,
        _radius: [f32; 4],
    ) -> Result<(), BackendError>;

    /// Drains underline/strike/overline rects queued by `draw_text` calls
    /// since the last call to this method.
    fn take_text_decorations(&mut self) -> Vec<RectCommand>;

    /// Drains text decorations into caller-owned frame scratch storage.
    ///
    /// Backends should override this when they can transfer elements while
    /// retaining their internal allocation. The default preserves source
    /// compatibility for third-party backends.
    fn drain_text_decorations(&mut self, out: &mut Vec<RectCommand>) {
        out.extend(self.take_text_decorations());
    }

    /// Flushes queued text to the GPU. Must be called after every
    /// `draw_text` and before anything meant to render above text
    /// (e.g. a focus ring).
    fn flush_text(&mut self) -> Result<(), BackendError>;

    /// Submits/presents the frame prepared by `begin_frame`.
    fn end_frame(&mut self);

    /// Returns or updates the `resize` value.
    fn resize(&mut self, width: u32, height: u32);
}

#[cfg(test)]
mod tests {
    use super::{BackendCapabilities, BackendFeature, UnsupportedFeaturePolicy};

    #[test]
    fn capabilities_are_explicit_per_feature() {
        let capabilities = BackendCapabilities {
            ripple: true,
            filter: false,
            backdrop_filter: false,
        };
        assert!(capabilities.supports(BackendFeature::Ripple));
        assert!(!capabilities.supports(BackendFeature::Filter));
        assert!(!capabilities.supports(BackendFeature::BackdropFilter));
    }

    #[test]
    fn unsupported_features_default_to_strict() {
        assert_eq!(
            UnsupportedFeaturePolicy::default(),
            UnsupportedFeaturePolicy::Strict
        );
    }
}
