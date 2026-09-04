// SPDX-License-Identifier: Apache-2.0
//! In-app render/repaint inspector, toggled by F12 (see `xenframe::App`).
//! Not meant to be added to a user's own tree directly.
use crate::{
    Align, AnimationManager, Background, Border, Button, Color, Constraints, Cursor, Display,
    Edges, ElementState, EventCtx, EventStatus, FlexDirection, InputEvent, Interaction,
    JustifyContent, Label, LayoutBox, Length, MeasureContext, MeasureResult, MouseButton, Overflow,
    PaintContext, RectCommand, Render, Size, Style, StyleBuilder, View, Widget, WidgetBase,
    WidgetId,
    devtools::{self, RenderEventKind},
    pct,
};
use smol_str::SmolStr;
use std::cell::Cell;
use std::rc::Rc;

const HANDLE_WIDTH: f32 = 6.0;
const MIN_PANEL_WIDTH: f32 = 240.0;
const MAX_PANEL_WIDTH: f32 = 900.0;
// How far past MIN_PANEL_WIDTH a drag can go before it counts as "drag to
// close" instead of just clamping at the minimum.
const CLOSE_DRAG_SLACK: f32 = 80.0;
// Logical px always left over for the app's own content, regardless of
// how wide the panel is asked to be.
const MIN_CONTENT_WIDTH: f32 = 200.0;
const MAX_VISIBLE_ENTRIES: usize = 300;

/// Thin draggable strip at the panel's left edge. Writes directly into a
/// shared width handle - the same shared-Cell pattern `ContextMenuHandle`
/// uses to let one part of the tree affect state another part owns.
pub struct DevtoolsResizeHandle {
    base: WidgetBase,
    layout_box: LayoutBox,
    width_handle: Rc<Cell<f32>>,
    close_handle: Rc<Cell<bool>>,
    dragging: Cell<bool>,
    drag_start_mouse_x: Cell<f32>,
    drag_start_width: Cell<f32>,
    scale_factor: Cell<f32>,
}

impl DevtoolsResizeHandle {
    /// Creates a value with its default configuration.
    pub fn new(width_handle: Rc<Cell<f32>>, close_handle: Rc<Cell<bool>>) -> Self {
        let mut interaction = Interaction::new();
        interaction.hover_cursor = Some(Cursor::EwResize);

        let mut base = WidgetBase::new(interaction);
        base.style.size = Some(Size::new(Length::px(HANDLE_WIDTH), Length::pct(100.0)));

        Self {
            base,
            layout_box: LayoutBox::default(),
            width_handle,
            close_handle,
            dragging: Cell::new(false),
            drag_start_mouse_x: Cell::new(0.0),
            drag_start_width: Cell::new(0.0),
            scale_factor: Cell::new(1.0),
        }
    }
}

