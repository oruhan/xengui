// SPDX-License-Identifier: Apache-2.0
//! Material Design 3-style floating "pill" navigation bar, meant for
//! narrow/mobile layouts. The widget itself only renders the pill shape
//! and dispatches selection - positioning (fixed, bottom, centered) is
//! left to the caller so it composes with any layout.
use crate::{
    Align, Border, BorderRadius, BoxShadow, Color, Display, Easing, Edges, Filter, FlexDirection,
    FontWeight, Interaction, JustifyContent, Label, LayoutBox, Length, Render, Style, StyleBuilder,
    Transition, VariableIcon, View, Widget, WidgetBase, WidgetId, pct,
};
use smol_str::SmolStr;
use std::rc::Rc;
use std::time::Duration;

/// A single destination in a [`NavigationBar`].
pub struct NavItem {
    /// The `codepoint` value carried by this type.
    pub codepoint: char,
    /// The `label` value carried by this type.
    pub label: SmolStr,
}

impl NavItem {
    /// Creates a value with its default configuration.
    pub fn new(codepoint: char, label: impl Into<SmolStr>) -> Self {
        Self {
            codepoint,
            label: label.into(),
        }
    }
}

/// Data and behavior represented by `NavigationBar`.
pub struct NavigationBar {
    base: WidgetBase,
    layout_box: LayoutBox,
    inner: Vec<Box<dyn Widget>>,
    hooks_id: WidgetId,

    items: Vec<NavItem>,
    active_index: usize,
    on_select: Option<Rc<dyn Fn(usize)>>,
}

impl NavigationBar {
    /// Creates a value with its default configuration.
    pub fn new() -> Self {
        Self {
            base: WidgetBase::new(Interaction::new()),
            layout_box: LayoutBox::default(),
            inner: Vec::new(),
            hooks_id: WidgetId::new_unique(),
            items: Vec::new(),
            active_index: 0,
            on_select: None,
        }
    }

    /// Returns or updates the `item` value.
    pub fn item(mut self, item: NavItem) -> Self {
        self.items.push(item);
        self
    }

    /// Returns or updates the `active_index` value.
    pub fn active_index(mut self, index: usize) -> Self {
        self.active_index = index;
        self
    }

    /// Registers the `on_select` callback.
    pub fn on_select(mut self, f: impl Fn(usize) + 'static) -> Self {
        self.on_select = Some(Rc::new(f));
        self
    }
}

impl Default for NavigationBar {
    fn default() -> Self {
        Self::new()
    }
}

impl StyleBuilder for NavigationBar {
    fn style_mut(&mut self) -> &mut Style {
        &mut self.base.style
    }

    fn mark_dirty(&mut self) {
        self.base.dirty = true;
    }
}

impl Render for NavigationBar {
    fn render(&self) -> Box<dyn Widget> {
        let theme = crate::current_theme();

        let mut row = View::new()
            .display(Display::Flex)
            .flex_direction(FlexDirection::Row)
            .align_items(Align::Center)
            .justify_content(JustifyContent::SpaceEvenly)
            .padding(Edges::symmetric(10.0, 10.0))
            .background(theme.surface_container_high.with_alpha_f32(0.65))
            .backdrop_filter(Filter::Blur(Length::px(24.0)))
            .border(
                Border::all(1.0, theme.outline_variant.with_alpha_f32(0.4))
                    .radius(BorderRadius::all(28.0)),
            )
            .box_shadow(BoxShadow::new(0.0, 6.0, 20.0, Color::BLACK.with_alpha(70)));

        for (index, item) in self.items.iter().enumerate() {
            let active = index == self.active_index;
            let (bg, fg) = if active {
                (theme.secondary_container, theme.on_secondary_container)
            } else {
                (Color::TRANSPARENT, theme.on_surface_variant)
            };

            let on_select = self.on_select.clone();

            let mut pill = View::new()
                .display(Display::Flex)
                .flex_direction(FlexDirection::Row)
                .align_items(Align::Center)
                .justify_content(JustifyContent::Center)
                .gap(6.0, 0.0)
                .color(fg)
                .padding(Edges::symmetric(if active { 18.0 } else { 12.0 }, 10.0))
                .background(bg)
                .border(Border::all(0.0, Color::TRANSPARENT).radius(BorderRadius::all(20.0)))
                .transition_all(Transition::new(Duration::from_millis(200)).easing(Easing::EaseOut))
                .child(VariableIcon::new(item.codepoint).size(22.0));

            if active {
                pill = pill.child(
                    Label::new()
                        .label(item.label.clone())
                        .font_size(Length::px(13.0))
                        .font_weight(FontWeight::Medium),
                );
            }

            pill = pill.on_click(move |_ctx| {
                if let Some(f) = &on_select {
                    f(index);
                }
            });

            row = row.child(pill);
        }

        let _ = pct!(100.0);
        Box::new(row)
    }
}

crate::impl_composite_widget!(NavigationBar);
