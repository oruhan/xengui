// SPDX-License-Identifier: Apache-2.0
use crate::{
    AnimationManager, Constraints, EventCtx, EventStatus, ImageCommand, ImageSource, InputEvent,
    Interaction, LayoutBox, MeasureContext, MeasureResult, PaintContext, Style, StyleBuilder,
    TriangleCommand, Widget, WidgetBase, WidgetId, image_source_from_rgba8,
    svg_compat::{IntoSvgColor, from_svg_color},
};
use smol_str::SmolStr;
use std::sync::Arc;
#[cfg(not(target_arch = "wasm32"))]
use xen_svg::SvgImageSource;
use xen_svg::{
    PathCommand, SvgAttributes, SvgDocument, SvgDrawOp, SvgElement, SvgTriangle, Transform2D,
    collect_draw_ops, parse_svg,
};

macro_rules! impl_svg_attrs_builder {
    ($ty:ident) => {
        impl $ty {
            /// Returns or updates the `fill` value.
            pub fn fill(mut self, color: impl IntoSvgColor) -> Self {
                self.attrs.fill = color.into_svg_color();
                self
            }

            /// Returns or updates the `stroke` value.
            pub fn stroke(mut self, color: impl IntoSvgColor) -> Self {
                self.attrs.stroke = color.into_svg_color();
                self
            }

            /// Returns or updates the `stroke_width` value.
            pub fn stroke_width(mut self, width: f32) -> Self {
                self.attrs.stroke_width = width;
                self
            }

            /// Returns or updates the `opacity` value.
            pub fn opacity(mut self, opacity: f32) -> Self {
                self.attrs.opacity = opacity;
                self
            }

            /// Returns or updates the `transform` value.
            pub fn transform(mut self, transform: Transform2D) -> Self {
                self.attrs.transform = transform;
                self
            }
        }
    };
}

/// Data and behavior represented by `SvgPathBuilder`.
pub struct SvgPathBuilder {
    commands: Vec<PathCommand>,
    attrs: SvgAttributes,
}

impl SvgPathBuilder {
    /// Creates a value with its default configuration.
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
            attrs: SvgAttributes::default(),
        }
    }

    /// Returns or updates the `move_to` value.
    pub fn move_to(mut self, x: f32, y: f32) -> Self {
        self.commands.push(PathCommand::MoveTo(x, y));
        self
    }

    /// Returns or updates the `line_to` value.
    pub fn line_to(mut self, x: f32, y: f32) -> Self {
        self.commands.push(PathCommand::LineTo(x, y));
        self
    }

    /// Returns or updates the `quad_to` value.
    pub fn quad_to(mut self, cx: f32, cy: f32, x: f32, y: f32) -> Self {
        self.commands.push(PathCommand::QuadTo(cx, cy, x, y));
        self
    }

    /// Returns or updates the `cubic_to` value.
    pub fn cubic_to(mut self, c1x: f32, c1y: f32, c2x: f32, c2y: f32, x: f32, y: f32) -> Self {
        self.commands
            .push(PathCommand::CubicTo(c1x, c1y, c2x, c2y, x, y));
        self
    }

    /// Returns or updates the `close` value.
    pub fn close(mut self) -> Self {
        self.commands.push(PathCommand::Close);
        self
    }

    fn build(self) -> SvgElement {
        SvgElement::Path {
            commands: self.commands,
            attrs: self.attrs,
        }
    }
}

impl Default for SvgPathBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl_svg_attrs_builder!(SvgPathBuilder);

/// Data and behavior represented by `SvgRectBuilder`.
pub struct SvgRectBuilder {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    rx: f32,
    attrs: SvgAttributes,
}

impl SvgRectBuilder {
    /// Creates a value with its default configuration.
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
            rx: 0.0,
            attrs: SvgAttributes::default(),
        }
    }

    /// Returns or updates the `radius` value.
    pub fn radius(mut self, rx: f32) -> Self {
        self.rx = rx;
        self
    }

    fn build(self) -> SvgElement {
        SvgElement::Rect {
            x: self.x,
            y: self.y,
            width: self.width,
            height: self.height,
            rx: self.rx,
            attrs: self.attrs,
        }
    }
}

impl_svg_attrs_builder!(SvgRectBuilder);

/// Data and behavior represented by `SvgCircleBuilder`.
pub struct SvgCircleBuilder {
    cx: f32,
    cy: f32,
    r: f32,
    attrs: SvgAttributes,
}

impl SvgCircleBuilder {
    /// Creates a value with its default configuration.
    pub fn new(cx: f32, cy: f32, r: f32) -> Self {
        Self {
            cx,
            cy,
            r,
            attrs: SvgAttributes::default(),
        }
    }

