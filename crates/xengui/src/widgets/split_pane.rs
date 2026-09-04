// SPDX-License-Identifier: Apache-2.0
//! IDE-style layout: optional left/right/bottom panels around a flexible
//! center area, each resizable by dragging its own `SplitHandle`.
use crate::{
    AnimationManager, Column, Constraints, Display, Interaction, LayoutBox, Length, MeasureContext,
    MeasureResult, Overflow, PaintContext, Row, SplitHandle, SplitSide, Style, StyleBuilder, View,
    Widget, WidgetBase,
};
use smol_str::SmolStr;
use std::cell::Cell;
use std::rc::Rc;

/// Configuration for one resizable panel: its content, current size
/// (width for left/right, height for top/bottom), and resize bounds.
/// `size` should be created once (e.g. via `use_state`) and kept alive
/// across rebuilds, or the panel forgets its size on every render.
pub struct SplitPanel {
    content: Box<dyn Widget>,
    size: Rc<Cell<f32>>,
    min_size: f32,
    max_size: f32,
    key: Option<SmolStr>,
}

impl SplitPanel {
    /// Creates a value with its default configuration.
    pub fn new(content: impl Widget + 'static, size: Rc<Cell<f32>>) -> Self {
        Self {
            content: Box::new(content),
            size,
            min_size: 150.0,
            max_size: 600.0,
            key: None,
        }
    }

    /// Returns or updates the `min_size` value.
    pub fn min_size(mut self, value: f32) -> Self {
        self.min_size = value;
        self
    }

    /// Returns or updates the `max_size` value.
    pub fn max_size(mut self, value: f32) -> Self {
        self.max_size = value;
        self
    }

    /// Stable identity across rebuilds, so the panel's content keeps its
    /// own interaction/hook state - see `View::key`.
    pub fn key(mut self, key: impl Into<SmolStr>) -> Self {
        self.key = Some(key.into());
        self
    }
}

/// Wraps a single panel's content and keeps its own width (or height,
/// for a top/bottom panel) synced to a shared `Rc<Cell<f32>>` every
/// frame via `cascade_style`, which the layout engine runs unconditionally
/// on every frame regardless of any dirty flag. This lets `SplitHandle`
/// resize the panel just by mutating the shared cell and requesting a
/// redraw, without forcing the owning composite widget to rebuild its
/// whole tree on every pixel of movement.
struct SplitSizedBox {
    base: WidgetBase,
    children: Vec<Box<dyn Widget>>,
    layout_box: LayoutBox,
    size: Rc<Cell<f32>>,
    min_size: f32,
    max_size: f32,
    horizontal: bool,
}

impl SplitSizedBox {
    fn new(
        content: Box<dyn Widget>,
        size: Rc<Cell<f32>>,
        min_size: f32,
        max_size: f32,
        horizontal: bool,
    ) -> Self {
        let mut base = WidgetBase::new(Interaction::new());
        base.style.display = Some(Display::Flex);
        base.style.overflow_x = Some(Overflow::Hidden);

        Self {
            base,
            children: vec![content],
            layout_box: LayoutBox::default(),
            size,
            min_size,
            max_size,
            horizontal,
        }
    }

    fn key(mut self, key: impl Into<SmolStr>) -> Self {
        self.base.key = Some(key.into());
        self
    }

    fn sync_size(&mut self) {
        let clamped = self.size.get().clamp(self.min_size, self.max_size);
        let target = Length::px(clamped);
        let mut size = self.base.style.size.unwrap_or_default();
        let axis = if self.horizontal {
            &mut size.width
        } else {
            &mut size.height
        };

        if *axis != Some(target) {
            *axis = Some(target);
            if self.horizontal {
                size.height = Some(Length::pct(100.0));
            } else {
                size.width = Some(Length::pct(100.0));
            }
            self.base.style.size = Some(size);
            self.base.dirty = true;
        }
    }
}

impl Widget for SplitSizedBox {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn debug_name(&self) -> &'static str {
        "Widget#SplitSizedBox"
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

    fn cascade_style(&mut self, parent: &Style, anim: &mut AnimationManager) {
        self.sync_size();
        self.base.inherited_style = parent.clone();
        self.base.computed_style = self.base.inherited_style.inherit_style(&self.base.style);
        for child in self.children.iter_mut() {
            child.cascade_style(&self.base.computed_style, anim);
        }
    }
}

fn side_panel_row(
    left: Option<SplitPanel>,
    center: Box<dyn Widget>,
    right: Option<SplitPanel>,
) -> View {
    let mut row = Row::new().size(Length::pct(100.0), Length::pct(100.0));

    if let Some(panel) = left {
        let mut wrapper = SplitSizedBox::new(
            panel.content,
            panel.size.clone(),
            panel.min_size,
            panel.max_size,
            true,
        );
        if let Some(key) = panel.key {
            wrapper = wrapper.key(key);
        }
        row = row.child(wrapper);
        row = row.child(SplitHandle::new(
            panel.size,
            SplitSide::Left,
            panel.min_size,
            panel.max_size,
        ));
    }

    let center_wrapper = View::new()
        .display(Display::Flex)
        .flex_grow(1.0)
        .height(Length::pct(100.0))
        .overflow_x(Overflow::Hidden)
        .children_vec(vec![center]);
    row = row.child(center_wrapper);

    if let Some(panel) = right {
        row = row.child(SplitHandle::new(
            panel.size.clone(),
            SplitSide::Right,
            panel.min_size,
            panel.max_size,
        ));
        let mut wrapper = SplitSizedBox::new(
            panel.content,
            panel.size,
            panel.min_size,
            panel.max_size,
            true,
        );
        if let Some(key) = panel.key {
            wrapper = wrapper.key(key);
        }
        row = row.child(wrapper);
    }

    row
}

/// Builds `[left] [handle] center [handle] [right]`, left/right panels optional.
pub fn split_pane(
    left: Option<SplitPanel>,
    center: impl Widget + 'static,
    right: Option<SplitPanel>,
) -> View {
    side_panel_row(left, Box::new(center), right)
}

/// Full IDE-style layout: left/right side panels around a center area,
/// plus an optional bottom panel (e.g. a terminal) spanning the whole
/// width beneath everything else.
pub fn split_pane_with_bottom(
    left: Option<SplitPanel>,
    center: impl Widget + 'static,
    right: Option<SplitPanel>,
    bottom: Option<SplitPanel>,
) -> View {
    let top_row = side_panel_row(left, Box::new(center), right);

    let mut column = Column::new().size(Length::pct(100.0), Length::pct(100.0));

    let top_wrapper = View::new()
        .display(Display::Flex)
        .flex_grow(1.0)
        .width(Length::pct(100.0))
        .overflow_y(Overflow::Hidden)
        .children_vec(vec![Box::new(top_row) as Box<dyn Widget>]);
    column = column.child(top_wrapper);

    if let Some(panel) = bottom {
        column = column.child(SplitHandle::new(
            panel.size.clone(),
            SplitSide::Bottom,
            panel.min_size,
            panel.max_size,
        ));
        let mut wrapper = SplitSizedBox::new(
            panel.content,
            panel.size,
            panel.min_size,
            panel.max_size,
            false,
        );
        if let Some(key) = panel.key {
            wrapper = wrapper.key(key);
        }
        column = column.child(wrapper);
    }

    column
}
