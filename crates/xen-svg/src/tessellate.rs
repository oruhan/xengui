use crate::{ CIRCLE_SEGMENTS, CORNER_SEGMENTS, TOLERANCE };

// SPDX-License-Identifier: Apache-2.0
use super::{
    FillRule as SvgFillRule,
    LineCap as SvgLineCap,
    LineJoin as SvgLineJoin,
    PathCommand,
    SvgAttributes,
    SvgColor,
    SvgDocument,
    SvgElement,
    Transform2D,
};
use lyon::math::{ point, Point };
use lyon::path::Path;
use lyon::tessellation::{
    BuffersBuilder,
    FillOptions,
    FillRule,
    FillTessellator,
    FillVertex,
    LineCap,
    LineJoin,
    StrokeOptions,
    StrokeTessellator,
    StrokeVertex,
    VertexBuffers,
};

/// A single filled triangle in the SVG's own `viewBox` coordinate space,
/// tagged with the paint it should be drawn with.
#[derive(Clone, Copy, Debug)]
pub struct SvgTriangle {
    pub p0: (f32, f32),
    pub p1: (f32, f32),
    pub p2: (f32, f32),
    pub paint: SvgColor,
    pub opacity: f32,
}

/// Antialiasing fringe width, in the element's own local (viewBox) units.
const AA_WIDTH: f32 = 1.1;
/// Number of stepped opacity bands the fringe is built from - more bands
/// approximate a smooth gradient more closely at the cost of more
/// triangles.
const AA_BANDS: u32 = 8;
/// Curve flattening granularity used only for the antialiasing fringe
/// (the fill/stroke tessellation itself still goes through lyon).
const AA_CURVE_SEGMENTS: u32 = 16;

/// Flattens an entire document into a triangle list, ready to be scaled
/// into a widget's layout box and handed to the triangle pipeline.
pub fn tessellate_document(doc: &SvgDocument) -> Vec<SvgTriangle> {
    let mut out = Vec::new();
    for element in &doc.elements {
        tessellate_element(element, Transform2D::IDENTITY, 1.0, &mut out);
    }
    out
}

fn tessellate_element(
    element: &SvgElement,
    parent_transform: Transform2D,
    parent_opacity: f32,
    out: &mut Vec<SvgTriangle>
) {
    // Element's own local transform must apply first, then the accumulated
    // ancestor chain - not the other way around.
    let transform = element.attrs().transform.then(parent_transform);
    let opacity = parent_opacity * element.attrs().opacity;

    let scale = transform_scale(transform);

    match element {
        SvgElement::Group { children, .. } => {
            for child in children {
                tessellate_element(child, transform, opacity, out);
            }
        }
        SvgElement::Path { commands, attrs } => {
            let path = build_path_from_commands(commands, transform);
            emit_shape(&path, attrs, opacity, scale, out);

            if !matches!(attrs.fill, SvgColor::None) {
                let loops = map_loops(&flatten_path_commands(commands), transform);
                add_fill_aa_fringe(&loops, attrs.fill, opacity, out);
            }
        }
        SvgElement::Rect { x, y, width, height, rx, attrs } => {
            let polygon = rect_polygon(*x, *y, *width, *height, *rx);
            let path = build_polygon_path(&polygon, true, transform);
            emit_shape(&path, attrs, opacity, scale, out);

            if !matches!(attrs.fill, SvgColor::None) {
                let mapped = map_points(&polygon, transform);
                add_fill_aa_fringe(&[mapped], attrs.fill, opacity, out);
            }
        }
        SvgElement::Circle { cx, cy, r, attrs } => {
            let polygon = circle_polygon(*cx, *cy, *r);
            let path = build_polygon_path(&polygon, true, transform);
            emit_shape(&path, attrs, opacity, scale, out);

            if !matches!(attrs.fill, SvgColor::None) {
                let mapped = map_points(&polygon, transform);
                add_fill_aa_fringe(&[mapped], attrs.fill, opacity, out);
            }
        }
        SvgElement::Line { x1, y1, x2, y2, attrs } => {
            let path = build_polygon_path(
                &[
                    (*x1, *y1),
                    (*x2, *y2),
                ],
                false,
                transform
            );
            emit_stroke(&path, attrs, opacity, scale, out);
        }
    }
}

