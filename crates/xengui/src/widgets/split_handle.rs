// SPDX-License-Identifier: Apache-2.0
//! Reusable draggable divider for resizing a side/bottom panel, shared by
//! any split-pane layout instead of being rebuilt per feature (see
//! `devtools_panel.rs` for the original single-purpose version this
//! generalizes).
use crate::{
    Background, Color, Constraints, Cursor, ElementState, EventCtx, EventStatus, InputEvent,
    Interaction, LayoutBox, Length, MeasureContext, MeasureResult, MouseButton, PaintContext,
    RectCommand, Size, Style, StyleBuilder, Widget, WidgetBase,
};
use std::cell::Cell;
use std::rc::Rc;

/// Which edge of the center content the panel this handle belongs to
/// sits on - determines the drag axis, cursor, and which direction grows
/// the panel.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SplitSide {
    /// The `Left` variant.
    Left,
    /// The `Right` variant.
    Right,
    /// The `Top` variant.
    Top,
    /// The `Bottom` variant.
    Bottom,
}

impl SplitSide {
    fn is_horizontal(self) -> bool {
        matches!(self, Self::Left | Self::Right)
    }

    // Left/Top: dragging toward positive screen coords grows the panel.
    // Right/Bottom: dragging the same way shrinks it (the divider moves
    // into the panel's own space instead of away from it).
    fn sign(self) -> f32 {
        match self {
            Self::Left | Self::Top => 1.0,
            Self::Right | Self::Bottom => -1.0,
        }
    }

    fn cursor(self) -> Cursor {
        if self.is_horizontal() {
            Cursor::EwResize
        } else {
            Cursor::NsResize
        }
    }
}

/// Data and behavior represented by `SplitHandle`.
pub struct SplitHandle {
    base: WidgetBase,
    layout_box: LayoutBox,
    size_handle: Rc<Cell<f32>>,
    side: SplitSide,
    min_size: f32,
    max_size: f32,
    dragging: Cell<bool>,
    drag_start_mouse: Cell<f32>,
    drag_start_size: Cell<f32>,
    scale_factor: Cell<f32>,
    idle_color: Color,
    hover_color: Color,
}

impl SplitHandle {
    /// Creates a value with its default configuration.
    pub fn new(size_handle: Rc<Cell<f32>>, side: SplitSide, min_size: f32, max_size: f32) -> Self {
        let mut interaction = Interaction::new();
        interaction.hover_cursor = Some(side.cursor());

        let mut base = WidgetBase::new(interaction);
        base.style.size = if side.is_horizontal() {
            Some(Size::new(Length::px(4.0), Length::pct(100.0)))
        } else {
            Some(Size::new(Length::pct(100.0), Length::px(4.0)))
        };

        Self {
            base,
            layout_box: LayoutBox::default(),
            size_handle,
            side,
            min_size,
            max_size,
            dragging: Cell::new(false),
            drag_start_mouse: Cell::new(0.0),
            drag_start_size: Cell::new(0.0),
            scale_factor: Cell::new(1.0),
            idle_color: Color::TRANSPARENT,
            hover_color: Color::BLUE_500,
        }
    }

    /// Overrides the idle/hover fill color. Idle defaults to transparent,
    /// so the handle is invisible until hovered or dragged.
    pub fn colors(mut self, idle: Color, hover: Color) -> Self {
        self.idle_color = idle;
        self.hover_color = hover;
        self
    }

    fn mouse_axis(&self, position: (f32, f32)) -> f32 {
        if self.side.is_horizontal() {
            position.0
        } else {
            position.1
        }
    }
}

impl StyleBuilder for SplitHandle {
    fn style_mut(&mut self) -> &mut Style {
        &mut self.base.style
    }

    fn mark_dirty(&mut self) {
        self.base.dirty = true;
    }
}

impl Widget for SplitHandle {
    crate::impl_widget_boilerplate!();

    fn debug_name(&self) -> &'static str {
        "Widget#SplitHandle"
    }

    fn measure(&self, _ctx: &mut MeasureContext, _constraints: Constraints) -> MeasureResult {
        MeasureResult::new(0.0, 0.0)
    }

    fn on_layout_pass(&self, ctx: &mut MeasureContext) {
        self.scale_factor.set(ctx.scale_factor);
    }

    fn paint(&self, ctx: &mut PaintContext) {
        let color = if self.dragging.get() || self.base.interaction.hovered {
            self.hover_color
        } else {
            self.idle_color
        };
        if color.a() <= 0.0 {
            return;
        }
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
                ctx.set_cursor_icon(self.side.cursor());
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
                self.drag_start_mouse.set(self.mouse_axis(*position));
                self.drag_start_size.set(self.size_handle.get());
                ctx.set_cursor_icon(self.side.cursor());
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
                let sf = self.scale_factor.get().max(0.0001);
                let delta_logical = ((self.mouse_axis(*position) - self.drag_start_mouse.get())
                    * self.side.sign())
                    / sf;
                let new_size = (self.drag_start_size.get() + delta_logical)
                    .clamp(self.min_size, self.max_size);
                self.size_handle.set(new_size);
                ctx.set_cursor_icon(self.side.cursor());
                self.base.dirty = true;
                ctx.request_redraw();
                EventStatus::Handled
            }
            _ => EventStatus::Ignored,
        }
    }

    fn transfer_interaction_state(&mut self, old: &dyn Widget) {
        if let Some(old) = old.as_any().downcast_ref::<SplitHandle>() {
            self.dragging.set(old.dragging.get());
            self.drag_start_mouse.set(old.drag_start_mouse.get());
            self.drag_start_size.set(old.drag_start_size.get());
            self.base.interaction.hovered = old.base.interaction.hovered;
            self.scale_factor.set(old.scale_factor.get());
        }
    }
}
