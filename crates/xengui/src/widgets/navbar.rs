// SPDX-License-Identifier: Apache-2.0
//! Material 3 Expressive flexible navigation bar for compact and medium
//! layouts. The caller owns bottom-edge positioning so the component can
//! compose with safe-area and application-shell layouts.
use crate::{
    Align, Border, BorderRadius, Breakpoint, Color, Display, Easing, Edges, ElementState,
    FlexDirection, FontWeight, Interaction, JustifyContent, Key, KeyState, Label, LayoutBox,
    Length, MouseButton, Render, Style, StyleBuilder, Transition, VariableIcon, View, Widget,
    WidgetBase, WidgetId, pct, px,
};
use smol_str::SmolStr;
use std::rc::Rc;
use std::time::Duration;
use xengui_icons::material_symbols::IconAxes;

const EFFECTS_TRANSITION: Transition =
    Transition::new(Duration::from_millis(150)).easing(Easing::cubic_bezier(0.31, 0.94, 0.34, 1.0));
const INDICATOR_TRANSITION: Transition = Transition::new(Duration::from_millis(350))
    .easing(Easing::cubic_bezier(0.42, 1.67, 0.21, 0.90));

const COMPACT_CONTAINER_HEIGHT: f32 = 80.0;
const MEDIUM_CONTAINER_HEIGHT: f32 = 64.0;
const VERTICAL_INDICATOR_WIDTH: f32 = 56.0;
const VERTICAL_INDICATOR_HEIGHT: f32 = 32.0;
const HORIZONTAL_INDICATOR_HEIGHT: f32 = 40.0;
// My design decision (not in M3): the spec requires fixed-width horizontal
// items but does not publish that width. 160dp leaves room for translated
// labels and flex-shrink keeps five destinations usable at 600dp.
const HORIZONTAL_ITEM_WIDTH: f32 = 160.0;

fn state_layer(base: Color, content: Color, opacity: f32) -> Color {
    let opacity = opacity.clamp(0.0, 1.0);
    Color::rgba_f32(
        base.r() * (1.0 - opacity) + content.r() * opacity,
        base.g() * (1.0 - opacity) + content.g() * opacity,
        base.b() * (1.0 - opacity) + content.b() * opacity,
        1.0,
    )
}

fn indicator_background(
    base: Color,
    content: Color,
    active: bool,
    hovered: bool,
    pressed: bool,
) -> Color {
    if pressed {
        state_layer(base, content, 0.10)
    } else if hovered {
        state_layer(base, content, 0.08)
    } else if active {
        base
    } else {
        Color::TRANSPARENT
    }
}

/// A single destination shared by navigation bars, rails, and modal rails.
pub struct NavItem {
    /// The `codepoint` value carried by this type.
    pub codepoint: char,
    /// The `label` value carried by this type.
    pub label: SmolStr,
}

