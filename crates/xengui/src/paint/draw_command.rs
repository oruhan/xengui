// SPDX-License-Identifier: Apache-2.0
use crate::{Background, BorderRadius, Color, Length, ShadowDirection, Style};
use smol_str::SmolStr;
use std::sync::Arc;
use xengui_icons::IconAxes;

#[derive(Clone, Debug)]
/// Data and behavior represented by `RectCommand`.
pub struct RectCommand {
    /// The `position` value carried by this type.
    pub position: (f32, f32),
    /// The `size` value carried by this type.
    pub size: (f32, f32),
    /// The `background` value carried by this type.
    pub background: Option<Background>,
    /// The `border_radius` value carried by this type.
    pub border_radius: Option<BorderRadius>,
    /// The `border_width` value carried by this type.
    pub border_width: Option<Length>,
    /// The `border_color` value carried by this type.
    pub border_color: Option<Color>,
    /// The `clip_rect` value carried by this type.
    pub clip_rect: Option<(f32, f32, f32, f32)>,
}

#[derive(Clone, Debug)]
/// Data and behavior represented by `TextCommand`.
pub struct TextCommand {
    /// The `text` value carried by this type.
    pub text: SmolStr,
    /// The `position` value carried by this type.
    pub position: (f32, f32),
    /// The `style` value carried by this type.
    pub style: Style,
    /// The `max_width` value carried by this type.
    pub max_width: Option<f32>,
    /// The `clip_rect` value carried by this type.
    pub clip_rect: Option<(f32, f32, f32, f32)>,
}

#[derive(Clone, Debug)]
/// Data and behavior represented by `ImageData`.
pub struct ImageData {
    /// The `id` value carried by this type.
    pub id: u64,
    /// The `width` value carried by this type.
    pub width: u32,
    /// The `height` value carried by this type.
    pub height: u32,
    /// The `rgba` value carried by this type.
    pub rgba: Vec<u8>,
}

#[derive(Clone, Debug)]
/// Data and behavior represented by `ImageCommand`.
pub struct ImageCommand {
    /// The `position` value carried by this type.
    pub position: (f32, f32),
    /// The `size` value carried by this type.
    pub size: (f32, f32),
    /// The `image` value carried by this type.
    pub image: Arc<ImageData>,
    /// The `border_radius` value carried by this type.
    pub border_radius: Option<BorderRadius>,
    /// The `tint` value carried by this type.
    pub tint: Option<Color>,
    /// The `clip_rect` value carried by this type.
    pub clip_rect: Option<(f32, f32, f32, f32)>,
}

#[derive(Clone, Debug)]
/// Data and behavior represented by `TriangleCommand`.
pub struct TriangleCommand {
    /// The `p0` value carried by this type.
    pub p0: (f32, f32),
    /// The `p1` value carried by this type.
    pub p1: (f32, f32),
    /// The `p2` value carried by this type.
    pub p2: (f32, f32),
    /// The `color` value carried by this type.
    pub color: Color,
    /// The `clip_rect` value carried by this type.
    pub clip_rect: Option<(f32, f32, f32, f32)>,
}

#[derive(Clone, Debug)]
/// Data and behavior represented by `BoxShadowCommand`.
pub struct BoxShadowCommand {
    /// Rect used for the blurred rounded-rect SDF. For an outset shadow
    /// this is the box shifted by the offset and grown by the spread; for
    /// an inset shadow it's the box shifted/shrunk instead - the "light"
    /// rect the inner shadow appears to be cast from.
    pub shadow_position: (f32, f32),
    /// The `shadow_size` value carried by this type.
    pub shadow_size: (f32, f32),
    /// The `shadow_radius` value carried by this type.
    pub shadow_radius: [f32; 4],
    /// The `blur` value carried by this type.
    pub blur: f32,
    /// The `color` value carried by this type.
    pub color: Color,
    /// The `inset` value carried by this type.
    pub inset: bool,
    /// The widget's real box; an inset shadow is masked to stay inside it.
    pub box_position: (f32, f32),
    /// The `box_size` value carried by this type.
    pub box_size: (f32, f32),
    /// The `box_radius` value carried by this type.
    pub box_radius: f32,
    /// The `clip_rect` value carried by this type.
    pub clip_rect: Option<(f32, f32, f32, f32)>,
    /// The `direction` value carried by this type.
    pub direction: ShadowDirection,
}

