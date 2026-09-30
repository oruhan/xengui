// SPDX-License-Identifier: Apache-2.0
//! Material-style single-selection combo box.

use crate::{
    Align, Border, BorderRadius, BoxShadow, Color, Cursor, Display, Easing, Edges, FlexDirection,
    FontWeight, Interaction, JustifyContent, Key, KeyState, Label, LayoutBox, Portal, Position,
    Render, Style, StyleBuilder, TransformOrigin, TransformOriginAxis, Transition, VariableIcon,
    View, Widget, WidgetBase, WidgetId, pct, px,
};
use smol_str::SmolStr;
use std::rc::Rc;
use web_time::Duration;
use xengui_icons::material_symbols::{IconAxes, codepoints};

const POPUP_OPACITY_TRANSITION: Transition =
    Transition::new(Duration::from_millis(350)).easing(Easing::cubic_bezier(0.31, 0.94, 0.34, 1.0));
const POPUP_SCALE_TRANSITION: Transition = Transition::new(Duration::from_millis(350))
    .easing(Easing::cubic_bezier(0.42, 1.67, 0.21, 0.90));
const POPUP_GATE_OPEN_TRANSITION: Transition = Transition::new(Duration::ZERO);
const POPUP_GATE_CLOSE_TRANSITION: Transition =
    Transition::new(Duration::ZERO).delay(Duration::from_millis(350));
const POPUP_CLOSED_SCALE: f32 = 0.96;
const POPUP_GAP: f32 = 8.0;
const POPUP_TRANSFORM_ORIGIN: TransformOrigin = TransformOrigin {
    x: TransformOriginAxis::Percent(50.0),
    // The popup begins below the gap, but its motion originates at the
    // trigger's bottom edge rather than at the popup surface itself.
    y: TransformOriginAxis::Px(-POPUP_GAP),
};

fn state_layer(base: Color, content: Color, opacity: f32) -> Color {
    let opacity = opacity.clamp(0.0, 1.0);
    Color::rgba_f32(
        base.r() * (1.0 - opacity) + content.r() * opacity,
        base.g() * (1.0 - opacity) + content.g() * opacity,
        base.b() * (1.0 - opacity) + content.b() * opacity,
        1.0,
    )
}

/// A compact, single-selection control that opens a menu of text options.
///
/// `selected_index` is controlled by the caller. The callback should update
/// that value and rebuild the widget, matching the rest of XenGui's controls.
pub struct ComboBox {
    base: WidgetBase,
    layout_box: LayoutBox,
    inner: Vec<Box<dyn Widget>>,
    hooks_id: WidgetId,
    options: Vec<SmolStr>,
    selected_index: usize,
    label: SmolStr,
    on_change: Option<Rc<dyn Fn(usize)>>,
}

impl ComboBox {
    /// Creates a combo box from the supplied display labels.
    pub fn new<I, S>(options: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<SmolStr>,
    {
        let mut base = WidgetBase::new(Interaction::new());
        base.style.size = Some(crate::Size {
            width: Some(pct!(100.0)),
            height: Some(px!(56.0)),
        });
        Self {
            base,
            layout_box: LayoutBox::default(),
            inner: Vec::new(),
            hooks_id: WidgetId::new_unique(),
            options: options.into_iter().map(Into::into).collect(),
            selected_index: 0,
            label: SmolStr::new("Seçenek"),
            on_change: None,
        }
    }

    /// Sets the currently selected option.
    pub fn selected_index(mut self, index: usize) -> Self {
        self.selected_index = index.min(self.options.len().saturating_sub(1));
        self
    }

    /// Sets the accessible label announced for the trigger.
    pub fn label(mut self, label: impl Into<SmolStr>) -> Self {
        self.label = label.into();
        self
    }

    /// Invoked with the selected option index.
    pub fn on_change(mut self, callback: impl Fn(usize) + 'static) -> Self {
        self.on_change = Some(Rc::new(callback));
        self
    }
}

impl StyleBuilder for ComboBox {
    fn style_mut(&mut self) -> &mut Style {
        &mut self.base.style
    }