/// Item arrangement used by a [`NavigationBar`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum NavigationBarLayout {
    /// Uses vertical items in compact windows and horizontal items otherwise.
    #[default]
    Adaptive,
    /// Places the label below its icon and active indicator.
    Vertical,
    /// Places the icon and label together inside the active indicator.
    Horizontal,
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
    layout: NavigationBarLayout,
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
            layout: NavigationBarLayout::Adaptive,
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

    /// Selects adaptive, vertical, or horizontal item arrangement.
    pub fn item_layout(mut self, layout: NavigationBarLayout) -> Self {
        self.layout = layout;
        self
    }

    /// Registers the `on_select` callback.
    pub fn on_select(mut self, f: impl Fn(usize) + 'static) -> Self {
        self.on_select = Some(Rc::new(f));
        self
    }

    fn resolved_layout(&self) -> NavigationBarLayout {
        match self.layout {
            NavigationBarLayout::Adaptive => {
                if crate::current_breakpoint() == Breakpoint::Compact {
                    NavigationBarLayout::Vertical
                } else {
                    NavigationBarLayout::Horizontal
                }
            }
            layout => layout,
        }
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
        let layout = self.resolved_layout();
        let vertical = layout == NavigationBarLayout::Vertical;

        let mut row = View::new()
            .width(pct!(100.0))
            .min_width(Length::px(0.0))
            .height(px!(if vertical {
                COMPACT_CONTAINER_HEIGHT
            } else {
                MEDIUM_CONTAINER_HEIGHT
            }))
            .display(Display::Flex)
            .flex_direction(FlexDirection::Row)
            .align_items(Align::Center)
            .justify_content(JustifyContent::Center)
            .background(theme.surface_container)
            .border(Border::none().radius(BorderRadius::all(0.0)));

        for (index, item) in self.items.iter().enumerate() {
            let active = index == self.active_index;
            let on_select = self.on_select.clone();
            let item_label = item.label.clone();
            let item_codepoint = item.codepoint;
            let component_key = format!("destination-{index}-{item_codepoint}-{item_label}");
            let destination = crate::component(component_key, || {
                let (hovered, set_hovered) = crate::use_state(false);
                let (pressed, set_pressed) = crate::use_state(false);
                let indicator_width = if active {
                    VERTICAL_INDICATOR_WIDTH
                } else {
                    VERTICAL_INDICATOR_HEIGHT
                };
                let indicator_base = if active {
                    theme.secondary_container
                } else {
                    theme.surface_container
                };
                let indicator_content = if active {
                    theme.on_secondary_container
                } else {
                    theme.on_surface_variant
                };
                let background = indicator_background(
                    indicator_base,
                    indicator_content,
                    active,
                    hovered,
                    pressed,
                );

                let indicator = if vertical {
                    View::new()
                        .width(px!(indicator_width))
                        .height(px!(VERTICAL_INDICATOR_HEIGHT))
                        .display(Display::Flex)
                        .align_items(Align::Center)
                        .justify_content(JustifyContent::Center)
                        .background(background)
                        .color(indicator_content)
                        .border(
                            Border::none()
                                .radius(BorderRadius::all(VERTICAL_INDICATOR_HEIGHT / 2.0)),
                        )
                        // Selection expands only horizontally, keeping the icon on
                        // the same flat plane as required by the M3 motion guidance.
                        .transition_all(INDICATOR_TRANSITION)
                        .transition_colors(EFFECTS_TRANSITION)
                        .scale(if pressed { 0.96 } else { 1.0 })
                        .content_scale(1.0)
                        .child(
                            VariableIcon::new(item_codepoint)
                                .size(24.0)
                                .axes(IconAxes::default().fill(if active { 1.0 } else { 0.0 })),
                        )
                } else {
                    View::new()
                        .height(px!(HORIZONTAL_INDICATOR_HEIGHT))
                        .display(Display::Flex)
                        .flex_direction(FlexDirection::Row)
                        .align_items(Align::Center)
                        .justify_content(JustifyContent::Center)
                        .gap(4.0, 0.0)
                        .padding(Edges::symmetric(16.0, 0.0))
                        .background(background)
                        .color(indicator_content)
                        .border(
                            Border::none()
                                .radius(BorderRadius::all(HORIZONTAL_INDICATOR_HEIGHT / 2.0)),
                        )
                        .transition_colors(EFFECTS_TRANSITION)
                        .scale(if pressed { 0.96 } else { 1.0 })
                        .content_scale(1.0)
                        .child(
                            VariableIcon::new(item_codepoint)
                                .size(24.0)
                                .axes(IconAxes::default().fill(if active { 1.0 } else { 0.0 })),
                        )
                        .child(
                            Label::new()
                                .label(item_label.clone())
                                .font_size(px!(12.0))
                                .line_height(px!(16.0))
                                .letter_spacing(px!(0.5))
                                .font_weight(if active {
                                    FontWeight::Bold
                                } else {
                                    FontWeight::Medium
                                }),
                        )
                };

                let mut destination = View::new()
                    .min_width(Length::px(0.0))
                    .height(pct!(100.0))
                    .display(Display::Flex)
                    .flex_direction(FlexDirection::Column)
                    .align_items(Align::Center)
                    .justify_content(JustifyContent::Center)
                    .gap(0.0, if vertical { 6.0 } else { 0.0 })
                    .focusable(true)
                    .accessible_label(item_label.clone())
                    .child(indicator);

                if vertical {
                    destination = destination.flex_grow(1.0).flex_basis(px!(0.0)).child(
                        Label::new()
                            .label(item_label.clone())
                            .font_size(px!(11.0))
                            .line_height(px!(16.0))
                            .letter_spacing(px!(0.5))
                            .font_weight(if active {
                                FontWeight::Bold
                            } else {
                                FontWeight::Medium
                            })
                            .color(if active {
                                theme.secondary
                            } else {
                                theme.on_surface_variant
                            }),
                    );
                } else {
                    destination = destination
                        .width(px!(HORIZONTAL_ITEM_WIDTH))
                        .max_width(px!(HORIZONTAL_ITEM_WIDTH))
                        .flex_shrink(1.0);
                }

                let hover_setter = set_hovered.clone();
                let hover_press_setter = set_pressed.clone();
                let mouse_press_setter = set_pressed.clone();
                let key_press_setter = set_pressed.clone();
                destination
                    // The destination owns the full hit target, while all
                    // visual state is deliberately confined to `indicator`.
                    .ripple(false)
                    .on_hover(move |value, _| {
                        hover_setter.set(value);
                        if !value {
                            hover_press_setter.set(false);
                        }
                    })
                    .on_mouse_input(move |state, button, _| {
                        if button == MouseButton::Left {
                            mouse_press_setter.set(state == ElementState::Pressed);
                        }
                    })
                    .on_key(move |event, _| {
                        if matches!(event.key, Key::Enter | Key::Space) {
                            key_press_setter.set(event.state == KeyState::Pressed);
                        }
                    })
                    .on_click(move |_| {
                        if let Some(callback) = &on_select {
                            callback(index);
                        }
                    })
            });
            row = row.child(destination);
        }

        Box::new(row)
    }
}

