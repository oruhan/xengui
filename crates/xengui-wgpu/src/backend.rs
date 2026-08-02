// SPDX-License-Identifier: Apache-2.0
use crate::pipelines::{
    BoxShadowPipeline,
    ImagePipeline,
    RectPipeline,
    TextPipeline,
    TrianglePipeline,
    FilterEngine,
};
use xengui::{
    BoxShadowCommand,
    Color,
    DrawCommand,
    FilterChain,
    ImageCommand,
    RectCommand,
    RenderBackend,
    SystemTheme,
    TextCommand,
    TextMeasurer,
    TriangleCommand,
};

/// Owns the four wgpu render pipelines xengui needs, built once against a
/// device and reused across every frame via `begin_frame`.
pub struct WgpuPipelines {
    pub(crate) rect: RectPipeline,
    triangle: TrianglePipeline,
    image: ImagePipeline,
    text: TextPipeline,
    pub(crate) box_shadow: BoxShadowPipeline,
    filters: FilterEngine,
    /// Resolved, adapter-clamped MSAA sample count every pipeline above
    /// was built with. `1` means MSAA is disabled entirely (either
    /// requested that way, or the adapter didn't support anything higher).
    sample_count: u32,
    /// Owned multisampled color target, `None` when `sample_count == 1`.
    /// Recreated on resize.
    msaa_texture: Option<wgpu::Texture>,
    msaa_view: Option<wgpu::TextureView>,
}

impl WgpuPipelines {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        adapter: &wgpu::Adapter,
        surface_format: wgpu::TextureFormat,
        user_fonts: Vec<(String, Vec<u8>)>,
        requested_samples: crate::SampleCount
    ) -> Result<Self, String> {
        let sample_count = requested_samples.clamp_to_adapter(adapter, surface_format).as_u32();

        Ok(Self {
            rect: RectPipeline::new(device, surface_format, sample_count),
            triangle: TrianglePipeline::new(device, surface_format, sample_count),
            image: ImagePipeline::new(device, surface_format, sample_count),
            text: TextPipeline::new(device, queue, surface_format, user_fonts, sample_count)?,
            box_shadow: BoxShadowPipeline::new(device, surface_format, sample_count),
            filters: FilterEngine::new(device, surface_format),
            sample_count,
            msaa_texture: None,
            msaa_view: None,
        })
    }

    /// (Re)allocates the MSAA color target for the given surface size.
    /// A no-op when MSAA is disabled (`sample_count == 1`).
    pub fn resize_msaa(
        &mut self,
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        width: u32,
        height: u32
    ) {
        if self.sample_count <= 1 {
            self.msaa_texture = None;
            self.msaa_view = None;
            return;
        }
        let texture = device.create_texture(
            &(wgpu::TextureDescriptor {
                label: Some("xengui msaa color target"),
                size: wgpu::Extent3d {
                    width: width.max(1),
                    height: height.max(1),
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: self.sample_count,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            })
        );
        self.msaa_view = Some(texture.create_view(&Default::default()));
        self.msaa_texture = Some(texture);
    }

    pub fn sample_count(&self) -> u32 {
        self.sample_count
    }

    pub fn begin_frame<'a>(
        &'a mut self,
        device: &'a wgpu::Device,
        queue: &'a wgpu::Queue,
        encoder: &'a mut wgpu::CommandEncoder,
        view: &'a wgpu::TextureView,
        width: u32,
        height: u32
    ) -> WgpuFrame<'a> {
        self.rect.reset_frame();
        self.triangle.reset_frame();
        self.image.reset_frame();
        self.box_shadow.reset_frame();

        WgpuFrame {
            pipelines: self,
            device,
            queue,
            encoder,
            view,
            width,
            height,
            background: Color::TRANSPARENT,
            scale_factor: 1.0,
            shape_pass_open: false,
            text_cmds: Vec::new(),
        }
    }
}

/// A single frame's worth of borrowed GPU resources; implements
/// `RenderBackend` so `xengui::FrameRenderer` can draw into it without
/// knowing anything about wgpu.
pub struct WgpuFrame<'a> {
    pipelines: &'a mut WgpuPipelines,
    device: &'a wgpu::Device,
    queue: &'a wgpu::Queue,
    encoder: &'a mut wgpu::CommandEncoder,
    view: &'a wgpu::TextureView,
    width: u32,
    height: u32,
    background: Color,
    scale_factor: f32,
    shape_pass_open: bool,
    // Kept only to redraw glyph buffers if `flush_text` needs a retry
    // after a text-atlas resize.
    text_cmds: Vec<(SystemTheme, TextCommand)>,
}

