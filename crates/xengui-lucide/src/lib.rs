// SPDX-License-Identifier: Apache-2.0
//! Prebuilt icon widgets. Each function returns a fresh `xengui::Svg` using
//! `currentColor`, so it automatically follows the parent widget's text
//! color without any manual styling.
//!
//! Adding a new icon: open the icon on lucide.dev, click "Copy SVG", paste
//! it as a `const &str` below, then add a one-line wrapper function - no
//! extra dependency is needed since this crate only ever parses raw SVG
//! markup through `xen-svg`.

pub const CHECK_SVG: &str =
    r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M20 6 9 17l-5-5"/></svg>"#;

pub const X_SVG: &str =
    r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M18 6 6 18"/><path d="m6 6 12 12"/></svg>"#;