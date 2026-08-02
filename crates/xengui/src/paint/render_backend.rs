// SPDX-License-Identifier: Apache-2.0
use crate::{
    BoxShadowCommand,
    Color,
    ImageCommand,
    RectCommand,
    SystemTheme,
    TextCommand,
    TextMeasurer,
    TriangleCommand,
};

/// Abstracts the GPU backend so xengui's core (layout, widgets,
/// reconciler, `FrameRenderer`) never depends on a concrete graphics API.
/// Implemented by `xengui-wgpu`; any other host (e.g. a Bevy render node)
/// can implement it too.
pub trait RenderBackend {
    fn text_measurer(&mut self) -> &mut dyn TextMeasurer;

    /// Prepares a new frame. Returning `false` skips the frame entirely
    /// (e.g. a native swapchain temporarily unavailable).
    fn begin_frame(&mut self, background: Color, width: u32, height: u32) -> bool;

    fn draw_rects(&mut self, cmds: &[RectCommand]);
    fn draw_triangles(&mut self, cmds: &[TriangleCommand]);
    fn draw_images(&mut self, cmds: &[ImageCommand]);
    fn draw_box_shadows(&mut self, cmds: &[BoxShadowCommand]);
    fn draw_text(&mut self, theme: SystemTheme, scale_factor: f32, cmd: &TextCommand);

    /// Renders `cmds` in isolation, runs `chain` over the result, and
    /// composites the filtered output at `bounds`. Backends without
    /// filter support may implement this as a no-op fallback that paints
    /// `cmds` directly (unfiltered) - correctness over a hard failure.
    fn draw_filtered(
        &mut self,
        cmds: &[crate::DrawCommand],
        chain: &crate::FilterChain,
        bounds: (f32, f32, f32, f32)
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
        _clip_rect: Option<(f32, f32, f32, f32)>
    ) {}

    /// Drains underline/strike/overline rects queued by `draw_text` calls
    /// since the last call to this method.
    fn take_text_decorations(&mut self) -> Vec<RectCommand>;

    /// Flushes queued text to the GPU. Must be called after every
    /// `draw_text` and before anything meant to render above text
    /// (e.g. a focus ring).
    fn flush_text(&mut self);

    /// Submits/presents the frame prepared by `begin_frame`.
    fn end_frame(&mut self);

    fn resize(&mut self, width: u32, height: u32);
}
