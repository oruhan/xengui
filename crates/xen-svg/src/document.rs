// SPDX-License-Identifier: Apache-2.0
use super::{ SvgElement, SvgImageSource };

/// Parsed SVG document: the `viewBox` (used as the local coordinate space
/// every element is authored in) plus the top-level element tree.
#[derive(Clone, Debug, PartialEq)]
pub struct SvgDocument {
    pub view_box: (f32, f32, f32, f32),
    pub elements: Vec<SvgElement>,
}

impl SvgDocument {
    pub fn new(view_box: (f32, f32, f32, f32)) -> Self {
        Self { view_box, elements: Vec::new() }
    }

    /// Replaces every `SvgImageSource::Unresolved(href)` in this document
    /// (recursively, including inside nested `<image>` SVGs) with
    /// whatever `resolve` returns for that `href` - lets a host with real
    /// filesystem/network access (unlike this crate) supply bytes for a
    /// bare reference like `icon.svg` or `icon.png`. Anything `resolve`
    /// still can't handle is left unresolved and simply renders nothing.
    pub fn resolve_images(&mut self, resolve: &mut dyn FnMut(&str) -> Option<SvgImageSource>) {
        for element in &mut self.elements {
            resolve_images_in_element(element, resolve);
        }
    }
}

fn resolve_images_in_element(
    element: &mut SvgElement,
    resolve: &mut dyn FnMut(&str) -> Option<SvgImageSource>
) {
    match element {
        SvgElement::Group { children, .. } => {
            for child in children {
                resolve_images_in_element(child, resolve);
            }
        }
        SvgElement::Image { source, .. } => {
            if let SvgImageSource::Unresolved(href) = source
                && let Some(resolved) = resolve(href) {
                    *source = resolved;
                }
            if let SvgImageSource::Svg(nested) = source {
                nested.resolve_images(resolve);
            }
        }
        _ => {}
    }
}

impl Default for SvgDocument {
    fn default() -> Self {
        Self::new((0.0, 0.0, 24.0, 24.0))
    }
}
