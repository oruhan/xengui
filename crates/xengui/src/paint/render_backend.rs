// SPDX-License-Identifier: Apache-2.0
use crate::{
    BoxShadowCommand, Color, ImageCommand, RectCommand, StrokeCommand, SystemTheme, TextCommand,
    TextMeasurer, TriangleCommand, VariableIconCommand,
};

/// Abstracts the GPU backend so xengui's core (layout, widgets,
/// reconciler, `FrameRenderer`) never depends on a concrete graphics API.
/// Implemented by `xengui-wgpu`; any other host (e.g. a Bevy render node)
/// can implement it too.
pub trait RenderBackend {
    /// Returns or updates the `text_measurer` value.
    fn text_measurer(&mut self) -> &mut dyn TextMeasurer;

    /// Prepares a new frame. Returning `false` skips the frame entirely
    /// (e.g. a native swapchain temporarily unavailable).
    fn begin_frame(&mut self, background: Color, width: u32, height: u32) -> bool;

    /// Returns or updates the `draw_rects` value.
    fn draw_rects(&mut self, cmds: &[RectCommand]);
    /// Returns or updates the `draw_triangles` value.
    fn draw_triangles(&mut self, cmds: &[TriangleCommand]);
    /// Returns or updates the `draw_images` value.
    fn draw_images(&mut self, cmds: &[ImageCommand]);
    /// Returns or updates the `draw_box_shadows` value.
    fn draw_box_shadows(&mut self, cmds: &[BoxShadowCommand]);
    /// Returns or updates the `draw_strokes` value.
    fn draw_strokes(&mut self, cmds: &[StrokeCommand]);
    /// Returns or updates the `draw_variable_icons` value.
    fn draw_variable_icons(&mut self, cmds: &[VariableIconCommand]);
    /// Returns or updates the `draw_text` value.
    fn draw_text(&mut self, theme: SystemTheme, scale_factor: f32, cmd: &TextCommand);

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
    );

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
    ) {
    }

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
    fn flush_text(&mut self);

    /// Submits/presents the frame prepared by `begin_frame`.
    fn end_frame(&mut self);

    /// Returns or updates the `resize` value.
    fn resize(&mut self, width: u32, height: u32);
}
