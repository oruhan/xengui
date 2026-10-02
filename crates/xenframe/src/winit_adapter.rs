//! Thin translation layer between winit events and the runtime state machines.

use crate::runtime::{ImeTransition, WindowMetrics};
use winit::event::WindowEvent;

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum RuntimeWindowEvent {
    Resized(WindowMetrics),
    ScaleFactorChanged(f64),
    FocusChanged(bool),
    Ime(ImeTransition),
}

pub(crate) struct WinitAdapter;

impl WinitAdapter {
    pub(crate) fn normalize(event: &WindowEvent) -> Option<RuntimeWindowEvent> {
        match event {
            WindowEvent::Resized(size) => Some(RuntimeWindowEvent::Resized(WindowMetrics::new(
                size.width,
                size.height,
                1.0,
            ))),
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                Some(RuntimeWindowEvent::ScaleFactorChanged(*scale_factor))
            }
            WindowEvent::Focused(focused) => Some(RuntimeWindowEvent::FocusChanged(*focused)),
            WindowEvent::Ime(ime) => Some(RuntimeWindowEvent::Ime(match ime {
                winit::event::Ime::Enabled => ImeTransition::Enabled,
                winit::event::Ime::Preedit(text, _) => ImeTransition::Preedit(text.clone()),
                winit::event::Ime::Commit(text) => ImeTransition::Commit(text.clone()),
                winit::event::Ime::Disabled => ImeTransition::Disabled,
            })),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use winit::dpi::PhysicalSize;

    #[test]
    fn resize_is_normalized_without_a_window() {
        assert_eq!(
            WinitAdapter::normalize(&WindowEvent::Resized(PhysicalSize::new(320, 240))),
            Some(RuntimeWindowEvent::Resized(WindowMetrics::new(
                320, 240, 1.0
            )))
        );
    }

    #[test]
    fn ime_is_normalized_without_a_window() {
        assert_eq!(
            WinitAdapter::normalize(&WindowEvent::Ime(winit::event::Ime::Preedit(
                "compose".into(),
                Some((0, 7)),
            ))),
            Some(RuntimeWindowEvent::Ime(ImeTransition::Preedit(
                "compose".into()
            )))
        );
    }
}
