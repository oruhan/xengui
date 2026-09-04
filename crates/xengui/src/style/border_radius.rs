// SPDX-License-Identifier: Apache-2.0
use super::Length;

/// Independent corner radii, matching CSS `border-radius: tl tr br bl`.
///
/// Every rounded-rect draw path (box background/border, image clipping,
/// box shadow, focus outline) accepts this instead of a single uniform
/// [`Length`], so a widget can round e.g. only its top corners.
///
/// A single value still works everywhere a `BorderRadius` is expected via
/// the blanket `From<L: Into<Length>>` impl, so existing code like
/// `.border(Border::new(1.0, color, 8.0))` keeps compiling unchanged and
/// applies `8.0` uniformly to all four corners.
///
/// # Example
/// ```
/// use xengui::{BorderRadius, Length};
///
/// // Only round the top corners, e.g. for a card header.
/// let radius = BorderRadius::top(12.0);
///
/// // Fully custom per-corner radii.
/// let radius = BorderRadius::only(4.0, 4.0, 16.0, 16.0);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct BorderRadius {
    /// The `top_left` value carried by this type.
    pub top_left: Length,
    /// The `top_right` value carried by this type.
    pub top_right: Length,
    /// The `bottom_right` value carried by this type.
    pub bottom_right: Length,
    /// The `bottom_left` value carried by this type.
    pub bottom_left: Length,
}

impl BorderRadius {
    /// Same radius on all four corners.
    pub fn all(radius: impl Into<Length>) -> Self {
        let radius = radius.into();
        Self {
            top_left: radius,
            top_right: radius,
            bottom_right: radius,
            bottom_left: radius,
        }
    }

    /// Explicit per-corner radii, in CSS `border-radius` order
    /// (top-left, top-right, bottom-right, bottom-left).
    pub fn only(
        top_left: impl Into<Length>,
        top_right: impl Into<Length>,
        bottom_right: impl Into<Length>,
        bottom_left: impl Into<Length>,
    ) -> Self {
        Self {
            top_left: top_left.into(),
            top_right: top_right.into(),
            bottom_right: bottom_right.into(),
            bottom_left: bottom_left.into(),
        }
    }

    /// Rounds only the top-left and top-right corners.
    pub fn top(radius: impl Into<Length>) -> Self {
        let radius = radius.into();
        Self {
            top_left: radius,
            top_right: radius,
            ..Self::default()
        }
    }

    /// Rounds only the bottom-left and bottom-right corners.
    pub fn bottom(radius: impl Into<Length>) -> Self {
        let radius = radius.into();
        Self {
            bottom_left: radius,
            bottom_right: radius,
            ..Self::default()
        }
    }

    /// Rounds only the top-left and bottom-left corners.
    pub fn left(radius: impl Into<Length>) -> Self {
        let radius = radius.into();
        Self {
            top_left: radius,
            bottom_left: radius,
            ..Self::default()
        }
    }

    /// Rounds only the top-right and bottom-right corners.
    pub fn right(radius: impl Into<Length>) -> Self {
        let radius = radius.into();
        Self {
            top_right: radius,
            bottom_right: radius,
            ..Self::default()
        }
    }

    /// Rounds a single corner, leaving the other three square.
    pub fn top_left(radius: impl Into<Length>) -> Self {
        Self {
            top_left: radius.into(),
            ..Self::default()
        }
    }

    /// Returns or updates the `top_right` value.
    pub fn top_right(radius: impl Into<Length>) -> Self {
        Self {
            top_right: radius.into(),
            ..Self::default()
        }
    }

    /// Returns or updates the `bottom_right` value.
    pub fn bottom_right(radius: impl Into<Length>) -> Self {
        Self {
            bottom_right: radius.into(),
            ..Self::default()
        }
    }

    /// Returns or updates the `bottom_left` value.
    pub fn bottom_left(radius: impl Into<Length>) -> Self {
        Self {
            bottom_left: radius.into(),
            ..Self::default()
        }
    }

    /// Whether all four corners share the same radius - lets callers keep
    /// using the cheaper uniform code path (e.g. simple hit-testing) when
    /// no widget actually asked for asymmetric corners.
    pub fn is_uniform(&self) -> bool {
        self.top_left == self.top_right
            && self.top_right == self.bottom_right
            && self.bottom_right == self.bottom_left
    }

    /// The largest of the four radii, in logical px - used where only an
    /// approximate/uniform radius is needed (e.g. clamping against half
    /// the box size before resolving per-corner).
    pub fn max_value(&self) -> f32 {
        self.top_left
            .value()
            .max(self.top_right.value())
            .max(self.bottom_right.value())
            .max(self.bottom_left.value())
    }

    /// Resolves every corner to physical px, clamped so no pair of
    /// adjacent corners can overlap (matches the CSS `border-radius`
    /// overlap-correction algorithm, applied independently per axis).
    pub fn to_physical_array(self, scale_factor: f32, width: f32, height: f32) -> [f32; 4] {
        let mut tl = self.top_left.to_physical(scale_factor).max(0.0);
        let mut tr = self.top_right.to_physical(scale_factor).max(0.0);
        let mut br = self.bottom_right.to_physical(scale_factor).max(0.0);
        let mut bl = self.bottom_left.to_physical(scale_factor).max(0.0);

        // CSS border-radius overlap correction: scale every radius pair
        // sharing an edge down by the same factor if their sum would
        // exceed that edge's length.
        let scale_pair = |a: &mut f32, b: &mut f32, limit: f32| {
            let sum = *a + *b;
            if sum > limit && sum > 0.0 {
                let f = limit / sum;
                *a *= f;
                *b *= f;
            }
        };

        scale_pair(&mut tl, &mut tr, width);
        scale_pair(&mut bl, &mut br, width);
        scale_pair(&mut tl, &mut bl, height);
        scale_pair(&mut tr, &mut br, height);

        [tl, tr, br, bl]
    }
}

impl<L: Into<Length>> From<L> for BorderRadius {
    fn from(value: L) -> Self {
        Self::all(value)
    }
}
