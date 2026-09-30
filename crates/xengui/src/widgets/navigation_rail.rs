// SPDX-License-Identifier: Apache-2.0
//! Material 3 Expressive standard expanded navigation rail.

use super::NavItem;
use crate::{
    Align, Border, BorderRadius, Color, Display, Easing, Edges, FlexDirection, FontWeight,
    Interaction, Label, LayoutBox, Length, Render, Style, StyleBuilder, StylePatch, Transition,
    VariableIcon, View, Widget, WidgetBase, WidgetId, pct, px,
};
use std::{rc::Rc, time::Duration};
use xengui_icons::material_symbols::IconAxes;

const MIN_WIDTH: f32 = 220.0;
const MAX_WIDTH: f32 = 360.0;

const EFFECTS_TRANSITION: Transition =
    Transition::new(Duration::from_millis(150)).easing(Easing::cubic_bezier(0.31, 0.94, 0.34, 1.0));
const ITEM_TRANSITION: Transition = Transition::new(Duration::from_millis(350))
    .easing(Easing::cubic_bezier(0.42, 1.67, 0.21, 0.90));

fn state_layer(base: Color, content: Color, opacity: f32) -> Color {
    let opacity = opacity.clamp(0.0, 1.0);
    Color::rgba_f32(
        base.r() * (1.0 - opacity) + content.r() * opacity,
        base.g() * (1.0 - opacity) + content.g() * opacity,
        base.b() * (1.0 - opacity) + content.b() * opacity,
        1.0,
    )
}

/// A persistent expanded navigation rail for medium and larger windows.
///
/// The rail owns its Material container and destination styling. Place it at
/// the leading edge of a row next to the application's content pane.
pub struct NavigationRail {
    base: WidgetBase,
    layout_box: LayoutBox,
    inner: Vec<Box<dyn Widget>>,
    hooks_id: WidgetId,
    items: Vec<NavItem>,
    active_index: usize,
    width: f32,
    on_select: Option<Rc<dyn Fn(usize)>>,
}

impl NavigationRail {
    /// Creates an empty standard expanded navigation rail.
    pub fn new() -> Self {
        Self {
            base: WidgetBase::new(Interaction::new()),
            layout_box: LayoutBox::default(),
            inner: Vec::new(),
            hooks_id: WidgetId::new_unique(),
            items: Vec::new(),
            active_index: 0,
            width: MAX_WIDTH,
            on_select: None,
        }
    }

    /// Adds a destination.
    pub fn item(mut self, item: NavItem) -> Self {
        self.items.push(item);
        self
    }

    /// Sets the selected destination index.
    pub fn active_index(mut self, index: usize) -> Self {
        self.active_index = index;
        self
    }

    /// Sets the rail width, clamped to the M3 expanded-rail range.
    pub fn rail_width(mut self, width: f32) -> Self {
        self.width = width.clamp(MIN_WIDTH, MAX_WIDTH);
        self
    }

    /// Registers destination selection.
    pub fn on_select(mut self, callback: impl Fn(usize) + 'static) -> Self {
        self.on_select = Some(Rc::new(callback));
        self
    }
}

impl Default for NavigationRail {
    fn default() -> Self {
        Self::new()
    }
}

impl StyleBuilder for NavigationRail {
    fn style_mut(&mut self) -> &mut Style {
        &mut self.base.style
    }

    fn mark_dirty(&mut self) {
        self.base.dirty = true;
    }
}

impl Render for NavigationRail {
    fn render(&self) -> Box<dyn Widget> {
        let theme = crate::current_theme();
        let mut destinations = View::new()
            .width(pct!(100.0))
            .min_width(Length::px(0.0))
            .display(Display::Flex)
            .flex_direction(FlexDirection::Column)
            .gap(0.0, 4.0);

        for (index, item) in self.items.iter().enumerate() {
            let selected = index == self.active_index;
            let on_select = self.on_select.clone();
            let item_label = item.label.clone();

            destinations = destinations.child(
                View::new()
                    .width(pct!(100.0))
                    .min_width(Length::px(0.0))
                    .height(px!(56.0))
                    .display(Display::Flex)
                    .flex_direction(FlexDirection::Row)
                    .align_items(Align::Center)
                    .gap(8.0, 0.0)
                    .padding(Edges::symmetric(16.0, 0.0))
                    .background(if selected {
                        theme.secondary_container
                    } else {
                        Color::TRANSPARENT
                    })
                    .color(if selected {
                        theme.on_secondary_container
                    } else {
                        theme.on_surface_variant
                    })
                    .border(Border::none().radius(BorderRadius::all(28.0)))
                    .hover_style(move |style: StylePatch, theme: &crate::Theme| {
                        style.background(if selected {
                            state_layer(
                                theme.secondary_container,
                                theme.on_secondary_container,
                                0.08,
                            )
                        } else {
                            state_layer(theme.surface_container, theme.on_surface, 0.08)
                        })
                    })
                    .pressed_style(|style: StylePatch, _| style.scale(0.96).content_scale(1.0))
                    .transition_colors(EFFECTS_TRANSITION)
                    .transition_transform(ITEM_TRANSITION)
                    .accessible_label(item_label)
                    .on_click(move |_| {
                        if let Some(callback) = &on_select {
                            callback(index);
                        }
                    })
                    .child(
                        VariableIcon::new(item.codepoint)
                            .size(24.0)
                            .axes(IconAxes::default().fill(if selected { 1.0 } else { 0.0 })),
                    )
                    .child(
                        Label::new()
                            .label(item.label.clone())
                            .font_size(px!(14.0))
                            .font_weight(if selected {
                                FontWeight::Bold
                            } else {
                                FontWeight::Medium
                            }),
                    ),
            );
        }

        Box::new(
            View::new()
                .width(px!(self.width))
                .min_width(px!(self.width))
                .height(pct!(100.0))
                .padding(Edges::all(16.0))
                .background(theme.surface_container)
                .border(Border::right(1.0, theme.outline_variant))
                .child(destinations),
        )
    }
}

crate::impl_composite_widget!(NavigationRail);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rail_width_stays_in_m3_expanded_range() {
        assert_eq!(NavigationRail::new().rail_width(100.0).width, MIN_WIDTH);
        assert_eq!(NavigationRail::new().rail_width(280.0).width, 280.0);
        assert_eq!(NavigationRail::new().rail_width(500.0).width, MAX_WIDTH);
    }
}
