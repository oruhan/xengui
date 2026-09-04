// SPDX-License-Identifier: Apache-2.0
//! Visual separators for grouping adjacent content.

use crate::{
    Color, Interaction, LayoutBox, Length, Render, Style, StyleBuilder, View, Widget, WidgetBase,
    WidgetId, pct,
};

/// Direction in which a [`Separator`] extends.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SeparatorOrientation {
    /// A horizontal line that fills the available width.
    #[default]
    Horizontal,
    /// A vertical line that fills the available height.
    Vertical,
}

/// A non-interactive line used to separate groups of content.
pub struct Separator {
    base: WidgetBase,
    layout_box: LayoutBox,
    inner: Vec<Box<dyn Widget>>,
    hooks_id: WidgetId,
    orientation: SeparatorOrientation,
    color: Option<Color>,
    thickness: Length,
}

impl Separator {
    /// Creates a horizontal separator.
    pub fn new() -> Self {
        Self {
            base: WidgetBase::new(Interaction::new()),
            layout_box: LayoutBox::default(),
            inner: Vec::new(),
            hooks_id: WidgetId::new_unique(),
            orientation: SeparatorOrientation::Horizontal,
            color: None,
            thickness: Length::px(1.0),
        }
    }

    /// Changes the separator's orientation.
    pub fn orientation(mut self, orientation: SeparatorOrientation) -> Self {
        self.orientation = orientation;
        self
    }

    /// Overrides the separator color.
    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    /// Sets the line thickness.
    pub fn thickness(mut self, thickness: impl Into<Length>) -> Self {
        self.thickness = thickness.into();
        self
    }
}

impl Default for Separator {
    fn default() -> Self {
        Self::new()
    }
}

impl StyleBuilder for Separator {
    fn style_mut(&mut self) -> &mut Style {
        &mut self.base.style
    }

    fn mark_dirty(&mut self) {
        self.base.dirty = true;
    }
}

impl Render for Separator {
    fn render(&self) -> Box<dyn Widget> {
        let color = self
            .color
            .unwrap_or_else(|| crate::current_theme().outline_variant);
        let line = match self.orientation {
            SeparatorOrientation::Horizontal => {
                View::new().width(pct!(100.0)).height(self.thickness)
            }
            SeparatorOrientation::Vertical => View::new().width(self.thickness).height(pct!(100.0)),
        };
        Box::new(line.background(color))
    }
}

crate::impl_composite_widget!(Separator);