    fn mark_dirty(&mut self) {
        self.base.dirty = true;
    }
}

impl Render for ComboBox {
    fn render(&self) -> Box<dyn Widget> {
        let theme = crate::current_theme();
        let (open, set_open) = crate::use_state(false);
        let selected = self
            .selected_index
            .min(self.options.len().saturating_sub(1));
        let value = self
            .options
            .get(selected)
            .cloned()
            .unwrap_or_else(|| SmolStr::new("—"));

        let set_open_from_click = set_open.clone();
        let set_open_from_key = set_open.clone();
        let options_len = self.options.len();
        let key_callback = self.on_change.clone();
        let mut root = View::new()
            .position(Position::Relative)
            .width(pct!(100.0))
            .height(px!(56.0))
            .min_width(px!(0.0))
            .overflow(crate::Overflow::Visible, crate::Overflow::Visible)
            .child(
                View::new()
                    .accessible_label(format!("{}: {value}", self.label))
                    .focusable(true)
                    .width(pct!(100.0))
                    .height(px!(56.0))
                    .display(Display::Flex)
                    .flex_direction(FlexDirection::Row)
                    .align_items(Align::Center)
                    .justify_content(JustifyContent::SpaceBetween)
                    .padding(Edges::symmetric(16.0, 0.0))
                    .background(theme.surface_container_highest)
                    .color(theme.on_surface)
                    .font_weight(FontWeight::Medium)
                    .border(
                        Border::all(
                            1.0,
                            if open {
                                theme.primary
                            } else {
                                theme.outline.with_alpha(180)
                            },
                        )
                        .radius(BorderRadius::all(16.0)),
                    )
                    .hover_background(state_layer(
                        theme.surface_container_highest,
                        theme.on_surface,
                        0.08,
                    ))
                    .pressed_background(state_layer(
                        theme.surface_container_highest,
                        theme.on_surface,
                        0.10,
                    ))
                    .transition_colors(
                        Transition::new(Duration::from_millis(150))
                            .easing(Easing::cubic_bezier(0.31, 0.94, 0.34, 1.0)),
                    )
                    .on_key(move |event, _ctx| {
                        if event.state != KeyState::Pressed || event.repeat || options_len == 0 {
                            return;
                        }
                        let next = match event.key {
                            Key::ArrowDown => Some((selected + 1).min(options_len - 1)),
                            Key::ArrowUp => Some(selected.saturating_sub(1)),
                            Key::Escape => {
                                set_open_from_key.set(false);
                                None
                            }
                            _ => None,
                        };
                        if let Some(next) = next
                            && let Some(callback) = &key_callback
                        {
                            callback(next);
                        }
                    })
                    .child(Label::new().label(value).font_size(px!(14.0)))
                    .child(
                        VariableIcon::new(codepoints::EXPAND_MORE)
                            .size(24.0)
                            .axes(IconAxes::default().weight(500.0).optical_size(24.0))
                            .rotation(if open { 180.0 } else { 0.0 })
                            .rotation_transition(
                                Transition::new(Duration::from_millis(350))
                                    .easing(Easing::cubic_bezier(0.42, 1.67, 0.21, 0.90)),
                            ),
                    )
                    .on_click(move |_| set_open_from_click.set(!open)),
            );

        let mut menu = View::new()
            .key("combo-box-popup")
            .display(Display::Flex)
            .flex_direction(FlexDirection::Column)
            .width(pct!(100.0))
            .min_width(px!(0.0))
            .max_width(px!(280.0))
            .gap(0.0, 8.0)
            .padding(Edges::all(8.0))
            .overflow(crate::Overflow::Hidden, crate::Overflow::Hidden)
            .background(theme.surface_container)
            .border(Border::none().radius(BorderRadius::all(20.0)))
            .box_shadow(BoxShadow::new(0.0, 8.0, 24.0, theme.shadow.with_alpha(90)));

        for (index, option) in self.options.iter().enumerate() {
            let callback = self.on_change.clone();
            let set_open_from_item = set_open.clone();
            let is_selected = index == selected;
            let leading: Box<dyn Widget> = if is_selected {
                Box::new(
                    VariableIcon::new(codepoints::CHECK)
                        .size(20.0)
                        .axes(IconAxes::default().weight(500.0).optical_size(20.0)),
                )
            } else {
                Box::new(View::new().size(px!(20.0), px!(20.0)))
            };
            menu = menu.child(
                View::new()
                    .accessible_label(option.clone())
                    .focusable(true)
                    .width(pct!(100.0))
                    .height(px!(48.0))
                    .display(Display::Flex)
                    .flex_direction(FlexDirection::Row)
                    .justify_content(JustifyContent::Start)
                    .align_items(Align::Center)
                    .gap(12.0, 0.0)
                    .padding(Edges::symmetric(12.0, 0.0))
                    .background(if is_selected {
                        theme.secondary_container
                    } else {
                        Color::TRANSPARENT
                    })
                    .color(if is_selected {
                        theme.on_secondary_container
                    } else {
                        theme.on_surface
                    })
                    .border(Border::none().radius(BorderRadius::all(12.0)))
                    .hover_background(state_layer(
                        if is_selected {
                            theme.secondary_container
                        } else {
                            theme.surface_container
                        },
                        if is_selected {
                            theme.on_secondary_container
                        } else {
                            theme.on_surface
                        },
                        0.08,
                    ))
                    .pressed_background(state_layer(
                        if is_selected {
                            theme.secondary_container
                        } else {
                            theme.surface_container
                        },
                        if is_selected {
                            theme.on_secondary_container
                        } else {
                            theme.on_surface
                        },
                        0.10,
                    ))
                    .transition_colors(
                        Transition::new(Duration::from_millis(150))
                            .easing(Easing::cubic_bezier(0.31, 0.94, 0.34, 1.0)),
                    )
                    .child_boxed(leading)
                    .child(Label::new().label(option.clone()).font_size(px!(14.0)))
                    .on_click(move |_| {
                        set_open_from_item.set(false);
                        if let Some(callback) = &callback {
                            callback(index);
                        }
                    }),
            );
        }

        // Keep the whole popup in one motion layer. Scaling the painted menu
        // itself splits its surface and descendants into separate compositor
        // layers; a paintless wrapper transforms them as one subtree.
        root = root.child(
            Portal::new()
                .position(Position::Absolute)
                .top(px!(0.0))
                .left(px!(0.0))
                .width(pct!(100.0))
                .child(
                    View::new()
                        .key("combo-box-popup-gate")
                        .position(Position::Absolute)
                        .top(px!(56.0 + POPUP_GAP))
                        .left(px!(0.0))
                        .z_index(1001)
                        .width(pct!(100.0))
                        .scale(if open { 1.0 } else { 0.0 })
                        .transition_transform(if open {
                            POPUP_GATE_OPEN_TRANSITION
                        } else {
                            POPUP_GATE_CLOSE_TRANSITION
                        })
                        .child(
                            View::new()
                                .key("combo-box-popup-motion")
                                .width(pct!(100.0))
                                .min_width(px!(0.0))
                                .transform_origin(POPUP_TRANSFORM_ORIGIN)
                                .scale(if open { 1.0 } else { POPUP_CLOSED_SCALE })
                                .opacity(if open { 1.0 } else { 0.0 })
                                .transition_transform(POPUP_SCALE_TRANSITION)
                                .transition_opacity(POPUP_OPACITY_TRANSITION)
                                .child(menu),
                        ),
                ),
        );

        // Append the conditional scrim after the always-mounted Portal so
        // opening the menu never changes the Portal's sibling index. The
        // explicit z-index still keeps the popup above this dismiss layer.
        if open {
            let set_open_from_scrim = set_open.clone();
            root = root.child(
                View::new()
                    .key("combo-box-scrim")
                    .position(Position::Fixed)
                    .top(px!(0.0))
                    .right(px!(0.0))
                    .bottom(px!(0.0))
                    .left(px!(0.0))
                    .z_index(1000)
                    .background(Color::TRANSPARENT)
                    .accessible_label("Seçim menüsünü kapat")
                    // A dismiss layer is not a visible control surface. Its
                    // full-window bounds must never become a ripple shape.
                    .ripple(false)
                    .on_click(move |_| set_open_from_scrim.set(false))
                    .cursor(Cursor::Default),
            );
        }

        Box::new(root)
    }
}

crate::impl_common_style_builders!(base ComboBox);
crate::impl_composite_widget!(ComboBox);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selected_index_is_clamped_to_available_options() {
        let combo = ComboBox::new(["one", "two"]).selected_index(99);
        assert_eq!(combo.selected_index, 1);
    }

