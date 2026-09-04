// SPDX-License-Identifier: Apache-2.0
use crate::{Cursor, Widget};
use std::future::Future;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// Available `KeyState` choices.
pub enum KeyState {
    /// The `Pressed` variant.
    Pressed,
    /// The `Released` variant.
    Released,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
/// Available `Key` choices.
pub enum Key {
    /// The `Escape` variant.
    Escape,

    /// The `F1` variant.
    F1,
    /// The `F2` variant.
    F2,
    /// The `F3` variant.
    F3,
    /// The `F4` variant.
    F4,
    /// The `F5` variant.
    F5,
    /// The `F6` variant.
    F6,
    /// The `F7` variant.
    F7,
    /// The `F8` variant.
    F8,
    /// The `F9` variant.
    F9,
    /// The `F10` variant.
    F10,
    /// The `F11` variant.
    F11,
    /// The `F12` variant.
    F12,
    /// The `F13` variant.
    F13,
    /// The `F14` variant.
    F14,
    /// The `F15` variant.
    F15,
    /// The `F16` variant.
    F16,
    /// The `F17` variant.
    F17,
    /// The `F18` variant.
    F18,
    /// The `F19` variant.
    F19,
    /// The `F20` variant.
    F20,
    /// The `F21` variant.
    F21,
    /// The `F22` variant.
    F22,
    /// The `F23` variant.
    F23,
    /// The `F24` variant.
    F24,
    /// The `F25` variant.
    F25,
    /// The `F26` variant.
    F26,
    /// The `F27` variant.
    F27,
    /// The `F28` variant.
    F28,
    /// The `F29` variant.
    F29,
    /// The `F30` variant.
    F30,
    /// The `F31` variant.
    F31,
    /// The `F32` variant.
    F32,
    /// The `F33` variant.
    F33,
    /// The `F34` variant.
    F34,
    /// The `F35` variant.
    F35,

    /// The `Pause` variant.
    Pause,
    /// The `PrintScreen` variant.
    PrintScreen,
    /// The `Delete` variant.
    Delete,
    /// The `Insert` variant.
    Insert,

    /// The `Home` variant.
    Home,
    /// The `End` variant.
    End,
    /// The `PageUp` variant.
    PageUp,
    /// The `PageDown` variant.
    PageDown,

    /// The `Backspace` variant.
    Backspace,
    /// The `NumLock` variant.
    NumLock,
    /// The `ScrollLock` variant.
    ScrollLock,

    /// The `Tab` variant.
    Tab,
    /// The `CapsLock` variant.
    CapsLock,
    /// The `Enter` variant.
    Enter,

    /// The `ShiftLeft` variant.
    ShiftLeft,
    /// The `ShiftRight` variant.
    ShiftRight,

    /// The `ControlLeft` variant.
    ControlLeft,
    /// The `ControlRight` variant.
    ControlRight,

    /// The `Fn` variant.
    Fn,
    /// The `SuperLeft` variant.
    SuperLeft,
    /// The `SuperRight` variant.
    SuperRight,
    /// The `AltLeft` variant.
    AltLeft,
    /// The `Space` variant.
    Space,
    /// The `AltRight` variant.
    AltRight,
    /// The `ContextMenu` variant.
    ContextMenu,

    /// The `ArrowUp` variant.
    ArrowUp,
    /// The `ArrowDown` variant.
    ArrowDown,
    /// The `ArrowLeft` variant.
    ArrowLeft,
    /// The `ArrowRight` variant.
    ArrowRight,

    /// The `Character` variant.
    Character(char),

