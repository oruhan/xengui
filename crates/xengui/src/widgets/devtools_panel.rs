// SPDX-License-Identifier: Apache-2.0
//! In-app render/repaint inspector, toggled by F12 (see `xenframe::App`).
//! Not meant to be added to a user's own tree directly.
use crate::{
    Align,
    Background,
    Border,
    Button,
    Color,
    Constraints,
    Cursor,
    Display,
    Edges,
    ElementState,
    EventCtx,
    EventStatus,
    FlexDirection,
    Interaction,
    InputEvent,
    JustifyContent,
    Label,
    LayoutBox,
    Length,
    MeasureContext,
    MeasureResult,
    MouseButton,
    Overflow,
    PaintContext,
    RectCommand,
    Render,
    Size,
    Style,
    StyleBuilder,
    View,
    Widget,
    WidgetBase,
    WidgetId,
    devtools::{ self, RenderEventKind },
    pct,
};
use smol_str::SmolStr;
use std::cell::Cell;
use std::rc::Rc;

const HANDLE_WIDTH: f32 = 4.0;
const MIN_PANEL_WIDTH: f32 = 240.0;
const MAX_PANEL_WIDTH: f32 = 900.0;
// Bounded so a long debug session doesn't force hundreds of Label widgets
// to be measured/painted every rebuild - only recent activity matters
// for live debugging.
const MAX_VISIBLE_ENTRIES: usize = 300;

/// Thin draggable strip at the panel's left edge. Writes directly into a
/// shared width handle - the same shared-Cell pattern `ContextMenuHandle`
/// uses to let one part of the tree affect state another part owns.
pub struct DevtoolsResizeHandle {
    base: WidgetBase,
    layout_box: LayoutBox,
    width_handle: Rc<Cell<f32>>,
    dragging: Cell<bool>,
    drag_start_mouse_x: Cell<f32>,
    drag_start_width: Cell<f32>,
}

impl DevtoolsResizeHandle {
    pub fn new(width_handle: Rc<Cell<f32>>) -> Self {
        let mut interaction = Interaction::new();
        interaction.hover_cursor = Some(Cursor::EwResize);

        let mut base = WidgetBase::new(interaction);
        base.style.size = Some(Size::new(Length::px(HANDLE_WIDTH), Length::pct(100.0)));

        Self {
            base,
            layout_box: LayoutBox::default(),
            width_handle,
            dragging: Cell::new(false),
            drag_start_mouse_x: Cell::new(0.0),
            drag_start_width: Cell::new(0.0),
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
                // The panel sits to the right of this handle, so dragging
                // left (negative delta) must grow it, not shrink it.
                let delta = position.0 - self.drag_start_mouse_x.get();
                let new_width = (self.drag_start_width.get() - delta).clamp(
                    MIN_PANEL_WIDTH,
                    MAX_PANEL_WIDTH
                );
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
        }
    }
}

pub struct DevtoolsPanel {
    base: WidgetBase,
    layout_box: LayoutBox,
    inner: Vec<Box<dyn Widget>>,
    hooks_id: WidgetId,
    width_handle: Rc<Cell<f32>>,
}

impl DevtoolsPanel {
    pub fn new(width_handle: Rc<Cell<f32>>) -> Self {
        Self {
            base: WidgetBase::new(Interaction::new()),
            layout_box: LayoutBox::default(),
            inner: Vec::new(),
            hooks_id: WidgetId::new_unique(),
            width_handle,
        }
    }

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

fn entry_row(entry: &devtools::RenderLogEntry, index: usize) -> Label {
    let (kind_label, kind_color) = match entry.kind {
        RenderEventKind::Rerender => ("RERENDER", Color::AMBER_400),
        RenderEventKind::Repaint => ("REPAINT ", Color::SKY_400),
    };

    let text = format!(
        "[{:>10}us] {kind_label}  {:<28} {:<20} {}",
        entry.t_micros,
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
        let width = self.width_handle.get();
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
            .overflow_y(Overflow::Auto)
            .padding(Edges::symmetric(8.0, 4.0))
            .child(log_column);

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
                    .font_size(Length::px(13.0))
            )
            .child(
                Button::new()
                    .label("Clear")
                    .font_size(Length::px(12.0))
                    .padding(Edges::symmetric(8.0, 4.0))
                    .background(Color::NEUTRAL_800)
                    .color(Color::NEUTRAL_100)
                    .on_click(|_ctx| devtools::clear())
            );

        let content = View::new()
            .key("devtools_content")
            .display(Display::Flex)
            .flex_direction(FlexDirection::Column)
            .flex_grow(1.0)
            .background(Color::NEUTRAL_950)
            .child(header)
            .child(log_body);

        Box::new(
            View::new()
                .display(Display::Flex)
                .flex_direction(FlexDirection::Row)
                .size(Length::px(width), pct!(100.0))
                .child(DevtoolsResizeHandle::new(self.width_handle.clone()))
                .child(content)
        )
    }
}

crate::impl_composite_widget!(DevtoolsPanel);
