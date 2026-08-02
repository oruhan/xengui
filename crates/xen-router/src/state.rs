// SPDX-License-Identifier: Apache-2.0
use std::cell::{ Cell, RefCell };
use std::collections::HashMap;
use xengui::hooks;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;

thread_local! {
    static CURRENT_PATH: RefCell<String> = RefCell::new(initial_path());
    static CURRENT_SEARCH: RefCell<HashMap<String, String>> = RefCell::new(initial_search());
    static POPSTATE_INSTALLED: Cell<bool> = const { Cell::new(false) };
}

#[cfg(target_arch = "wasm32")]
fn initial_path() -> String {
    web_sys
        ::window()
        .and_then(|w| w.location().pathname().ok())
        .unwrap_or_else(|| "/".to_string())
}

#[cfg(not(target_arch = "wasm32"))]
fn initial_path() -> String {
    "/".to_string()
}

#[cfg(target_arch = "wasm32")]
fn initial_search() -> HashMap<String, String> {
    web_sys
        ::window()
        .and_then(|w| w.location().search().ok())
        .map(|s| parse_query(&s))
        .unwrap_or_default()
}

#[cfg(not(target_arch = "wasm32"))]
fn initial_search() -> HashMap<String, String> {
    HashMap::new()
}

// Parses a "?a=1&b=2" style query string, percent-decoding both keys and
// values the same way a browser's URLSearchParams would.
#[cfg(target_arch = "wasm32")]
fn parse_query(query: &str) -> HashMap<String, String> {
    let query = query.strip_prefix('?').unwrap_or(query);
    let mut map = HashMap::new();
    if query.is_empty() {
        return map;
    }
    for pair in query.split('&') {
        let mut parts = pair.splitn(2, '=');
        let Some(key) = parts.next() else {
            continue;
        };
        let value = parts.next().unwrap_or("");
        map.insert(percent_decode(key), percent_decode(value));
    }
    map
}

#[cfg(target_arch = "wasm32")]
fn percent_decode(s: &str) -> String {
    let bytes = s.replace('+', " ");
    let bytes = bytes.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if
            bytes[i] == b'%' &&
            i + 2 < bytes.len() &&
            let Ok(byte) = u8::from_str_radix(
                std::str::from_utf8(&bytes[i + 1..i + 3]).unwrap_or(""),
                16
            )
        {
            out.push(byte);
            i += 3;
            continue;
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8(out).unwrap_or_default()
}

/// Returns the current URL's query-string parameters as a map, matching
/// Next.js's `useSearchParams()`. Always empty on native targets, since
/// there is no real URL there.
pub fn search_params() -> HashMap<String, String> {
    ensure_popstate_listener();
    CURRENT_SEARCH.with(|s| s.borrow().clone())
}

pub fn current_path() -> String {
    ensure_popstate_listener();
    CURRENT_PATH.with(|p| p.borrow().clone())
}

/// Navigates to `path`, pushing a new browser history entry on wasm32.
pub fn push(path: impl Into<String>) {
    set_path(path.into(), true);
}

/// Like `push`, but replaces the current history entry instead of
/// pushing a new one - useful for redirects that shouldn't be reachable
/// via the back button.
pub fn replace(path: impl Into<String>) {
    set_path(path.into(), false);
}

/// Navigates one entry back in browser history, mirroring the browser's
/// own back button. No-op on native targets, where there is no real
/// history stack.
pub fn back() {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window() && let Ok(history) = window.history() {
            let _ = history.back();
        }
    }
}

/// Navigates one entry forward in browser history. No-op on native targets.
pub fn forward() {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window() && let Ok(history) = window.history() {
            let _ = history.forward();
        }
    }
}

fn set_path(path: String, _push: bool) {
    #[cfg(target_arch = "wasm32")]
    sync_browser_url(&path, _push);

    CURRENT_PATH.with(|p| {
        *p.borrow_mut() = path;
    });

    #[cfg(target_arch = "wasm32")]
    CURRENT_SEARCH.with(|s| {
        *s.borrow_mut() = initial_search();
    });

    hooks::mark_dirty_and_redraw();
}

#[cfg(target_arch = "wasm32")]
fn sync_browser_url(path: &str, push: bool) {
    let Some(window) = web_sys::window() else {
        return;
    };
    let Ok(history) = window.history() else {
        return;
    };
    let _ = if push {
        history.push_state_with_url(&wasm_bindgen::JsValue::NULL, "", Some(path))
    } else {
        history.replace_state_with_url(&wasm_bindgen::JsValue::NULL, "", Some(path))
    };
}

// Installed lazily on first read instead of requiring a separate init
// call - simply depending on this crate and calling current_path()/
// push() is enough to get popstate sync on wasm32.
fn ensure_popstate_listener() {
    #[cfg(target_arch = "wasm32")]
    {
        if POPSTATE_INSTALLED.with(Cell::get) {
            return;
        }
        POPSTATE_INSTALLED.with(|f| f.set(true));

        let Some(window) = web_sys::window() else {
            return;
        };
        let window_for_closure = window.clone();

        let closure = wasm_bindgen::closure::Closure::<dyn FnMut(web_sys::Event)>::new(
            move |_event: web_sys::Event| {
                let path = window_for_closure
                    .location()
                    .pathname()
                    .unwrap_or_else(|_| "/".to_string());
                CURRENT_PATH.with(|p| {
                    *p.borrow_mut() = path;
                });
                CURRENT_SEARCH.with(|s| {
                    *s.borrow_mut() = initial_search();
                });
                hooks::mark_dirty_and_redraw();
            }
        );
        let _ = window.add_event_listener_with_callback(
            "popstate",
            closure.as_ref().unchecked_ref()
        );
        closure.forget();
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        POPSTATE_INSTALLED.with(|f| f.set(true));
    }
}
