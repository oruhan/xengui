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
    surface_format: wgpu::TextureFormat,
    /// Resolved, adapter-clamped MSAA sample count every pipeline above
    /// was built with. `1` means MSAA is disabled entirely (either
    /// requested that way, or the adapter didn't support anything higher).
    sample_count: u32,
    /// Owned multisampled color target, `None` when `sample_count == 1`.
    /// Recreated on resize.
    msaa_texture: Option<wgpu::Texture>,
    msaa_view: Option<wgpu::TextureView>,
    // Everything is painted into this offscreen target instead of the
    // swapchain directly, so a backdrop-filter widget can read back
    // already-painted content mid-frame - not possible against a
    // swapchain image on most backends.
    scene_texture: wgpu::Texture,
    scene_view: wgpu::TextureView,
    scene_width: u32,
    scene_height: u32,
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

        // Placeholder 1x1 target; the real size is (re)created lazily by
        // `ensure_scene_target` on the first `begin_frame` call, once the
        // actual surface dimensions are known.
        let scene_texture = device.create_texture(
            &(wgpu::TextureDescriptor {
                label: Some("xengui scene target"),
                size: wgpu::Extent3d { width: 1, height: 1, depth_or_array_layers: 1 },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: surface_format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT |
                wgpu::TextureUsages::TEXTURE_BINDING |
                wgpu::TextureUsages::COPY_SRC,
                view_formats: &[],
            })
        );
        let scene_view = scene_texture.create_view(&Default::default());

        Ok(Self {
            rect: RectPipeline::new(device, surface_format, sample_count),
            triangle: TrianglePipeline::new(device, surface_format, sample_count),
            image: ImagePipeline::new(device, surface_format, sample_count),
            text: TextPipeline::new(device, queue, surface_format, user_fonts, sample_count)?,
            box_shadow: BoxShadowPipeline::new(device, surface_format, sample_count),
            filters: FilterEngine::new(device, surface_format),
            surface_format,
            sample_count,
            msaa_texture: None,
            msaa_view: None,
            scene_texture,
            scene_view,
            scene_width: 0,
            scene_height: 0,
        })
    }

    // (Re)creates the scene target when the surface size changes. `new()`
    // seeds scene_width/height at 0 so the very first begin_frame call
    // always recreates it at the real size.
    fn ensure_scene_target(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        let width = width.max(1);
        let height = height.max(1);
        if self.scene_width == width && self.scene_height == height {
            return;
        }
        let texture = device.create_texture(
            &(wgpu::TextureDescriptor {
                label: Some("xengui scene target"),
                size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: self.surface_format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT |
                wgpu::TextureUsages::TEXTURE_BINDING |
                wgpu::TextureUsages::COPY_SRC,
                view_formats: &[],
            })
        );
        self.scene_view = texture.create_view(&Default::default());
        self.scene_texture = texture;
        self.scene_width = width;
        self.scene_height = height;
    }

    /// Composites the accumulated scene target onto `target` (the real
    /// swapchain view), overwriting it entirely - the final step of every
    /// frame.
    pub fn present_scene(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        width: u32,
        height: u32
    ) {
        self.filters.blit_full(device, queue, encoder, &self.scene_view, target, width, height);
    }

    /// The format every pipeline (including the filter engine's own
    /// offscreen textures) was built against.
    pub fn surface_format(&self) -> wgpu::TextureFormat {
        self.surface_format
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
        width: u32,
        height: u32
    ) -> WgpuFrame<'a> {
        self.ensure_scene_target(device, width, height);
        self.rect.reset_frame();
        self.triangle.reset_frame();
        self.image.reset_frame();
        self.box_shadow.reset_frame();
        self.filters.reset_frame();

        let view = self.scene_view.clone();

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
    view: wgpu::TextureView,
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
                        view: &self.view,
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

    /// Renders `cmds` (already translated into the subtree's own local
    /// coordinate space, i.e. as if the widget's own top-left sat at the
    /// origin) into a freshly cleared `target_view`, batching same-type
    /// draw calls into runs the same way the main frame loop batches
    /// commands against `self.view` - only the destination texture and
    /// its dimensions differ.
    fn paint_subtree_to_offscreen(
        &mut self,
        cmds: &[DrawCommand],
        target_view: &wgpu::TextureView,
        target_width: u32,
        target_height: u32
    ) {
        #[derive(PartialEq, Clone, Copy)]
        enum RunKind {
            Rect,
            Triangle,
            Image,
            Text,
            BoxShadow,
        }

        let mut current_kind: Option<RunKind> = None;
        let mut rect_buf: Vec<RectCommand> = Vec::new();
        let mut tri_buf: Vec<TriangleCommand> = Vec::new();
        let mut img_buf: Vec<ImageCommand> = Vec::new();
        let mut shadow_buf: Vec<BoxShadowCommand> = Vec::new();
        // Whether the offscreen texture has already been cleared, so
        // every following pass loads instead of wiping earlier layers.
        let mut cleared = false;

        macro_rules! shape_pass {
            () => {
        {
                let load = if cleared {
                    wgpu::LoadOp::Load
                } else {
                    cleared = true;
                    wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT)
                };
                self.encoder.begin_render_pass(
                    &(wgpu::RenderPassDescriptor {
                        label: Some("xengui filtered subtree shape pass"),
                        color_attachments: &[
                            Some(wgpu::RenderPassColorAttachment {
                                view: target_view,
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
                )
        }
            };
        }

        macro_rules! flush_run {
            () => {
                match current_kind {
                    Some(RunKind::Rect) => {
                        let mut pass = shape_pass!();
                        self.pipelines.rect.draw_batch(
                            self.device,
                            self.queue,
                            &mut pass,
                            target_width,
                            target_height,
                            &rect_buf
                        );
                    }
                    Some(RunKind::Triangle) => {
                        let mut pass = shape_pass!();
                        self.pipelines.triangle.draw_batch(
                            self.device,
                            self.queue,
                            &mut pass,
                            target_width,
                            target_height,
                            &tri_buf
                        );
                    }
                    Some(RunKind::Image) => {
                        let mut pass = shape_pass!();
                        self.pipelines.image.draw_batch(
                            self.device,
                            self.queue,
                            &mut pass,
                            target_width,
                            target_height,
                            &img_buf
                        );
                    }
                    Some(RunKind::BoxShadow) => {
                        let mut pass = shape_pass!();
                        self.pipelines.box_shadow.draw_batch(
                            self.device,
                            self.queue,
                            &mut pass,
                            target_width,
                            target_height,
                            &shadow_buf
                        );
                    }
                    Some(RunKind::Text) => {
                        if !cleared {
                            let _ = shape_pass!();
                        }
                        if
                            let Err(err) = self.pipelines.text.flush(
                                self.device,
                                self.queue,
                                self.encoder,
                                target_view,
                                target_width,
                                target_height
                            )
                        {
                            log::warn!("xengui-wgpu: filtered subtree text flush failed: {err}");
                        }
                        let decorations = self.pipelines.text.take_decorations();
                        if !decorations.is_empty() {
                            let mut pass = shape_pass!();
                            self.pipelines.rect.draw_batch(
                                self.device,
                                self.queue,
                                &mut pass,
                                target_width,
                                target_height,
                                &decorations
                            );
                        }
                    }
                    None => {}
                }
                rect_buf.clear();
                tri_buf.clear();
                img_buf.clear();
                shadow_buf.clear();
            };
        }

        for command in cmds {
            match command {
                DrawCommand::Text(cmd) => {
                    if current_kind != Some(RunKind::Text) {
                        flush_run!();
                        current_kind = Some(RunKind::Text);
                    }
                    self.pipelines.text.draw(self.scale_factor, SystemTheme::Dark, cmd);
                }
                DrawCommand::Rect(cmd) => {
                    if current_kind != Some(RunKind::Rect) {
                        flush_run!();
                        current_kind = Some(RunKind::Rect);
                    }
                    rect_buf.push(cmd.clone());
                }
                DrawCommand::Triangle(cmd) => {
                    if current_kind != Some(RunKind::Triangle) {
                        flush_run!();
                        current_kind = Some(RunKind::Triangle);
                    }
                    tri_buf.push(cmd.clone());
                }
                DrawCommand::Image(cmd) => {
                    if current_kind != Some(RunKind::Image) {
                        flush_run!();
                        current_kind = Some(RunKind::Image);
                    }
                    img_buf.push((**cmd).clone());
                }
                DrawCommand::BoxShadow(cmd) => {
                    if current_kind != Some(RunKind::BoxShadow) {
                        flush_run!();
                        current_kind = Some(RunKind::BoxShadow);
                    }
                    shadow_buf.push(cmd.clone());
                }
                // paint_subtree_for_filter (xengui core) never records a
                // nested Filtered command - it inlines every descendant's
                // own paint() call directly. This arm only guards against
                // that changing later; it inlines the nested subtree
                // unfiltered rather than silently dropping its content.
                DrawCommand::Filtered(nested) => {
                    flush_run!();
                    current_kind = None;
                    self.paint_subtree_to_offscreen(
                        &nested.commands,
                        target_view,
                        target_width,
                        target_height
                    );
                }
                // An isolated filtered subtree has no "behind" content of
                // its own to snapshot, so a nested backdrop-filter inside
                // it is skipped - its own background/children still paint.
                DrawCommand::BackdropFilter(_) => {}
            }
        }
        flush_run!();

        if !cleared {
            let _ = shape_pass!();
        }

        let _ = cleared;
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
                        view: &self.view,
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
                        view: &self.view,
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
                        view: &self.view,
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
                        view: &self.view,
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
                    &self.view,
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
        chain: &FilterChain,
        bounds: (f32, f32, f32, f32)
    ) {
        let (bx, by, bw, bh) = bounds;
        let width = bw.round().max(1.0) as u32;
        let height = bh.round().max(1.0) as u32;

        let translated: Vec<DrawCommand> = cmds
            .iter()
            .map(|c| translate_draw_command(c, bx, by))
            .collect();

        let format = self.pipelines.surface_format();
        let source_texture = self.device.create_texture(
            &(wgpu::TextureDescriptor {
                label: Some("xengui filtered subtree source"),
                size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT |
                wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            })
        );
        let source_view = source_texture.create_view(&Default::default());

        self.paint_subtree_to_offscreen(&translated, &source_view, width, height);

        let filtered = self.pipelines.filters.apply(
            self.device,
            self.queue,
            self.encoder,
            &source_view,
            width,
            height,
            chain,
            self.scale_factor
        );

        // Clamped to the frame's own top-left: a large blur/drop-shadow
        // padding on a widget sitting right at the edge would otherwise
        // push the destination viewport into negative territory, which
        // wgpu rejects.
        let dest_rect = (
            (bx - filtered.padding).max(0.0),
            (by - filtered.padding).max(0.0),
            filtered.width as f32,
            filtered.height as f32,
        );

        self.pipelines.filters.composite(
            self.device,
            self.queue,
            self.encoder,
            &filtered.view,
            &self.view,
            dest_rect,
            None,
            self.width,
            self.height,
            (0.0, 0.0, 1.0, 1.0)
        );
    }

    fn draw_backdrop_filtered(
        &mut self,
        chain: &FilterChain,
        bounds: (f32, f32, f32, f32),
        clip_rect: Option<(f32, f32, f32, f32)>
    ) {
        let (bx, by, bw, bh) = bounds;

        // Clamps the capture rect to the scene's own bounds and to any
        // ancestor clip, so a widget straddling the edge (or scrolled
        // partly out of view) never asks the GPU to copy outside the
        // texture.
        let (cx, cy, cw, ch) = clip_rect.unwrap_or((
            0.0,
            0.0,
            self.width as f32,
            self.height as f32,
        ));
        let left = bx.max(cx).max(0.0);
        let top = by.max(cy).max(0.0);
        let right = (bx + bw).min(cx + cw).min(self.width as f32);
        let bottom = (by + bh).min(cy + ch).min(self.height as f32);

        let src_x = left.round() as u32;
        let src_y = top.round() as u32;
        let src_w = (right - left).round().max(0.0) as u32;
        let src_h = (bottom - top).round().max(0.0) as u32;

        if src_w == 0 || src_h == 0 {
            return;
        }

        // Live snapshot of the scene as painted so far, taken into its own
        // texture since the scene target can't be bound as a shader
        // resource while it's still the active render target.
        let snapshot = self.device.create_texture(
            &(wgpu::TextureDescriptor {
                label: Some("xengui backdrop snapshot"),
                size: wgpu::Extent3d { width: src_w, height: src_h, depth_or_array_layers: 1 },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: self.pipelines.surface_format(),
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            })
        );

        self.encoder.copy_texture_to_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &self.pipelines.scene_texture,
                mip_level: 0,
                origin: wgpu::Origin3d { x: src_x, y: src_y, z: 0 },
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyTextureInfo {
                texture: &snapshot,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::Extent3d { width: src_w, height: src_h, depth_or_array_layers: 1 }
        );

        let snapshot_view = snapshot.create_view(&Default::default());

        let filtered = self.pipelines.filters.apply(
            self.device,
            self.queue,
            self.encoder,
            &snapshot_view,
            src_w,
            src_h,
            chain,
            self.scale_factor
        );

        // Backdrop-filter must stay exactly clipped to the widget's own
        // box - unlike a foreground filter, blurred backdrop content must
        // never bleed beyond the element it belongs to (matching CSS
        // `backdrop-filter` semantics). The padding FilterEngine::apply
        // added around the source exists only so the blur convolution has
        // enough neighboring pixels to sample correctly at the edges; the
        // unpadded source always sits centered at (pad, pad) within the
        // filtered texture regardless of where the widget sits on screen,
        // so cropping exactly that inner (src_w x src_h) region back out
        // and compositing it at the widget's real position/size recovers
        // the correct result without any screen-edge-relative math.
        let pad = filtered.padding;
        let filtered_w = filtered.width as f32;
        let filtered_h = filtered.height as f32;

        let dest_rect = (src_x as f32, src_y as f32, src_w as f32, src_h as f32);
        let source_uv_rect = (
            pad / filtered_w.max(1.0),
            pad / filtered_h.max(1.0),
            (src_w as f32) / filtered_w.max(1.0),
            (src_h as f32) / filtered_h.max(1.0),
        );

        self.pipelines.filters.composite(
            self.device,
            self.queue,
            self.encoder,
            &filtered.view,
            &self.view,
            dest_rect,
            clip_rect,
            self.width,
            self.height,
            source_uv_rect
        );
    }

    fn resize(&mut self, _width: u32, _height: u32) {}
}

/// Shifts a draw command by `(-ox, -oy)`, converting it from the main
/// frame's absolute paint coordinates into a filtered subtree's own local
/// space, where the widget's own top-left lands at the texture origin.
fn translate_draw_command(command: &DrawCommand, ox: f32, oy: f32) -> DrawCommand {
    let shift_clip = |clip: Option<(f32, f32, f32, f32)>| {
        clip.map(|(x, y, w, h)| (x - ox, y - oy, w, h))
    };

    match command {
        DrawCommand::Rect(c) => {
            let mut c = c.clone();
            c.position.0 -= ox;
            c.position.1 -= oy;
            c.clip_rect = shift_clip(c.clip_rect);
            DrawCommand::Rect(c)
        }
        DrawCommand::Triangle(c) => {
            let mut c = c.clone();
            c.p0.0 -= ox;
            c.p0.1 -= oy;
            c.p1.0 -= ox;
            c.p1.1 -= oy;
            c.p2.0 -= ox;
            c.p2.1 -= oy;
            c.clip_rect = shift_clip(c.clip_rect);
            DrawCommand::Triangle(c)
        }
        DrawCommand::Text(c) => {
            let mut c = c.clone();
            c.position.0 -= ox;
            c.position.1 -= oy;
            c.clip_rect = shift_clip(c.clip_rect);
            DrawCommand::Text(c)
        }
        DrawCommand::Image(c) => {
            let mut c = c.clone();
            c.position.0 -= ox;
            c.position.1 -= oy;
            c.clip_rect = shift_clip(c.clip_rect);
            DrawCommand::Image(c)
        }
        DrawCommand::BoxShadow(c) => {
            let mut c = c.clone();
            c.shadow_position.0 -= ox;
            c.shadow_position.1 -= oy;
            c.box_position.0 -= ox;
            c.box_position.1 -= oy;
            c.clip_rect = shift_clip(c.clip_rect);
            DrawCommand::BoxShadow(c)
        }
        DrawCommand::Filtered(nested) => {
            let mut nested = nested.clone();
            nested.bounds.0 -= ox;
            nested.bounds.1 -= oy;
            nested.clip_rect = shift_clip(nested.clip_rect);
            nested.commands = nested.commands
                .iter()
                .map(|c| translate_draw_command(c, ox, oy))
                .collect();
            DrawCommand::Filtered(nested)
        }
        DrawCommand::BackdropFilter(cmd) => {
            let mut cmd = cmd.clone();
            cmd.bounds.0 -= ox;
            cmd.bounds.1 -= oy;
            cmd.clip_rect = shift_clip(cmd.clip_rect);
            DrawCommand::BackdropFilter(cmd)
        }
    }
}