    #[test]
    fn empty_combo_box_keeps_a_safe_zero_index() {
        let combo = ComboBox::new(Vec::<String>::new()).selected_index(4);
        assert_eq!(combo.selected_index, 0);
    }

    #[test]
    fn closed_combo_keeps_popup_content_mounted() {
        let runtime = crate::RuntimeContext::new();
        let _guard = runtime.enter();
        crate::hooks::begin_render();
        let rendered =
            crate::hooks::component("combo-test", || ComboBox::new(["one", "two"]).render());
        crate::hooks::end_render();

        let popup_portal = rendered
            .children()
            .iter()
            .find(|child| child.is_portal())
            .expect("combo popup should escape ancestor clipping");
        let popup_gate = popup_portal
            .children()
            .first()
            .expect("closed combo should retain its popup gate");
        assert_eq!(
            popup_gate.get_key().map(|key| key.as_str()),
            Some("combo-box-popup-gate")
        );
        let popup_motion = popup_gate
            .children()
            .first()
            .expect("popup gate should retain its motion layer");
        let popup = popup_motion
            .children()
            .first()
            .expect("popup motion layer should retain popup content");
        assert_eq!(
            popup.get_key().map(|key| key.as_str()),
            Some("combo-box-popup")
        );
        assert_eq!(popup.children().len(), 2);
        assert_eq!(popup_motion.style().scale, Some(POPUP_CLOSED_SCALE));
        assert_eq!(popup_motion.style().opacity, Some(0.0));
        let transitions = popup_motion
            .style()
            .transition_properties
            .expect("popup should animate scale and opacity");
        assert!(transitions.contains(crate::TransitionProperty::TRANSFORM));
        assert!(transitions.contains(crate::TransitionProperty::OPACITY));
    }
}
