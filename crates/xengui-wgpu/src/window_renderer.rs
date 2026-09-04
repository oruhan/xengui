// SPDX-License-Identifier: Apache-2.0
use crate::{SampleCount, WgpuPipelines};
use std::fmt;
use std::sync::{Arc, Mutex};
use xengui::{FrameRenderer, SystemTheme, Widget};

/// Controls how the swapchain presentation mode is selected.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PresentModePreference {
    /// Tear-free, power-conscious presentation. Falls back to any mode the
    /// surface supports only if FIFO is unexpectedly unavailable.
    #[default]
    Vsync,
    /// Prefer mailbox for low latency while retaining a FIFO fallback.
    LowLatency,
    /// Prefer immediate presentation, accepting possible tearing.
    Immediate,
}

#[cfg(test)]
mod tests {
    use super::{PresentModePreference, select_present_mode};
    use wgpu::PresentMode;

    #[test]
    fn vsync_prefers_fifo_over_faster_modes() {
        let available = [
            PresentMode::Immediate,
            PresentMode::Mailbox,
            PresentMode::Fifo,
        ];
        assert_eq!(
            select_present_mode(&available, PresentModePreference::Vsync),
            Some(PresentMode::Fifo)
        );
    }

    #[test]
    fn low_latency_falls_back_to_fifo() {
        assert_eq!(
            select_present_mode(&[PresentMode::Fifo], PresentModePreference::LowLatency),
            Some(PresentMode::Fifo)
        );
    }

    #[test]
    fn empty_present_modes_are_rejected() {
        assert_eq!(
            select_present_mode(&[], PresentModePreference::Immediate),
            None
        );
    }
}

/// GPU and presentation policy used when creating a window renderer.
#[derive(Clone, Copy, Debug)]
pub struct RendererOptions {
    /// Graphics backends that wgpu may use.
    ///
    /// WebAssembly defaults to WebGL2 because some browsers expose a partial
    /// WebGPU surface but return a null adapter. Applications that require
    /// WebGPU can opt in with [`wgpu::Backends::BROWSER_WEBGPU`].
    pub backends: wgpu::Backends,
    /// Adapter power preference.
    pub power_preference: wgpu::PowerPreference,
    /// Swapchain presentation policy.
    pub present_mode: PresentModePreference,
    /// Multisample anti-aliasing sample count.
    pub sample_count: SampleCount,
    /// Preferred number of frames queued by the presentation engine.
    pub desired_maximum_frame_latency: u32,
}

impl Default for RendererOptions {
    fn default() -> Self {
        Self {
            backends: if cfg!(target_arch = "wasm32") {
                wgpu::Backends::GL
            } else {
                wgpu::Backends::PRIMARY
            },
            power_preference: wgpu::PowerPreference::None,
            present_mode: PresentModePreference::Vsync,
            sample_count: SampleCount::X4,
            desired_maximum_frame_latency: 2,
        }
    }
}

/// Initialization and runtime failures reported by the renderer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RendererError {
    SurfaceCreation(String),
    AdapterUnavailable(String),
    DeviceRequest(String),
    SurfaceUnsupported(&'static str),
    PipelineCreation(String),
    DeviceLost(String),
    OutOfMemory(String),
    Validation(String),
    Internal(String),
}

impl fmt::Display for RendererError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SurfaceCreation(message) => write!(f, "cannot create GPU surface: {message}"),
            Self::AdapterUnavailable(message) => write!(f, "no compatible GPU adapter: {message}"),
            Self::DeviceRequest(message) => write!(f, "cannot create GPU device: {message}"),
            Self::SurfaceUnsupported(message) => write!(f, "unsupported GPU surface: {message}"),
            Self::PipelineCreation(message) => write!(f, "cannot create GPU pipelines: {message}"),
            Self::DeviceLost(message) => write!(f, "GPU device lost: {message}"),
            Self::OutOfMemory(message) => write!(f, "GPU out of memory: {message}"),
            Self::Validation(message) => write!(f, "GPU validation error: {message}"),
            Self::Internal(message) => write!(f, "internal GPU error: {message}"),
        }
    }
}

