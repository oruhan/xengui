// SPDX-License-Identifier: Apache-2.0
//! A minimal DOM-like control surface: give any widget an `.id(...)` and
//! trigger it from anywhere in application code, mirroring HTML's
//! `id="..."` + JS's `document.getElementById(id).click()`. Actions are
//! queued by id in a thread-local mailbox; the targeted widget picks them
//! up on its next animation tick and interprets them itself (a `Click` on
//! a `Checkbox` toggles it, on a `TextBox` it focuses it, etc), so there's
//! no central knowledge of what every widget type does with every action.

use smol_str::SmolStr;
use std::cell::RefCell;
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
/// Available `DomAction` choices.
pub enum DomAction {
    /// Synthesizes a primary activation - a click on `Button`, a toggle
    /// on `Checkbox`/`Switch`, a selection on `RadioButton`, a focus on
    /// `TextBox`.
    Click,
    /// Moves keyboard focus to the target widget.
    Focus,
    /// Explicitly sets a boolean state without firing the widget's own
    /// change callback - like `checkbox.checked = true` in JS, as
    /// opposed to a user click.
    SetChecked(bool),
    /// Replaces a `TextBox`'s content without firing `on_change` - like
    /// `input.value = "..."` in JS.
    SetValue(String),
    /// Escape hatch for application-defined widgets: any string payload,
    /// interpreted however the target widget's own `event()` wants.
    Custom(SmolStr),
}

thread_local! {
    static MAILBOX: RefCell<HashMap<SmolStr, Vec<DomAction>>> = RefCell::new(HashMap::new());
}

/// Queues `action` for whichever widget is registered under `id` via
/// `.id(id)`. Delivered on that widget's next animation tick.
pub fn dispatch(id: &str, action: DomAction) {
    MAILBOX.with(|m| {
        m.borrow_mut()
            .entry(SmolStr::new(id))
            .or_default()
            .push(action);
    });
    crate::hooks::mark_dirty_and_redraw();
}

/// Returns or updates the `click` value.
pub fn click(id: &str) {
    dispatch(id, DomAction::Click);
}

/// Returns or updates the `focus` value.
pub fn focus(id: &str) {
    dispatch(id, DomAction::Focus);
}

/// Updates the `set_checked` value.
pub fn set_checked(id: &str, checked: bool) {
    dispatch(id, DomAction::SetChecked(checked));
}

/// Updates the `set_value` value.
pub fn set_value(id: &str, value: impl Into<String>) {
    dispatch(id, DomAction::SetValue(value.into()));
}

/// Widgets use this in `wants_animation_frame` to opt into ticking only
/// while something is actually pending for their own id.
pub fn has_pending(id: &str) -> bool {
    MAILBOX.with(|m| m.borrow().get(id).is_some_and(|q| !q.is_empty()))
}

/// Drains and returns every action currently queued for `id`, in the
/// order they were dispatched.
pub fn take_actions(id: &str) -> Vec<DomAction> {
    MAILBOX.with(|m| m.borrow_mut().remove(id).unwrap_or_default())
}
