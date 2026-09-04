// SPDX-License-Identifier: Apache-2.0
use crate::{AnimationManager, TextMeasurer};

/// Data and behavior represented by `LayoutContext`.
pub struct LayoutContext<'a> {
    /// The `text` value carried by this type.
    pub text: &'a mut dyn TextMeasurer,
    /// The `anim` value carried by this type.
    pub anim: &'a mut AnimationManager,
    /// The `scale_factor` value carried by this type.
    pub scale_factor: f32,
}
