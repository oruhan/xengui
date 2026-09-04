// SPDX-License-Identifier: Apache-2.0
//! Minimal event-timeline recorder used to diagnose frame-timing issues
//! (e.g. the borderless-window resize jitter on Windows). Any crate in
//! the workspace can push timestamped events into it and dump them as a
//! single ordered log at a convenient moment (e.g. WM_EXITSIZEMOVE).

mod render_log;
pub use render_log::*;

use std::cell::RefCell;
use std::collections::VecDeque;
use web_time::Instant;

struct Event {
    t: Instant,
    tag: &'static str,
    size: Option<(u32, u32)>,
    note: Option<String>,
}

const CAPACITY: usize = 16384;

thread_local! {
    static EVENTS: RefCell<VecDeque<Event>> = RefCell::new(VecDeque::with_capacity(CAPACITY));
    static START: Instant = Instant::now();
}

/// Returns or updates the `record` value.
pub fn record(tag: &'static str) {
    push(tag, None, None);
}

/// Returns or updates the `record_size` value.
pub fn record_size(tag: &'static str, width: u32, height: u32) {
    push(tag, Some((width, height)), None);
}

/// Returns or updates the `record_note` value.
pub fn record_note(tag: &'static str, note: impl Into<String>) {
    push(tag, None, Some(note.into()));
}

/// Returns or updates the `record_size_note` value.
pub fn record_size_note(tag: &'static str, width: u32, height: u32, note: impl Into<String>) {
    push(tag, Some((width, height)), Some(note.into()));
}

fn push(tag: &'static str, size: Option<(u32, u32)>, note: Option<String>) {
    EVENTS.with(|events| {
        let mut events = events.borrow_mut();
        if events.len() == CAPACITY {
            events.pop_front();
        }
        events.push_back(Event {
            t: Instant::now(),
            tag,
            size,
            note,
        });
    });
}

/// Prints every buffered event as a `log::info!` line, oldest first, with
/// a microsecond offset from the first-ever recorded event so deltas
/// between lines are directly readable. Clears the buffer afterward.
pub fn dump(label: &str) {
    let start = START.with(|s| *s);
    EVENTS.with(|events| {
        let mut events = events.borrow_mut();
        log::info!("=== devtools dump: {label} ({} events) ===", events.len());
        for event in events.iter() {
            let us = event.t.duration_since(start).as_micros();
            match (&event.size, &event.note) {
                (Some((w, h)), Some(note)) => {
                    log::info!("[{us:>12}us] {:<28} {w}x{h}  {note}", event.tag)
                }
                (Some((w, h)), None) => log::info!("[{us:>12}us] {:<28} {w}x{h}", event.tag),
                (None, Some(note)) => log::info!("[{us:>12}us] {:<28} {note}", event.tag),
                (None, None) => log::info!("[{us:>12}us] {}", event.tag),
            }
        }
        log::info!("=== end dump ===");
        events.clear();
    });
}

/// Performs the `clear` operation.
pub fn clear() {
    EVENTS.with(|events| events.borrow_mut().clear());
}