impl<'a> WgpuFrame<'a> {
    // Every frame always starts from the background color, regardless of
    // what gets painted - otherwise a frame with zero rect/triangle/image
    // commands leaves the swapchain's previous (differently-sized) content
    // on screen.
    fn clear_frame(&mut self) {
        let bg = self.background;
        let _ = self.encoder.begin_render_pass(
            &(wgpu::RenderPassDescriptor {
                label: Some("xengui clear pass"),
                color_attachments: &[
                    Some(wgpu::RenderPassColorAttachment {
                        view: self.view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: bg.r() as f64,
                                g: bg.g() as f64,
                                b: bg.b() as f64,
                                a: bg.a() as f64,
                            }),
                            store: wgpu::StoreOp::Store,
                        },
                        depth_slice: None,
                    }),
                ],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            })
        );
        self.shape_pass_open = true;
    }

    fn shape_pass_load(&mut self) -> wgpu::LoadOp<wgpu::Color> {
        if self.shape_pass_open {
            return wgpu::LoadOp::Load;
        }
        self.shape_pass_open = true;
        let bg = self.background;
        wgpu::LoadOp::Clear(wgpu::Color {
            r: bg.r() as f64,
            g: bg.g() as f64,
            b: bg.b() as f64,
            a: bg.a() as f64,
        })
    }

    /// Marks this frame as already having visible content in `view` (e.g.
    /// a window-chrome shadow drawn before this frame started), so the
    /// first shape draw call loads instead of clearing it away.
    pub fn preserve_existing_content(&mut self) {
        self.shape_pass_open = true;
    }
}

impl<'a> RenderBackend for WgpuFrame<'a> {
    fn text_measurer(&mut self) -> &mut dyn TextMeasurer {
        &mut self.pipelines.text
    }

    fn begin_frame(&mut self, background: Color, _width: u32, _height: u32) -> bool {
        self.background = background;
        if !self.shape_pass_open {
            self.clear_frame();
        }
        true
    }

    fn draw_rects(&mut self, cmds: &[RectCommand]) {
        if cmds.is_empty() {
            return;
        }
        let load = self.shape_pass_load();
        let mut pass = self.encoder.begin_render_pass(
            &(wgpu::RenderPassDescriptor {
                label: Some("xengui shape pass"),
                color_attachments: &[
                    Some(wgpu::RenderPassColorAttachment {
                        view: self.view,
                        resolve_target: None,
                        ops: wgpu::Operations { load, store: wgpu::StoreOp::Store },
                        depth_slice: None,
                    }),
                ],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            })
        );
        self.pipelines.rect.draw_batch(
            self.device,
            self.queue,
            &mut pass,
            self.width,
            self.height,
            cmds
        );
    }

    fn draw_triangles(&mut self, cmds: &[TriangleCommand]) {
        if cmds.is_empty() {
            return;
        }
        let load = self.shape_pass_load();
        let mut pass = self.encoder.begin_render_pass(
            &(wgpu::RenderPassDescriptor {
                label: Some("xengui shape pass"),
                color_attachments: &[
                    Some(wgpu::RenderPassColorAttachment {
                        view: self.view,
                        resolve_target: None,
                        ops: wgpu::Operations { load, store: wgpu::StoreOp::Store },
                        depth_slice: None,
                    }),
                ],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            })
        );
        self.pipelines.triangle.draw_batch(
            self.device,
            self.queue,
            &mut pass,
            self.width,
            self.height,
            cmds
        );
    }

    fn draw_images(&mut self, cmds: &[ImageCommand]) {
        if cmds.is_empty() {
            return;
        }
        let load = self.shape_pass_load();
        let mut pass = self.encoder.begin_render_pass(
            &(wgpu::RenderPassDescriptor {
                label: Some("xengui shape pass"),
                color_attachments: &[
                    Some(wgpu::RenderPassColorAttachment {
                        view: self.view,
                        resolve_target: None,
                        ops: wgpu::Operations { load, store: wgpu::StoreOp::Store },
                        depth_slice: None,
                    }),
                ],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            })
        );
        self.pipelines.image.draw_batch(
            self.device,
            self.queue,
            &mut pass,
            self.width,
            self.height,
            cmds
        );
    }