    fn build(self) -> SvgElement {
        SvgElement::Circle {
            cx: self.cx,
            cy: self.cy,
            r: self.r,
            attrs: self.attrs,
        }
    }
}

impl_svg_attrs_builder!(SvgCircleBuilder);

/// Data and behavior represented by `SvgLineBuilder`.
pub struct SvgLineBuilder {
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
    attrs: SvgAttributes,
}

impl SvgLineBuilder {
    /// Creates a value with its default configuration.
    pub fn new(x1: f32, y1: f32, x2: f32, y2: f32) -> Self {
        Self {
            x1,
            y1,
            x2,
            y2,
            attrs: SvgAttributes::default(),
        }
    }

    fn build(self) -> SvgElement {
        SvgElement::Line {
            x1: self.x1,
            y1: self.y1,
            x2: self.x2,
            y2: self.y2,
            attrs: self.attrs,
        }
    }
}

impl_svg_attrs_builder!(SvgLineBuilder);

/// Data and behavior represented by `SvgGroupBuilder`.
pub struct SvgGroupBuilder {
    children: Vec<SvgElement>,
    attrs: SvgAttributes,
}

impl SvgGroupBuilder {
    /// Creates a value with its default configuration.
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
            attrs: SvgAttributes::default(),
        }
    }

    /// Returns or updates the `path` value.
    pub fn path(mut self, build: impl FnOnce(SvgPathBuilder) -> SvgPathBuilder) -> Self {
        self.children.push(build(SvgPathBuilder::new()).build());
        self
    }

    /// Returns or updates the `rect` value.
    pub fn rect(
        mut self,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        build: impl FnOnce(SvgRectBuilder) -> SvgRectBuilder,
    ) -> Self {
        self.children
            .push(build(SvgRectBuilder::new(x, y, w, h)).build());
        self
    }

    /// Returns or updates the `circle` value.
    pub fn circle(
        mut self,
        cx: f32,
        cy: f32,
        r: f32,
        build: impl FnOnce(SvgCircleBuilder) -> SvgCircleBuilder,
    ) -> Self {
        self.children
            .push(build(SvgCircleBuilder::new(cx, cy, r)).build());
        self
    }

    /// Returns or updates the `line` value.
    pub fn line(
        mut self,
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        build: impl FnOnce(SvgLineBuilder) -> SvgLineBuilder,
    ) -> Self {
        self.children
            .push(build(SvgLineBuilder::new(x1, y1, x2, y2)).build());
        self
    }

    /// Returns or updates the `group` value.
    pub fn group(mut self, build: impl FnOnce(SvgGroupBuilder) -> SvgGroupBuilder) -> Self {
        self.children.push(build(SvgGroupBuilder::new()).build());
        self
    }

    fn build(self) -> SvgElement {
        SvgElement::Group {
            children: self.children,
            attrs: self.attrs,
        }
    }
}

impl Default for SvgGroupBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl_svg_attrs_builder!(SvgGroupBuilder);

/// A single `<image>` placement resolved to something xengui's own image
/// pipeline can draw - the `ImageSource` is built once (when the document
/// is set) instead of re-hashing pixel data on every frame.
struct ResolvedRasterImage {
    position: (f32, f32),
    size: (f32, f32),
    transform: Transform2D,
    opacity: f32,
    source: ImageSource,
    clip: Option<(f32, f32, f32, f32)>,
}

enum ResolvedDrawOp {
    Triangle(SvgTriangle),
    Image(ResolvedRasterImage),
}

fn transform_uniform_scale(t: Transform2D) -> f32 {
    let sx = (t.a * t.a + t.b * t.b).sqrt();
    let sy = (t.c * t.c + t.d * t.d).sqrt();
    (sx + sy) * 0.5
}

// Resolves any <image> href xen-svg's own parser couldn't decode (a bare
// file path rather than a data: URI), reading it relative to the current
// working directory - the same convention Image::path already uses.
#[cfg(not(target_arch = "wasm32"))]
fn resolve_document_images(document: &mut SvgDocument) {
    document.resolve_images(
        &mut (|href| {
            let bytes = std::fs::read(href).ok()?;
            let is_svg = href
                .rsplit('.')
                .next()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("svg"));

            if is_svg {
                let text = std::str::from_utf8(&bytes).ok()?;
                parse_svg(text)
                    .ok()
                    .map(|doc| SvgImageSource::Svg(Box::new(doc)))
            } else {
                let decoded = image::load_from_memory(&bytes).ok()?.to_rgba8();
                let (width, height) = decoded.dimensions();
                Some(SvgImageSource::Raster {
                    width,
                    height,
                    rgba: Arc::new(decoded.into_raw()),
                })
            }
        }),
    );
}

