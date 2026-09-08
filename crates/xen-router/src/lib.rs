// SPDX-License-Identifier: Apache-2.0
//! A lightweight client-side router for xengui.
//!
//! Route state lives in a single thread-local, the same pattern xengui
//! itself uses for the active theme and for `use_state`: navigating just
//! updates that state and asks xengui to redraw, and the app's `render`
//! closure re-evaluates `Router::build` against the new path on the next
//! frame.
//!
//! On `wasm32`, the current path is kept in sync with the browser's
//! address bar via the History API (`pushState`/`popstate`), so links
//! are shareable and the back/forward buttons work. On native targets
//! there is no real URL - navigation only changes in-memory state.

mod link;
mod route_match;
mod router;
mod state;

pub use link::link;
pub use route_match::{RouteParams, match_route};
pub use router::Router;
pub use state::{
    NavigationDirection, back, current_path, forward, navigation_direction, push, replace,
    search_params,
};