    /// The `Unknown` variant.
    Unknown,
}

#[derive(Clone, Debug)]
/// Data and behavior represented by `KeyboardEvent`.
pub struct KeyboardEvent {
    /// The `key` value carried by this type.
    pub key: Key,
    /// The `state` value carried by this type.
    pub state: KeyState,
    /// The `repeat` value carried by this type.
    pub repeat: bool,
}

#[derive(Clone, Copy, Debug, Default)]
/// Data and behavior represented by `ModifiersState`.
pub struct ModifiersState {
    /// The `ctrl` value carried by this type.
    pub ctrl: bool,
    /// The `shift` value carried by this type.
    pub shift: bool,
    /// The `alt` value carried by this type.
    pub alt: bool,
    /// The `super_key` value carried by this type.
    pub super_key: bool,
}

/// Pressed/released state of a mouse button, independent of any windowing backend.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ElementState {
    /// The `Pressed` variant.
    Pressed,
    /// The `Released` variant.
    Released,
}

/// A mouse button identifier, independent of any windowing backend.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MouseButton {
    /// The `Left` variant.
    Left,
    /// The `Right` variant.
    Right,
    /// The `Middle` variant.
    Middle,
    /// The `Back` variant.
    Back,
    /// The `Forward` variant.
    Forward,
    /// The `Other` variant.
    Other(u16),
}

/// A single scroll-wheel step, independent of any windowing backend.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MouseScrollDelta {
    /// The `LineDelta` variant.
    LineDelta(f32, f32),
    /// The `PixelDelta` variant.
    PixelDelta(f64, f64),
}

/// IME composition state, independent of any windowing backend.
#[derive(Clone, Debug, PartialEq)]
pub enum ImeEvent {
    /// The `Enabled` variant.
    Enabled,
    /// The `Preedit` variant.
    Preedit(String, Option<(usize, usize)>),
    /// The `Commit` variant.
    Commit(String),
    /// The `Disabled` variant.
    Disabled,
}

/// Phase of a touch-driven pan gesture. Dispatched positionally alongside
/// (not instead of) the ordinary mouse-shaped events touch input already
/// synthesizes for hover/press/click compatibility, so a scrollable widget
/// can turn a finger drag into a scroll without any widget needing to know
/// whether its mouse events came from a real mouse or from touch.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TouchPanPhase {
    /// The `Start` variant.
    Start,
    /// The `Move` variant.
    Move,
    /// The `End` variant.
    End,
    /// The `Cancel` variant.
    Cancel,
}

#[derive(Clone, Debug)]
/// Available `InputEvent` choices.
pub enum InputEvent {
    /// The `MouseMoved` variant.
    MouseMoved {
        /// The `item` value carried by this type.
        position: (f32, f32),
    },
    /// The `MouseEntered` variant.
    MouseEntered,
    /// The `MouseExited` variant.
    MouseExited,
    /// The `MouseInput` variant.
    MouseInput {
        /// The `item` value carried by this type.
        state: ElementState,
        /// The `item` value carried by this type.
        button: MouseButton,
        /// The `item` value carried by this type.
        position: (f32, f32),
    },
    /// The `MouseWheel` variant.
    MouseWheel {
        /// The `item` value carried by this type.
        delta: MouseScrollDelta,
        /// The `item` value carried by this type.
        position: (f32, f32),
        /// The `item` value carried by this type.
        modifiers: ModifiersState,
    },
    /// The `KeyInput` variant.
    KeyInput {
        /// The `item` value carried by this type.
        event: KeyboardEvent,
        /// The `item` value carried by this type.
        modifiers: ModifiersState,
    },
    /// The `ModifiersChanged` variant.
    ModifiersChanged(ModifiersState),
    /// The `Ime` variant.
    Ime(ImeEvent),
    /// The `FocusGained` variant.
    FocusGained {
        /// The `item` value carried by this type.
        via_keyboard: bool,
    },
    /// The `FocusLost` variant.
    FocusLost,
    /// The `BlinkTick` variant.
    BlinkTick,
    /// The `AnimationTick` variant.
    AnimationTick {
        /// The `item` value carried by this type.
        dt: f32,
    },
    /// The `TouchPan` variant.
    TouchPan {
        /// The `item` value carried by this type.
        phase: TouchPanPhase,
        /// The `item` value carried by this type.
        position: (f32, f32),
    },
    /// The `FocusWithinChanged` variant.
    FocusWithinChanged(bool),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// Available `EventStatus` choices.
pub enum EventStatus {
    /// The `Ignored` variant.
    Ignored,
    /// The `Handled` variant.
    Handled,
}

#[derive(Default)]
/// Data and behavior represented by `EventCtx`.
pub struct EventCtx {
    redraw_requested: bool,
    cursor_icon: Option<crate::Cursor>,
    focus_requested: bool,
    focus_released: bool,
    /// The `focus_target` value carried by this type.
    pub focus_target: Option<String>,
    /// The `clear_focus` value carried by this type.
    pub clear_focus: bool,
    suppress_text_drag: bool,
}

impl EventCtx {
    /// Creates a value with its default configuration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Spawns a future on the framework's GUI-thread executor - shorthand
    /// for `xengui::spawn` usable directly from an event callback.
    pub fn spawn<F>(&self, future: F)
    where
        F: Future + 'static,
    {
        crate::task::spawn(future);
    }