#[cfg(target_arch = "wasm32")]
fn resolve_document_images(_document: &mut SvgDocument) {}

/// A vector-graphics widget rendering a small subset of SVG (path, rect,
/// circle, line, image, group) through the existing triangle pipeline
/// (vector content) and the existing image pipeline (raster `<image>`s).
///
/// Colors may use [`xen_svg::SvgColor::CURRENT`] instead of a fixed [`crate::Color`]
/// to follow the widget's inherited `color` at render time, the same way
/// CSS's `currentColor` works - this is what lets `xengui-icons` ship icons
/// that automatically match surrounding text color.
pub struct Svg {
    base: WidgetBase,
    anim_id: WidgetId,
    document: Arc<SvgDocument>,
    // Tessellated once per document change instead of on every paint, since
    // flattening curves and triangulating shapes isn't free. Kept in
    // document order so overlapping vector/raster content composites
    // correctly (see paint()).
    draw_ops: Arc<Vec<ResolvedDrawOp>>,
    layout_box: LayoutBox,
}

impl Svg {
    /// Creates a value with its default configuration.
    pub fn new() -> Self {
        let document = SvgDocument::default();
        let interaction = Interaction::new();

        Self {
            base: WidgetBase::new(interaction),
            anim_id: WidgetId::new_unique(),
            document: Arc::new(document),
            draw_ops: Arc::new(Vec::new()),
            layout_box: LayoutBox::default(),
        }
    }

    /// Parses a full `<svg>...</svg>` document string.
    pub fn from_string(source: &str) -> Self {
        let mut svg = Self::new();
        match parse_svg(source) {
            Ok(mut document) => {
                resolve_document_images(&mut document);
                svg.set_document(document);
            }
            Err(err) => log::error!("Svg::from_string parse error: {err}"),
        }
        svg
    }

    /// Parses raw UTF-8 SVG bytes; invalid UTF-8 or malformed markup logs
    /// an error and leaves the widget empty, matching `Image::bytes`.
    pub fn from_bytes(bytes: &[u8]) -> Self {
        match std::str::from_utf8(bytes) {
            Ok(source) => Self::from_string(source),
            Err(err) => {
                log::error!("Svg::from_bytes invalid utf-8: {err}");
                Self::new()
            }
        }
    }

    /// Returns or updates the `key` value.
    pub fn key(mut self, key: impl Into<SmolStr>) -> Self {
        self.base.key = Some(key.into());
        self
    }

    /// Returns or updates the `view_box` value.
    pub fn view_box(mut self, x: f32, y: f32, width: f32, height: f32) -> Self {
        let mut document = (*self.document).clone();
        document.view_box = (x, y, width, height);
        self.set_document(document);
        self
    }

    /// Returns or updates the `path` value.
    pub fn path(mut self, build: impl FnOnce(SvgPathBuilder) -> SvgPathBuilder) -> Self {
        self.push_element(build(SvgPathBuilder::new()).build());
        self
    }

    /// Returns or updates the `rect` value.
    pub fn rect(
        mut self,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        build: impl FnOnce(SvgRectBuilder) -> SvgRectBuilder,
    ) -> Self {
        self.push_element(build(SvgRectBuilder::new(x, y, w, h)).build());
        self
    }

    /// Returns or updates the `circle` value.
    pub fn circle(
        mut self,
        cx: f32,
        cy: f32,
        r: f32,
        build: impl FnOnce(SvgCircleBuilder) -> SvgCircleBuilder,
    ) -> Self {
        self.push_element(build(SvgCircleBuilder::new(cx, cy, r)).build());
        self
    }

    /// Returns or updates the `line` value.
    pub fn line(
        mut self,
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        build: impl FnOnce(SvgLineBuilder) -> SvgLineBuilder,
    ) -> Self {
        self.push_element(build(SvgLineBuilder::new(x1, y1, x2, y2)).build());
        self
    }

    /// Returns or updates the `group` value.
    pub fn group(mut self, build: impl FnOnce(SvgGroupBuilder) -> SvgGroupBuilder) -> Self {
        self.push_element(build(SvgGroupBuilder::new()).build());
        self
    }

    /// Overrides the fill paint of the element with the given `id`
    /// (the SVG's own `id="..."` attribute), re-tessellating so the
    /// change is visible next frame. No-op if no element with that id
    /// exists in the current document.
    pub fn fill_by_id(mut self, id: &str, color: impl IntoSvgColor) -> Self {
        let mut document = (*self.document).clone();
        if let Some(element) = document.find_by_id_mut(id) {
            element.attrs_mut().fill = color.into_svg_color();
        }
        self.set_document(document);
        self
    }