impl std::error::Error for RendererError {}

/// Result of a frame request that did not fail fatally.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FrameOutcome {
    Presented,
    SkippedOccluded,
    SkippedTimeout,
}

type PendingGpuError = Arc<Mutex<Option<RendererError>>>;

fn select_present_mode(
    available: &[wgpu::PresentMode],
    preference: PresentModePreference,
) -> Option<wgpu::PresentMode> {
    let preferred = match preference {
        PresentModePreference::Vsync => [
            wgpu::PresentMode::Fifo,
            wgpu::PresentMode::FifoRelaxed,
            wgpu::PresentMode::Mailbox,
            wgpu::PresentMode::Immediate,
        ],
        PresentModePreference::LowLatency => [
            wgpu::PresentMode::Mailbox,
            wgpu::PresentMode::Fifo,
            wgpu::PresentMode::FifoRelaxed,
            wgpu::PresentMode::Immediate,
        ],
        PresentModePreference::Immediate => [
            wgpu::PresentMode::Immediate,
            wgpu::PresentMode::Mailbox,
            wgpu::PresentMode::Fifo,
            wgpu::PresentMode::FifoRelaxed,
        ],
    };

    preferred
        .into_iter()
        .find(|candidate| available.contains(candidate))
        .or_else(|| available.first().copied())
}

fn install_error_handlers(device: &wgpu::Device, pending: PendingGpuError) {
    let device_lost = Arc::clone(&pending);
    device.set_device_lost_callback(move |_reason, message| {
        if let Ok(mut error) = device_lost.lock() {
            *error = Some(RendererError::DeviceLost(message));
        }
    });

    device.on_uncaptured_error(Arc::new(move |error| {
        let renderer_error = match error {
            wgpu::Error::OutOfMemory { source } => RendererError::OutOfMemory(source.to_string()),
            wgpu::Error::Validation { description, .. } => RendererError::Validation(description),
            wgpu::Error::Internal { description, .. } => RendererError::Internal(description),
        };
        if let Ok(mut error) = pending.lock() {
            *error = Some(renderer_error);
        }
    }));
}

/// Owns a wgpu device/surface for a native window and drives xengui's
/// `FrameRenderer` against it every frame. This is xenframe's default
/// integration point. Not winit-specific: `W` only needs to provide a
/// raw window/display handle, so any windowing crate works.
pub struct WgpuWindowRenderer {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pipelines: WgpuPipelines,
    frame: FrameRenderer,
    options: RendererOptions,
    pending_gpu_error: PendingGpuError,
}

impl WgpuWindowRenderer {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn new<W>(
        window: Arc<W>,
        width: u32,
        height: u32,
        user_fonts: Vec<(String, Vec<u8>)>,
    ) -> Result<Self, String>
    where
        W: wgpu::WindowHandle + raw_window_handle::HasDisplayHandle + 'static,
    {
        Self::new_with_options(
            window,
            width,
            height,
            user_fonts,
            RendererOptions::default(),
        )
        .map_err(|error| error.to_string())
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn new_with_options<W>(
        window: Arc<W>,
        width: u32,
        height: u32,
        user_fonts: Vec<(String, Vec<u8>)>,
        options: RendererOptions,
    ) -> Result<Self, RendererError>
    where
        W: wgpu::WindowHandle + raw_window_handle::HasDisplayHandle + 'static,
    {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: options.backends,
            flags: wgpu::InstanceFlags::default(),
            memory_budget_thresholds: wgpu::MemoryBudgetThresholds::default(),
            backend_options: wgpu::BackendOptions::default(),
            display: None,
        });

