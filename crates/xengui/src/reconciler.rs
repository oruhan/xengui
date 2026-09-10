// SPDX-License-Identifier: Apache-2.0
//! Interruptible tree reconciler.
//!
//! Reconciliation is driven by an explicit work stack. Each stack frame
//! represents one level of sibling traversal and stores the state needed
//! to continue processing that level. `WorkLoop::perform_work` repeatedly
//! pops the current frame, processes a single widget, updates the frame's
//! progress, and pushes any newly discovered child work back onto the
//! stack. Because the traversal state lives entirely in the work stack,
//! execution can stop after any processed node and resume later by
//! continuing from the remaining frames.
//!
//! Widgets are matched one sibling level at a time. Keyed widgets are
//! looked up by key, while unkeyed widgets consume the next available
//! unkeyed sibling in order. A match is considered valid only when both
//! widgets have the same concrete type; otherwise the existing widget is
//! discarded and the new widget is treated as a fresh insertion.
//!
//! Rather than storing references into the old tree, each frame records
//! its location as a path of child indices from the root. On every
//! `perform_work` invocation, this path is resolved against `old_root` to
//! obtain the current sibling slice, ensuring that no references into the
//! old tree are kept alive across yields.

use crate::Widget;
use smol_str::SmolStr;
use std::collections::{HashMap, HashSet};
use web_time::{Duration, Instant};

struct Frame {
    new_siblings: Vec<Box<dyn Widget>>,
    old_path: Vec<usize>,
    keyed_old: HashMap<SmolStr, usize>,
    consumed: Vec<bool>,
    positional_cursor: usize,
    next_index: usize,
}

impl Frame {
    fn new(
        new_siblings: Vec<Box<dyn Widget>>,
        old_siblings: &[Box<dyn Widget>],
        old_path: Vec<usize>,
    ) -> Self {
        let mut keyed_old = HashMap::new();
        let mut new_keys = HashSet::new();

        for sibling in &new_siblings {
            if let Some(key) = sibling.get_key() {
                assert!(
                    new_keys.insert(key.clone()),
                    "duplicate sibling key `{key}` during reconciliation"
                );
            }
        }

        for (i, old) in old_siblings.iter().enumerate() {
            if let Some(key) = old.get_key() {
                assert!(
                    keyed_old.insert(key.clone(), i).is_none(),
                    "duplicate sibling key `{key}` in committed tree"
                );
            }
        }

        Self {
            new_siblings,
            old_path,
            keyed_old,
            consumed: vec![false; old_siblings.len()],
            positional_cursor: 0,
            next_index: 0,
        }
    }
}

/// Result of a single [`WorkLoop::perform_work`] call.
pub enum WorkLoopStatus {
    /// The time budget ran out before the tree was fully reconciled; call
    /// `perform_work` again with a fresh deadline to keep going.
    Yielded,
    /// Reconciliation finished; this is the fully reconciled tree, ready
    /// to be committed as the new current tree.
    Complete(Vec<Box<dyn Widget>>),
}

const YIELD_CHECK_INTERVAL: u32 = 8;

/// An in-progress, interruptible reconciliation pass.
pub struct WorkLoop {
    stack: Vec<Frame>,
    units_since_check: u32,
}

// Walks down `root` following a path of child indices, returning the
// sibling slice at that depth.
fn resolve_old_siblings<'a>(root: &'a [Box<dyn Widget>], path: &[usize]) -> &'a [Box<dyn Widget>] {
    let mut current = root;
    for &idx in path {
        current = current[idx].children();
    }
    current
}

fn resolve_old_siblings_mut<'a>(
    root: &'a mut [Box<dyn Widget>],
    path: &[usize],
) -> &'a mut [Box<dyn Widget>] {
    let mut current = root;
    for &idx in path {
        current = current[idx]
            .children_mut()
            .expect("old tree structure changed during reconciliation")
            .as_mut_slice();
    }
    current
}

impl WorkLoop {
    /// Begins reconciling `new_root` against `old_root`.
    pub fn new(new_root: Vec<Box<dyn Widget>>, old_root: &[Box<dyn Widget>]) -> Self {
        Self {
            stack: vec![Frame::new(new_root, old_root, Vec::new())],
            units_since_check: 0,
        }
    }

