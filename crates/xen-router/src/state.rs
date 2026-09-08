// SPDX-License-Identifier: Apache-2.0
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use xengui::hooks;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;

/// Direction of the navigation operation that selected the current route.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum NavigationDirection {
    #[default]
    Forward,
    Backward,
    Replace,
}

thread_local! {
    static CURRENT_PATH: RefCell<String> = RefCell::new(initial_path());
    static CURRENT_SEARCH: RefCell<HashMap<String, String>> = RefCell::new(initial_search());
    static POPSTATE_INSTALLED: Cell<bool> = const { Cell::new(false) };
    static NAVIGATION_DIRECTION: Cell<NavigationDirection> = const { Cell::new(NavigationDirection::Forward) };
    #[cfg(not(target_arch = "wasm32"))]
    static NATIVE_HISTORY: RefCell<Vec<String>> = RefCell::new(vec![initial_path()]);
    #[cfg(not(target_arch = "wasm32"))]
    static NATIVE_HISTORY_INDEX: Cell<usize> = const { Cell::new(0) };
}

#[cfg(target_arch = "wasm32")]
fn initial_path() -> String {
    web_sys::window()
        .and_then(|w| w.location().pathname().ok())
        .unwrap_or_else(|| "/".to_string())
}

#[cfg(not(target_arch = "wasm32"))]
fn initial_path() -> String {
    "/".to_string()
}

#[cfg(target_arch = "wasm32")]
fn initial_search() -> HashMap<String, String> {
    web_sys::window()
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
        if bytes[i] == b'%'
            && i + 2 < bytes.len()
            && let Ok(byte) =
                u8::from_str_radix(std::str::from_utf8(&bytes[i + 1..i + 3]).unwrap_or(""), 16)
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

pub fn navigation_direction() -> NavigationDirection {
    NAVIGATION_DIRECTION.with(Cell::get)
}

/// Navigates to `path`, pushing a new browser history entry on wasm32.
pub fn push(path: impl Into<String>) {
    NAVIGATION_DIRECTION.with(|direction| direction.set(NavigationDirection::Forward));
    set_path(path.into(), true);
}

/// Like `push`, but replaces the current history entry instead of
/// pushing a new one - useful for redirects that shouldn't be reachable
/// via the back button.
pub fn replace(path: impl Into<String>) {
    NAVIGATION_DIRECTION.with(|direction| direction.set(NavigationDirection::Replace));
    set_path(path.into(), false);
}

/// Navigates one entry back in browser history, mirroring the browser's
/// own back button. No-op on native targets, where there is no real
/// history stack.
pub fn back() -> bool {
    NAVIGATION_DIRECTION.with(|direction| direction.set(NavigationDirection::Backward));
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window()
            && let Ok(history) = window.history()
        {
            let _ = history.back();
            return true;
        }
        false
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        navigate_native_history(-1)
    }
}

/// Navigates one entry forward in browser history. No-op on native targets.
pub fn forward() {
    NAVIGATION_DIRECTION.with(|direction| direction.set(NavigationDirection::Forward));
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window()
            && let Ok(history) = window.history()
        {
            let _ = history.forward();
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    navigate_native_history(1);
}

fn set_path(path: String, push: bool) {
    #[cfg(target_arch = "wasm32")]
    sync_browser_url(&path, push);

    #[cfg(not(target_arch = "wasm32"))]
    update_native_history(&path, push);

    CURRENT_PATH.with(|p| {
        *p.borrow_mut() = path;
    });

    #[cfg(target_arch = "wasm32")]
    CURRENT_SEARCH.with(|s| {
        *s.borrow_mut() = initial_search();
    });

    hooks::mark_dirty_and_redraw();
}

#[cfg(not(target_arch = "wasm32"))]
fn update_native_history(path: &str, push: bool) {
    NATIVE_HISTORY.with(|history| {
        NATIVE_HISTORY_INDEX.with(|index| {
            let mut history = history.borrow_mut();
            let current = index.get().min(history.len().saturating_sub(1));
            if push {
                history.truncate(current + 1);
                history.push(path.to_owned());
                index.set(history.len() - 1);
            } else {
                history[current] = path.to_owned();
                index.set(current);
            }
        });
    });
}

#[cfg(not(target_arch = "wasm32"))]
fn navigate_native_history(offset: isize) -> bool {
    let next_path = NATIVE_HISTORY.with(|history| {
        NATIVE_HISTORY_INDEX.with(|index| {
            let history = history.borrow();
            let current = index.get() as isize;
            let next = (current + offset).clamp(0, history.len().saturating_sub(1) as isize);
            if next == current {
                return None;
            }
            index.set(next as usize);
            history.get(next as usize).cloned()
        })
    });

    if let Some(path) = next_path {
        CURRENT_PATH.with(|current| *current.borrow_mut() = path);
        hooks::mark_dirty_and_redraw();
        true
    } else {
        false
    }
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
            },
        );
        let _ =
            window.add_event_listener_with_callback("popstate", closure.as_ref().unchecked_ref());
        closure.forget();
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        POPSTATE_INSTALLED.with(|f| f.set(true));
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;

    fn reset_history() {
        CURRENT_PATH.with(|path| *path.borrow_mut() = "/".to_owned());
        NATIVE_HISTORY.with(|history| *history.borrow_mut() = vec!["/".to_owned()]);
        NATIVE_HISTORY_INDEX.with(|index| index.set(0));
    }

    #[test]
    fn native_history_supports_push_back_forward_and_replace() {
        reset_history();
        push("/network");
        push("/network/vpn");
        back();
        assert_eq!(current_path(), "/network");
        forward();
        assert_eq!(current_path(), "/network/vpn");
        replace("/network/private-dns");
        assert_eq!(current_path(), "/network/private-dns");
        back();
        assert_eq!(current_path(), "/network");
    }

    #[test]
    fn push_after_back_discards_forward_entries() {
        reset_history();
        push("/apps");
        push("/notifications");
        back();
        push("/display");
        forward();
        assert_eq!(current_path(), "/display");
    }
}