    /// Overrides the stroke paint of the element with the given `id`.
    pub fn stroke_by_id(mut self, id: &str, color: impl IntoSvgColor) -> Self {
        let mut document = (*self.document).clone();
        if let Some(element) = document.find_by_id_mut(id) {
            element.attrs_mut().stroke = color.into_svg_color();
        }
        self.set_document(document);
        self
    }

    /// Overrides the opacity of the element with the given `id`.
    pub fn opacity_by_id(mut self, id: &str, opacity: f32) -> Self {
        let mut document = (*self.document).clone();
        if let Some(element) = document.find_by_id_mut(id) {
            element.attrs_mut().opacity = opacity;
        }
        self.set_document(document);
        self
    }

    fn push_element(&mut self, element: SvgElement) {
        let mut document = (*self.document).clone();
        document.elements.push(element);
        self.set_document(document);
    }

    fn set_document(&mut self, document: SvgDocument) {
        let ops = collect_draw_ops(&document);
        self.draw_ops = Arc::new(
            ops.into_iter()
                .map(|op| match op {
                    SvgDrawOp::Triangle(tri) => ResolvedDrawOp::Triangle(tri),
                    SvgDrawOp::Image(img) => ResolvedDrawOp::Image(ResolvedRasterImage {
                        position: img.position,
                        size: img.size,
                        transform: img.transform,
                        opacity: img.opacity,
                        source: image_source_from_rgba8((*img.rgba).clone(), img.width, img.height),
                        clip: img.clip,
                    }),
                })
                .collect(),
        );
        self.document = Arc::new(document);
        self.mark_dirty();
    }

    fn recompute_style(&mut self) {
        self.base.computed_style = self.base.inherited_style.inherit_style(&self.base.style);
    }
}

impl Default for Svg {
    fn default() -> Self {
        Self::new()
    }
}

impl StyleBuilder for Svg {
    fn style_mut(&mut self) -> &mut Style {
        &mut self.base.style
    }

    fn mark_dirty(&mut self) {
        self.base.dirty = true;
        self.recompute_style();
    }
}

crate::impl_interaction_builders!(base Svg);

