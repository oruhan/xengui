// SPDX-License-Identifier: Apache-2.0
//! Prebuilt icon widgets. Each constant is raw SVG markup using
//! `currentColor`, so it automatically follows the parent widget's text
//! color without any manual styling. No dependency on xengui itself -
//! consumers parse/render the string however their own GUI stack does.
//!
//! Adding a new icon: open it on lucide.dev, click "Copy SVG", save it as
//! `icons/<name>.svg`, then add one `include_str!` line below.

pub const X_SVG: &str = include_str!("../icons/x.svg");
pub const CHECK_SVG: &str = include_str!("../icons/check.svg");
pub const SEARCH_SVG: &str = include_str!("../icons/search.svg");
pub const PLUS_SVG: &str = include_str!("../icons/plus.svg");
pub const MINUS_SVG: &str = include_str!("../icons/minus.svg");