// Approximate uniform scale of an affine transform, used to keep stroke
// width consistent with geometry already baked in by a `transform="..."` attribute.
fn transform_scale(t: Transform2D) -> f32 {
    let sx = (t.a * t.a + t.b * t.b).sqrt();
    let sy = (t.c * t.c + t.d * t.d).sqrt();
    (sx + sy) * 0.5
}

fn map_point(transform: Transform2D, x: f32, y: f32) -> Point {
    let (tx, ty) = transform.apply(x, y);
    point(tx, ty)
}

fn map_points(points: &[(f32, f32)], transform: Transform2D) -> Vec<(f32, f32)> {
    points
        .iter()
        .map(|&(x, y)| transform.apply(x, y))
        .collect()
}

fn map_loops(loops: &[Vec<(f32, f32)>], transform: Transform2D) -> Vec<Vec<(f32, f32)>> {
    loops
        .iter()
        .map(|points| map_points(points, transform))
        .collect()
}

// Flattens a path's commands (bezier segments included) into closed point
// loops, in the element's own local (untransformed) coordinate space -
// used only to build the antialiasing fringe, independent of lyon's own
// internal flattening so this stays self-contained.
fn flatten_path_commands(commands: &[PathCommand]) -> Vec<Vec<(f32, f32)>> {
    let mut loops = Vec::new();
    let mut current: Vec<(f32, f32)> = Vec::new();
    let mut cursor = (0.0, 0.0);

    fn push_point(current: &mut Vec<(f32, f32)>, p: (f32, f32)) {
        if current.last() != Some(&p) {
            current.push(p);
        }
    }

    for command in commands {
        match *command {
            PathCommand::MoveTo(x, y) => {
                if current.len() > 1 {
                    loops.push(std::mem::take(&mut current));
                } else {
                    current.clear();
                }
                cursor = (x, y);
                current.push(cursor);
            }
            PathCommand::LineTo(x, y) => {
                cursor = (x, y);
                push_point(&mut current, cursor);
            }
            PathCommand::QuadTo(cx, cy, x, y) => {
                let p0 = cursor;
                for i in 1..=AA_CURVE_SEGMENTS {
                    let t = (i as f32) / (AA_CURVE_SEGMENTS as f32);
                    let mt = 1.0 - t;
                    let px = mt * mt * p0.0 + 2.0 * mt * t * cx + t * t * x;
                    let py = mt * mt * p0.1 + 2.0 * mt * t * cy + t * t * y;
                    push_point(&mut current, (px, py));
                }
                cursor = (x, y);
            }
            PathCommand::CubicTo(c1x, c1y, c2x, c2y, x, y) => {
                let p0 = cursor;
                for i in 1..=AA_CURVE_SEGMENTS {
                    let t = (i as f32) / (AA_CURVE_SEGMENTS as f32);
                    let mt = 1.0 - t;
                    let px =
                        mt * mt * mt * p0.0 +
                        3.0 * mt * mt * t * c1x +
                        3.0 * mt * t * t * c2x +
                        t * t * t * x;
                    let py =
                        mt * mt * mt * p0.1 +
                        3.0 * mt * mt * t * c1y +
                        3.0 * mt * t * t * c2y +
                        t * t * t * y;
                    push_point(&mut current, (px, py));
                }
                cursor = (x, y);
            }
            PathCommand::Close => {
                if current.len() > 1 {
                    loops.push(std::mem::take(&mut current));
                } else {
                    current.clear();
                }
            }
        }
    }
    if current.len() > 1 {
        loops.push(current);
    }

    loops
}

// Extrudes a thin, opacity-stepped quad along one edge's normal - the
// basic building block of the antialiasing fringe.
#[allow(clippy::too_many_arguments)]
fn push_quad_band(
    a: (f32, f32),
    b: (f32, f32),
    normal: (f32, f32),
    w0: f32,
    w1: f32,
    paint: SvgColor,
    opacity: f32,
    out: &mut Vec<SvgTriangle>
) {
    let ext = |p: (f32, f32), w: f32| (p.0 + normal.0 * w, p.1 + normal.1 * w);
    let a0 = ext(a, w0);
    let a1 = ext(a, w1);
    let b0 = ext(b, w0);
    let b1 = ext(b, w1);

    out.push(SvgTriangle { p0: a0, p1: a1, p2: b1, paint, opacity });
    out.push(SvgTriangle { p0: a0, p1: b1, p2: b0, paint, opacity });
}