crate::impl_composite_widget!(NavigationBar);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adaptive_layout_uses_vertical_items_only_for_compact_windows() {
        let runtime = crate::RuntimeContext::new();
        let _guard = runtime.enter();
        let nav = NavigationBar::new();

        crate::style::responsive::set_current_breakpoint_from_width(599.0);
        assert_eq!(nav.resolved_layout(), NavigationBarLayout::Vertical);

        crate::style::responsive::set_current_breakpoint_from_width(600.0);
        assert_eq!(nav.resolved_layout(), NavigationBarLayout::Horizontal);
    }

    #[test]
    fn explicit_item_layout_overrides_the_breakpoint() {
        let runtime = crate::RuntimeContext::new();
        let _guard = runtime.enter();
        crate::style::responsive::set_current_breakpoint_from_width(400.0);

        assert_eq!(
            NavigationBar::new()
                .item_layout(NavigationBarLayout::Horizontal)
                .resolved_layout(),
            NavigationBarLayout::Horizontal
        );
    }

    #[test]
    fn destination_exposes_distinct_enabled_hover_and_pressed_states() {
        let runtime = crate::RuntimeContext::new();
        let _guard = runtime.enter();
        let base = crate::Theme::light().secondary_container;
        let content = crate::Theme::light().on_secondary_container;

        let enabled = indicator_background(base, content, true, false, false);
        let hovered = indicator_background(base, content, true, true, false);
        let pressed = indicator_background(base, content, true, false, true);

        assert_ne!(enabled, hovered);
        assert_ne!(enabled, pressed);
        assert_ne!(hovered, pressed);
    }

    #[test]
    fn full_destination_is_interactive_but_indicator_is_not_a_hit_target() {
        let runtime = crate::RuntimeContext::new();
        let _guard = runtime.enter();
        let rendered = NavigationBar::new()
            .item(NavItem::new('x', "Example"))
            .item_layout(NavigationBarLayout::Vertical)
            .render();
        let destination = rendered
            .children()
            .first()
            .expect("navigation row should expose its destination");
        let destination_interaction = destination
            .interaction()
            .expect("destination should be interactive");
        assert!(destination_interaction.on_hover.is_some());
        assert!(destination_interaction.on_mouse_input.is_some());
        assert!(destination_interaction.on_click.is_some());

        let indicator = destination
            .children()
            .first()
            .expect("destination should expose its indicator");
        assert!(
            !indicator
                .interaction()
                .expect("views expose interaction state")
                .is_active()
        );
    }

    #[test]
    fn variants_use_m3_container_heights() {
        let runtime = crate::RuntimeContext::new();
        let _guard = runtime.enter();

        for (layout, expected_height) in [
            (NavigationBarLayout::Vertical, COMPACT_CONTAINER_HEIGHT),
            (NavigationBarLayout::Horizontal, MEDIUM_CONTAINER_HEIGHT),
        ] {
            let rendered = NavigationBar::new().item_layout(layout).render();
            assert_eq!(
                rendered.style().size.and_then(|size| size.height),
                Some(px!(expected_height))
            );
        }
    }
}
