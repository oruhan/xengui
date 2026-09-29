// SPDX-License-Identifier: Apache-2.0
//! Material 3 Expressive modal expanded navigation rail.

use super::NavItem;
use crate::{
    Align, Border, BorderRadius, BoxShadow, Color, Constraints, Cursor, Display, Easing, Edges,
    ElementState, EventCtx, EventStatus, FlexDirection, FontWeight, InputEvent, Interaction,
    JustifyContent, Label, LayoutBox, MeasureContext, MeasureResult, MouseButton, PaintContext,
    Position, Render, SetState, Style, StyleBuilder, StylePatch, Transition, VariableIcon, View,
    Widget, WidgetBase, WidgetId, pct, px,
};
use smol_str::SmolStr;
use std::{cell::Cell, rc::Rc, time::Duration};
use xengui_icons::material_symbols::{IconAxes, codepoints};

const MIN_WIDTH: f32 = 220.0;
const MAX_WIDTH: f32 = 360.0;
const EDGE_GESTURE_WIDTH: f32 = 24.0;
const DRAG_THRESHOLD: f32 = 0.5;

const SPATIAL_TRANSITION: Transition =
    Transition::new(Duration::from_millis(500)).easing(Easing::cubic_bezier(0.38, 1.21, 0.22, 1.0));
const EFFECTS_TRANSITION: Transition =
    Transition::new(Duration::from_millis(200)).easing(Easing::cubic_bezier(0.34, 0.80, 0.34, 1.0));

fn state_layer(base: Color, content: Color, opacity: f32) -> Color {
    let opacity = opacity.clamp(0.0, 1.0);
    Color::rgba_f32(
        base.r() * (1.0 - opacity) + content.r() * opacity,
        base.g() * (1.0 - opacity) + content.g() * opacity,
        base.b() * (1.0 - opacity) + content.b() * opacity,
        1.0,
    )
}

struct RailSwipeHandle {
    base: WidgetBase,
    layout_box: LayoutBox,
    dragging: Cell<bool>,
    start_x: Cell<f32>,
    start_progress: Cell<f32>,
    scale_factor: Cell<f32>,
    progress: f32,
    rail_width: f32,
    set_drag_progress: SetState<Option<f32>>,
    on_open_change: Option<Rc<dyn Fn(bool)>>,
}

impl RailSwipeHandle {
    fn new(
        progress: f32,
        rail_width: f32,
        set_drag_progress: SetState<Option<f32>>,
        on_open_change: Option<Rc<dyn Fn(bool)>>,
    ) -> Self {
        let mut interaction = Interaction::new();
        interaction.hover_cursor = Some(Cursor::EwResize);
        Self {
            base: WidgetBase::new(interaction),
            layout_box: LayoutBox::default(),
            dragging: Cell::new(false),
            start_x: Cell::new(0.0),
            start_progress: Cell::new(progress),
            scale_factor: Cell::new(1.0),
            progress,
            rail_width,
            set_drag_progress,
            on_open_change,
        }
    }
}

impl StyleBuilder for RailSwipeHandle {
    fn style_mut(&mut self) -> &mut Style {
        &mut self.base.style
    }

    fn mark_dirty(&mut self) {
        self.base.dirty = true;
    }
}

impl Widget for RailSwipeHandle {
    crate::impl_widget_boilerplate!();

