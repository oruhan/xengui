// SPDX-License-Identifier: Apache-2.0
#![deny(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]
//! Cross-platform, declarative user-interface primitives for Rust.
//!
//! `xengui` owns the widget tree, styling, layout, input dispatch, hooks,
//! animation, and platform-independent paint commands. Use [`xenframe`](https://docs.rs/xenframe)
//! for a ready-made native/web application loop and
//! [`xengui-wgpu`](https://docs.rs/xengui-wgpu) for GPU rendering.
//!
//! Most applications can start with the crate prelude:
//!
//! ```no_run
//! use xengui::*;
//!
//! let content = Column::new()
//!     .gap(0.0, 12.0)
//!     .child(Label::new().label("Hello"))
//!     .child(Button::new().label("Continue"));
//! # let _: View = content;
//! ```
//!
//! The public API is grouped into [`widgets`], [`style`], [`layout`],
//! [`hooks`], [`input`], [`animation`], and [`paint`].

/// Animation values, transitions, easing, and runtime state.
pub mod animation;
/// Support for user-defined widgets built from existing widgets.
pub mod composite;
/// Framework-wide interaction and rendering constants.
pub mod constants;
/// Typed context values scoped to a render operation.
pub mod context;
/// Runtime render diagnostics and development tooling.
pub mod devtools;
/// Standalone input dispatcher for host integrations.
pub mod dispatcher;
/// Programmatic actions targeting widgets by identifier.
pub mod dom;
/// Component state, effects, and asynchronous resource hooks.
pub mod hooks;
/// Platform-independent input events and event context.
pub mod input;
/// Reusable pointer and keyboard interaction state.
pub mod interaction;
/// Layout constraints, measurement, and Taffy integration.
pub mod layout;
/// Declarative view and widget implementation macros.
pub mod macros;
/// Platform-independent paint commands and frame traversal.
pub mod paint;
/// Target capability detection.
pub mod platform;
/// Keyed widget-tree reconciliation.
pub mod reconciler;
/// Redraw scheduling abstraction.
pub mod redraw;
/// Theme, layout, typography, and visual style values.
pub mod style;
/// Compatibility conversions for SVG colors.
pub mod svg_compat;
/// Lightweight GUI-thread asynchronous task execution.
pub mod task;
/// Text measurement and shaped glyph data.
pub mod text;
/// Shared geometry and result types.
pub mod types;
/// The core [`Widget`] contract and widget helpers.
pub mod widget;
/// Shared state embedded by built-in widgets.
pub mod widget_base;
/// Built-in controls, content, layout, and feedback widgets.
pub mod widgets;

pub use animation::{
    AnimKey, AnimLayer, AnimProperty, AnimValue, AnimationManager, Easing, Transition,
    TransitionOverrides, TransitionProperty, animate_computed_style,
};
pub use composite::Render;
pub use hooks::{
    ComponentId, ComponentKey, EffectCleanup, EffectDeps, Resource, ResourceState, SetState,
    component, use_effect, use_resource, use_resource_once, use_state,
};
pub use input::*;
pub use interaction::*;
pub use layout::*;
pub use macros::WidgetContent;
pub use paint::*;
pub use style::system_theme::SystemTheme;
pub use style::*;
pub use text::*;
pub use widget::{Widget, scaled_layout_box, scaled_layout_box_with_origin};
pub use widget_base::WidgetBase;

pub use constants::*;
pub use context::{ContextGuard, provide_context, use_context, with_context};
pub use dispatcher::Dispatcher;
pub use input::{
    InputEvent, Key, KeyState, any_wants_animation, dispatch_animation_tick, find_widget_mut,
};
pub use platform::{is_touch_platform, set_is_touch_platform};
pub use redraw::RedrawRequester;
pub use style::{
    Border, BoxShadow, BoxSizing, Color, Cursor, Edges, FlexDirection, FlexWrap, FontStyle,
    FontWeight, IntoThemed, Length, Overflow, Overscroll, Style, StyleBuilder, Theme, ThemeMode,
    current_theme, set_active_theme, set_active_theme_by_name,
};
pub use svg_compat::IntoSvgColor;
pub use task::spawn;
#[cfg(not(target_arch = "wasm32"))]
pub use task::spawn_blocking;
pub use types::*;
pub use widgets::*;
pub use xen_svg::{SvgColor, SvgDocument, SvgElement, Transform2D};

#[cfg(not(target_arch = "wasm32"))]
pub use widgets::image_source_from_path;
