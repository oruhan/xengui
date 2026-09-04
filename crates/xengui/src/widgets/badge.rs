// SPDX-License-Identifier: Apache-2.0
//! Compact status and category labels.

use crate::{
    Align, Border, Color, Display, Edges, FlexDirection, FontWeight, Interaction, Label, LayoutBox,
    Length, Render, Style, StyleBuilder, View, Widget, WidgetBase, WidgetId,
};
use smol_str::SmolStr;

/// A compact, non-interactive label used for statuses, counts, or categories.
pub struct Badge {
    base: WidgetBase,
    layout_box: LayoutBox,
    inner: Vec<Box<dyn Widget>>,
    hooks_id: WidgetId,
    label: SmolStr,
    background: Option<Color>,
    color: Option<Color>,
}

impl Badge {
    /// Creates an empty badge.
    pub fn new() -> Self {
        Self {
            base: WidgetBase::new(Interaction::new()),
            layout_box: LayoutBox::default(),
            inner: Vec::new(),
            hooks_id: WidgetId::new_unique(),
            label: SmolStr::new(""),
            background: None,
            color: None,
        }
    }

    /// Sets the text displayed inside the badge.
    pub fn label(mut self, label: impl Into<SmolStr>) -> Self {
        self.label = label.into();
        self
    }

    /// Overrides the badge background color.
    pub fn background(mut self, color: Color) -> Self {
        self.background = Some(color);
        self
    }

    /// Overrides the badge text color.
    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }
}

impl Default for Badge {
    fn default() -> Self {
        Self::new()
    }
}

impl StyleBuilder for Badge {
    fn style_mut(&mut self) -> &mut Style {
        &mut self.base.style
    }

    fn mark_dirty(&mut self) {
        self.base.dirty = true;
    }
}

impl Render for Badge {
    fn render(&self) -> Box<dyn Widget> {
        let theme = crate::current_theme();
        Box::new(
            View::new()
                .display(Display::Flex)
                .flex_direction(FlexDirection::Row)
                .align_items(Align::Center)
                .padding(Edges::symmetric(8.0, 3.0))
                .background(self.background.unwrap_or(theme.secondary_container))
                .border(Border::all(0.0, Color::TRANSPARENT).radius(Length::px(999.0)))
                .child(
                    Label::new()
                        .label(self.label.clone())
                        .font_size(Length::px(12.0))
                        .font_weight(FontWeight::Medium)
                        .color(self.color.unwrap_or(theme.on_secondary_container)),
                ),
        )
    }
}

crate::impl_composite_widget!(Badge);