    /// Returns or updates the `request_redraw` value.
    pub fn request_redraw(&mut self) {
        self.redraw_requested = true;
    }

    // Tells the cross-widget drag-selection mechanism to skip this press.
    /// Returns or updates the `suppress_text_drag` value.
    pub fn suppress_text_drag(&mut self) {
        self.suppress_text_drag = true;
    }

    /// Returns or updates the `take_suppress_text_drag` value.
    pub fn take_suppress_text_drag(&mut self) -> bool {
        std::mem::take(&mut self.suppress_text_drag)
    }

    /// Returns or updates the `redraw_requested` value.
    pub fn redraw_requested(&self) -> bool {
        self.redraw_requested
    }

    /// Updates the `set_cursor_icon` value.
    pub fn set_cursor_icon(&mut self, icon: crate::Cursor) {
        self.cursor_icon = Some(icon);
    }

    /// Returns or updates the `take_cursor_icon` value.
    pub fn take_cursor_icon(&mut self) -> Option<crate::Cursor> {
        self.cursor_icon.take()
    }

    /// Returns or updates the `request_focus` value.
    pub fn request_focus(&mut self) {
        self.focus_requested = true;
    }

    /// Returns or updates the `release_focus` value.
    pub fn release_focus(&mut self) {
        self.focus_released = true;
    }

    fn take_focus_request(&mut self) -> bool {
        std::mem::take(&mut self.focus_requested)
    }

    fn take_release_focus_request(&mut self) -> bool {
        std::mem::take(&mut self.focus_released)
    }
}

/// Returns or updates the `ancestor_paths` value.
pub fn ancestor_paths(path: &str) -> Vec<String> {
    let parts: Vec<&str> = path.split('.').collect();
    (1..=parts.len()).map(|n| parts[..n].join(".")).collect()
}

/// Returns or updates the `path_segment` value.
pub fn path_segment(widget: &dyn Widget, index: usize) -> String {
    match widget.get_key() {
        Some(key) => format!("k{key}"),
        None => index.to_string(),
    }
}

fn resolve_segment<'a>(
    siblings: &'a mut [Box<dyn Widget>],
    segment: &str,
) -> Option<&'a mut dyn Widget> {
    if let Some(key) = segment.strip_prefix('k') {
        siblings
            .iter_mut()
            .find(|w| w.get_key().is_some_and(|k| k.as_str() == key))
            .map(|w| w.as_mut())
    } else {
        let idx: usize = segment.parse().ok()?;
        siblings.get_mut(idx).map(|w| w.as_mut())
    }
}

