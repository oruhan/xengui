// SPDX-License-Identifier: Apache-2.0
use crate::{
    ElementState, EventCtx, InputEvent, InputState, KeyboardEvent, ModifiersState, MouseButton,
    Widget, cancel_auto_scroll_recursive, dispatch_hover_transition, dispatch_positional,
    hit_test_path, resolve_hover_cursor,
};
use std::rc::Rc;

/// High-level pointer/keyboard/focus dispatcher built on top of the
/// low-level primitives in `input.rs`. Platform crates (e.g. xenframe)
/// can own a single `Dispatcher` instead of re-implementing hover,
/// pointer-capture and focus bookkeeping themselves.
pub struct Dispatcher {
    /// The `state` value carried by this type.
    pub state: InputState,
    platform_services: Rc<dyn crate::PlatformServices>,
}

impl Default for Dispatcher {
    fn default() -> Self {
        Self {
            state: InputState::default(),
            platform_services: crate::platform_services::default_platform_services(),
        }
    }
}

impl Dispatcher {
    /// Creates a value with its default configuration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a dispatcher whose event contexts use `services`.
    pub fn with_platform_services(services: Rc<dyn crate::PlatformServices>) -> Self {
        Self {
            state: InputState::default(),
            platform_services: services,
        }
    }

    fn event_ctx(&self) -> EventCtx {
        EventCtx::with_platform_services(Rc::clone(&self.platform_services))
    }

    /// Updates hover state for the widget under `point` and forwards the
    /// move event to whichever widget currently has pointer capture.
    pub fn pointer_moved(&mut self, tree: &mut [Box<dyn Widget>], point: (f32, f32)) -> EventCtx {
        let mut ctx = self.event_ctx();
        self.state.cursor_pos = Some(point);

        let new_hover = hit_test_path(tree, point);
        if new_hover != self.state.hovered_path {
            dispatch_hover_transition(
                tree,
                self.state.hovered_path.as_ref(),
                new_hover.as_ref(),
                &mut ctx,
            );
            self.state.hovered_path = new_hover.clone();
        }

        let move_target = self.state.pointer_capture.target().cloned().or(new_hover);
        if let Some(path) = &move_target {
            dispatch_positional(
                tree,
                path,
                &(InputEvent::MouseMoved { position: point }),
                &mut ctx,
            );
        }

        if let Some(path) = self
            .state
            .pointer_capture
            .target()
            .or(self.state.hovered_path.as_ref())
            && let Some(cursor) = resolve_hover_cursor(tree, path)
        {
            ctx.set_cursor_icon(cursor);
        }

        ctx
    }

    /// Returns or updates the `pointer_input` value.
    pub fn pointer_input(
        &mut self,
        tree: &mut [Box<dyn Widget>],
        point: (f32, f32),
        input_state: ElementState,
        button: MouseButton,
    ) -> EventCtx {
        let mut ctx = self.event_ctx();

        // A press with any button other than Middle cancels any in-progress
        // AutoScroll gesture across the whole tree: a click landing on an
        // interactive descendant (Button, Switch, Checkbox, TextBox, ...) is
        // consumed there and never bubbles up to the scrollable View that
        // started the gesture. Middle itself is excluded since View's own
        // handler needs to see its unmodified state to toggle AutoScroll
        // on/off correctly.
        if input_state == ElementState::Pressed && button != MouseButton::Middle {
            cancel_auto_scroll_recursive(tree, &mut ctx);
        }

        let path = if input_state == ElementState::Released {
            self.state.pointer_capture.target().cloned()
        } else {
            self.state
                .hovered_path
                .clone()
                .or_else(|| hit_test_path(tree, point))
        };
        if input_state == ElementState::Pressed {
            self.state.pointer_capture.capture(path.clone(), button);
        }

        if let Some(path) = &path {
            dispatch_positional(
                tree,
                path,
                &(InputEvent::MouseInput {
                    state: input_state,
                    button,
                    position: point,
                }),
                &mut ctx,
            );
        }

        if input_state == ElementState::Released {
            self.state.pointer_capture.release(button);
        }

        ctx
    }

    /// Dispatches a keyboard event to the currently focused widget.
    pub fn key_input(
        &mut self,
        tree: &mut [Box<dyn Widget>],
        event: KeyboardEvent,
        modifiers: ModifiersState,
    ) -> EventCtx {
        let mut ctx = self.event_ctx();
        if let Some(path) = self.state.focus.focused_path().cloned() {
            dispatch_positional(
                tree,
                &path,
                &(InputEvent::KeyInput { event, modifiers }),
                &mut ctx,
            );
        }
        ctx
    }

    /// Moves keyboard focus to the next (or previous) focusable widget,
    /// wrapping at the boundaries.
    pub fn advance_focus(&mut self, tree: &mut [Box<dyn Widget>], backward: bool) -> EventCtx {
        let mut ctx = self.event_ctx();
        self.state.focus.advance(tree, backward, &mut ctx);

        ctx
    }
}
