use xengui_wgpu::WgpuWindowRenderer;

pub enum XenEvent {
    RendererReady(Box<WgpuWindowRenderer>),
    CancelSelection,
    SystemThemeChanged(winit::window::Theme),
    NativeInputChanged(String),
    /// Sent from any thread purely to wake a blocked event loop once an
    /// async task becomes ready; the actual polling happens once per
    /// iteration in `about_to_wait`.
    PollTasks,
}
