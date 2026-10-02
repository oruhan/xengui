//! Platform-independent runtime state machines.
//!
//! This module deliberately contains no `Window`, event-loop, or GPU handles.
//! The winit adapter applies the returned effects to those platform objects.

use web_time::Instant;
use xengui::{Breakpoint, ModifiersState, WidgetPath, reconciler};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DirtyCause {
    Update,
    Resize,
    ScaleFactor,
    Theme,
    Animation,
    RendererRecovery,
}

pub(crate) struct AppRuntime {
    pub(crate) reconcile_work: Option<reconciler::WorkLoop>,
    pub(crate) last_breakpoint: Breakpoint,
}

impl Default for AppRuntime {
    fn default() -> Self {
        Self {
            reconcile_work: None,
            last_breakpoint: Breakpoint::Compact,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WindowLifecycle {
    Suspended,
    Resuming,
    Active,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct WindowMetrics {
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) scale_factor: f64,
}

impl WindowMetrics {
    pub(crate) const fn new(width: u32, height: u32, scale_factor: f64) -> Self {
        Self {
            width,
            height,
            scale_factor,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WindowEffect {
    CreateWindow,
    SuspendSurface,
    ResumeSurface,
    ReconfigureSurface,
    Relayout,
    RequestRedraw,
}

#[derive(Debug)]
pub(crate) struct WindowRuntime {
    pub(crate) lifecycle: WindowLifecycle,
    pub(crate) metrics: Option<WindowMetrics>,
    pub(crate) is_visible: bool,
    pub(crate) pending_maximize: bool,
}

impl Default for WindowRuntime {
    fn default() -> Self {
        Self {
            lifecycle: WindowLifecycle::Suspended,
            metrics: None,
            is_visible: false,
            pending_maximize: false,
        }
    }
}

impl WindowRuntime {
    pub(crate) fn resume_requested(&mut self) -> Vec<WindowEffect> {
        if self.lifecycle != WindowLifecycle::Suspended {
            return Vec::new();
        }
        self.lifecycle = WindowLifecycle::Resuming;
        vec![WindowEffect::CreateWindow]
    }

    pub(crate) fn resumed(&mut self, metrics: WindowMetrics) -> Vec<WindowEffect> {
        self.lifecycle = WindowLifecycle::Active;
        self.metrics = Some(metrics);
        vec![
            WindowEffect::ResumeSurface,
            WindowEffect::Relayout,
            WindowEffect::RequestRedraw,
        ]
    }

    pub(crate) fn suspend(&mut self) -> Vec<WindowEffect> {
        if self.lifecycle == WindowLifecycle::Suspended {
            return Vec::new();
        }
        self.lifecycle = WindowLifecycle::Suspended;
        self.is_visible = false;
        vec![WindowEffect::SuspendSurface]
    }

    pub(crate) fn resize(&mut self, width: u32, height: u32) -> Vec<WindowEffect> {
        let scale_factor = self.metrics.map_or(1.0, |metrics| metrics.scale_factor);
        self.metrics = Some(WindowMetrics::new(width, height, scale_factor));
        if self.lifecycle != WindowLifecycle::Active {
            return Vec::new();
        }
        vec![
            WindowEffect::ReconfigureSurface,
            WindowEffect::Relayout,
            WindowEffect::RequestRedraw,
        ]
    }

    pub(crate) fn scale_factor_changed(&mut self, scale_factor: f64) -> Vec<WindowEffect> {
        if let Some(metrics) = &mut self.metrics {
            metrics.scale_factor = scale_factor;
        }
        if self.lifecycle != WindowLifecycle::Active {
            return Vec::new();
        }
        vec![WindowEffect::Relayout, WindowEffect::RequestRedraw]
    }

    pub(crate) fn reveal(&mut self) -> bool {
        if self.lifecycle != WindowLifecycle::Active || self.is_visible {
            return false;
        }
        self.is_visible = true;
        true
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum InputEffect {
    CancelPointerCapture,
    ClearHover,
    ClearModifiers,
}

#[derive(Debug, Default)]
pub(crate) struct InputRouter {
    pub(crate) has_focus: bool,
    pub(crate) modifiers: ModifiersState,
    pub(crate) pointer_captured: bool,
    pub(crate) pointer_hovered: bool,
}

impl InputRouter {
    pub(crate) fn focus_changed(&mut self, focused: bool) -> Vec<InputEffect> {
        self.has_focus = focused;
        if focused {
            return Vec::new();
        }

        let mut effects = vec![InputEffect::ClearModifiers];
        self.modifiers = ModifiersState::default();
        if std::mem::take(&mut self.pointer_captured) {
            effects.push(InputEffect::CancelPointerCapture);
        }
        if std::mem::take(&mut self.pointer_hovered) {
            effects.push(InputEffect::ClearHover);
        }
        effects
    }
}

#[derive(Debug, Default)]
pub(crate) struct GestureArena {
    pub(crate) pending_long_press: Option<(Instant, (f32, f32), WidgetPath)>,
    pub(crate) touch_pan_owner: Option<WidgetPath>,
    pub(crate) touch_start_point: Option<(f32, f32)>,
    pub(crate) touch_activation_cancelled: bool,
}

impl GestureArena {
    pub(crate) fn reset(&mut self) {
        self.pending_long_press = None;
        self.touch_pan_owner = None;
        self.touch_start_point = None;
        self.touch_activation_cancelled = false;
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ImeTransition {
    Enabled,
    Preedit(String),
    Commit(String),
    Disabled,
}

#[derive(Debug, Default)]
pub(crate) struct TextInputSession {
    pub(crate) native_ime_allowed: bool,
    pub(crate) platform_enabled: bool,
    pub(crate) preedit: Option<String>,
}

impl TextInputSession {
    pub(crate) fn set_allowed(&mut self, allowed: bool, force_show: bool) -> bool {
        let changed = allowed != self.native_ime_allowed;
        self.native_ime_allowed = allowed;
        changed || (force_show && allowed)
    }

    pub(crate) fn transition(&mut self, transition: ImeTransition) {
        match transition {
            ImeTransition::Enabled => self.platform_enabled = true,
            ImeTransition::Preedit(text) => self.preedit = Some(text),
            ImeTransition::Commit(_) => self.preedit = None,
            ImeTransition::Disabled => {
                self.platform_enabled = false;
                self.preedit = None;
            }
        }
    }

    pub(crate) fn focus_lost(&mut self) {
        self.platform_enabled = false;
        self.preedit = None;
    }
}

#[derive(Debug, Default)]
pub(crate) struct FrameScheduler {
    pub(crate) next_blink: Option<Instant>,
    pub(crate) next_animation: Option<Instant>,
    dirty_causes: Vec<DirtyCause>,
}

impl FrameScheduler {
    pub(crate) fn mark_dirty(&mut self, cause: DirtyCause) {
        if !self.dirty_causes.contains(&cause) {
            self.dirty_causes.push(cause);
        }
    }

    pub(crate) fn take_dirty_causes(&mut self) -> Vec<DirtyCause> {
        std::mem::take(&mut self.dirty_causes)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RendererState {
    Ready,
    SurfaceSuspended,
    Recovering,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RendererFailure {
    DeviceLost,
    Internal,
    Surface,
    OutOfMemory,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RendererEffect {
    RebuildDevice,
    ReportFatal,
}

#[derive(Debug)]
pub(crate) struct RendererSupervisor {
    pub(crate) state: RendererState,
}

impl Default for RendererSupervisor {
    fn default() -> Self {
        Self {
            state: RendererState::Ready,
        }
    }
}

impl RendererSupervisor {
    pub(crate) fn suspend_surface(&mut self) {
        self.state = RendererState::SurfaceSuspended;
    }

    pub(crate) fn surface_resumed(&mut self) {
        self.state = RendererState::Ready;
    }

    pub(crate) fn failed(&mut self, failure: RendererFailure) -> RendererEffect {
        match failure {
            RendererFailure::DeviceLost | RendererFailure::Internal => {
                self.state = RendererState::Recovering;
                RendererEffect::RebuildDevice
            }
            RendererFailure::Surface | RendererFailure::OutOfMemory | RendererFailure::Other => {
                self.state = RendererState::Failed;
                RendererEffect::ReportFatal
            }
        }
    }

    pub(crate) fn recovery_finished(&mut self, succeeded: bool) {
        self.state = if succeeded {
            RendererState::Ready
        } else {
            RendererState::Failed
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resize_transition_needs_no_window() {
        let mut runtime = WindowRuntime::default();
        assert!(runtime.resize(800, 600).is_empty());
        assert_eq!(runtime.metrics, Some(WindowMetrics::new(800, 600, 1.0)));

        runtime.resume_requested();
        runtime.resumed(WindowMetrics::new(800, 600, 2.0));
        assert_eq!(
            runtime.resize(1024, 768),
            vec![
                WindowEffect::ReconfigureSurface,
                WindowEffect::Relayout,
                WindowEffect::RequestRedraw
            ]
        );
    }

    #[test]
    fn suspend_resume_preserves_metrics_and_reactivates_surface() {
        let mut runtime = WindowRuntime::default();
        assert_eq!(runtime.resume_requested(), vec![WindowEffect::CreateWindow]);
        runtime.resumed(WindowMetrics::new(640, 480, 1.5));
        runtime.is_visible = true;
        assert_eq!(runtime.suspend(), vec![WindowEffect::SuspendSurface]);
        assert_eq!(runtime.lifecycle, WindowLifecycle::Suspended);
        assert!(!runtime.is_visible);
        assert_eq!(runtime.metrics, Some(WindowMetrics::new(640, 480, 1.5)));
        assert_eq!(runtime.resume_requested(), vec![WindowEffect::CreateWindow]);
        assert_eq!(
            runtime.resumed(WindowMetrics::new(640, 480, 1.5)),
            vec![
                WindowEffect::ResumeSurface,
                WindowEffect::Relayout,
                WindowEffect::RequestRedraw
            ]
        );
        assert_eq!(runtime.lifecycle, WindowLifecycle::Active);
    }

    #[test]
    fn focus_loss_cancels_transient_input_state() {
        let mut router = InputRouter {
            has_focus: true,
            pointer_captured: true,
            pointer_hovered: true,
            ..InputRouter::default()
        };
        assert_eq!(
            router.focus_changed(false),
            vec![
                InputEffect::ClearModifiers,
                InputEffect::CancelPointerCapture,
                InputEffect::ClearHover
            ]
        );
    }

    #[test]
    fn ime_session_tracks_preedit_commit_and_focus_loss() {
        let mut session = TextInputSession::default();
        assert!(session.set_allowed(true, false));
        session.transition(ImeTransition::Enabled);
        session.transition(ImeTransition::Preedit("x".into()));
        assert_eq!(session.preedit.as_deref(), Some("x"));
        session.transition(ImeTransition::Commit("x".into()));
        assert_eq!(session.preedit, None);
        session.focus_lost();
        assert!(!session.platform_enabled);
    }

    #[test]
    fn device_loss_enters_recovery_without_a_renderer() {
        let mut supervisor = RendererSupervisor::default();
        assert_eq!(
            supervisor.failed(RendererFailure::DeviceLost),
            RendererEffect::RebuildDevice
        );
        assert_eq!(supervisor.state, RendererState::Recovering);
        supervisor.recovery_finished(true);
        assert_eq!(supervisor.state, RendererState::Ready);
    }
}
