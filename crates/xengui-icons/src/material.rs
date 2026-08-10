pub struct Material;

impl Material {
    pub const FONT: &[u8] = include_bytes!("../fonts/material-symbols-rounded.woff2");

    pub const CHECK: &str = "check";
    pub const ADD: &str = "add";
    pub const PLUS: &str = "add";
    pub const REMOVE: &str = "remove";
    pub const MINUS: &str = "remove";
    pub const CLOSE: &str = "close";
    pub const CLOSE_SMALL: &str = "close_small";
    pub const X: &str = "close";
    pub const X_SMALL: &str = "close_small";
    pub const SEARCH: &str = "search";
}