#[derive(Clone, Debug)]
/// Data and behavior represented by `StrokeCommand`.
pub struct StrokeCommand {
    /// The `p0` value carried by this type.
    pub p0: (f32, f32),
    /// The `p1` value carried by this type.
    pub p1: (f32, f32),
    /// The `thickness` value carried by this type.
    pub thickness: f32,
    /// The `color` value carried by this type.
    pub color: Color,
    /// The `clip_rect` value carried by this type.
    pub clip_rect: Option<(f32, f32, f32, f32)>,
}

/// One rasterized Material Symbols (or any other variable-font) glyph
/// draw, resolved by the backend's own variable-icon pipeline instead of
/// the glyphon-backed text pipeline - the only path that can blend
/// FILL/wght/GRAD/opsz continuously.
#[derive(Clone, Debug)]
pub struct VariableIconCommand {
    /// The `position` value carried by this type.
    pub position: (f32, f32),
    /// The `size` value carried by this type.
    pub size: (f32, f32),
    /// The `codepoint` value carried by this type.
    pub codepoint: char,
    /// The `font` value carried by this type.
    pub font: &'static [u8],
    /// The `axes` value carried by this type.
    pub axes: IconAxes,
    /// The `color` value carried by this type.
    pub color: Color,
    /// The `clip_rect` value carried by this type.
    pub clip_rect: Option<(f32, f32, f32, f32)>,
}

/// A subtree's own draw commands, rendered in isolation to an offscreen
/// texture and processed through `chain` before being composited back
/// into the frame. Produced by `FrameRenderer` for any widget whose
/// `computed_style().filter` is set; consumed by `RenderBackend::draw_filtered`.
#[derive(Clone, Debug)]
pub struct FilteredCommand {
    /// The `commands` value carried by this type.
    pub commands: Vec<DrawCommand>,
    /// The `chain` value carried by this type.
    pub chain: crate::FilterChain,
    /// The widget's own layout box, in the *unfiltered* subtree's local
    /// paint coordinates (same space `commands` was recorded in).
    pub bounds: (f32, f32, f32, f32),
    /// The `clip_rect` value carried by this type.
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
    /// The `chain` value carried by this type.
    pub chain: crate::FilterChain,
    /// The `bounds` value carried by this type.
    pub bounds: (f32, f32, f32, f32),
    /// The `clip_rect` value carried by this type.
    pub clip_rect: Option<(f32, f32, f32, f32)>,
    /// The `radius` value carried by this type.
    pub radius: [f32; 4],
}

#[derive(Clone, Debug)]
/// Available `DrawCommand` choices.
pub enum DrawCommand {
    /// The `Rect` variant.
    Rect(RectCommand),
    /// The `Triangle` variant.
    Triangle(TriangleCommand),
    /// The `Text` variant.
    Text(Box<TextCommand>),
    /// The `Image` variant.
    Image(Box<ImageCommand>),
    /// The `BoxShadow` variant.
    BoxShadow(BoxShadowCommand),
    /// The `Stroke` variant.
    Stroke(StrokeCommand),
    /// The `Filtered` variant.
    Filtered(Box<FilteredCommand>),
    /// The `BackdropFilter` variant.
    BackdropFilter(Box<BackdropFilterCommand>),
    /// The `VariableIcon` variant.
    VariableIcon(Box<VariableIconCommand>),
}

// Converts a logical clip rect (top-left origin) into a physical scissor
// rect clamped to the surface bounds. `None` means the full surface.
/// Returns or updates the `scissor_for_clip` value.
pub fn scissor_for_clip(
    clip: Option<(f32, f32, f32, f32)>,
    surface_width: u32,
    surface_height: u32,
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