    /// Runs until either the whole tree has been reconciled or `deadline`
    /// is reached, whichever comes first. `old_root` must be the same
    /// tree (unchanged in structure) this `WorkLoop` was created against.
    pub fn perform_work(
        &mut self,
        old_root: &mut [Box<dyn Widget>],
        deadline: Instant,
    ) -> WorkLoopStatus {
        loop {
            let frame_done = {
                let frame = self.stack.last().expect("root frame always present");
                frame.next_index >= frame.new_siblings.len()
            };

            if frame_done {
                let finished = self.stack.pop().expect("frame exists");

                let old_siblings = resolve_old_siblings_mut(old_root, &finished.old_path);
                let mut any_removed = false;
                for (i, consumed) in finished.consumed.iter().enumerate() {
                    if !consumed {
                        unmount_subtree(old_siblings[i].as_mut());
                        any_removed = true;
                    }
                }
                match self.stack.last_mut() {
                    Some(parent) => {
                        let parent_idx = parent.next_index - 1;
                        let parent_widget = &mut parent.new_siblings[parent_idx];
                        if any_removed {
                            // A child actually disappeared from beneath this
                            // widget, so its taffy node shape changed even
                            // though its own style stayed content_eq - without
                            // this, the widget keeps its stale (pre-removal)
                            // layout box until something unrelated marks the
                            // tree dirty again.
                            parent_widget.set_dirty(true);
                        }
                        if let Some(slot) = parent_widget.children_mut() {
                            *slot = finished.new_siblings;
                        }
                    }
                    None => {
                        return WorkLoopStatus::Complete(finished.new_siblings);
                    }
                }
                continue;
            }

            self.process_one_node(old_root);

            self.units_since_check += 1;
            if self.units_since_check >= YIELD_CHECK_INTERVAL {
                self.units_since_check = 0;
                if Instant::now() >= deadline {
                    return WorkLoopStatus::Yielded;
                }
            }
        }
    }

    fn process_one_node(&mut self, old_root: &mut [Box<dyn Widget>]) {
        let frame = self.stack.last_mut().expect("root frame always present");
        let idx = frame.next_index;
        frame.next_index += 1;

        let Some(old_idx) = Self::find_match(frame, old_root, idx) else {
            mount_subtree(frame.new_siblings[idx].as_mut());
            return;
        };
        if frame.consumed[old_idx] {
            mount_subtree(frame.new_siblings[idx].as_mut());
            return;
        }

        frame.consumed[old_idx] = true;
        let old_path = frame.old_path.clone();

        let old_siblings = resolve_old_siblings_mut(old_root, &old_path);

        if frame.new_siblings[idx].as_any().type_id() != old_siblings[old_idx].as_any().type_id() {
            if crate::devtools::is_enabled() {
                let name = frame.new_siblings[idx].debug_name();
                let path = reconcile_path(&old_path, idx);
                crate::devtools::log_rerender(&path, name, "widget type changed");
            }
            unmount_subtree(old_siblings[old_idx].as_mut());
            mount_subtree(frame.new_siblings[idx].as_mut());
            return;
        }

        let new_node = &mut frame.new_siblings[idx];
        let old_node = &mut old_siblings[old_idx];

        new_node.transfer_interaction_state(old_node.as_ref());
        let content_equal = new_node.content_eq(old_node.as_ref());

        if !content_equal && crate::devtools::is_enabled() {
            let path = reconcile_path(&old_path, idx);
            crate::devtools::log_rerender(&path, new_node.debug_name(), "content changed");
        }

        new_node.after_interaction_transfer();
        new_node.transfer_composite_children(old_node.as_mut());

        if content_equal {
            new_node.transfer_measured_state(old_node.as_ref());
            new_node.layout(*old_node.layout_box());
            new_node.set_dirty(false);
            new_node.set_layout_dirty(false);
        }

        let has_children = new_node.children_mut().is_some_and(|c| !c.is_empty());

        if has_children {
            let mut child_path = old_path;
            child_path.push(old_idx);

            let taken_children = std::mem::take(new_node.children_mut().unwrap());
            let old_children = resolve_old_siblings(old_root, &child_path);
            self.stack
                .push(Frame::new(taken_children, old_children, child_path));
        }
    }

    fn find_match(frame: &mut Frame, old_root: &[Box<dyn Widget>], idx: usize) -> Option<usize> {
        let key = frame.new_siblings[idx].get_key().cloned();
        if let Some(key) = key {
            return frame.keyed_old.get(&key).copied();
        }

        let old_siblings = resolve_old_siblings(old_root, &frame.old_path);
        while frame.positional_cursor < old_siblings.len() {
            let candidate = frame.positional_cursor;
            frame.positional_cursor += 1;
            if frame.consumed[candidate] || old_siblings[candidate].get_key().is_some() {
                continue;
            }
            return Some(candidate);
        }
        None
    }
}