/// Returns or updates the `find_widget_mut` value.
pub fn find_widget_mut<'a>(
    tree: &'a mut [Box<dyn Widget>],
    path: &str,
) -> Option<&'a mut dyn Widget> {
    let mut parts = path.split('.');
    let mut current: &mut dyn Widget = resolve_segment(tree, parts.next()?)?;

    for part in parts {
        let children = current.children_mut()?;
        current = resolve_segment(children, part)?;
    }

    Some(current)
}

/// Returns or updates the `hit_test_path` value.
pub fn hit_test_path(tree: &[Box<dyn Widget>], point: (f32, f32)) -> Option<String> {
    hit_test_children(tree, point, 0, "")
}

// Tests widgets in the same stacking order FrameRenderer paints them: sorted
// by z_index (inherited from `parent_z` when unset), highest first - so a
// widget with a higher z_index (e.g. a sticky header) can intercept a hit
// even when it's earlier in the sibling list than overlapping content.
fn hit_test_children(
    widgets: &[Box<dyn Widget>],
    point: (f32, f32),
    parent_z: i32,
    parent_path: &str,
) -> Option<String> {
    let mut order: Vec<usize> = (0..widgets.len()).collect();
    order.sort_by_key(|&i| widgets[i].computed_style().z_index.unwrap_or(parent_z));

    for &i in order.iter().rev() {
        let widget = &widgets[i];
        let segment = path_segment(widget.as_ref(), i);
        let path = if parent_path.is_empty() {
            segment
        } else {
            format!("{parent_path}.{segment}")
        };
        let z = widget.computed_style().z_index.unwrap_or(parent_z);
        if let Some(hit) = hit_test_recursive(widget.as_ref(), &path, point, z) {
            return Some(hit);
        }
    }
    None
}

fn hit_test_recursive(
    widget: &dyn Widget,
    path: &str,
    point: (f32, f32),
    z: i32,
) -> Option<String> {
    if !widget.hit_test(point) {
        return None;
    }

    if !widget.blocks_children_hit_test(point)
        && let Some(hit) = hit_test_children(widget.children(), point, z, path)
    {
        return Some(hit);
    }

    Some(path.to_string())
}

/// Collects the paths of every active, focusable widget in the tree in
/// depth-first order, used to build the Tab / Shift+Tab sequence.
pub fn collect_focusable_paths(tree: &[Box<dyn Widget>]) -> Vec<String> {
    let mut paths = Vec::new();
    for (i, node) in tree.iter().enumerate() {
        let segment = path_segment(node.as_ref(), i);
        collect_focusable_recursive(node.as_ref(), &segment, &mut paths);
    }
    paths
}

fn collect_focusable_recursive(widget: &dyn Widget, path: &str, out: &mut Vec<String>) {
    if widget
        .interaction()
        .is_some_and(|i| i.focusable && i.enabled)
    {
        out.push(path.to_string());
    }

    for (i, child) in widget.children().iter().enumerate() {
        let segment = path_segment(child.as_ref(), i);
        let child_path = format!("{path}.{segment}");
        collect_focusable_recursive(child.as_ref(), &child_path, out);
    }
}

// True if `path` is `ancestor` itself or one of its descendants.
/// Returns or updates the `path_is_within` value.
pub fn path_is_within(path: &str, ancestor: &str) -> bool {
    path == ancestor || path.starts_with(&format!("{ancestor}."))
}

// Walks the hover path from root to leaf and returns the deepest widget's
// own hover cursor, so a plain (non-interactive) child inside a clickable
// ancestor doesn't shadow that ancestor's cursor.
/// Returns or updates the `resolve_hover_cursor` value.
pub fn resolve_hover_cursor(tree: &[Box<dyn Widget>], path: &str) -> Option<Cursor> {
    let mut current: &[Box<dyn Widget>] = tree;
    let mut resolved = None;

    for segment in path.split('.') {
        let widget = if let Some(key) = segment.strip_prefix('k') {
            current
                .iter()
                .find(|w| w.get_key().is_some_and(|k| k.as_str() == key))?
        } else {
            let idx: usize = segment.parse().ok()?;
            current.get(idx)?
        };

        if let Some(cursor) = widget.interaction().and_then(|i| i.hover_cursor) {
            resolved = Some(cursor);
        }

        current = widget.children();
    }

    resolved
}