// Builds a soft edge around every polygon loop by extruding thin,
// progressively more transparent quads along each edge, symmetrically on
// both sides of it. The half extruded inward lands on top of the
// already-opaque fill of the same color, so its translucency is
// invisible there - alpha-blending translucent color C over opaque C
// leaves C unchanged. Only the outward half is a visible fringe, which is
// what lets this skip figuring out the polygon's winding/outward
// direction entirely.
fn add_fill_aa_fringe(
    loops: &[Vec<(f32, f32)>],
    paint: SvgColor,
    opacity: f32,
    out: &mut Vec<SvgTriangle>
) {
    if matches!(paint, SvgColor::None) {
        return;
    }

    for points in loops {
        if points.len() < 2 {
            continue;
        }

        for i in 0..points.len() {
            let a = points[i];
            let b = points[(i + 1) % points.len()];
            let (dx, dy) = (b.0 - a.0, b.1 - a.1);
            let len = (dx * dx + dy * dy).sqrt();
            if len < 0.0001 {
                continue;
            }
            let normal = (-dy / len, dx / len);

            for band in 0..AA_BANDS {
                let t0 = (band as f32) / (AA_BANDS as f32);
                let t1 = ((band + 1) as f32) / (AA_BANDS as f32);
                let w0 = AA_WIDTH * t0;
                let w1 = AA_WIDTH * t1;
                let band_opacity = opacity * (1.0 - (t0 + t1) * 0.5);

                push_quad_band(a, b, normal, w0, w1, paint, band_opacity, out);
                push_quad_band(a, b, (-normal.0, -normal.1), w0, w1, paint, band_opacity, out);
            }
        }
    }
}

fn build_path_from_commands(commands: &[PathCommand], transform: Transform2D) -> Path {
    let mut builder = Path::builder();
    let mut in_subpath = false;

    for command in commands {
        match *command {
            PathCommand::MoveTo(x, y) => {
                if in_subpath {
                    builder.end(false);
                }
                builder.begin(map_point(transform, x, y));
                in_subpath = true;
            }
            PathCommand::LineTo(x, y) => {
                builder.line_to(map_point(transform, x, y));
            }
            PathCommand::QuadTo(cx, cy, x, y) => {
                builder.quadratic_bezier_to(
                    map_point(transform, cx, cy),
                    map_point(transform, x, y)
                );
            }
            PathCommand::CubicTo(c1x, c1y, c2x, c2y, x, y) => {
                builder.cubic_bezier_to(
                    map_point(transform, c1x, c1y),
                    map_point(transform, c2x, c2y),
                    map_point(transform, x, y)
                );
            }
            PathCommand::Close => {
                builder.close();
                in_subpath = false;
            }
        }
    }

    if in_subpath {
        builder.end(false);
    }

    builder.build()
}

fn build_polygon_path(points: &[(f32, f32)], closed: bool, transform: Transform2D) -> Path {
    let mut builder = Path::builder();
    let mut iter = points.iter();

    let Some(&(x0, y0)) = iter.next() else {
        return builder.build();
    };

    builder.begin(map_point(transform, x0, y0));
    for &(x, y) in iter {
        builder.line_to(map_point(transform, x, y));
    }
    builder.end(closed);

    builder.build()
}

fn emit_shape(
    path: &Path,
    attrs: &SvgAttributes,
    opacity: f32,
    scale: f32,
    out: &mut Vec<SvgTriangle>
) {
    if !matches!(attrs.fill, SvgColor::None) {
        tessellate_fill(path, attrs, opacity, out);
    }
    emit_stroke(path, attrs, opacity, scale, out);
}

fn emit_stroke(
    path: &Path,
    attrs: &SvgAttributes,
    opacity: f32,
    scale: f32,
    out: &mut Vec<SvgTriangle>
) {
    if !matches!(attrs.stroke, SvgColor::None) && attrs.stroke_width > 0.0 {
        tessellate_stroke(path, attrs, opacity, scale, out);
    }
}

