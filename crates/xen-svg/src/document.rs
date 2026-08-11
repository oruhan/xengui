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

    /// Finds the first element (searching recursively into groups) whose
    /// `id="..."` attribute matches, for runtime style overrides.
    pub fn find_by_id_mut(&mut self, id: &str) -> Option<&mut SvgElement> {
        find_by_id_in_slice(&mut self.elements, id)
    }

    pub fn resolve_images(&mut self, resolve: &mut dyn FnMut(&str) -> Option<SvgImageSource>) {
        for element in &mut self.elements {
            resolve_images_in_element(element, resolve);
        }
    }
}

fn find_by_id_in_slice<'a>(elements: &'a mut [SvgElement], id: &str) -> Option<&'a mut SvgElement> {
    for element in elements.iter_mut() {
        if element.attrs().id.as_deref() == Some(id) {
            return Some(element);
        }
        if
            let SvgElement::Group { children, .. } = element &&
            let Some(found) = find_by_id_in_slice(children, id)
        {
            return Some(found);
        }
    }
    None
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
            if let SvgImageSource::Unresolved(href) = source && let Some(resolved) = resolve(href) {
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
