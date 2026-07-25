/// Maximum distance (in the SVG's own user-space units) between a curve
/// and its flattened approximation.
pub const TOLERANCE: f32 = 0.01;

/// Segment count used to approximate a full circle as a polygon before
/// handing it to lyon - lyon's tessellators only consume straight/bezier
/// path segments, not native arcs.
pub const CIRCLE_SEGMENTS: u32 = 128;

/// Segment count per rounded-rect corner, approximated the same way.
pub const CORNER_SEGMENTS: u32 = 32;
