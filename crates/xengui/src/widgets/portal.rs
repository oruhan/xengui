// SPDX-License-Identifier: Apache-2.0
use crate::{
    AnimationManager, Constraints, LayoutBox, MeasureContext, MeasureResult, PaintContext, Style,
    StyleBuilder, Widget, WidgetBase,
};

/// Wraps exactly one child and paints it unclipped in the top layer,
/// after every other widget's own content - letting it escape an
/// ancestor's overflow clipping (e.g. a dropdown opened from inside a
/// scrollable list). Still occupies space in the normal layout flow
/// like any other widget; only painting is redirected.
pub struct Portal {
    base: WidgetBase,
    children: Vec<Box<dyn Widget>>,
    layout_box: LayoutBox,
}

impl Portal {
    /// Creates a value with its default configuration.
    pub fn new() -> Self {
        Self {
            base: WidgetBase::new(crate::Interaction::new()),
            children: Vec::new(),
            layout_box: LayoutBox::default(),
        }
    }

    /// Returns or updates the `child` value.
    pub fn child(mut self, child: impl Widget + 'static) -> Self {
        self.children = vec![Box::new(child)];
        self
    }
}

impl Default for Portal {
    fn default() -> Self {
        Self::new()
    }
}

impl StyleBuilder for Portal {
    fn style_mut(&mut self) -> &mut Style {
        &mut self.base.style
    }

    fn mark_dirty(&mut self) {
        self.base.dirty = true;
    }
}

impl Widget for Portal {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn debug_name(&self) -> &'static str {
        "Widget#Portal"
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
        &self.children
    }

    fn children_mut(&mut self) -> Option<&mut Vec<Box<dyn Widget>>> {
        Some(&mut self.children)
    }

    fn measure(&self, _ctx: &mut MeasureContext, _constraints: Constraints) -> MeasureResult {
        MeasureResult::new(0.0, 0.0)
    }

    fn layout(&mut self, rect: LayoutBox) {
        self.layout_box = rect;
    }

    fn layout_box(&self) -> &LayoutBox {
        &self.layout_box
    }

    fn paint(&self, _ctx: &mut PaintContext) {}

    fn is_portal(&self) -> bool {
        true
    }

    fn cascade_style(&mut self, parent: &Style, anim: &mut AnimationManager) {
        self.base.inherited_style = parent.clone();
        self.base.computed_style = self.base.inherited_style.inherit_style(&self.base.style);
        for child in self.children.iter_mut() {
            child.cascade_style(&self.base.computed_style, anim);
        }
    }
}
