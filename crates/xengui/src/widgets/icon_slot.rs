// SPDX-License-Identifier: Apache-2.0
//! Shared icon-rendering helper for widgets that paint a small optional
//! SVG glyph (Checkbox's check/dash, Switch's on/off icon), mapped into
//! an arbitrary square box at paint time. Default icons (from
//! xengui-lucide) are parsed once per process and shared via `Arc`
//! instead of being re-tessellated on every widget instance.
use crate::{ Color, PaintContext, TriangleCommand };
use std::sync::{ Arc, OnceLock };
use xen_svg::{ SvgDocument, SvgTriangle, parse_svg, tessellate_document };

static DEFAULT_CHECK: OnceLock<(Arc<SvgDocument>, Arc<Vec<SvgTriangle>>)> = OnceLock::new();
static DEFAULT_MINUS: OnceLock<(Arc<SvgDocument>, Arc<Vec<SvgTriangle>>)> = OnceLock::new();

fn parse_and_tessellate(source: &str) -> (Arc<SvgDocument>, Arc<Vec<SvgTriangle>>) {
    match parse_svg(source) {
        Ok(document) => {
            let triangles = Arc::new(tessellate_document(&document));
            (Arc::new(document), triangles)
        }
        Err(err) => {
            log::error!("IconSlot: svg parse error: {err}");
            (Arc::new(SvgDocument::default()), Arc::new(Vec::new()))
        }
    }
}

pub(super) struct IconSlot {
    enabled: bool,
    document: Option<Arc<SvgDocument>>,
    triangles: Arc<Vec<SvgTriangle>>,
}

impl IconSlot {
    pub(super) fn default_check() -> Self {
        let (document, triangles) = DEFAULT_CHECK.get_or_init(||
            parse_and_tessellate(xengui_lucide::CHECK_SVG)
        ).clone();
        Self { enabled: true, document: Some(document), triangles }
    }

    pub(super) fn default_minus() -> Self {
        let (document, triangles) = DEFAULT_MINUS.get_or_init(||
            parse_and_tessellate(xengui_lucide::MINUS_SVG)
        ).clone();
        Self { enabled: true, document: Some(document), triangles }
    }

    /// Replaces the icon with arbitrary SVG source - a custom icon or
    /// another `xengui-lucide` constant are both just a `&str` here.
    pub(super) fn set_svg(&mut self, source: &str) {
        let (document, triangles) = parse_and_tessellate(source);
        self.document = Some(document);
        self.triangles = triangles;
        self.enabled = true;
    }

    pub(super) fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub(super) fn paint(
        &self,
        ctx: &mut PaintContext,
        box_rect: (f32, f32, f32, f32),
        color: Color,
        opacity: f32
    ) {
        if !self.enabled || opacity <= 0.001 {
            return;
        }
        let Some(doc) = &self.document else {
            return;
        };
        let (vb_x, vb_y, vb_w, vb_h) = doc.view_box;
        if self.triangles.is_empty() || vb_w <= 0.0 || vb_h <= 0.0 {
            return;
        }

        let (bx, by, bw, bh) = box_rect;
        let scale = (bw / vb_w).min(bh / vb_h);
        let offset_x = bx + (bw - vb_w * scale) * 0.5;
        let offset_y = by + (bh - vb_h * scale) * 0.5;

        let inherited = xen_svg::Color::rgba_f32(color.r(), color.g(), color.b(), color.a());

        for triangle in self.triangles.iter() {
            let Some(paint) = triangle.paint.resolve(inherited) else {
                continue;
            };
            let paint = crate::svg_compat::from_svg_color(paint);
            let paint = paint.with_alpha_f32(paint.a() * triangle.opacity * opacity);

            let map = |p: (f32, f32)| -> (f32, f32) {
                (offset_x + (p.0 - vb_x) * scale, offset_y + (p.1 - vb_y) * scale)
            };

            ctx.draw_triangle(TriangleCommand {
                p0: map(triangle.p0),
                p1: map(triangle.p1),
                p2: map(triangle.p2),
                color: paint,
                clip_rect: None,
            });
        }
    }
}

impl PartialEq for IconSlot {
    fn eq(&self, other: &Self) -> bool {
        self.enabled == other.enabled && self.document == other.document
    }
}