/// Returns or updates the `dispatch_positional` value.
pub fn dispatch_positional(
    tree: &mut [Box<dyn Widget>],
    leaf_path: &str,
    event: &InputEvent,
    ctx: &mut EventCtx,
) -> EventStatus {
    dispatch_positional_capturing(tree, leaf_path, event, ctx).0
}

/// Same bubbling as `dispatch_positional`, but also returns the exact
/// ancestor path that consumed the event - lets a caller (e.g. touch pan
/// gesture tracking) cache that path and reach the same widget directly
/// via `dispatch_to_path` on later events of the same gesture, instead of
/// re-walking the whole ancestor chain from the original leaf every time.
pub fn dispatch_positional_capturing(
    tree: &mut [Box<dyn Widget>],
    leaf_path: &str,
    event: &InputEvent,
    ctx: &mut EventCtx,
) -> (EventStatus, Option<String>) {
    for path in ancestor_paths(leaf_path).into_iter().rev() {
        let Some(widget) = find_widget_mut(tree, &path) else {
            continue;
        };

        let redraw_before = ctx.redraw_requested();
        let status = widget.event(event, ctx);

        if !redraw_before && ctx.redraw_requested() && crate::devtools::is_enabled() {
            crate::devtools::log_repaint(&path, widget.debug_name(), format!("{event:?}"));
        }

        if ctx.take_focus_request() {
            ctx.focus_target = Some(path.clone());
        }
        if ctx.take_release_focus_request() {
            ctx.clear_focus = true;
        }

        if status == EventStatus::Handled {
            return (EventStatus::Handled, Some(path));
        }
    }
    (EventStatus::Ignored, None)
}

/// Returns or updates the `dispatch_to_path` value.
pub fn dispatch_to_path(
    tree: &mut [Box<dyn Widget>],
    path: &str,
    event: &InputEvent,
    ctx: &mut EventCtx,
) -> EventStatus {
    match find_widget_mut(tree, path) {
        Some(widget) => {
            let redraw_before = ctx.redraw_requested();
            let status = widget.event(event, ctx);
            if !redraw_before && ctx.redraw_requested() && crate::devtools::is_enabled() {
                crate::devtools::log_repaint(path, widget.debug_name(), format!("{event:?}"));
            }
            status
        }
        None => EventStatus::Ignored,
    }
}

/// Transitions hover state from `old_path` to `new_path`, dispatching
/// MouseExited/MouseEntered to every ancestor exclusive to one side of
/// the change - not just the leaf that was actually hit. Without this, a
/// non-interactive child (e.g. an icon) wrapped by a clickable ancestor
/// never sets that ancestor's own `hovered` flag.
pub fn dispatch_hover_transition(
    tree: &mut [Box<dyn Widget>],
    old_path: Option<&str>,
    new_path: Option<&str>,
    ctx: &mut EventCtx,
) {
    let old_chain: Vec<String> = old_path.map(ancestor_paths).unwrap_or_default();
    let new_chain: Vec<String> = new_path.map(ancestor_paths).unwrap_or_default();

    for path in &old_chain {
        if !new_chain.contains(path) {
            dispatch_to_path(tree, path, &InputEvent::MouseExited, ctx);
        }
    }
    for path in &new_chain {
        if !old_chain.contains(path) {
            dispatch_to_path(tree, path, &InputEvent::MouseEntered, ctx);
        }
    }
}

