// SPDX-License-Identifier: Apache-2.0
use super::Length;

#[derive(Clone, Copy, Debug, PartialEq, Default)]
/// Data and behavior represented by `Edges`.
pub struct Edges {
    /// The `left` value carried by this type.
    pub left: Length,
    /// The `top` value carried by this type.
    pub top: Length,
    /// The `right` value carried by this type.
    pub right: Length,
    /// The `bottom` value carried by this type.
    pub bottom: Length,
}

impl Edges {
    /// Returns or updates the `all` value.
    pub fn all<L>(value: L) -> Self
    where
        L: Into<Length>,
    {
        let value = value.into();

        Self {
            left: value,
            top: value,
            right: value,
            bottom: value,
        }
    }

    /// Returns or updates the `symmetric` value.
    pub fn symmetric<H, V>(horizontal: H, vertical: V) -> Self
    where
        H: Into<Length>,
        V: Into<Length>,
    {
        let horizontal = horizontal.into();
        let vertical = vertical.into();

        Self {
            left: horizontal,
            right: horizontal,
            top: vertical,
            bottom: vertical,
        }
    }

    /// Returns or updates the `only` value.
    pub fn only<L, T, R, B>(left: L, top: T, right: R, bottom: B) -> Self
    where
        L: Into<Length>,
        T: Into<Length>,
        R: Into<Length>,
        B: Into<Length>,
    {
        Self {
            left: left.into(),
            top: top.into(),
            right: right.into(),
            bottom: bottom.into(),
        }
    }

    /// Returns or updates the `left` value.
    pub const fn left(&self) -> Length {
        self.left
    }

    /// Returns or updates the `top` value.
    pub const fn top(&self) -> Length {
        self.top
    }

    /// Returns or updates the `right` value.
    pub const fn right(&self) -> Length {
        self.right
    }

    /// Returns or updates the `bottom` value.
    pub const fn bottom(&self) -> Length {
        self.bottom
    }
}

impl From<Length> for Edges {
    fn from(value: Length) -> Self {
        Self::all(value)
    }
}