    fn debug_name(&self) -> &'static str {
        "Widget#RailSwipeHandle"
    }

    fn measure(&self, _ctx: &mut MeasureContext, _constraints: Constraints) -> MeasureResult {
        MeasureResult::new(0.0, 0.0)
    }

    fn on_layout_pass(&self, ctx: &mut MeasureContext) {
        self.scale_factor.set(ctx.scale_factor);
    }

    fn paint(&self, _ctx: &mut PaintContext) {}

    fn event(&mut self, event: &InputEvent, ctx: &mut EventCtx) -> EventStatus {
        match event {
            InputEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Left,
                position,
            } => {
                self.dragging.set(true);
                self.start_x.set(position.0);
                self.start_progress.set(self.progress);
                self.set_drag_progress.set(Some(self.progress));
                ctx.suppress_text_drag();
                ctx.set_cursor_icon(Cursor::EwResize);
                EventStatus::Handled
            }
            InputEvent::MouseMoved { position } if self.dragging.get() => {
                let logical_delta =
                    (position.0 - self.start_x.get()) / self.scale_factor.get().max(0.0001);
                let progress =
                    (self.start_progress.get() + logical_delta / self.rail_width).clamp(0.0, 1.0);
                self.set_drag_progress.set(Some(progress));
                ctx.set_cursor_icon(Cursor::EwResize);
                EventStatus::Handled
            }
            InputEvent::MouseInput {
                state: ElementState::Released,
                button: MouseButton::Left,
                ..
            } if self.dragging.replace(false) => {
                let open = self.progress >= DRAG_THRESHOLD;
                self.set_drag_progress.set(None);
                if let Some(callback) = &self.on_open_change {
                    callback(open);
                }
                ctx.set_cursor_icon(Cursor::Default);
                EventStatus::Handled
            }
            InputEvent::PointerCancel if self.dragging.replace(false) => {
                self.set_drag_progress.set(None);
                ctx.set_cursor_icon(Cursor::Default);
                EventStatus::Handled
            }
            _ => EventStatus::Ignored,
        }
    }

    fn transfer_interaction_state(&mut self, old: &dyn Widget) {
        if let Some(old) = old.as_any().downcast_ref::<Self>() {
            self.dragging.set(old.dragging.get());
            self.start_x.set(old.start_x.get());
            self.start_progress.set(old.start_progress.get());
            self.scale_factor.set(old.scale_factor.get());
        }
    }
}

/// A controlled, modal expanded navigation rail for compact and medium windows.
///
/// The rail opens from the leading edge, dismisses through its scrim or close
/// button, and supports dragging its trailing-edge handle in either direction.
pub struct ModalNavigationRail {
    base: WidgetBase,
    layout_box: LayoutBox,
    inner: Vec<Box<dyn Widget>>,
    hooks_id: WidgetId,
    items: Vec<NavItem>,
    active_index: usize,
    open: bool,
    headline: SmolStr,
    width: f32,
    on_select: Option<Rc<dyn Fn(usize)>>,
    on_open_change: Option<Rc<dyn Fn(bool)>>,
}

