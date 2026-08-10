// SPDX-License-Identifier: Apache-2.0
use crate::{ Background, BorderRadius, Color, Length, ShadowDirection, Style };
use smol_str::SmolStr;
use xengui_icons::IconAxes;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct RectCommand {
    pub position: (f32, f32),
    pub size: (f32, f32),
    pub background: Option<Background>,
    pub border_radius: Option<BorderRadius>,
    pub border_width: Option<Length>,
    pub border_color: Option<Color>,
    pub clip_rect: Option<(f32, f32, f32, f32)>,
}

#[derive(Clone, Debug)]
pub struct TextCommand {
    pub text: SmolStr,
    pub position: (f32, f32),
    pub style: Style,
    pub max_width: Option<f32>,
    pub clip_rect: Option<(f32, f32, f32, f32)>,
}

#[derive(Clone, Debug)]
pub struct ImageData {
    pub id: u64,
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

#[derive(Clone, Debug)]
pub struct ImageCommand {
    pub position: (f32, f32),
    pub size: (f32, f32),
    pub image: Arc<ImageData>,
    pub border_radius: Option<BorderRadius>,
    pub tint: Option<Color>,
    pub clip_rect: Option<(f32, f32, f32, f32)>,
}

#[derive(Clone, Debug)]
pub struct TriangleCommand {
    pub p0: (f32, f32),
    pub p1: (f32, f32),
    pub p2: (f32, f32),
    pub color: Color,
    pub clip_rect: Option<(f32, f32, f32, f32)>,
}

#[derive(Clone, Debug)]
pub struct BoxShadowCommand {
    /// Rect used for the blurred rounded-rect SDF. For an outset shadow
    /// this is the box shifted by the offset and grown by the spread; for
    /// an inset shadow it's the box shifted/shrunk instead - the "light"
    /// rect the inner shadow appears to be cast from.
    pub shadow_position: (f32, f32),
    pub shadow_size: (f32, f32),
    pub shadow_radius: [f32; 4],
    pub blur: f32,
    pub color: Color,
    pub inset: bool,
    /// The widget's real box; an inset shadow is masked to stay inside it.
    pub box_position: (f32, f32),
    pub box_size: (f32, f32),
    pub box_radius: f32,
    pub clip_rect: Option<(f32, f32, f32, f32)>,
    pub direction: ShadowDirection,
}

#[derive(Clone, Debug)]
pub struct StrokeCommand {
    pub p0: (f32, f32),
    pub p1: (f32, f32),
    pub thickness: f32,
    pub color: Color,
    pub clip_rect: Option<(f32, f32, f32, f32)>,
}

/// One rasterized Material Symbols (or any other variable-font) glyph
/// draw, resolved by the backend's own variable-icon pipeline instead of
/// the glyphon-backed text pipeline - the only path that can blend
/// FILL/wght/GRAD/opsz continuously.
#[derive(Clone, Debug)]
pub struct VariableIconCommand {
    pub position: (f32, f32),
    pub size: (f32, f32),
    pub codepoint: char,
    pub font: &'static [u8],
    pub axes: IconAxes,
    pub color: Color,
    pub clip_rect: Option<(f32, f32, f32, f32)>,
}

/// A subtree's own draw commands, rendered in isolation to an offscreen
/// texture and processed through `chain` before being composited back
/// into the frame. Produced by `FrameRenderer` for any widget whose
/// `computed_style().filter` is set; consumed by `RenderBackend::draw_filtered`.
#[derive(Clone, Debug)]
pub struct FilteredCommand {
    pub commands: Vec<DrawCommand>,
    pub chain: crate::FilterChain,
    /// The widget's own layout box, in the *unfiltered* subtree's local
    /// paint coordinates (same space `commands` was recorded in).
    pub bounds: (f32, f32, f32, f32),
    pub clip_rect: Option<(f32, f32, f32, f32)>,
}

/// A live snapshot-and-filter pass: captures whatever has already been
/// painted within `bounds` so far this frame, runs `chain` (typically a
/// blur) over that snapshot, and composites the result back at the same
/// spot - matches CSS `backdrop-filter`. Unlike `FilteredCommand`, this
/// carries no commands of its own; it only reads the frame as it stands
/// at this point in paint order. Produced by `FrameRenderer` for any
/// widget whose `computed_style().backdrop_filter` is set.
#[derive(Clone, Debug)]
pub struct BackdropFilterCommand {
    pub chain: crate::FilterChain,
    pub bounds: (f32, f32, f32, f32),
    pub clip_rect: Option<(f32, f32, f32, f32)>,
}

#[derive(Clone, Debug)]
pub enum DrawCommand {
    Rect(RectCommand),
    Triangle(TriangleCommand),
    Text(Box<TextCommand>),
    Image(Box<ImageCommand>),
    BoxShadow(BoxShadowCommand),
    Stroke(StrokeCommand),
    Filtered(Box<FilteredCommand>),
    BackdropFilter(Box<BackdropFilterCommand>),
    VariableIcon(Box<VariableIconCommand>),
}

// Converts a logical clip rect (top-left origin) into a physical scissor
// rect clamped to the surface bounds. `None` means the full surface.
pub fn scissor_for_clip(
    clip: Option<(f32, f32, f32, f32)>,
    surface_width: u32,
    surface_height: u32
) -> (u32, u32, u32, u32) {
    let Some((x, y, w, h)) = clip else {
        return (0, 0, surface_width, surface_height);
    };

    let x0 = x.max(0.0).min(surface_width as f32);
    let y0 = y.max(0.0).min(surface_height as f32);
    let x1 = (x + w).max(0.0).min(surface_width as f32);
    let y1 = (y + h).max(0.0).min(surface_height as f32);

    // Each edge is rounded independently and width/height derived from the
    // rounded edges, instead of rounding the origin and the span on their
    // own - the latter can round both up and overshoot the (already
    // clamped) surface bound by a texel.
    let x0 = x0.round() as u32;
    let y0 = y0.round() as u32;
    let x1 = x1.round() as u32;
    let y1 = y1.round() as u32;

    (x0, y0, x1.saturating_sub(x0), y1.saturating_sub(y0))
}
