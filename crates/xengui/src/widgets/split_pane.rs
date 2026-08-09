// SPDX-License-Identifier: Apache-2.0
//! IDE-style layout: optional left/right/bottom panels around a flexible
//! center area, each resizable by dragging its own `SplitHandle`.
use crate::{
    Column,
    Display,
    Length,
    Overflow,
    Row,
    SplitHandle,
    SplitSide,
    StyleBuilder,
    View,
    Widget,
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
    pub fn new(content: impl Widget + 'static, size: Rc<Cell<f32>>) -> Self {
        Self {
            content: Box::new(content),
            size,
            min_size: 150.0,
            max_size: 600.0,
            key: None,
        }
    }

    pub fn min_size(mut self, value: f32) -> Self {
        self.min_size = value;
        self
    }

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

fn side_panel_row(
    left: Option<SplitPanel>,
    center: Box<dyn Widget>,
    right: Option<SplitPanel>
) -> View {
    let mut row = Row::new().size(Length::pct(100.0), Length::pct(100.0));

    if let Some(panel) = left {
        let width = panel.size.get().clamp(panel.min_size, panel.max_size);
        let mut wrapper = View::new()
            .display(Display::Flex)
            .width(Length::px(width))
            .height(Length::pct(100.0))
            .overflow_x(Overflow::Hidden)
            .children_vec(vec![panel.content]);
        if let Some(key) = panel.key {
            wrapper = wrapper.key(key);
        }
        row = row.child(wrapper);
        row = row.child(
            SplitHandle::new(panel.size, SplitSide::Left, panel.min_size, panel.max_size)
        );
    }

    let center_wrapper = View::new()
        .display(Display::Flex)
        .flex_grow(1.0)
        .height(Length::pct(100.0))
        .overflow_x(Overflow::Hidden)
        .children_vec(vec![center]);
    row = row.child(center_wrapper);

    if let Some(panel) = right {
        let width = panel.size.get().clamp(panel.min_size, panel.max_size);
        row = row.child(
            SplitHandle::new(panel.size, SplitSide::Right, panel.min_size, panel.max_size)
        );
        let mut wrapper = View::new()
            .display(Display::Flex)
            .width(Length::px(width))
            .height(Length::pct(100.0))
            .overflow_x(Overflow::Hidden)
            .children_vec(vec![panel.content]);
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
    right: Option<SplitPanel>
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
    bottom: Option<SplitPanel>
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
        let height = panel.size.get().clamp(panel.min_size, panel.max_size);
        column = column.child(
            SplitHandle::new(panel.size, SplitSide::Bottom, panel.min_size, panel.max_size)
        );
        let mut wrapper = View::new()
            .display(Display::Flex)
            .height(Length::px(height))
            .width(Length::pct(100.0))
            .overflow_y(Overflow::Hidden)
            .children_vec(vec![panel.content]);
        if let Some(key) = panel.key {
            wrapper = wrapper.key(key);
        }
        column = column.child(wrapper);
    }

    column
}
