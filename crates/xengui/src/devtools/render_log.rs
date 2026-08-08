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
    pub t: Instant,
    pub kind: RenderEventKind,
    pub widget_path: String,
    pub widget_name: &'static str,
    pub reason: String,
}

const CAPACITY: usize = 2048;

thread_local! {
    static LOG: RefCell<VecDeque<RenderLogEntry>> = RefCell::new(VecDeque::with_capacity(CAPACITY));
    static START: Instant = Instant::now();
    static ENABLED: Cell<bool> = const { Cell::new(false) };
}

/// Toggled by the DevTools panel (F12). While disabled, `log_rerender`/
/// `log_repaint` are a single thread-local bool read - callers on hot
/// paths (interaction dispatch, reconciliation) still guard the more
/// expensive `format!()` call sites with `is_enabled()` themselves.
pub fn set_enabled(enabled: bool) {
    ENABLED.with(|e| e.set(enabled));
    if !enabled {
        clear();
    }
}

pub fn is_enabled() -> bool {
    ENABLED.with(Cell::get)
}

fn push(kind: RenderEventKind, widget_path: &str, widget_name: &'static str, reason: String) {
    if !is_enabled() {
        return;
    }
    LOG.with(|log| {
        let mut log = log.borrow_mut();
        if log.len() == CAPACITY {
            log.pop_front();
        }
        log.push_back(RenderLogEntry {
            t: Instant::now(),
            kind,
            widget_path: widget_path.to_string(),
            widget_name,
            reason,
        });
    });
}

pub fn log_rerender(widget_path: &str, widget_name: &'static str, reason: impl Into<String>) {
    push(RenderEventKind::Rerender, widget_path, widget_name, reason.into());
}

pub fn log_repaint(widget_path: &str, widget_name: &'static str, reason: impl Into<String>) {
    push(RenderEventKind::Repaint, widget_path, widget_name, reason.into());
}

/// Snapshot of every entry recorded so far, oldest first, with a
/// microsecond offset from the first-ever recorded event.
pub fn snapshot() -> Vec<(u128, RenderEventKind, String, &'static str, String)> {
    let start = START.with(|s| *s);
    LOG.with(|log| {
        log.borrow()
            .iter()
            .map(|e| (
                e.t.duration_since(start).as_micros(),
                e.kind,
                e.widget_path.clone(),
                e.widget_name,
                e.reason.clone(),
            ))
            .collect()
    })
}

pub fn clear() {
    LOG.with(|log| log.borrow_mut().clear());
}