        let surface = instance
            .create_surface(window)
            .map_err(|error| RendererError::SurfaceCreation(error.to_string()))?;

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: options.power_preference,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
            apply_limit_buckets: false,
        }))
        .map_err(|error| RendererError::AdapterUnavailable(error.to_string()))?;

        let supported_limits = adapter.limits();
        let required_limits = wgpu::Limits::downlevel_defaults()
            .using_resolution(supported_limits.clone())
            .using_alignment(supported_limits);

        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            required_limits,
            ..Default::default()
        }))
        .map_err(|error| RendererError::DeviceRequest(error.to_string()))?;

        Self::init_common(
            surface,
            &adapter,
            device,
            queue,
            (width, height),
            user_fonts,
            options,
        )
    }

    #[cfg(target_arch = "wasm32")]
    pub async fn new<W>(
        window: Arc<W>,
        width: u32,
        height: u32,
        user_fonts: Vec<(String, Vec<u8>)>,
    ) -> Result<Self, String>
    where
        W: wgpu::WindowHandle + raw_window_handle::HasDisplayHandle + 'static,
    {
        Self::new_with_options(
            window,
            width,
            height,
            user_fonts,
            RendererOptions::default(),
        )
        .await
        .map_err(|error| error.to_string())
    }

    #[cfg(target_arch = "wasm32")]
    pub async fn new_with_options<W>(
        window: Arc<W>,
        width: u32,
        height: u32,
        user_fonts: Vec<(String, Vec<u8>)>,
        options: RendererOptions,
    ) -> Result<Self, RendererError>
    where
        W: wgpu::WindowHandle + raw_window_handle::HasDisplayHandle + 'static,
    {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: options.backends,
            flags: wgpu::InstanceFlags::default(),
            memory_budget_thresholds: wgpu::MemoryBudgetThresholds::default(),
            backend_options: wgpu::BackendOptions::default(),
            display: None,
        });

        let t_surface = web_time::Instant::now();
        let surface = instance
            .create_surface(window)
            .map_err(|error| RendererError::SurfaceCreation(error.to_string()))?;
        log::info!("phase: surface {:?}", t_surface.elapsed());

        let t_adapter = web_time::Instant::now();
        let adapter = instance
            .request_adapter(
                &(wgpu::RequestAdapterOptions {
                    power_preference: options.power_preference,
                    compatible_surface: Some(&surface),
                    force_fallback_adapter: false,
                    apply_limit_buckets: false,
                }),
            )
            .await
            .map_err(|error| RendererError::AdapterUnavailable(error.to_string()))?;
        log::info!("phase: adapter {:?}", t_adapter.elapsed());

        let t_device = web_time::Instant::now();
        let supported_limits = adapter.limits();
        let required_limits = wgpu::Limits::downlevel_webgl2_defaults()
            .using_resolution(supported_limits.clone())
            .using_alignment(supported_limits);
        let (device, queue) = adapter
            .request_device(
                &(wgpu::DeviceDescriptor {
                    required_limits,
                    ..Default::default()
                }),
            )
            .await
            .map_err(|error| RendererError::DeviceRequest(error.to_string()))?;
        log::info!("phase: device {:?}", t_device.elapsed());

        let t_pipelines = web_time::Instant::now();
        let result = Self::init_common(
            surface,
            &adapter,
            device,
            queue,
            (width, height),
            user_fonts,
            options,
        );
        log::info!("phase: pipelines+fonts {:?}", t_pipelines.elapsed());
        result
    }

    fn init_common(
        surface: wgpu::Surface<'static>,
        adapter: &wgpu::Adapter,
        device: wgpu::Device,
        queue: wgpu::Queue,
        size: (u32, u32),
        user_fonts: Vec<(String, Vec<u8>)>,
        options: RendererOptions,
    ) -> Result<Self, RendererError> {
        let (width, height) = size;
        let surface_caps = surface.get_capabilities(adapter);
        let Some(surface_format) = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| {
                f == &wgpu::TextureFormat::Bgra8Unorm || f == &wgpu::TextureFormat::Rgba8Unorm
            })
            .or_else(|| surface_caps.formats.first().copied())
        else {
            return Err(RendererError::SurfaceUnsupported("no texture formats"));
        };

        let pipelines = WgpuPipelines::new(
            &device,
            &queue,
            adapter,
            surface_format,
            user_fonts,
            options.sample_count,
        )
        .map_err(RendererError::PipelineCreation)?;

        let alpha_mode = if surface_caps
            .alpha_modes
            .contains(&wgpu::CompositeAlphaMode::PreMultiplied)
        {
            wgpu::CompositeAlphaMode::PreMultiplied
        } else {
            surface_caps
                .alpha_modes
                .first()
                .copied()
                .ok_or(RendererError::SurfaceUnsupported("no alpha modes"))?
        };

        log::info!(
            "surface alpha_mode selected: {:?} (available: {:?})",
            alpha_mode,
            surface_caps.alpha_modes
        );

        let present_mode = select_present_mode(&surface_caps.present_modes, options.present_mode)
            .ok_or(RendererError::SurfaceUnsupported("no presentation modes"))?;

        log::info!(
            "surface present_mode selected: {:?} (available: {:?})",
            present_mode,
            surface_caps.present_modes
        );

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: width.max(1),
            height: height.max(1),
            present_mode,
            desired_maximum_frame_latency: options.desired_maximum_frame_latency.max(1),
            alpha_mode,
            view_formats: vec![],
            color_space: wgpu::SurfaceColorSpace::Auto,
        };
        surface.configure(&device, &config);

        let pending_gpu_error = Arc::new(Mutex::new(None));
        install_error_handlers(&device, Arc::clone(&pending_gpu_error));

        Ok(Self {
            surface,
            device,
            queue,
            config,
            pipelines,
            frame: FrameRenderer::new(),
            options,
            pending_gpu_error,
        })
    }

    pub fn is_animating(&self) -> bool {
        self.frame.is_animating()
    }

    pub fn options(&self) -> RendererOptions {
        self.options
    }

    fn take_gpu_error(&self) -> Option<RendererError> {
        self.pending_gpu_error
            .lock()
            .ok()
            .and_then(|mut error| error.take())
    }

    pub fn render_frame(
        &mut self,
        tree: &mut [Box<dyn Widget>],
        theme: SystemTheme,
        scale_factor: f32,
    ) {
        if let Err(error) = self.try_render_frame(tree, theme, scale_factor) {
            log::error!("renderer frame failed: {error}");
        }
    }

    pub fn try_render_frame(
        &mut self,
        tree: &mut [Box<dyn Widget>],
        theme: SystemTheme,
        scale_factor: f32,
    ) -> Result<FrameOutcome, RendererError> {
        if let Some(error) = self.take_gpu_error() {
            return Err(error);
        }

        log::trace!(
            "render_frame: {}x{} at {:?}",
            self.config.width,
            self.config.height,
            web_time::Instant::now()
        );

        const MAX_ACQUIRE_ATTEMPTS: u32 = 2;
        let mut frame = None;
        let mut timed_out = false;
        for attempt in 0..MAX_ACQUIRE_ATTEMPTS {
            match self.surface.get_current_texture() {
                wgpu::CurrentSurfaceTexture::Success(t) => {
                    xengui::devtools::record_note(
                        "surface:acquire",
                        format!("success attempt={attempt}"),
                    );
                    frame = Some(t);
                    break;
                }
                wgpu::CurrentSurfaceTexture::Suboptimal(texture) => {
                    xengui::devtools::record_note(
                        "surface:acquire",
                        format!("suboptimal attempt={attempt}"),
                    );
                    drop(texture);
                    self.surface.configure(&self.device, &self.config);
                }
                wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                    xengui::devtools::record_note(
                        "surface:acquire",
                        format!("outdated/lost attempt={attempt}"),
                    );
                    self.surface.configure(&self.device, &self.config);
                }
                wgpu::CurrentSurfaceTexture::Timeout => {
                    timed_out = true;
                    xengui::devtools::record_note(
                        "surface:acquire",
                        format!("timeout attempt={attempt}"),
                    );
                }
                wgpu::CurrentSurfaceTexture::Occluded => {
                    xengui::devtools::record("surface:occluded-skip");
                    log::trace!("Surface occluded, skipping frame.");
                    return Ok(FrameOutcome::SkippedOccluded);
                }
                wgpu::CurrentSurfaceTexture::Validation => {
                    xengui::devtools::record("surface:validation-error");
                    return Err(RendererError::Validation(
                        "surface texture acquisition failed validation".to_string(),
                    ));
                }
                #[allow(unreachable_patterns)]
                _ => {
                    xengui::devtools::record("surface:unhandled-error");
                    return Err(RendererError::Internal(
                        "surface returned an unknown texture state".to_string(),
                    ));
                }
            }
        }
        let Some(frame) = frame else {
            xengui::devtools::record("surface:acquire-failed-skip");
            if timed_out {
                log::debug!("Surface acquisition timed out; skipping frame.");
                return Ok(FrameOutcome::SkippedTimeout);
            }
            return Err(RendererError::Internal(
                "surface acquisition failed after recovery attempt".to_string(),
            ));
        };

        let frame_size = frame.texture.size();
        let (frame_width, frame_height) = (frame_size.width, frame_size.height);
        xengui::devtools::record_size_note(
            "frame:acquired",
            frame_width,
            frame_height,
            format!("config={}x{}", self.config.width, self.config.height),
        );

        let mut encoder = self.device.create_command_encoder(&Default::default());

        {
            let mut backend = self.pipelines.begin_frame(
                &self.device,
                &self.queue,
                &mut encoder,
                frame_width,
                frame_height,
                scale_factor,
            );
            self.frame.render_frame(
                tree,
                &mut backend,
                theme,
                scale_factor,
                frame_width,
                frame_height,
            );
        }

        // Everything above painted into an offscreen scene target instead
        // of the swapchain directly, so a backdrop-blur widget could read
        // back already-painted content mid-frame - this final blit is what
        // actually presents that scene onto the real surface.
        let surface_view = frame.texture.create_view(&Default::default());
        self.pipelines.present_scene(
            &self.device,
            &self.queue,
            &mut encoder,
            &surface_view,
            frame_width,
            frame_height,
        );

        xengui::devtools::record("frame:submit");
        self.queue.submit(Some(encoder.finish()));
        xengui::devtools::record("frame:present");
        self.queue.present(frame);
        xengui::devtools::record("frame:presented");

        if let Some(error) = self.take_gpu_error() {
            return Err(error);
        }
        Ok(FrameOutcome::Presented)
    }

    pub fn resize(
        &mut self,
        tree: &mut [Box<dyn Widget>],
        theme: SystemTheme,
        scale_factor: f32,
        width: u32,
        height: u32,
    ) {
        if let Err(error) = self.try_resize(tree, theme, scale_factor, width, height) {
            log::error!("renderer resize failed: {error}");
        }
    }

    pub fn try_resize(
        &mut self,
        tree: &mut [Box<dyn Widget>],
        theme: SystemTheme,
        scale_factor: f32,
        width: u32,
        height: u32,
    ) -> Result<Option<FrameOutcome>, RendererError> {
        if width == 0 || height == 0 {
            return Ok(None);
        }
        if width != self.config.width || height != self.config.height {
            xengui::devtools::record_size("surface:reconfigure", width, height);
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
            self.frame.resize();
        }
        self.try_render_frame(tree, theme, scale_factor).map(Some)
    }

    /// Reconfigures the swapchain to `width`/`height` without drawing a
    /// frame. Lets a burst of resize events keep the surface's own size in
    /// sync immediately while the actual (expensive) redraw is deferred to
    /// a single coalesced `RedrawRequested`, so the GPU never has to submit
    /// and present a frame per intermediate resize step.
    pub fn reconfigure_surface(&mut self, width: u32, height: u32) {
        log::info!(
            "reconfigure_surface: {}x{} at {:?}",
            width,
            height,
            web_time::Instant::now()
        );
        if width == 0 || height == 0 || (width == self.config.width && height == self.config.height)
        {
            return;
        }
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);
        self.frame.resize();
    }
}