/// Bubbles a `focus_within` flag up every ancestor exclusive to one side
/// of the change, the same way `dispatch_hover_transition` bubbles hover -
/// lets a container react when any descendant gains/loses focus.
pub fn dispatch_focus_within_transition(
    tree: &mut [Box<dyn Widget>],
    old_path: Option<&str>,
    new_path: Option<&str>,
    ctx: &mut EventCtx,
) {
    let old_chain: Vec<String> = old_path.map(ancestor_paths).unwrap_or_default();
    let new_chain: Vec<String> = new_path.map(ancestor_paths).unwrap_or_default();

    for path in &old_chain {
        if !new_chain.contains(path) {
            dispatch_to_path(tree, path, &InputEvent::FocusWithinChanged(false), ctx);
        }
    }
    for path in &new_chain {
        if !old_chain.contains(path) {
            dispatch_to_path(tree, path, &InputEvent::FocusWithinChanged(true), ctx);
        }
    }
}

/// Returns or updates the `any_wants_animation` value.
pub fn any_wants_animation(tree: &[Box<dyn Widget>]) -> bool {
    tree.iter()
        .any(|w| widget_wants_animation_recursive(w.as_ref()))
}

fn widget_wants_animation_recursive(widget: &dyn Widget) -> bool {
    if widget.wants_animation_frame() {
        return true;
    }
    widget
        .children()
        .iter()
        .any(|c| widget_wants_animation_recursive(c.as_ref()))
}

/// Returns or updates the `dispatch_animation_tick` value.
pub fn dispatch_animation_tick(tree: &mut [Box<dyn Widget>], dt: f32, ctx: &mut EventCtx) {
    for (i, widget) in tree.iter_mut().enumerate() {
        let segment = path_segment(widget.as_ref(), i);
        dispatch_animation_tick_recursive(widget.as_mut(), &segment, dt, ctx);
    }
}

fn dispatch_animation_tick_recursive(
    widget: &mut dyn Widget,
    path: &str,
    dt: f32,
    ctx: &mut EventCtx,
) {
    if widget.wants_animation_frame() {
        widget.event(&(InputEvent::AnimationTick { dt }), ctx);

        // AnimationTick has no ancestor-chain lookup of its own, unlike
        // dispatch_positional, so a focus request raised from it must be
        // resolved against this path explicitly.
        if ctx.take_focus_request() {
            ctx.focus_target = Some(path.to_string());
        }
        if ctx.take_release_focus_request() {
            ctx.clear_focus = true;
        }
    }
    if let Some(children) = widget.children_mut() {
        for (i, child) in children.iter_mut().enumerate() {
            let segment = path_segment(child.as_ref(), i);
            let child_path = format!("{path}.{segment}");
            dispatch_animation_tick_recursive(child.as_mut(), &child_path, dt, ctx);
        }
    }
}

#[derive(Default)]
/// Data and behavior represented by `InputState`.
pub struct InputState {
    /// The `cursor_pos` value carried by this type.
    pub cursor_pos: Option<(f32, f32)>,
    /// The `hovered_path` value carried by this type.
    pub hovered_path: Option<String>,
    /// The `pressed_path` value carried by this type.
    pub pressed_path: Option<String>,
    /// The `focused_path` value carried by this type.
    pub focused_path: Option<String>,
    /// The `modifiers` value carried by this type.
    pub modifiers: ModifiersState,
    /// Screen point where a cross-widget text-selection drag started;
    /// `None` when no such drag is in progress.
    pub text_drag_anchor: Option<(f32, f32)>,
}

/// Returns or updates the `select_all_text_recursive` value.
pub fn select_all_text_recursive(tree: &mut [Box<dyn Widget>]) {
    for widget in tree.iter_mut() {
        widget.select_all_text();
        if let Some(children) = widget.children_mut() {
            select_all_text_recursive(children);
        }
    }
}