impl Widget for DevtoolsResizeHandle {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn debug_name(&self) -> &'static str {
        "Widget#DevtoolsResizeHandle"
    }

    fn is_dirty(&self) -> bool {
        self.base.dirty
    }

    fn set_dirty(&mut self, dirty: bool) {
        self.base.dirty = dirty;
    }

    fn style(&self) -> &Style {
        &self.base.style
    }

    fn style_mut(&mut self) -> &mut Style {
        &mut self.base.style
    }

    fn computed_style(&self) -> &Style {
        &self.base.computed_style
    }

    fn interaction(&self) -> Option<&Interaction> {
        Some(&self.base.interaction)
    }

    fn interaction_mut(&mut self) -> Option<&mut Interaction> {
        Some(&mut self.base.interaction)
    }

    fn measure(&self, _ctx: &mut MeasureContext, _constraints: Constraints) -> MeasureResult {
        MeasureResult::new(0.0, 0.0)
    }

    fn on_layout_pass(&self, ctx: &mut MeasureContext) {
        self.scale_factor.set(ctx.scale_factor);
    }

    fn layout(&mut self, rect: LayoutBox) {
        self.layout_box = rect;
    }

    fn layout_box(&self) -> &LayoutBox {
        &self.layout_box
    }

    fn paint(&self, ctx: &mut PaintContext) {
        let color = if self.dragging.get() || self.base.interaction.hovered {
            Color::BLUE_500
        } else {
            Color::NEUTRAL_700
        };
        let b = self.layout_box;
        ctx.draw_rect(RectCommand {
            position: (b.x, b.y),
            size: (b.width, b.height),
            background: Some(Background::Color(color)),
            border_radius: None,
            border_width: None,
            border_color: None,
            clip_rect: None,
        });
    }

    fn event(&mut self, event: &InputEvent, ctx: &mut EventCtx) -> EventStatus {
        match event {
            InputEvent::MouseEntered => {
                self.base.interaction.hovered = true;
                ctx.set_cursor_icon(Cursor::EwResize);
                self.base.dirty = true;
                ctx.request_redraw();
                EventStatus::Handled
            }
            InputEvent::MouseExited => {
                self.base.interaction.hovered = false;
                if !self.dragging.get() {
                    ctx.set_cursor_icon(Cursor::Default);
                }
                self.base.dirty = true;
                ctx.request_redraw();
                EventStatus::Handled
            }
            InputEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Left,
                position,
            } => {
                self.dragging.set(true);
                self.drag_start_mouse_x.set(position.0);
                self.drag_start_width.set(self.width_handle.get());
                ctx.set_cursor_icon(Cursor::EwResize);
                self.base.dirty = true;
                ctx.request_redraw();
                EventStatus::Handled
            }
            InputEvent::MouseInput {
                state: ElementState::Released,
                button: MouseButton::Left,
                ..
            } => {
                self.dragging.set(false);
                self.base.dirty = true;
                ctx.request_redraw();
                EventStatus::Handled
            }
            InputEvent::MouseMoved { position } if self.dragging.get() => {
                let delta = position.0 - self.drag_start_mouse_x.get();
                let raw_width = self.drag_start_width.get() - delta;

                if raw_width < MIN_PANEL_WIDTH - CLOSE_DRAG_SLACK {
                    self.dragging.set(false);
                    self.close_handle.set(true);
                    ctx.set_cursor_icon(Cursor::Default);
                    self.base.dirty = true;
                    ctx.request_redraw();
                    return EventStatus::Handled;
                }

                let (viewport_w, _) = crate::viewport_size();
                let sf = self.scale_factor.get().max(0.0001);
                let max_width = if viewport_w > 0.0 {
                    (viewport_w / sf - MIN_CONTENT_WIDTH).clamp(MIN_PANEL_WIDTH, MAX_PANEL_WIDTH)
                } else {
                    MAX_PANEL_WIDTH
                };

                let new_width = raw_width.clamp(MIN_PANEL_WIDTH, max_width);
                self.width_handle.set(new_width);
                ctx.set_cursor_icon(Cursor::EwResize);
                self.base.dirty = true;
                ctx.request_redraw();
                EventStatus::Handled
            }
            _ => EventStatus::Ignored,
        }
    }

    fn transfer_interaction_state(&mut self, old: &dyn Widget) {
        if let Some(old) = old.as_any().downcast_ref::<DevtoolsResizeHandle>() {
            self.dragging.set(old.dragging.get());
            self.drag_start_mouse_x.set(old.drag_start_mouse_x.get());
            self.drag_start_width.set(old.drag_start_width.get());
            self.base.interaction.hovered = old.base.interaction.hovered;
            self.scale_factor.set(old.scale_factor.get());
        }
    }
}

/// Data and behavior represented by `DevtoolsPanel`.
pub struct DevtoolsPanel {
    base: WidgetBase,
    layout_box: LayoutBox,
    inner: Vec<Box<dyn Widget>>,
    hooks_id: WidgetId,
    width_handle: Rc<Cell<f32>>,
    close_handle: Rc<Cell<bool>>,
    scale_factor: Cell<f32>,
}

