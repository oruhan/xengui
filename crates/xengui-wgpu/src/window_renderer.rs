// SPDX-License-Identifier: Apache-2.0
use crate::{ SampleCount, WgpuPipelines };
use std::sync::Arc;
use xengui::{ FrameRenderer, SystemTheme, Widget };

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
}

impl WgpuWindowRenderer {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn new<W>(
        window: Arc<W>,
        width: u32,
        height: u32,
        user_fonts: Vec<(String, Vec<u8>)>
    ) -> Result<Self, String>
        where W: wgpu::WindowHandle + raw_window_handle::HasDisplayHandle + 'static
    {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: if cfg!(target_os = "windows") {
                wgpu::Backends::VULKAN
            } else {
                wgpu::Backends::PRIMARY
            },
            flags: wgpu::InstanceFlags::default(),
            memory_budget_thresholds: wgpu::MemoryBudgetThresholds::default(),
            backend_options: wgpu::BackendOptions::default(),
            display: None,
        });

        let surface = instance
            .create_surface(window)
            .map_err(|e| format!("Cannot create surface: {}", e))?;

        let adapter = pollster
            ::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))
            .expect("Cannot find a compatible adapter");

        let (device, queue) = pollster
            ::block_on(
                adapter.request_device(
                    &(wgpu::DeviceDescriptor {
                        required_limits: adapter.limits(),
                        ..Default::default()
                    })
                )
            )
            .map_err(|e| format!("Cannot start GPU (device): {}", e))?;

        Self::init_common(surface, &adapter, device, queue, width, height, user_fonts)
    }

    #[cfg(target_arch = "wasm32")]
    pub async fn new<W>(
        window: Arc<W>,
        width: u32,
        height: u32,
        user_fonts: Vec<(String, Vec<u8>)>
    ) -> Result<Self, String>
        where W: wgpu::WindowHandle + raw_window_handle::HasDisplayHandle + 'static
    {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            flags: wgpu::InstanceFlags::default(),
            memory_budget_thresholds: wgpu::MemoryBudgetThresholds::default(),
            backend_options: wgpu::BackendOptions::default(),
            display: None,
        });

        let t_surface = web_time::Instant::now();
        let surface = instance
            .create_surface(window)
            .map_err(|e| format!("Cannot create surface: {}", e))?;
        log::info!("phase: surface {:?}", t_surface.elapsed());

        let t_adapter = web_time::Instant::now();
        let adapter = instance
            .request_adapter(
                &(wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::HighPerformance,
                    compatible_surface: Some(&surface),
                    force_fallback_adapter: false,
                    apply_limit_buckets: false,
                })
            ).await
            .map_err(|e| format!("Cannot find a compatible adapter: {}", e))?;
        log::info!("phase: adapter {:?}", t_adapter.elapsed());

        let t_device = web_time::Instant::now();
        let (device, queue) = adapter
            .request_device(
                &(wgpu::DeviceDescriptor {
                    required_limits: adapter.limits(),
                    ..Default::default()
                })
            ).await
            .map_err(|e| format!("Cannot start GPU (device): {}", e))?;
        log::info!("phase: device {:?}", t_device.elapsed());

        let t_pipelines = web_time::Instant::now();
        let result = Self::init_common(surface, &adapter, device, queue, width, height, user_fonts);
        log::info!("phase: pipelines+fonts {:?}", t_pipelines.elapsed());
        result
    }

    fn init_common(
        surface: wgpu::Surface<'static>,
        adapter: &wgpu::Adapter,
        device: wgpu::Device,
        queue: wgpu::Queue,
        width: u32,
        height: u32,
        user_fonts: Vec<(String, Vec<u8>)>
    ) -> Result<Self, String> {
        let surface_caps = surface.get_capabilities(adapter);
        let Some(surface_format) = surface_caps.formats
            .iter()
            .copied()
            .find(|f| {
                f == &wgpu::TextureFormat::Bgra8Unorm || f == &wgpu::TextureFormat::Rgba8Unorm
            })
            .or_else(|| surface_caps.formats.first().copied()) else {
            return Err(
                "Surface reports no supported texture formats (GPU/browser incompatibility).".to_string()
            );
        };

        // MSAA's resolve target (WgpuPipelines::resize_msaa) is currently
        // never wired into an actual render pass - every draw call targets
        // a sample_count:1 attachment (the swapchain view, or a filtered
        // subtree's offscreen texture). Requesting a higher sample count
        // here would build shape pipelines that mismatch those attachments,
        // so X1 is the only value that's actually consistent right now.
        let pipelines = WgpuPipelines::new(
            &device,
            &queue,
            adapter,
            surface_format,
            user_fonts,
            SampleCount::X1
        )?;

        let alpha_mode = if surface_caps.alpha_modes.contains(&wgpu::CompositeAlphaMode::Opaque) {
            wgpu::CompositeAlphaMode::Opaque
        } else {
            surface_caps.alpha_modes[0]
        };

        log::info!(
            "surface alpha_mode selected: {:?} (available: {:?})",
            alpha_mode,
            surface_caps.alpha_modes
        );

        // Fifo blocks get_current_texture() until the previous frame's
        // vsync completes. The whole resize pipeline runs synchronously
        // inside WM_SIZE (Windows never pumps RedrawRequested during a
        // live resize), so a blocked acquire stalls that handler - DWM
        // then stretches the last presented (stale-sized) buffer to fill
        // the already-grown window until the stalled call returns. A
        // non-blocking present mode keeps acquire immediate so every
        // WM_SIZE can finish its own layout/raster/present without
        // falling behind the drag.
        let present_mode = surface_caps.present_modes
            .iter()
            .copied()
            .find(|m| *m == wgpu::PresentMode::Immediate)
            .or_else(||
                surface_caps.present_modes
                    .iter()
                    .copied()
                    .find(|m| *m == wgpu::PresentMode::Mailbox)
            )
            .unwrap_or(wgpu::PresentMode::Fifo);

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
            desired_maximum_frame_latency: 2,
            alpha_mode,
            view_formats: vec![],
            color_space: wgpu::SurfaceColorSpace::Auto,
        };
        surface.configure(&device, &config);

        Ok(Self {
            surface,
            device,
            queue,
            config,
            pipelines,
            frame: FrameRenderer::new(),
        })
    }

    pub fn is_animating(&self) -> bool {
        self.frame.is_animating()
    }

    pub fn render_frame(
        &mut self,
        tree: &mut [Box<dyn Widget>],
        theme: SystemTheme,
        scale_factor: f32
    ) {
        log::info!(
            "render_frame: {}x{} at {:?}",
            self.config.width,
            self.config.height,
            web_time::Instant::now()
        );

        const MAX_ACQUIRE_ATTEMPTS: u32 = 8;
        let mut frame = None;
        for attempt in 0..MAX_ACQUIRE_ATTEMPTS {
            match self.surface.get_current_texture() {
                wgpu::CurrentSurfaceTexture::Success(t) => {
                    xengui::devtools::record_note(
                        "surface:acquire",
                        format!("success attempt={attempt}")
                    );
                    frame = Some(t);
                    break;
                }
                wgpu::CurrentSurfaceTexture::Suboptimal(texture) => {
                    xengui::devtools::record_note(
                        "surface:acquire",
                        format!("suboptimal attempt={attempt}")
                    );
                    drop(texture);
                    self.surface.configure(&self.device, &self.config);
                }
                wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                    xengui::devtools::record_note(
                        "surface:acquire",
                        format!("outdated/lost attempt={attempt}")
                    );
                    self.surface.configure(&self.device, &self.config);
                }
                wgpu::CurrentSurfaceTexture::Timeout => {
                    xengui::devtools::record_note(
                        "surface:acquire",
                        format!("timeout attempt={attempt}")
                    );
                    if attempt >= MAX_ACQUIRE_ATTEMPTS / 2 {
                        self.surface.configure(&self.device, &self.config);
                    }
                }
                wgpu::CurrentSurfaceTexture::Occluded => {
                    xengui::devtools::record("surface:occluded-skip");
                    log::trace!("Surface occluded, skipping frame.");
                    return;
                }
                wgpu::CurrentSurfaceTexture::Validation => {
                    xengui::devtools::record("surface:validation-skip");
                    log::warn!("Surface validation error, skipping frame.");
                    return;
                }
                #[allow(unreachable_patterns)]
                _ => {
                    xengui::devtools::record("surface:unhandled-skip");
                    log::warn!("Unhandled surface texture state, skipping frame.");
                    return;
                }
            }
        }
        let Some(frame) = frame else {
            xengui::devtools::record("surface:acquire-failed-skip");
            log::warn!("Surface acquire failed after retries, skipping frame.");
            return;
        };

        let frame_size = frame.texture.size();
        let (frame_width, frame_height) = (frame_size.width, frame_size.height);
        xengui::devtools::record_size_note(
            "frame:acquired",
            frame_width,
            frame_height,
            format!("config={}x{}", self.config.width, self.config.height)
        );

        let view = frame.texture.create_view(&Default::default());
        let mut encoder = self.device.create_command_encoder(&Default::default());

        {
            let mut backend = self.pipelines.begin_frame(
                &self.device,
                &self.queue,
                &mut encoder,
                &view,
                frame_width,
                frame_height
            );
            self.frame.render_frame(
                tree,
                &mut backend,
                theme,
                scale_factor,
                frame_width,
                frame_height
            );
        }

        xengui::devtools::record("frame:submit");
        self.queue.submit(Some(encoder.finish()));
        xengui::devtools::record("frame:present");
        self.queue.present(frame);
        xengui::devtools::record("frame:presented");
    }

    pub fn resize(
        &mut self,
        tree: &mut [Box<dyn Widget>],
        theme: SystemTheme,
        scale_factor: f32,
        width: u32,
        height: u32
    ) {
        if width == 0 || height == 0 {
            return;
        }
        if width != self.config.width || height != self.config.height {
            xengui::devtools::record_size("surface:reconfigure", width, height);
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
            self.frame.resize();
        }
        self.render_frame(tree, theme, scale_factor);
    }

    /// Reconfigures the swapchain to `width`/`height` without drawing a
    /// frame. Lets a burst of resize events keep the surface's own size in
    /// sync immediately while the actual (expensive) redraw is deferred to
    /// a single coalesced `RedrawRequested`, so the GPU never has to submit
    /// and present a frame per intermediate resize step.
    pub fn reconfigure_surface(&mut self, width: u32, height: u32) {
        log::info!("reconfigure_surface: {}x{} at {:?}", width, height, web_time::Instant::now());
        if
            width == 0 ||
            height == 0 ||
            (width == self.config.width && height == self.config.height)
        {
            return;
        }
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);
        self.frame.resize();
    }
}
