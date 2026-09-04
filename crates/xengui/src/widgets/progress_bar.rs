// SPDX-License-Identifier: Apache-2.0
//! Determinate progress indicators.

use crate::{
    Border, Color, Interaction, LayoutBox, Length, Render, Style, StyleBuilder, View, Widget,
    WidgetBase, WidgetId, pct,
};

/// A horizontal determinate progress indicator.
///
/// Values are clamped to the inclusive `0.0..=1.0` range.
pub struct ProgressBar {
    base: WidgetBase,
    layout_box: LayoutBox,
    inner: Vec<Box<dyn Widget>>,
    hooks_id: WidgetId,
    value: f32,
    track_color: Option<Color>,
    fill_color: Option<Color>,
    height: Length,
}

impl ProgressBar {
    /// Creates an empty progress bar.
    pub fn new() -> Self {
        Self {
            base: WidgetBase::new(Interaction::new()),
            layout_box: LayoutBox::default(),
            inner: Vec::new(),
            hooks_id: WidgetId::new_unique(),
            value: 0.0,
            track_color: None,
            fill_color: None,
            height: Length::px(6.0),
        }
    }

    /// Sets progress as a fraction in `0.0..=1.0`.
    pub fn value(mut self, value: f32) -> Self {
        self.value = if value.is_finite() {
            value.clamp(0.0, 1.0)
        } else {
            0.0
        };
        self
    }

    /// Overrides the unfilled track color.
    pub fn track_color(mut self, color: Color) -> Self {
        self.track_color = Some(color);
        self
    }

    /// Overrides the filled portion's color.
    pub fn fill_color(mut self, color: Color) -> Self {
        self.fill_color = Some(color);
        self
    }

    /// Sets the bar height in logical units.
    pub fn bar_height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }
}

impl Default for ProgressBar {
    fn default() -> Self {
        Self::new()
    }
}

impl StyleBuilder for ProgressBar {
    fn style_mut(&mut self) -> &mut Style {
        &mut self.base.style
    }

    fn mark_dirty(&mut self) {
        self.base.dirty = true;
    }
}

impl Render for ProgressBar {
    fn render(&self) -> Box<dyn Widget> {
        let theme = crate::current_theme();
        Box::new(
            View::new()
                .width(pct!(100.0))
                .height(self.height)
                .background(self.track_color.unwrap_or(theme.surface_container_highest))
                .border(Border::all(0.0, Color::TRANSPARENT).radius(Length::px(999.0)))
                .overflow_x(crate::Overflow::Hidden)
                .child(
                    View::new()
                        .width(pct!(self.value * 100.0))
                        .height(pct!(100.0))
                        .background(self.fill_color.unwrap_or(theme.primary))
                        .border(Border::all(0.0, Color::TRANSPARENT).radius(Length::px(999.0))),
                ),
        )
    }
}

crate::impl_composite_widget!(ProgressBar);

#[cfg(test)]
mod tests {
    use super::ProgressBar;

    #[test]
    fn clamps_value_to_valid_fraction() {
        assert_eq!(ProgressBar::new().value(-1.0).value, 0.0);
        assert_eq!(ProgressBar::new().value(2.0).value, 1.0);
        assert_eq!(ProgressBar::new().value(f32::NAN).value, 0.0);
    }
}