// Returns whether any widget actually had a selection to clear, so the
// caller can skip a redraw when nothing changed.
/// Performs the `clear_text_selection_recursive` operation.
pub fn clear_text_selection_recursive(tree: &mut [Box<dyn Widget>]) -> bool {
    let mut cleared = false;
    for widget in tree.iter_mut() {
        if widget.text_selection().is_some() {
            cleared = true;
        }
        widget.cancel_text_selection();
        if let Some(children) = widget.children_mut() {
            cleared |= clear_text_selection_recursive(children);
        }
    }
    cleared
}

/// Recomputes every selectable widget's own text selection from two
/// screen points, so a single mouse drag can span multiple widgets like
/// a browser selection.
pub fn update_global_text_selection(
    tree: &mut [Box<dyn Widget>],
    anchor: (f32, f32),
    current: (f32, f32),
) -> bool {
    let (start, end) = if (anchor.1, anchor.0) <= (current.1, current.0) {
        (anchor, current)
    } else {
        (current, anchor)
    };
    update_global_text_selection_recursive(tree, start, end)
}

// Returns whether any widget's selection actually changed, so callers can
// skip requesting a redraw when the drag moved but nothing selectable was
// underneath it (e.g. dragging across a plain View).
fn update_global_text_selection_recursive(
    widgets: &mut [Box<dyn Widget>],
    start: (f32, f32),
    end: (f32, f32),
) -> bool {
    let mut changed = false;

    for widget in widgets.iter_mut() {
        if widget.selectable_text().is_some() {
            let before = widget.text_selection();
            let b = *widget.layout_box();
            let top = b.y;
            let bottom = b.y + b.height;

            if bottom <= start.1 || top >= end.1 {
                widget.set_text_selection(None);
            } else {
                let overlaps_start = top <= start.1 && bottom > start.1;
                let overlaps_end = top <= end.1 && bottom > end.1;

                let from = if overlaps_start {
                    widget.text_index_at(start)
                } else {
                    0
                };
                let to = if overlaps_end {
                    widget.text_index_at(end)
                } else {
                    widget
                        .selectable_text()
                        .map(|t| t.chars().count())
                        .unwrap_or(0)
                };

                widget.set_text_selection(Some((from, to)));
            }

            if widget.text_selection() != before {
                changed = true;
            }
        }

        if let Some(children) = widget.children_mut() {
            changed |= update_global_text_selection_recursive(children, start, end);
        }
    }

    changed
}

/// Returns or updates the `collect_selected_text_recursive` value.
pub fn collect_selected_text_recursive(tree: &[Box<dyn Widget>], out: &mut String) {
    for widget in tree.iter() {
        if let (Some(text), Some((start, end))) =
            (widget.selectable_text(), widget.text_selection())
        {
            let chars: Vec<char> = text.chars().collect();
            let s = start.min(chars.len());
            let e = end.min(chars.len());
            if e > s {
                if !out.is_empty() {
                    out.push('\n');
                }
                out.extend(&chars[s..e]);
            }
        }
        collect_selected_text_recursive(widget.children(), out);
    }
}

/// Forces every widget in the tree to repaint on the next frame, even
/// when its own dirty flag and layout box are otherwise unchanged -
/// needed for style inputs that live outside any individual widget's own
/// props, like the active theme.
pub fn mark_tree_dirty(tree: &mut [Box<dyn Widget>]) {
    for widget in tree.iter_mut() {
        widget.set_dirty(true);
        if let Some(children) = widget.children_mut() {
            mark_tree_dirty(children);
        }
    }
}

/// Cancels any in-progress AutoScroll gesture across the whole tree. See
/// `Widget::cancel_auto_scroll`.
pub fn cancel_auto_scroll_recursive(tree: &mut [Box<dyn Widget>], ctx: &mut EventCtx) {
    for widget in tree.iter_mut() {
        widget.cancel_auto_scroll(ctx);
        if let Some(children) = widget.children_mut() {
            cancel_auto_scroll_recursive(children, ctx);
        }
    }
}