fn tessellate_fill(path: &Path, attrs: &SvgAttributes, opacity: f32, out: &mut Vec<SvgTriangle>) {
    let mut geometry: VertexBuffers<[f32; 2], u16> = VertexBuffers::new();
    let options = FillOptions::default()
        .with_tolerance(TOLERANCE)
        .with_fill_rule(map_fill_rule(attrs.fill_rule));

    let result = FillTessellator::new().tessellate_path(
        path,
        &options,
        &mut BuffersBuilder::new(&mut geometry, |vertex: FillVertex| {
            vertex.position().to_array()
        })
    );

    if let Err(err) = result {
        log::error!("xen-svg: fill tessellation failed: {err:?}");
        return;
    }

    push_triangles(&geometry, attrs.fill, opacity, out);
}

fn tessellate_stroke(
    path: &Path,
    attrs: &SvgAttributes,
    opacity: f32,
    scale: f32,
    out: &mut Vec<SvgTriangle>
) {
    let mut geometry: VertexBuffers<[f32; 2], u16> = VertexBuffers::new();
    let options = StrokeOptions::default()
        .with_tolerance(TOLERANCE)
        // stroke_width lives in the element's own coordinate space, so it
        // must scale together with the already-transformed path it strokes.
        .with_line_width(attrs.stroke_width * scale)
        .with_line_join(map_line_join(attrs.line_join))
        .with_line_cap(map_line_cap(attrs.line_cap))
        .with_miter_limit(attrs.miter_limit);

    let result = StrokeTessellator::new().tessellate_path(
        path,
        &options,
        &mut BuffersBuilder::new(&mut geometry, |vertex: StrokeVertex| {
            vertex.position().to_array()
        })
    );

    if let Err(err) = result {
        log::error!("xen-svg: stroke tessellation failed: {err:?}");
        return;
    }

    push_triangles(&geometry, attrs.stroke, opacity, out);
}

fn map_line_cap(cap: SvgLineCap) -> LineCap {
    match cap {
        SvgLineCap::Butt => LineCap::Butt,
        SvgLineCap::Round => LineCap::Round,
        SvgLineCap::Square => LineCap::Square,
    }
}

fn map_line_join(join: SvgLineJoin) -> LineJoin {
    match join {
        SvgLineJoin::Miter => LineJoin::Miter,
        SvgLineJoin::Round => LineJoin::Round,
        SvgLineJoin::Bevel => LineJoin::Bevel,
    }
}

fn map_fill_rule(rule: SvgFillRule) -> FillRule {
    match rule {
        SvgFillRule::NonZero => FillRule::NonZero,
        SvgFillRule::EvenOdd => FillRule::EvenOdd,
    }
}

fn push_triangles(
    geometry: &VertexBuffers<[f32; 2], u16>,
    paint: SvgColor,
    opacity: f32,
    out: &mut Vec<SvgTriangle>
) {
    for tri in geometry.indices.chunks_exact(3) {
        let p0 = geometry.vertices[tri[0] as usize];
        let p1 = geometry.vertices[tri[1] as usize];
        let p2 = geometry.vertices[tri[2] as usize];
        out.push(SvgTriangle {
            p0: (p0[0], p0[1]),
            p1: (p1[0], p1[1]),
            p2: (p2[0], p2[1]),
            paint,
            opacity,
        });
    }
}

fn rect_polygon(x: f32, y: f32, width: f32, height: f32, rx: f32) -> Vec<(f32, f32)> {
    if rx <= 0.0 {
        return vec![(x, y), (x + width, y), (x + width, y + height), (x, y + height)];
    }

    let r = rx.min(width * 0.5).min(height * 0.5);
    let mut points = Vec::new();
    let corners = [
        (x + width - r, y + r, -90.0f32, 0.0f32),
        (x + width - r, y + height - r, 0.0, 90.0),
        (x + r, y + height - r, 90.0, 180.0),
        (x + r, y + r, 180.0, 270.0),
    ];
    for &(cx, cy, start_deg, end_deg) in &corners {
        for i in 0..=CORNER_SEGMENTS {
            let t = start_deg + (end_deg - start_deg) * ((i as f32) / (CORNER_SEGMENTS as f32));
            let rad = t.to_radians();
            points.push((cx + rad.cos() * r, cy + rad.sin() * r));
        }
    }
    points
}

fn circle_polygon(cx: f32, cy: f32, r: f32) -> Vec<(f32, f32)> {
    (0..CIRCLE_SEGMENTS)
        .map(|i| {
            let t = ((i as f32) / (CIRCLE_SEGMENTS as f32)) * std::f32::consts::TAU;
            (cx + t.cos() * r, cy + t.sin() * r)
        })
        .collect()
}