impl DevtoolsPanel {
    /// Creates a value with its default configuration.
    pub fn new(width_handle: Rc<Cell<f32>>, close_handle: Rc<Cell<bool>>) -> Self {
        Self {
            base: WidgetBase::new(Interaction::new()),
            layout_box: LayoutBox::default(),
            inner: Vec::new(),
            hooks_id: WidgetId::new_unique(),
            width_handle,
            close_handle,
            scale_factor: Cell::new(1.0),
        }
    }

    /// Returns or updates the `key` value.
    pub fn key(mut self, key: impl Into<SmolStr>) -> Self {
        self.base.key = Some(key.into());
        self
    }
}

impl StyleBuilder for DevtoolsPanel {
    fn style_mut(&mut self) -> &mut Style {
        &mut self.base.style
    }

    fn mark_dirty(&mut self) {
        self.base.dirty = true;
    }
}

// Formats milliseconds-since-epoch as a plain HH:MM:SS wall-clock string
// (UTC, since no timezone database is available here) - readable at a
// glance, unlike a raw microsecond counter.
fn format_clock(epoch_millis: u128) -> String {
    let total_seconds = epoch_millis / 1000;
    let seconds_of_day = total_seconds % 86400;
    let hours = seconds_of_day / 3600;
    let minutes = (seconds_of_day % 3600) / 60;
    let seconds = seconds_of_day % 60;
    format!("{hours:02}:{minutes:02}:{seconds:02}")
}

fn entry_row(entry: &devtools::RenderLogEntry, index: usize) -> Label {
    let (kind_label, kind_color) = match entry.kind {
        RenderEventKind::Rerender => ("RERENDER", Color::WHITE),
        RenderEventKind::Repaint => ("REPAINT ", Color::WHITE),
        RenderEventKind::Layout => ("LAYOUT  ", Color::WHITE),
        RenderEventKind::Warning => ("WARNING ", Color::AMBER_400),
        RenderEventKind::Error => ("ERROR   ", Color::RED_400),
    };

    let text = format!(
        "[{}] {kind_label}  {:<28} {:<20} {}",
        format_clock(entry.epoch_millis),
        entry.widget_path,
        entry.widget_name,
        entry.reason
    );

    Label::new()
        .label(text)
        .selectable(true)
        .color(kind_color)
        .font_size(Length::px(11.0))
        .key(format!("row_{index}"))
}

impl Render for DevtoolsPanel {
    fn render(&self) -> Box<dyn Widget> {
        let (viewport_w, _) = crate::viewport_size();
        let sf = self.scale_factor.get().max(0.0001);
        let max_width = if viewport_w > 0.0 {
            (viewport_w / sf - MIN_CONTENT_WIDTH).clamp(MIN_PANEL_WIDTH, MAX_PANEL_WIDTH)
        } else {
            MAX_PANEL_WIDTH
        };
        let width = self.width_handle.get().clamp(MIN_PANEL_WIDTH, max_width);

        let entries = devtools::snapshot();
        let visible = if entries.len() > MAX_VISIBLE_ENTRIES {
            &entries[entries.len() - MAX_VISIBLE_ENTRIES..]
        } else {
            &entries[..]
        };

        let mut log_column = View::new()
            .key("devtools_log_rows")
            .display(Display::Flex)
            .flex_direction(FlexDirection::Column);

        for (i, entry) in visible.iter().enumerate() {
            log_column = log_column.child(entry_row(entry, i));
        }

        let log_body = View::new()
            .key("devtools_log_body")
            .display(Display::Flex)
            .flex_direction(FlexDirection::Column)
            .flex_grow(1.0)
            .overflow_x(Overflow::Hidden)
            .overflow_y(Overflow::Auto)
            .padding(Edges::symmetric(8.0, 4.0))
            .pin_scroll_to_bottom(true)
            .child(log_column);

        let close_handle = self.close_handle.clone();

        let header_buttons = View::new()
            .display(Display::Flex)
            .flex_direction(FlexDirection::Row)
            .align_items(Align::Center)
            .gap(6.0, 0.0)
            .child(
                Button::new()
                    .label("Clear")
                    .font_size(Length::px(12.0))
                    .padding(Edges::symmetric(8.0, 4.0))
                    .background(Color::NEUTRAL_800)
                    .color(Color::NEUTRAL_100)
                    .on_click(|_ctx| devtools::clear()),
            )
            .child(
                Button::new()
                    .label("✕")
                    .font_size(Length::px(12.0))
                    .padding(Edges::symmetric(8.0, 4.0))
                    .background(Color::NEUTRAL_800)
                    .color(Color::NEUTRAL_100)
                    .on_click(move |ctx| {
                        close_handle.set(true);
                        ctx.request_redraw();
                    }),
            );

        let header = View::new()
            .display(Display::Flex)
            .flex_direction(FlexDirection::Row)
            .justify_content(JustifyContent::SpaceBetween)
            .align_items(Align::Center)
            .padding(Edges::symmetric(10.0, 8.0))
            .border(Border::bottom(1.0, Color::NEUTRAL_800))
            .child(
                Label::new()
                    .label(format!("XenGui DevTools ({} events)", entries.len()))
                    .color(Color::NEUTRAL_100)
                    .font_size(Length::px(13.0)),
            )
            .child(header_buttons);

        let content = View::new()
            .key("devtools_content")
            .display(Display::Flex)
            .flex_direction(FlexDirection::Column)
            .flex_grow(1.0)
            .overflow_x(Overflow::Hidden)
            .background(Color::NEUTRAL_950)
            .child(header)
            .child(log_body);

        Box::new(
            View::new()
                .display(Display::Flex)
                .flex_direction(FlexDirection::Row)
                .size(Length::px(width), pct!(100.0))
                .child(DevtoolsResizeHandle::new(
                    self.width_handle.clone(),
                    self.close_handle.clone(),
                ))
                .child(content),
        )
    }
}

