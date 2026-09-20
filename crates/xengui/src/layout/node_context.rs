// SPDX-License-Identifier: Apache-2.0

use crate::{Widget, WidgetPath};

/// Immutable data needed to measure one Taffy leaf during layout.
///
/// The Taffy node stores this value's index in a layout-pass-local table.
/// Keeping the widget borrow outside Taffy makes ownership explicit and
/// ensures no layout context can survive beyond the pass that created it.
pub(crate) struct NodeContext<'a> {
    pub(crate) widget: &'a dyn Widget,
    pub(crate) path: WidgetPath,
}

impl<'a> NodeContext<'a> {
    pub(crate) fn new(widget: &'a dyn Widget, path: &WidgetPath) -> Self {
        Self {
            widget,
            path: path.clone(),
        }
    }
}