impl Widget for Svg {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn debug_name(&self) -> &'static str {
        "Widget#Svg"
    }

    fn get_key(&self) -> Option<&SmolStr> {
        self.base.key.as_ref()
    }

    fn is_dirty(&self) -> bool {
        self.base.dirty
    }

    fn set_dirty(&mut self, dirty: bool) {
        self.base.dirty = dirty;
    }

    fn style(&self) -> &Style {
        &self.base.style
    }

    fn style_mut(&mut self) -> &mut Style {
        &mut self.base.style
    }

    fn computed_style(&self) -> &Style {
        &self.base.computed_style
    }

    fn children(&self) -> &[Box<dyn Widget>] {
        &[]
    }

    fn interaction(&self) -> Option<&Interaction> {
        Some(&self.base.interaction)
    }

    fn interaction_mut(&mut self) -> Option<&mut Interaction> {
        Some(&mut self.base.interaction)
    }

    fn measure(&self, ctx: &mut MeasureContext, constraints: Constraints) -> MeasureResult {
        let (_, _, vb_w, vb_h) = self.document.view_box;
        if vb_w <= 0.0 || vb_h <= 0.0 {
            return MeasureResult::new(0.0, 0.0);
        }

        let intrinsic_w = vb_w * ctx.scale_factor;
        let intrinsic_h = vb_h * ctx.scale_factor;

        let (width, height) = match (constraints.known_width, constraints.known_height) {
            (Some(w), Some(h)) => (w, h),
            (Some(w), None) => (w, (w * intrinsic_h) / intrinsic_w),
            (None, Some(h)) => ((h * intrinsic_w) / intrinsic_h, h),
            (None, None) => (intrinsic_w, intrinsic_h),
        };

        MeasureResult::new(width, height)
    }

    fn layout(&mut self, rect: LayoutBox) {
        self.layout_box = rect;
    }

    fn layout_box(&self) -> &LayoutBox {
        &self.layout_box
    }

    fn paint(&self, ctx: &mut PaintContext) {
        self.paint_box(ctx);
        self.paint_outline(ctx);

        let (vb_x, vb_y, vb_w, vb_h) = self.document.view_box;
        if vb_w <= 0.0 || vb_h <= 0.0 {
            return;
        }

        let style = &self.base.computed_style;
        let content_scale = style.content_scale.unwrap_or(style.scale.unwrap_or(1.0));
        let b = crate::scaled_layout_box_with_origin(
            self.layout_box,
            content_scale,
            style.transform_origin.unwrap_or_default(),
            ctx.scale_factor,
        );
        let scale = (b.width / vb_w).min(b.height / vb_h);
        let offset_x = (b.x + (b.width - vb_w * scale) * 0.5).round();
        let offset_y = (b.y + (b.height - vb_h * scale) * 0.5).round();

        let map = |p: (f32, f32)| -> (f32, f32) {
            (
                offset_x + (p.0 - vb_x) * scale,
                offset_y + (p.1 - vb_y) * scale,
            )
        };

        let inherited_color = self
            .base
            .computed_style
            .color
            .unwrap_or(crate::Color::BLACK);
        let inherited_svg_color = xen_svg::Color::rgba_f32(
            inherited_color.r(),
            inherited_color.g(),
            inherited_color.b(),
            inherited_color.a(),
        );

        // Draws vector triangles and raster images in the same order they
        // appear in the source document instead of batching all triangles
        // before all images, so a shape stacked on top of (or under) an
        // embedded image composites in the correct visual order.
        for op in self.draw_ops.iter() {
            match op {
                ResolvedDrawOp::Triangle(triangle) => {
                    let Some(color) = triangle.paint.resolve(inherited_svg_color) else {
                        continue;
                    };
                    let color = from_svg_color(color);
                    let color = color.with_alpha_f32(color.a() * triangle.opacity);

                    ctx.draw_triangle(TriangleCommand {
                        p0: map(triangle.p0),
                        p1: map(triangle.p1),
                        p2: map(triangle.p2),
                        color,
                        clip_rect: None,
                    });
                }
                ResolvedDrawOp::Image(image) => {
                    let (lx, ly) = image.transform.apply(image.position.0, image.position.1);
                    let (px, py) = map((lx, ly));
                    let img_scale = transform_uniform_scale(image.transform) * scale;

                    // Maps the local clip rect through the same
                    // element-transform + viewBox chain used for the
                    // image itself, then takes the axis-aligned bounding
                    // box of the mapped corners.
                    let clip_rect = image.clip.map(|(cx, cy, cw, ch)| {
                        let corners = [(cx, cy), (cx + cw, cy), (cx, cy + ch), (cx + cw, cy + ch)];
                        let mapped: Vec<(f32, f32)> = corners
                            .iter()
                            .map(|&(x, y)| {
                                let (tx, ty) = image.transform.apply(x, y);
                                map((tx, ty))
                            })
                            .collect();
                        let min_x = mapped.iter().map(|p| p.0).fold(f32::MAX, f32::min);
                        let max_x = mapped.iter().map(|p| p.0).fold(f32::MIN, f32::max);
                        let min_y = mapped.iter().map(|p| p.1).fold(f32::MAX, f32::min);
                        let max_y = mapped.iter().map(|p| p.1).fold(f32::MIN, f32::max);
                        (min_x, min_y, max_x - min_x, max_y - min_y)
                    });

                    ctx.draw_image(ImageCommand {
                        position: (px, py),
                        size: (image.size.0 * img_scale, image.size.1 * img_scale),
                        image: image.source.clone(),
                        border_radius: None,
                        tint: (image.opacity < 1.0)
                            .then(|| crate::Color::WHITE.with_alpha_f32(image.opacity)),
                        clip_rect,
                    });
                }
            }
        }
    }

    fn event(&mut self, event: &InputEvent, ctx: &mut EventCtx) -> EventStatus {
        if !self.base.interaction.is_active() {
            return EventStatus::Ignored;
        }
        self.base.interaction.handle(event, ctx)
    }

    fn content_eq(&self, other: &dyn Widget) -> bool {
        let Some(other) = other.as_any().downcast_ref::<Svg>() else {
            return false;
        };
        *self.document == *other.document && self.base.authored_styles_eq(&other.base)
    }

    fn cascade_style(&mut self, parent: &Style, anim: &mut AnimationManager) {
        self.base.inherited_style = parent.clone();
        self.recompute_style();
        if crate::animate_computed_style(self.anim_id, &mut self.base.computed_style, anim) {
            self.base.dirty = true;
        }
    }

    fn transfer_measured_state(&mut self, old: &dyn Widget) {
        if let Some(old) = old.as_any().downcast_ref::<Svg>() {
            self.anim_id = old.anim_id;
        }
    }

    fn anim_id(&self) -> WidgetId {
        self.anim_id
    }
}