// Debug-only display path built from reconciler frame indices - not the
// same format as the paint/input system's key-based WidgetPath, but
// stable enough to eyeball in the DevTools log.
fn reconcile_path(old_path: &[usize], idx: usize) -> String {
    if old_path.is_empty() {
        idx.to_string()
    } else {
        let parent: Vec<String> = old_path.iter().map(usize::to_string).collect();
        format!("{}.{}", parent.join("."), idx)
    }
}

fn mount_subtree(widget: &mut dyn Widget) {
    widget.on_mount();
    if let Some(children) = widget.children_mut() {
        for child in children.iter_mut() {
            mount_subtree(child.as_mut());
        }
    }
}

fn unmount_subtree(widget: &mut dyn Widget) {
    widget.on_unmount();
    if let Some(children) = widget.children_mut() {
        for child in children.iter_mut() {
            unmount_subtree(child.as_mut());
        }
    }
}

/// Fully reconciles a single-child new tree against `old_root`
/// synchronously, with no time budget - used by composite widgets, where
/// the tree being diffed is small enough that yielding mid-reconcile
/// isn't worth the added complexity.
pub fn reconcile_now(
    new_root: Vec<Box<dyn Widget>>,
    old_root: &mut [Box<dyn Widget>],
) -> Vec<Box<dyn Widget>> {
    let mut work = WorkLoop::new(new_root, old_root);
    let far_future = Instant::now() + Duration::from_secs(3600);
    loop {
        match work.perform_work(old_root, far_future) {
            WorkLoopStatus::Complete(tree) => {
                return tree;
            }
            WorkLoopStatus::Yielded => {
                continue;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Constraints, LayoutBox, MeasureContext, MeasureResult, PaintContext, Style, Widget,
    };
    use std::{any::Any, cell::RefCell, rc::Rc};

    type EventLog = Rc<RefCell<Vec<String>>>;

    struct TestWidget {
        name: &'static str,
        key: Option<SmolStr>,
        retained_state: u32,
        dirty: bool,
        style: Style,
        layout: LayoutBox,
        children: Vec<Box<dyn Widget>>,
        events: EventLog,
        drain_old_children_on_transfer: bool,
    }

    impl TestWidget {
        fn new(name: &'static str, key: Option<&str>, state: u32, events: &EventLog) -> Self {
            Self {
                name,
                key: key.map(SmolStr::new),
                retained_state: state,
                dirty: true,
                style: Style::default(),
                layout: LayoutBox::default(),
                children: Vec::new(),
                events: events.clone(),
                drain_old_children_on_transfer: false,
            }
        }

        fn with_children(mut self, children: Vec<Box<dyn Widget>>) -> Self {
            self.children = children;
            self
        }

        fn draining_old_children(mut self) -> Self {
            self.drain_old_children_on_transfer = true;
            self
        }
    }

    impl Widget for TestWidget {
        fn as_any(&self) -> &dyn Any {
            self
        }

        fn as_any_mut(&mut self) -> &mut dyn Any {
            self
        }

        fn debug_name(&self) -> &'static str {
            self.name
        }

        fn get_key(&self) -> Option<&SmolStr> {
            self.key.as_ref()
        }

        fn is_dirty(&self) -> bool {
            self.dirty
        }

        fn set_dirty(&mut self, dirty: bool) {
            self.dirty = dirty;
        }

        fn style(&self) -> &Style {
            &self.style
        }

        fn style_mut(&mut self) -> &mut Style {
            &mut self.style
        }

        fn on_mount(&mut self) {
            self.events
                .borrow_mut()
                .push(format!("mount:{}", self.name));
        }

        fn on_unmount(&mut self) {
            self.events
                .borrow_mut()
                .push(format!("unmount:{}", self.name));
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
            self.layout = rect;
        }

        fn layout_box(&self) -> &LayoutBox {
            &self.layout
        }

        fn paint(&self, _ctx: &mut PaintContext) {}

        fn content_eq(&self, other: &dyn Widget) -> bool {
            other
                .as_any()
                .downcast_ref::<Self>()
                .is_some_and(|old| self.name == old.name)
        }

        fn transfer_measured_state(&mut self, old: &dyn Widget) {
            if let Some(old) = old.as_any().downcast_ref::<Self>() {
                self.retained_state = old.retained_state;
            }
        }

        fn transfer_composite_children(&mut self, old: &mut dyn Widget) {
            if self.drain_old_children_on_transfer
                && let Some(old) = old.as_any_mut().downcast_mut::<Self>()
            {
                self.children = std::mem::take(&mut old.children);
            }
        }
    }

    fn widget(
        name: &'static str,
        key: Option<&str>,
        state: u32,
        events: &EventLog,
    ) -> Box<dyn Widget> {
        Box::new(TestWidget::new(name, key, state, events))
    }

    fn state(widget: &dyn Widget) -> u32 {
        widget
            .as_any()
            .downcast_ref::<TestWidget>()
            .expect("test widget")
            .retained_state
    }

    #[test]
    fn keyed_reorder_preserves_each_widgets_state() {
        let events = EventLog::default();
        let mut old = vec![
            widget("alpha", Some("a"), 10, &events),
            widget("beta", Some("b"), 20, &events),
        ];
        let new = vec![
            widget("beta", Some("b"), 0, &events),
            widget("alpha", Some("a"), 0, &events),
        ];

        let reconciled = reconcile_now(new, &mut old);

        assert_eq!(state(reconciled[0].as_ref()), 20);
        assert_eq!(state(reconciled[1].as_ref()), 10);
        assert!(events.borrow().is_empty());
    }

    #[test]
    fn keyed_insert_and_remove_fire_lifecycle_once() {
        let events = EventLog::default();
        let mut old = vec![
            widget("removed", Some("a"), 1, &events),
            widget("retained", Some("b"), 2, &events),
        ];
        let new = vec![
            widget("retained", Some("b"), 0, &events),
            widget("inserted", Some("c"), 0, &events),
        ];

        let reconciled = reconcile_now(new, &mut old);

        assert_eq!(state(reconciled[0].as_ref()), 2);
        assert_eq!(
            events.borrow().as_slice(),
            ["mount:inserted", "unmount:removed"]
        );
    }

    #[test]
    #[should_panic(expected = "duplicate sibling key `same` during reconciliation")]
    fn duplicate_new_sibling_keys_are_rejected() {
        let events = EventLog::default();
        let _ = WorkLoop::new(
            vec![
                widget("first", Some("same"), 0, &events),
                widget("second", Some("same"), 0, &events),
            ],
            &[],
        );
    }

    #[test]
    fn yielded_work_resumes_to_the_same_result_as_synchronous_work() {
        let events = EventLog::default();
        let mut old: Vec<Box<dyn Widget>> = (0..16)
            .map(|index| widget("item", Some(&format!("key-{index}")), index, &events))
            .collect();
        let new: Vec<Box<dyn Widget>> = (0..16)
            .rev()
            .map(|index| widget("item", Some(&format!("key-{index}")), 0, &events))
            .collect();
        let mut work = WorkLoop::new(new, &old);

        assert!(matches!(
            work.perform_work(&mut old, Instant::now()),
            WorkLoopStatus::Yielded
        ));

        let complete = loop {
            match work.perform_work(&mut old, Instant::now() + Duration::from_secs(60)) {
                WorkLoopStatus::Yielded => continue,
                WorkLoopStatus::Complete(tree) => break tree,
            }
        };

        let states: Vec<u32> = complete.iter().map(|item| state(item.as_ref())).collect();
        assert_eq!(states, (0..16).rev().collect::<Vec<_>>());
        assert!(events.borrow().is_empty());
    }

    #[test]
    #[ignore = "Phase 2: composite transfer currently mutates the committed tree before commit"]
    fn yielded_reconciliation_does_not_mutate_the_committed_tree() {
        let events = EventLog::default();
        let old_children = (0..8)
            .map(|index| widget("child", Some(&format!("child-{index}")), index, &events))
            .collect();
        let mut old: Vec<Box<dyn Widget>> = vec![Box::new(
            TestWidget::new("composite", Some("root"), 1, &events).with_children(old_children),
        )];
        let new_children = (0..8)
            .map(|index| widget("child", Some(&format!("child-{index}")), 0, &events))
            .collect();
        let new: Vec<Box<dyn Widget>> = vec![Box::new(
            TestWidget::new("composite", Some("root"), 0, &events)
                .with_children(new_children)
                .draining_old_children(),
        )];
        let mut work = WorkLoop::new(new, &old);

        assert!(matches!(
            work.perform_work(&mut old, Instant::now()),
            WorkLoopStatus::Yielded
        ));
        assert_eq!(old[0].children().len(), 8);
    }
}
