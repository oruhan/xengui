// SPDX-License-Identifier: Apache-2.0
//! Platform-agnostic SVG support: a small element model, a `d`/`viewBox`
//! parser, and a triangle tessellator. Has no rendering or windowing
//! dependencies of its own - any GUI framework can consume the tessellated
//! triangle list and draw it through its own pipeline.

mod base64;
mod color;
mod document;
mod element;
mod parser;
mod tessellate;
mod transform;
mod constants;

pub use color::{ Color, SvgColor };
pub use document::SvgDocument;
pub use element::{
    FillRule,
    LineCap,
    LineJoin,
    PathCommand,
    SvgAttributes,
    SvgElement,
    SvgImageSource,
};
pub use parser::parse_svg;
pub use tessellate::{ tessellate_document, collect_raster_images, SvgRasterImage, SvgTriangle };
pub use transform::{ parse_transform, Transform2D };
pub use constants::*;
