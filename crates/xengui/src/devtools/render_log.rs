// SPDX-License-Identifier: Apache-2.0
//! Structured, timestamped rerender/repaint log for the in-app DevTools
//! panel. Separate from `devtools::record*` (which targets frame-timing/
//! resize debugging) - this one is keyed by widget path and carries a
//! human-readable reason string.

use std::cell::{ Cell, RefCell };
use std::collections::VecDeque;
use web_time::Instant;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderEventKind {
    Rerender,
    Repaint,
}

#[derive(Clone, Debug)]
pub struct RenderLogEntry {
    pub t_micros: u128,
    pub kind: RenderEventKind,
    pub widget_path: String,
    pub widget_name: &'static str,
    pub reason: String,
}

const CAPACITY: usize = 2048;
const NOTIFY_THROTTLE_MS: u64 = 300;

thread_local! {
    static LOG: RefCell<VecDeque<RenderLogEntry>> = RefCell::new(VecDeque::with_capacity(CAPACITY));
    static START: Instant = Instant::now();
    static ENABLED: Cell<bool> = const { Cell::new(false) };
    static LAST_NOTIFY: Cell<Option<Instant>> = const { Cell::new(None) };
    static SUPPRESS_DEPTH: Cell<u32> = const { Cell::new(0) };
}

pub fn set_enabled(enabled: bool) {
    ENABLED.with(|e| e.set(enabled));
    if !enabled {
        clear();
    }
}

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

fn notify_new_entry() {
    let should_notify = LAST_NOTIFY.with(|cell| {
        let now = Instant::now();
        let due = cell
            .get()
            .is_none_or(|last| {
                (now.duration_since(last).as_millis() as u64) >= NOTIFY_THROTTLE_MS
            });
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
    if !is_enabled() || is_suppressed() {
        return;
    }
    let t_micros = START.with(|s| Instant::now().duration_since(*s).as_micros());
    LOG.with(|log| {
        let mut log = log.borrow_mut();
        if log.len() == CAPACITY {
            log.pop_front();
        }
        log.push_back(RenderLogEntry {
            t_micros,
            kind,
            widget_path: widget_path.to_string(),
            widget_name,
            reason,
        });
    });
    notify_new_entry();
}

pub fn log_rerender(widget_path: &str, widget_name: &'static str, reason: impl Into<String>) {
    push(RenderEventKind::Rerender, widget_path, widget_name, reason.into());
}

pub fn log_repaint(widget_path: &str, widget_name: &'static str, reason: impl Into<String>) {
    push(RenderEventKind::Repaint, widget_path, widget_name, reason.into());
}

/// Snapshot of every entry recorded so far, oldest first.
pub fn snapshot() -> Vec<RenderLogEntry> {
    LOG.with(|log| log.borrow().iter().cloned().collect())
}

pub fn clear() {
    LOG.with(|log| log.borrow_mut().clear());
}
