//! Shared visual tokens for the XenGui website.
//!
//! Typography and shape values follow the local Material 3 specification.
//! Page gutters and shell geometry are XenGui website design decisions.
#![allow(dead_code)]

use std::time::Duration;
use xengui::{Easing, Edges, Responsive, Transition};

pub mod type_scale {
    pub const DISPLAY_LG: f32 = 57.0;
    pub const DISPLAY_LG_LINE: f32 = 64.0;
    pub const DISPLAY_MD: f32 = 45.0;
    pub const DISPLAY_MD_LINE: f32 = 52.0;
    pub const DISPLAY_SM: f32 = 36.0;
    pub const DISPLAY_SM_LINE: f32 = 44.0;
    pub const HEADLINE_LG: f32 = 32.0;
    pub const HEADLINE_LG_LINE: f32 = 40.0;
    pub const HEADLINE_MD: f32 = 28.0;
    pub const HEADLINE_MD_LINE: f32 = 36.0;
    pub const HEADLINE_SM: f32 = 24.0;
    pub const HEADLINE_SM_LINE: f32 = 32.0;
    pub const TITLE_LG: f32 = 22.0;
    pub const TITLE_LG_LINE: f32 = 28.0;
    pub const TITLE_MD: f32 = 16.0;
    pub const TITLE_MD_LINE: f32 = 24.0;
    pub const TITLE_SM: f32 = 14.0;
    pub const TITLE_SM_LINE: f32 = 20.0;
    pub const BODY_LG: f32 = 16.0;
    pub const BODY_LG_LINE: f32 = 24.0;
    pub const BODY_MD: f32 = 14.0;
    pub const BODY_MD_LINE: f32 = 20.0;
    pub const BODY_SM: f32 = 12.0;
    pub const BODY_SM_LINE: f32 = 16.0;
    pub const LABEL_LG: f32 = 14.0;
    pub const LABEL_LG_LINE: f32 = 20.0;
    pub const LABEL_MD: f32 = 12.0;
    pub const LABEL_MD_LINE: f32 = 16.0;
    pub const LABEL_SM: f32 = 11.0;
    pub const LABEL_SM_LINE: f32 = 16.0;
}

pub mod radius {
    pub const XS: f32 = 4.0;
    pub const SM: f32 = 8.0;
    pub const MD: f32 = 12.0;
    pub const LG: f32 = 16.0;
    pub const LG_INCREASED: f32 = 20.0;
    pub const XL: f32 = 28.0;
    pub const XL_INCREASED: f32 = 32.0;
    pub const XXL: f32 = 48.0;
    pub const FULL: f32 = 999.0;
}

pub mod space {
    pub const XS: f32 = 4.0;
    pub const SM: f32 = 8.0;
    pub const MD: f32 = 12.0;
    pub const LG: f32 = 16.0;
    pub const XL: f32 = 24.0;
    pub const XXL: f32 = 32.0;
    pub const SECTION: f32 = 48.0;
}

pub const CONTENT_MAX: f32 = 1120.0;
pub const ARTICLE_MAX: f32 = 960.0;

pub fn page_gutter(top: f32, bottom: f32) -> Responsive<Edges> {
    Responsive::new(Edges::only(20.0, top, 20.0, bottom))
        .md(Edges::only(48.0, top, 48.0, bottom))
        .lg(Edges::only(72.0, top, 72.0, bottom))
}

pub fn fast_effect() -> Transition {
    Transition::new(Duration::from_millis(150)).easing(Easing::cubic_bezier(0.31, 0.94, 0.34, 1.0))
}

pub fn fast_spatial() -> Transition {
    Transition::new(Duration::from_millis(350)).easing(Easing::cubic_bezier(0.42, 1.67, 0.21, 0.90))
}
