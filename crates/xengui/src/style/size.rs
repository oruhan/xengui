// SPDX-License-Identifier: Apache-2.0
use super::Length;

#[derive(Clone, Copy, Debug, PartialEq, Default)]
/// Data and behavior represented by `Size`.
pub struct Size {
    /// The `width` value carried by this type.
    pub width: Option<Length>,
    /// The `height` value carried by this type.
    pub height: Option<Length>,
}

impl Size {
    /// Creates a value with its default configuration.
    pub const fn new(width: Length, height: Length) -> Self {
        Self {
            width: Some(width),
            height: Some(height),
        }
    }
}
