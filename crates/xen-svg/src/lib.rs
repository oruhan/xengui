// SPDX-License-Identifier: Apache-2.0
//! Platform-agnostic SVG support: a small element model, a `d`/`viewBox`
//! parser, and a triangle tessellator. Has no rendering or windowing
//! dependencies of its own - any GUI framework can consume the tessellated
//! triangle list and draw it through its own pipeline.

mod base64;
mod color;
mod constants;
mod document;
mod element;
mod parser;
mod tessellate;
mod transform;

pub use color::{Color, SvgColor};
pub use constants::*;
pub use document::SvgDocument;
pub use element::{
    FillRule, LineCap, LineJoin, PathCommand, SvgAttributes, SvgElement, SvgImageSource,
};
pub use parser::parse_svg;
pub use tessellate::{
    SvgDrawOp, SvgRasterImage, SvgTriangle, collect_draw_ops, collect_raster_images,
    tessellate_document,
};
pub use transform::{Transform2D, parse_transform};
