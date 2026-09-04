// SPDX-License-Identifier: Apache-2.0
use crate::{Display, FlexDirection, StyleBuilder, View};

/// Sugar for `View::new().display(Display::Flex).flex_direction(FlexDirection::Row)`.
/// `Row::new()` returns a plain [`View`], so every builder method
/// (including `.display(...)` and `.flex_direction(...)` themselves) can
/// still override the defaults afterward exactly like on any other `View`.
pub struct Row;

impl Row {
    #[allow(clippy::new_ret_no_self)]
    /// Creates a value with its default configuration.
    pub fn new() -> View {
        View::new()
            .display(Display::Flex)
            .flex_direction(FlexDirection::Row)
    }
}

/// Sugar for `View::new().display(Display::Flex).flex_direction(FlexDirection::Column)`.
/// `Column::new()` returns a plain [`View`], so every builder method
/// (including `.display(...)` and `.flex_direction(...)` themselves) can
/// still override the defaults afterward exactly like on any other `View`.
pub struct Column;

impl Column {
    #[allow(clippy::new_ret_no_self)]
    /// Creates a value with its default configuration.
    pub fn new() -> View {
        View::new()
            .display(Display::Flex)
            .flex_direction(FlexDirection::Column)
    }
}