impl ModalNavigationRail {
    /// Creates an empty modal expanded navigation rail.
    pub fn new() -> Self {
        Self {
            base: WidgetBase::new(Interaction::new()),
            layout_box: LayoutBox::default(),
            inner: Vec::new(),
            hooks_id: WidgetId::new_unique(),
            items: Vec::new(),
            active_index: 0,
            open: false,
            headline: SmolStr::new("Navigation"),
            width: MAX_WIDTH,
            on_select: None,
            on_open_change: None,
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

    /// Controls whether the rail is open.
    pub fn open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }

    /// Sets the rail header text.
    pub fn headline(mut self, headline: impl Into<SmolStr>) -> Self {
        self.headline = headline.into();
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

    /// Registers open-state changes from buttons, scrim, or swipe gestures.
    pub fn on_open_change(mut self, callback: impl Fn(bool) + 'static) -> Self {
        self.on_open_change = Some(Rc::new(callback));
        self
    }
}

impl Default for ModalNavigationRail {
    fn default() -> Self {
        Self::new()
    }
}

impl StyleBuilder for ModalNavigationRail {
    fn style_mut(&mut self) -> &mut Style {
        &mut self.base.style
    }

    fn mark_dirty(&mut self) {
        self.base.dirty = true;
    }
}

impl Render for ModalNavigationRail {
    fn render(&self) -> Box<dyn Widget> {
        let theme = crate::current_theme();
        let (drag_progress, set_drag_progress) = crate::use_state(None::<f32>);
        let target = drag_progress.unwrap_or(if self.open { 1.0 } else { 0.0 });
        let dragging = drag_progress.is_some();

        let close_from_scrim = self.on_open_change.clone();
        let close_from_button = self.on_open_change.clone();
        let scrim_gate = self.open || target > 0.0;

        let scrim = View::new()
            .key("modal-navigation-rail-scrim")
            .position(Position::Fixed)
            .top(px!(0.0))
            .right(px!(0.0))
            .bottom(px!(0.0))
            .left(px!(0.0))
            .z_index(2000)
            // M3 specifies the Scrim role but not an opacity value here.
            // 32% is this component's restrained project-level treatment.
            .background(theme.scrim.with_alpha_f32(0.32))
            .ripple(false)
            .opacity(target)
            .scale(if scrim_gate { 1.0 } else { 0.0 })
            .transition_opacity(EFFECTS_TRANSITION)
            .accessible_label("Close navigation")
            .on_click(move |_| {
                if let Some(callback) = &close_from_scrim {
                    callback(false);
                }
            });

        let mut destinations = View::new()
            .display(Display::Flex)
            .flex_direction(FlexDirection::Column)
            .width(pct!(100.0))
            .gap(0.0, 4.0);

        for (index, item) in self.items.iter().enumerate() {
            let selected = index == self.active_index;
            let on_select = self.on_select.clone();
            let on_open_change = self.on_open_change.clone();
            let item_label = item.label.clone();
            destinations = destinations.child(
                View::new()
                    .height(px!(56.0))
                    .width(pct!(100.0))
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
                            state_layer(theme.surface_container_low, theme.on_surface, 0.08)
                        })
                    })
                    .pressed_style(|style: StylePatch, _| style.scale(0.96).content_scale(1.0))
                    .transition_colors(EFFECTS_TRANSITION)
                    .transition_transform(SPATIAL_TRANSITION)
                    .accessible_label(item_label)
                    .on_click(move |_| {
                        if let Some(callback) = &on_select {
                            callback(index);
                        }
                        if let Some(callback) = &on_open_change {
                            callback(false);
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

        let mut rail = View::new()
            .key("modal-navigation-rail-sheet")
            .position(Position::Fixed)
            .top(px!(0.0))
            .bottom(px!(0.0))
            .left(px!((target - 1.0) * self.width))
            .z_index(2001)
            .width(px!(self.width))
            .min_width(px!(self.width))
            .display(Display::Flex)
            .flex_direction(FlexDirection::Column)
            .padding(Edges::all(12.0))
            .gap(0.0, 12.0)
            .overflow_y(crate::Overflow::Auto)
            .background(theme.surface_container_low)
            .color(theme.on_surface)
            .border(Border::none().radius(BorderRadius::only(0.0, 16.0, 16.0, 0.0)))
            .box_shadow(BoxShadow::new(4.0, 0.0, 18.0, theme.shadow.with_alpha(64)))
            .child(
                View::new()
                    .height(px!(56.0))
                    .display(Display::Flex)
                    .flex_direction(FlexDirection::Row)
                    .align_items(Align::Center)
                    .justify_content(JustifyContent::SpaceBetween)
                    .padding(Edges::symmetric(4.0, 0.0))
                    .child(
                        Label::new()
                            .label(self.headline.clone())
                            .font_size(px!(18.0))
                            .font_weight(FontWeight::Bold),
                    )
                    .child(
                        View::new()
                            .size(px!(48.0), px!(48.0))
                            .display(Display::Flex)
                            .align_items(Align::Center)
                            .justify_content(JustifyContent::Center)
                            .border(Border::none().radius(BorderRadius::all(24.0)))
                            .accessible_label("Close navigation")
                            .child(VariableIcon::new(codepoints::MENU_OPEN).size(24.0))
                            .on_click(move |_| {
                                if let Some(callback) = &close_from_button {
                                    callback(false);
                                }
                            }),
                    ),
            )
            .child(destinations);
        if !dragging {
            rail = rail.transition_all(SPATIAL_TRANSITION);
        }

        let handle_left = (target * self.width - EDGE_GESTURE_WIDTH).max(0.0);
        let handle = RailSwipeHandle::new(
            target,
            self.width,
            set_drag_progress,
            self.on_open_change.clone(),
        )
        .position(Position::Fixed)
        .top(px!(0.0))
        .bottom(px!(0.0))
        .left(px!(handle_left))
        .z_index(2002)
        .width(px!(EDGE_GESTURE_WIDTH));

        Box::new(
            View::new()
                .position(Position::Relative)
                .size(px!(0.0), px!(0.0))
                .overflow(crate::Overflow::Visible, crate::Overflow::Visible)
                .child(scrim)
                .child(rail)
                .child(handle),
        )
    }
}

crate::impl_composite_widget!(ModalNavigationRail);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rail_width_stays_in_m3_expanded_range() {
        assert_eq!(
            ModalNavigationRail::new().rail_width(100.0).width,
            MIN_WIDTH
        );
        assert_eq!(ModalNavigationRail::new().rail_width(280.0).width, 280.0);
        assert_eq!(
            ModalNavigationRail::new().rail_width(500.0).width,
            MAX_WIDTH
        );
    }
}
