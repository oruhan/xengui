// SPDX-License-Identifier: Apache-2.0

use crate::{
    EventCtx, InputEvent, Widget, WidgetPath, collect_focusable_paths,
    dispatch_focus_within_transition, dispatch_to_path,
};

/// Owns keyboard focus for one widget tree.
///
/// All focus transitions pass through this manager so `FocusLost`,
/// `FocusGained`, and `FocusWithinChanged` are emitted in one deterministic
/// order. A manager belongs to one runtime/window and must not be shared.
#[derive(Default)]
pub struct FocusManager {
    focused: Option<WidgetPath>,
}

impl FocusManager {
    /// Creates an empty focus manager.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the currently focused widget path.
    pub fn focused_path(&self) -> Option<&WidgetPath> {
        self.focused.as_ref()
    }

    /// Moves focus to `target`, returning whether the focused widget changed.
    pub fn focus(
        &mut self,
        tree: &mut [Box<dyn Widget>],
        target: WidgetPath,
        via_keyboard: bool,
        ctx: &mut EventCtx,
    ) -> bool {
        if self.focused.as_ref() == Some(&target) {
            return false;
        }

        let old = self.focused.take();
        if let Some(old_path) = old.as_ref() {
            dispatch_to_path(tree, old_path, &InputEvent::FocusLost, ctx);
        }

        dispatch_to_path(
            tree,
            &target,
            &InputEvent::FocusGained { via_keyboard },
            ctx,
        );
        dispatch_focus_within_transition(tree, old.as_ref(), Some(&target), ctx);
        self.focused = Some(target);
        true
    }

    /// Clears focus, returning whether a widget previously held focus.
    pub fn clear(&mut self, tree: &mut [Box<dyn Widget>], ctx: &mut EventCtx) -> bool {
        let Some(old) = self.focused.take() else {
            return false;
        };

        dispatch_to_path(tree, &old, &InputEvent::FocusLost, ctx);
        dispatch_focus_within_transition(tree, Some(&old), None, ctx);
        true
    }

    /// Advances through the active focusable widgets in tree order.
    pub fn advance(
        &mut self,
        tree: &mut [Box<dyn Widget>],
        backward: bool,
        ctx: &mut EventCtx,
    ) -> bool {
        let focusable = collect_focusable_paths(tree);
        if focusable.is_empty() {
            return self.clear(tree, ctx);
        }

        let current_index = self
            .focused
            .as_ref()
            .and_then(|current| focusable.iter().position(|candidate| candidate == current));
        let next_index = match (current_index, backward) {
            (None, false) => 0,
            (None, true) => focusable.len() - 1,
            (Some(index), false) => (index + 1) % focusable.len(),
            (Some(index), true) => (index + focusable.len() - 1) % focusable.len(),
        };

        self.focus(tree, focusable[next_index].clone(), true, ctx)
    }
}

#[cfg(test)]
mod tests {
    use super::FocusManager;
    use crate::{EventCtx, View, Widget, collect_focusable_paths, find_widget_mut};

    #[test]
    fn advance_emits_one_coherent_focus_transition() {
        let mut tree: Vec<Box<dyn Widget>> = vec![Box::new(
            View::new()
                .key("root")
                .child(View::new().key("first").focusable(true))
                .child(View::new().key("second").focusable(true)),
        )];
        let paths = collect_focusable_paths(&tree);
        let mut focus = FocusManager::new();

        assert!(focus.advance(&mut tree, false, &mut EventCtx::new()));
        assert_eq!(focus.focused_path(), Some(&paths[0]));
        assert!(
            find_widget_mut(&mut tree, &paths[0])
                .and_then(|widget| widget.interaction())
                .is_some_and(|interaction| interaction.focused)
        );

        assert!(focus.advance(&mut tree, false, &mut EventCtx::new()));
        assert_eq!(focus.focused_path(), Some(&paths[1]));
        assert!(
            find_widget_mut(&mut tree, &paths[0])
                .and_then(|widget| widget.interaction())
                .is_some_and(|interaction| !interaction.focused)
        );
        assert!(
            find_widget_mut(&mut tree, &paths[1])
                .and_then(|widget| widget.interaction())
                .is_some_and(|interaction| interaction.focused)
        );
    }

    #[test]
    fn focus_managers_are_runtime_isolated() {
        let mut first_tree: Vec<Box<dyn Widget>> =
            vec![Box::new(View::new().key("first").focusable(true))];
        let second_tree: Vec<Box<dyn Widget>> =
            vec![Box::new(View::new().key("second").focusable(true))];
        let mut first = FocusManager::new();
        let second = FocusManager::new();

        first.advance(&mut first_tree, false, &mut EventCtx::new());

        assert!(first.focused_path().is_some());
        assert!(second.focused_path().is_none());
        assert!(
            second_tree[0]
                .interaction()
                .is_some_and(|interaction| !interaction.focused)
        );
    }
}
