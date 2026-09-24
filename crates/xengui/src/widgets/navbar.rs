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

fn state_layer(base: Color, content: Color, opacity: f32) -> Color {
    let opacity = opacity.clamp(0.0, 1.0);
    Color::rgba_f32(
        base.r() * (1.0 - opacity) + content.r() * opacity,
        base.g() * (1.0 - opacity) + content.g() * opacity,
        base.b() * (1.0 - opacity) + content.b() * opacity,
        1.0,
    )
}

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
            .width(pct!(100.0))
            .min_width(Length::px(0.0))
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

            let pill = View::new()
                .flex_grow(1.0)
                .min_width(Length::px(0.0))
                .height(Length::px(56.0))
                .display(Display::Flex)
                .flex_direction(FlexDirection::Row)
                .align_items(Align::Center)
                .justify_content(JustifyContent::Center)
                .gap(6.0, 0.0)
                .color(fg)
                .padding(Edges::symmetric(12.0, 10.0))
                .background(bg)
                .border(Border::none().radius(BorderRadius::all(28.0)))
                .hover_style(move |style: crate::StylePatch, theme: &crate::Theme| {
                    style.background(if active {
                        state_layer(
                            theme.secondary_container,
                            theme.on_secondary_container,
                            0.08,
                        )
                    } else {
                        state_layer(theme.surface_container_high, theme.on_surface, 0.08)
                    })
                })
                .pressed_style(|style: crate::StylePatch, _theme: &crate::Theme| {
                    style.scale(0.96).content_scale(1.0)
                })
                .transition_colors(
                    Transition::new(Duration::from_millis(150))
                        .easing(Easing::cubic_bezier(0.31, 0.94, 0.34, 1.0)),
                )
                .transition_transform(
                    Transition::new(Duration::from_millis(350))
                        .easing(Easing::cubic_bezier(0.42, 1.67, 0.21, 0.90)),
                )
                .child(VariableIcon::new(item.codepoint).size(22.0))
                .child(
                    Label::new()
                        .label(item.label.clone())
                        .font_size(Length::px(12.0))
                        .font_weight(FontWeight::Medium),
                )
                .on_click(move |_ctx| {
                    if let Some(f) = &on_select {
                        f(index);
                    }
                });

            row = row.child(pill);
        }

        Box::new(row)
    }
}

crate::impl_composite_widget!(NavigationBar);
