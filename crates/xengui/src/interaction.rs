// SPDX-License-Identifier: Apache-2.0
use crate::{
    Cursor, ElementState, EventCtx, EventStatus, InputEvent, Key, KeyState, KeyboardEvent,
    MouseButton,
};

type Callback = Box<dyn FnMut(&mut EventCtx)>;
type HoverCallback = Box<dyn FnMut(bool, &mut EventCtx)>;
type MouseInputCallback = Box<dyn FnMut(ElementState, MouseButton, &mut EventCtx)>;
type KeyCallback = Box<dyn FnMut(&KeyboardEvent, &mut EventCtx)>;

/// Data and behavior represented by `Interaction`.
pub struct Interaction {
    /// The `enabled` value carried by this type.
    pub enabled: bool,

    /// The `focusable` value carried by this type.
    pub focusable: bool,
    /// The `hover_cursor` value carried by this type.
    pub hover_cursor: Option<Cursor>,

    /// Marks this widget as a window drag handle: a left-button press on
    /// it moves the OS window instead of going through the normal press
    /// dispatch, so a custom titlebar can be built from ordinary widgets.
    pub drag_region: bool,

    /// The `hovered` value carried by this type.
    pub hovered: bool,
    /// The `pressed` value carried by this type.
    pub pressed: bool,
    /// The `focused` value carried by this type.
    pub focused: bool,
    /// The `focus_visible` value carried by this type.
    pub focus_visible: bool,
    /// The `focus_within` value carried by this type.
    pub focus_within: bool,

    /// The `on_mouse_enter` value carried by this type.
    pub on_mouse_enter: Option<Callback>,
    /// The `on_mouse_leave` value carried by this type.
    pub on_mouse_leave: Option<Callback>,

    /// The `on_hover` value carried by this type.
    pub on_hover: Option<HoverCallback>,

    /// The `on_mouse_input` value carried by this type.
    pub on_mouse_input: Option<MouseInputCallback>,
    /// The `on_key` value carried by this type.
    pub on_key: Option<KeyCallback>,
    /// The `on_click` value carried by this type.
    pub on_click: Option<Callback>,
}

impl Interaction {
    /// Creates a value with its default configuration.
    pub fn new() -> Self {
        Self {
            enabled: true,
            focusable: false,
            hover_cursor: None,
            drag_region: false,
            hovered: false,
            pressed: false,
            focused: false,
            focus_visible: false,
            focus_within: false,
            on_mouse_enter: None,
            on_mouse_leave: None,
            on_hover: None,
            on_mouse_input: None,
            on_key: None,
            on_click: None,
        }
    }

