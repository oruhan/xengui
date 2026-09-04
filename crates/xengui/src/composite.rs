// SPDX-License-Identifier: Apache-2.0
use crate::Widget;

/// Implemented by user-defined widgets composed from existing xengui
/// widgets. `render` runs once per instance, the first time it becomes
/// part of the tree - a fresh instance (and a fresh render) is produced
/// every time this widget's parent re-renders, the same way a React
/// function component reruns on every parent render. Props are just the
/// struct's own fields; `use_state` works normally inside `render`.
pub trait Render {
    /// Builds the widget subtree represented by this component's current properties.
    fn render(&self) -> Box<dyn Widget>;
}