    fn draw_box_shadows(&mut self, cmds: &[BoxShadowCommand]) {
        if cmds.is_empty() {
            return;
        }
        let load = self.shape_pass_load();
        let mut pass = self.encoder.begin_render_pass(
            &(wgpu::RenderPassDescriptor {
                label: Some("xengui shape pass"),
                color_attachments: &[
                    Some(wgpu::RenderPassColorAttachment {
                        view: self.view,
                        resolve_target: None,
                        ops: wgpu::Operations { load, store: wgpu::StoreOp::Store },
                        depth_slice: None,
                    }),
                ],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            })
        );
        self.pipelines.box_shadow.draw_batch(
            self.device,
            self.queue,
            &mut pass,
            self.width,
            self.height,
            cmds
        );
    }

    fn draw_text(&mut self, theme: SystemTheme, scale_factor: f32, cmd: &TextCommand) {
        self.scale_factor = scale_factor;
        self.pipelines.text.draw(scale_factor, theme, cmd);
        self.text_cmds.push((theme, cmd.clone()));
    }

    fn take_text_decorations(&mut self) -> Vec<RectCommand> {
        self.pipelines.text.take_decorations()
    }

    fn flush_text(&mut self) {
        const MAX_RETRIES: u32 = 3;
        let mut attempts = 0;
        loop {
            match
                self.pipelines.text.flush(
                    self.device,
                    self.queue,
                    self.encoder,
                    self.view,
                    self.width,
                    self.height
                )
            {
                Ok(()) => {
                    break;
                }
                Err(e) if attempts < MAX_RETRIES => {
                    attempts += 1;
                    log::warn!("text cache resize, retrying flush ({attempts}/{MAX_RETRIES}): {e}");
                    for (theme, cmd) in &self.text_cmds {
                        self.pipelines.text.draw(self.scale_factor, *theme, cmd);
                    }
                }
                Err(e) => {
                    log::error!("text drawing failed permanently, skipping frame: {e}");
                    break;
                }
            }
        }
        self.text_cmds.clear();

        // glyphon's TextRenderer reuses ONE internal vertex buffer across
        // prepare() calls (overwrite, not append) — unlike our own shape
        // pipelines. flush_text() now runs once per text run instead of once
        // per frame, so a later run's prepare() would clobber an earlier run's
        // glyph data before the GPU ever executes that earlier run's render()
        // call, since both were recorded into the same not-yet-submitted
        // encoder. Submitting right here — and swapping in a fresh encoder for
        // whatever comes next — forces this run to actually execute before its
        // buffer can be reused by the next one.
        let finished = std::mem::replace(
            &mut *self.encoder,
            self.device.create_command_encoder(
                &(wgpu::CommandEncoderDescriptor {
                    label: Some("xengui frame encoder (continued)"),
                })
            )
        );
        self.queue.submit(Some(finished.finish()));
    }

    fn end_frame(&mut self) {
        self.pipelines.text.trim_atlas();
    }

    fn draw_filtered(
        &mut self,
        cmds: &[DrawCommand],
        _chain: &FilterChain,
        _bounds: (f32, f32, f32, f32)
    ) {
        // Unfiltered fallback, explicitly allowed by RenderBackend::draw_filtered's
        // own contract: paints the subtree directly instead of running the GPU
        // filter chain, so a filtered widget still renders rather than vanishing.
        let mut rect_buf = Vec::new();
        let mut tri_buf = Vec::new();
        let mut img_buf = Vec::new();
        let mut shadow_buf = Vec::new();

        for cmd in cmds {
            match cmd {
                DrawCommand::Rect(c) => rect_buf.push(c.clone()),
                DrawCommand::Triangle(c) => tri_buf.push(c.clone()),
                DrawCommand::Image(c) => img_buf.push((**c).clone()),
                DrawCommand::BoxShadow(c) => shadow_buf.push(c.clone()),
                DrawCommand::Text(c) => {
                    let scale_factor = self.scale_factor;
                    self.draw_text(SystemTheme::Dark, scale_factor, c);
                }
                DrawCommand::Filtered(nested) => {
                    self.draw_filtered(&nested.commands, &nested.chain, nested.bounds);
                }
            }
        }

        if !rect_buf.is_empty() {
            self.draw_rects(&rect_buf);
        }
        if !tri_buf.is_empty() {
            self.draw_triangles(&tri_buf);
        }
        if !img_buf.is_empty() {
            self.draw_images(&img_buf);
        }
        if !shadow_buf.is_empty() {
            self.draw_box_shadows(&shadow_buf);
        }
        self.flush_text();
    }

    fn resize(&mut self, _width: u32, _height: u32) {}
}