    /// Updates the `set_enabled` value.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            self.pressed = false;
        }
    }

    /// Returns whether the `is_active` condition is satisfied.
    pub fn is_active(&self) -> bool {
        self.focusable
            || self.hover_cursor.is_some()
            || self.drag_region
            || self.on_mouse_enter.is_some()
            || self.on_mouse_leave.is_some()
            || self.on_hover.is_some()
            || self.on_mouse_input.is_some()
            || self.on_key.is_some()
            || self.on_click.is_some()
    }

    /// Returns or updates the `transfer_from` value.
    pub fn transfer_from(&mut self, old: &Interaction) {
        self.hovered = old.hovered;
        self.pressed = old.pressed;
        self.focused = old.focused;
        self.focus_visible = old.focus_visible;
        self.focus_within = old.focus_within;
    }

    fn is_activation_key(key: Key) -> bool {
        matches!(key, Key::Enter | Key::Space)
    }

    /// Returns or updates the `handle` value.
    pub fn handle(&mut self, event: &InputEvent, ctx: &mut EventCtx) -> EventStatus {
        if !self.enabled {
            // Hover must keep reflecting the real cursor position even while
            // disabled, so a later transfer_from during reconciliation can
            // never resurrect a stale flag from before the widget was disabled.
            match event {
                InputEvent::MouseEntered => {
                    self.hovered = true;
                    if let Some(icon) = self.hover_cursor {
                        ctx.set_cursor_icon(icon);
                    }
                }
                InputEvent::MouseExited => {
                    self.hovered = false;
                    self.pressed = false;
                    if self.hover_cursor.is_some() {
                        ctx.set_cursor_icon(Cursor::Default);
                    }
                }
                _ => {}
            }
            return EventStatus::Ignored;
        }

        match event {
            InputEvent::MouseEntered => {
                self.hovered = true;
                if let Some(icon) = self.hover_cursor {
                    ctx.set_cursor_icon(icon);
                }
                if let Some(cb) = self.on_mouse_enter.as_mut() {
                    cb(ctx);
                }
                if let Some(cb) = self.on_hover.as_mut() {
                    cb(true, ctx);
                }
                EventStatus::Handled
            }

            InputEvent::MouseExited => {
                self.hovered = false;
                self.pressed = false;
                if self.hover_cursor.is_some() {
                    ctx.set_cursor_icon(Cursor::Default);
                }
                if let Some(cb) = self.on_mouse_leave.as_mut() {
                    cb(ctx);
                }
                if let Some(cb) = self.on_hover.as_mut() {
                    cb(false, ctx);
                }
                EventStatus::Handled
            }

            InputEvent::MouseInput { state, button, .. } => {
                let mut handled = false;

                if let Some(cb) = self.on_mouse_input.as_mut() {
                    cb(*state, *button, ctx);
                    handled = true;
                }

                if *button == MouseButton::Left {
                    match state {
                        ElementState::Pressed => {
                            self.pressed = true;
                            self.focus_visible = false;
                            // Cursor is already owned by whichever widget is
                            // actually hovered (set via MouseEntered/MouseExited);
                            // re-setting it here would let an ancestor that
                            // merely claims this click for its own on_click
                            // overwrite a still-hovered child's own cursor.
                            if self.focusable {
                                ctx.request_focus();
                            }
                        }
                        ElementState::Released => {
                            let was_click = self.pressed && self.hovered;
                            self.pressed = false;
                            if was_click && let Some(cb) = self.on_click.as_mut() {
                                cb(ctx);
                            }
                        }
                    }

                    // Only claim the event when something actually reacts to clicks;
                    // a widget that merely tracks pressed/hover state for its own
                    // cursor shouldn't swallow the press from an ancestor with real
                    // click handling (e.g. a Label nested inside a clickable View).
                    if self.on_click.is_some() || self.focusable {
                        handled = true;
                    }
                }

                if handled {
                    EventStatus::Handled
                } else {
                    EventStatus::Ignored
                }
            }

            InputEvent::KeyInput {
                event: key_event, ..
            } => {
                let mut consumed = self.on_key.is_some();

                if let Some(cb) = self.on_key.as_mut() {
                    cb(key_event, ctx);
                }

                if self.focused
                    && key_event.key == Key::Escape
                    && key_event.state == KeyState::Pressed
                {
                    ctx.release_focus();
                    consumed = true;
                }

                if self.focused && Self::is_activation_key(key_event.key) {
                    match key_event.state {
                        KeyState::Pressed if !key_event.repeat => {
                            self.pressed = true;
                            if let Some(cb) = self.on_click.as_mut() {
                                cb(ctx);
                            }
                        }
                        KeyState::Released => {
                            self.pressed = false;
                        }
                        _ => {}
                    }
                    consumed = true;
                }

                if consumed {
                    EventStatus::Handled
                } else {
                    EventStatus::Ignored
                }
            }

            InputEvent::FocusGained { via_keyboard } => {
                self.focused = true;
                self.focus_visible = *via_keyboard;
                if let Some(icon) = self.hover_cursor {
                    ctx.set_cursor_icon(icon);
                }
                EventStatus::Handled
            }

            InputEvent::FocusLost => {
                self.focused = false;
                self.pressed = false;
                self.focus_visible = false;
                EventStatus::Handled
            }

            _ => EventStatus::Ignored,
        }
    }
}

impl Default for Interaction {
    fn default() -> Self {
        Self::new()
    }
}

#[macro_export]
/// Implements common pointer and keyboard callback builders for a widget.
macro_rules! impl_interaction_builders {
    (base $ty:ty) => {
        impl $ty {
            /// Registers a callback invoked when the widget is activated.
            pub fn on_click(mut self, f: impl FnMut(&mut $crate::EventCtx) + 'static) -> Self {
                self.base.interaction.on_click = Some(Box::new(f));
                self
            }

            /// Registers a callback invoked when hover state changes.
            pub fn on_hover(
                mut self,
                f: impl FnMut(bool, &mut $crate::EventCtx) + 'static,
            ) -> Self {
                self.base.interaction.on_hover = Some(Box::new(f));
                self
            }

            /// Registers a callback invoked when the pointer enters the widget.
            pub fn on_mouse_enter(
                mut self,
                f: impl FnMut(&mut $crate::EventCtx) + 'static,
            ) -> Self {
                self.base.interaction.on_mouse_enter = Some(Box::new(f));
                self
            }

            /// Registers a callback invoked when the pointer leaves the widget.
            pub fn on_mouse_leave(
                mut self,
                f: impl FnMut(&mut $crate::EventCtx) + 'static,
            ) -> Self {
                self.base.interaction.on_mouse_leave = Some(Box::new(f));
                self
            }

            /// Registers a callback for raw mouse-button input.
            pub fn on_mouse_input(
                mut self,
                f: impl FnMut($crate::ElementState, $crate::MouseButton, &mut $crate::EventCtx)
                + 'static,
            ) -> Self {
                self.base.interaction.on_mouse_input = Some(Box::new(f));
                self
            }

            /// Registers a callback for keyboard input while focused.
            pub fn on_key(
                mut self,
                f: impl FnMut(&$crate::KeyboardEvent, &mut $crate::EventCtx) + 'static,
            ) -> Self {
                self.base.interaction.on_key = Some(Box::new(f));
                self
            }

            /// Marks the widget as a native window-drag region.
            pub fn window_drag_region(mut self, value: bool) -> Self {
                self.base.interaction.drag_region = value;
                self
            }
        }
    };
}