impl Widget for DevtoolsPanel {
    crate::impl_widget_boilerplate!();

    fn debug_name(&self) -> &'static str {
        "DevtoolsPanel"
    }

    fn children(&self) -> &[Box<dyn Widget>] {
        &self.inner
    }

    fn children_mut(&mut self) -> Option<&mut Vec<Box<dyn Widget>>> {
        Some(&mut self.inner)
    }

    fn measure(&self, _ctx: &mut MeasureContext, _constraints: Constraints) -> MeasureResult {
        MeasureResult::new(0.0, 0.0)
    }

    fn on_layout_pass(&self, ctx: &mut MeasureContext) {
        self.scale_factor.set(ctx.scale_factor);
    }

    fn paint(&self, _ctx: &mut PaintContext) {}

    fn cascade_style(&mut self, parent: &Style, anim: &mut AnimationManager) {
        self.base.inherited_style = parent.clone();
        self.base.recompute_style();

        if self.inner.is_empty() {
            let key = format!("DevtoolsPanel#{}", self.hooks_id.get());
            let built =
                devtools::with_suppressed(|| crate::component(key, || Render::render(self)));
            self.inner = vec![built];
        }

        for child in self.inner.iter_mut() {
            child.cascade_style(&self.base.computed_style, anim);
        }
    }

    fn transfer_interaction_state(&mut self, old: &dyn Widget) {
        if let (Some(new), Some(old_i)) = (self.interaction_mut(), old.interaction()) {
            new.transfer_from(old_i);
        }
        if let Some(old) = old.as_any().downcast_ref::<DevtoolsPanel>() {
            self.hooks_id = old.hooks_id;
            self.scale_factor.set(old.scale_factor.get());
        }
    }

    // Its own internal rebuild/reconcile never gets logged and never
    // wakes another rebuild, so opening or closing the panel can't feed
    // back into itself.
    fn transfer_composite_children(&mut self, old: &mut dyn Widget) {
        let key = format!("DevtoolsPanel#{}", self.hooks_id.get());

        devtools::with_suppressed(|| {
            let rendered = crate::component(key, || Render::render(self));

            if let Some(old) = old.as_any_mut().downcast_mut::<DevtoolsPanel>() {
                let mut old_inner = std::mem::take(&mut old.inner);
                self.inner = crate::reconciler::reconcile_now(vec![rendered], &mut old_inner);
            } else {
                self.inner = vec![rendered];
            }
        });
    }
}
