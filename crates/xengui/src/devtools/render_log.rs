// SPDX-License-Identifier: Apache-2.0
//! Structured, timestamped rerender/repaint log for the in-app DevTools
//! panel. Separate from `devtools::record*` (which targets frame-timing/
//! resize debugging) - this one is keyed by widget path and carries a
//! human-readable reason string.

use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use web_time::{Instant, SystemTime, UNIX_EPOCH};

/// Widget key assigned to the DevTools panel when it's mounted into the
/// app's root wrapper (see `xenframe::App::schedule_render`). Shared here
/// so the render log can recognize - and skip logging - anything that
/// happened inside the panel's own subtree, instead of the panel
/// perpetually reporting on its own churn.
pub const DEVTOOLS_PANEL_KEY: &str = "xengui_devtools_panel";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// Available `RenderEventKind` choices.
pub enum RenderEventKind {
    /// The `Rerender` variant.
    Rerender,
    /// The `Repaint` variant.
    Repaint,
    /// The `Layout` variant.
    Layout,
    /// The `Warning` variant.
    Warning,
    /// The `Error` variant.
    Error,
}

#[derive(Clone, Debug)]
/// Data and behavior represented by `RenderLogEntry`.
pub struct RenderLogEntry {
    /// The `epoch_millis` value carried by this type.
    pub epoch_millis: u128,
    /// The `kind` value carried by this type.
    pub kind: RenderEventKind,
    /// The `widget_path` value carried by this type.
    pub widget_path: String,
    /// The `widget_name` value carried by this type.
    pub widget_name: &'static str,
    /// The `reason` value carried by this type.
    pub reason: String,
}

const CAPACITY: usize = 2048;
const NOTIFY_THROTTLE_MS: u64 = 300;

thread_local! {
    static LOG: RefCell<VecDeque<RenderLogEntry>> = RefCell::new(VecDeque::with_capacity(CAPACITY));
    static ENABLED: Cell<bool> = const { Cell::new(false) };
    static LAST_NOTIFY: Cell<Option<Instant>> = const { Cell::new(None) };
    static SUPPRESS_DEPTH: Cell<u32> = const { Cell::new(0) };
}

/// Updates the `set_enabled` value.
pub fn set_enabled(enabled: bool) {
    ENABLED.with(|e| e.set(enabled));
    if !enabled {
        clear();
    }
}

/// Returns whether the `is_enabled` condition is satisfied.
pub fn is_enabled() -> bool {
    ENABLED.with(Cell::get)
}

/// Runs `f` with render-log recording paused. A widget that rebuilds its
/// own display of the log (the DevTools panel itself) must wrap that
/// rebuild in this, or its own churn shows up as new entries and keeps
/// waking itself back up.
pub fn with_suppressed<R>(f: impl FnOnce() -> R) -> R {
    SUPPRESS_DEPTH.with(|d| d.set(d.get() + 1));
    let result = f();
    SUPPRESS_DEPTH.with(|d| d.set(d.get().saturating_sub(1)));
    result
}

fn is_suppressed() -> bool {
    SUPPRESS_DEPTH.with(|d| d.get() > 0)
}

// A path is "inside" the DevTools panel when one of its dot-separated
// segments is the panel's own keyed segment - matches the panel widget
// itself and everything nested under it (resize handle, buttons, rows...).
fn is_devtools_panel_path(widget_path: &str) -> bool {
    widget_path.split('.').any(|segment| {
        segment
            .strip_prefix('k')
            .is_some_and(|key| key == DEVTOOLS_PANEL_KEY)
    })
}

fn notify_new_entry() {
    let should_notify = LAST_NOTIFY.with(|cell| {
        let now = Instant::now();
        let due = cell
            .get()
            .is_none_or(|last| (now.duration_since(last).as_millis() as u64) >= NOTIFY_THROTTLE_MS);
        if due {
            cell.set(Some(now));
        }
        due
    });
    if should_notify {
        crate::hooks::mark_dirty_and_redraw();
    }
}

fn push(kind: RenderEventKind, widget_path: &str, widget_name: &'static str, reason: String) {
    if !is_enabled() || is_suppressed() || is_devtools_panel_path(widget_path) {
        return;
    }
    let epoch_millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    LOG.with(|log| {
        let mut log = log.borrow_mut();
        if log.len() == CAPACITY {
            log.pop_front();
        }
        log.push_back(RenderLogEntry {
            epoch_millis,
            kind,
            widget_path: widget_path.to_string(),
            widget_name,
            reason,
        });
    });
    notify_new_entry();
}

/// Returns or updates the `log_rerender` value.
pub fn log_rerender(widget_path: &str, widget_name: &'static str, reason: impl Into<String>) {
    push(
        RenderEventKind::Rerender,
        widget_path,
        widget_name,
        reason.into(),
    );
}

/// Returns or updates the `log_repaint` value.
pub fn log_repaint(widget_path: &str, widget_name: &'static str, reason: impl Into<String>) {
    push(
        RenderEventKind::Repaint,
        widget_path,
        widget_name,
        reason.into(),
    );
}

/// Logs that a full layout pass (taffy tree rebuild + re-apply) actually
/// ran this frame, as opposed to the cheaper cascade/reflow-only path.
pub fn log_layout(widget_path: &str, widget_name: &'static str, reason: impl Into<String>) {
    push(
        RenderEventKind::Layout,
        widget_path,
        widget_name,
        reason.into(),
    );
}

/// Returns or updates the `log_warning` value.
pub fn log_warning(widget_path: &str, widget_name: &'static str, reason: impl Into<String>) {
    push(
        RenderEventKind::Warning,
        widget_path,
        widget_name,
        reason.into(),
    );
}

/// Returns or updates the `log_error` value.
pub fn log_error(widget_path: &str, widget_name: &'static str, reason: impl Into<String>) {
    push(
        RenderEventKind::Error,
        widget_path,
        widget_name,
        reason.into(),
    );
}

/// Snapshot of every entry recorded so far, oldest first.
pub fn snapshot() -> Vec<RenderLogEntry> {
    LOG.with(|log| log.borrow().iter().cloned().collect())
}

pub fn clear() {
    LOG.with(|log| log.borrow_mut().clear());
}
