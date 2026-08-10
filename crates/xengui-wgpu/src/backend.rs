// SPDX-License-Identifier: Apache-2.0
use crate::pipelines::{
    ImagePipeline,
    PostProcessEngine,
    RectPipeline,
    StrokePipeline,
    TextPipeline,
    TrianglePipeline,
};
use crate::pipelines::postprocess::{ BlitPass, directional_shadow_padding, padding_for_chain };
use xengui::{
    BoxShadowCommand,
    Color,
    DrawCommand,
    FilterChain,
    ImageCommand,
    RectCommand,
    RenderBackend,
    StrokeCommand,
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
    triangle_offscreen: TrianglePipeline,
    stroke: StrokePipeline,
    image: ImagePipeline,
    text: TextPipeline,
    postprocess: PostProcessEngine,
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
    // MSAA
    triangle_sample_count: u32,
    triangle_msaa_seed: BlitPass,
    triangle_msaa_texture: Option<wgpu::Texture>,
    triangle_msaa_view: Option<wgpu::TextureView>,
    triangle_msaa_width: u32,
    triangle_msaa_height: u32,
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

        // Tessellated SVG triangles (icons, checkmarks) have no analytic AA
        // of their own the way the rect/image SDF pipelines do, so they get
        // a dedicated MSAA target independent of the rest of the scene.
        let triangle_sample_count = crate::SampleCount::X4
            .clamp_to_adapter(adapter, surface_format)
            .as_u32();

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
            triangle: TrianglePipeline::new(device, surface_format, triangle_sample_count),
            triangle_offscreen: TrianglePipeline::new(device, surface_format, 1),
            stroke: StrokePipeline::new(device, surface_format, sample_count),
            image: ImagePipeline::new(device, surface_format, sample_count),
            text: TextPipeline::new(device, queue, surface_format, user_fonts, sample_count)?,
            postprocess: PostProcessEngine::new(device, surface_format),
            surface_format,
            sample_count,
            msaa_texture: None,
            msaa_view: None,
            triangle_sample_count,
            triangle_msaa_seed: BlitPass::new(device, surface_format, triangle_sample_count),
            triangle_msaa_texture: None,
            triangle_msaa_view: None,
            triangle_msaa_width: 0,
            triangle_msaa_height: 0,
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

    fn ensure_triangle_msaa_target(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        if self.triangle_sample_count <= 1 {
            return;
        }
        let width = width.max(1);
        let height = height.max(1);
        if self.triangle_msaa_width == width && self.triangle_msaa_height == height {
            return;
        }
        let texture = device.create_texture(
            &(wgpu::TextureDescriptor {
                label: Some("xengui triangle msaa target"),
                size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
                mip_level_count: 1,
                sample_count: self.triangle_sample_count,
                dimension: wgpu::TextureDimension::D2,
                format: self.surface_format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            })
        );
        self.triangle_msaa_view = Some(texture.create_view(&Default::default()));
        self.triangle_msaa_texture = Some(texture);
        self.triangle_msaa_width = width;
        self.triangle_msaa_height = height;
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
        self.postprocess.blit_full(device, queue, encoder, &self.scene_view, target, width, height);
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
        height: u32,
        scale_factor: f32
    ) -> WgpuFrame<'a> {
        log::trace!("WgpuPipelines::begin_frame size={width}x{height} scale_factor={scale_factor}");

        self.ensure_scene_target(device, width, height);
        self.ensure_triangle_msaa_target(device, width, height);
        self.rect.reset_frame();
        self.triangle.reset_frame();
        self.triangle_offscreen.reset_frame();
        self.stroke.reset_frame();
        self.image.reset_frame();
        self.postprocess.reset_frame();

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
            scale_factor,
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
            Stroke,
        }

        let mut current_kind: Option<RunKind> = None;
        let mut rect_buf: Vec<RectCommand> = Vec::new();
        let mut tri_buf: Vec<TriangleCommand> = Vec::new();
        let mut img_buf: Vec<ImageCommand> = Vec::new();
        let mut shadow_buf: Vec<BoxShadowCommand> = Vec::new();
        let mut stroke_buf: Vec<StrokeCommand> = Vec::new();
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
                        self.pipelines.triangle_offscreen.draw_batch(
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
                    Some(RunKind::Stroke) => {
                        let mut pass = shape_pass!();
                        self.pipelines.stroke.draw_batch(
                            self.device,
                            self.queue,
                            &mut pass,
                            target_width,
                            target_height,
                            &stroke_buf
                        );
                    }
                    Some(RunKind::BoxShadow) => {
                        if !cleared {
                            let _ = shape_pass!();
                        }
                        self.pipelines.postprocess.draw_box_shadows(
                            self.device,
                            self.queue,
                            self.encoder,
                            target_view,
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
                stroke_buf.clear();
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
                DrawCommand::Stroke(cmd) => {
                    if current_kind != Some(RunKind::Stroke) {
                        flush_run!();
                        current_kind = Some(RunKind::Stroke);
                    }
                    stroke_buf.push(cmd.clone());
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

        let Some(msaa_view) = self.pipelines.triangle_msaa_view.clone() else {
            // Adapter has no MSAA support for this format - falls back
            // to the plain single-sample path instead of failing.
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
            return;
        };

        if !self.shape_pass_open {
            self.clear_frame();
        }

        // The MSAA resolve at the end of this pass overwrites the whole
        // target, so every sample first needs to see what's already
        // painted in the scene, not just the pixels the triangles
        // themselves cover.
        self.pipelines.triangle_msaa_seed.run(
            self.device,
            self.queue,
            self.encoder,
            &self.view,
            &msaa_view,
            self.width,
            self.height,
            (0.0, 0.0),
            (1.0, 1.0),
            None
        );

        let mut pass = self.encoder.begin_render_pass(
            &(wgpu::RenderPassDescriptor {
                label: Some("xengui triangle msaa pass"),
                color_attachments: &[
                    Some(wgpu::RenderPassColorAttachment {
                        view: &msaa_view,
                        resolve_target: Some(&self.view),
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
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
        self.pipelines.triangle.draw_batch(
            self.device,
            self.queue,
            &mut pass,
            self.width,
            self.height,
            cmds
        );
    }

    fn draw_strokes(&mut self, cmds: &[StrokeCommand]) {
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
        self.pipelines.stroke.draw_batch(
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
        // Composited through its own mask/blur/composite passes instead
        // of the shared shape pass every other draw_* call batches into.
        if !self.shape_pass_open {
            self.clear_frame();
        }
        let view = self.view.clone();
        self.pipelines.postprocess.draw_box_shadows(
            self.device,
            self.queue,
            self.encoder,
            &view,
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
        let (pad_left, pad_top, pad_right, pad_bottom) = box_shadow_overflow(cmds, bounds);
        let cap_x = bx - pad_left;
        let cap_y = by - pad_top;
        let cap_w = bw + pad_left + pad_right;
        let cap_h = bh + pad_top + pad_bottom;

        let width = cap_w.round().max(1.0) as u32;
        let height = cap_h.round().max(1.0) as u32;

        log::trace!(
            "draw_filtered bounds={bounds:?} shadow_overflow=({pad_left},{pad_top},{pad_right},{pad_bottom}) scale_factor={} size={width}x{height}",
            self.scale_factor
        );

        let translated: Vec<DrawCommand> = cmds
            .iter()
            .map(|c| translate_draw_command(c, cap_x, cap_y))
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

        let filtered = self.pipelines.postprocess.apply(
            self.device,
            self.queue,
            self.encoder,
            &source_view,
            width,
            height,
            chain,
            self.scale_factor
        );

        let dest_rect = (
            (cap_x - filtered.padding).max(0.0),
            (cap_y - filtered.padding).max(0.0),
            filtered.width as f32,
            filtered.height as f32,
        );

        log::trace!("draw_filtered dest_rect={dest_rect:?} filtered_padding={}", filtered.padding);

        self.pipelines.postprocess.composite(
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
        let padding_px = padding_for_chain(chain, self.scale_factor);
        let screen_w = self.width as f32;
        let screen_h = self.height as f32;

        let Some((cap_x, cap_y, cap_w, cap_h, left_pad, top_pad, right_pad, bottom_pad)) =
            backdrop_capture_rect(bounds, clip_rect, padding_px, screen_w, screen_h) else {
            log::trace!(
                "draw_backdrop_filtered: empty capture rect, skipping bounds={bounds:?} clip={clip_rect:?}"
            );
            return;
        };

        // The widget's own visible rect, recovered from the capture rect
        // and whatever padding actually survived clamping - this (not
        // the padded capture) is where the filtered result gets
        // composited back, keeping the effect confined to the widget.
        let dst_x = (cap_x as f32) + left_pad;
        let dst_y = (cap_y as f32) + top_pad;
        let dst_w = (cap_w as f32) - left_pad - right_pad;
        let dst_h = (cap_h as f32) - top_pad - bottom_pad;

        log::trace!(
            "draw_backdrop_filtered bounds={bounds:?} clip={clip_rect:?} scale_factor={} \
             padding_px={padding_px} capture=({cap_x},{cap_y},{cap_w},{cap_h}) \
             pad(l,t,r,b)=({left_pad},{top_pad},{right_pad},{bottom_pad}) dst=({dst_x},{dst_y},{dst_w},{dst_h})",
            self.scale_factor
        );

        if dst_w <= 0.0 || dst_h <= 0.0 {
            return;
        }

        // Live snapshot of the padded capture area (not just the widget's
        // own box), so blur has real surrounding scene content to sample
        // instead of fading into synthetic transparency at the edges.
        let snapshot = self.device.create_texture(
            &(wgpu::TextureDescriptor {
                label: Some("xengui backdrop snapshot"),
                size: wgpu::Extent3d { width: cap_w, height: cap_h, depth_or_array_layers: 1 },
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
                origin: wgpu::Origin3d { x: cap_x, y: cap_y, z: 0 },
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyTextureInfo {
                texture: &snapshot,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::Extent3d { width: cap_w, height: cap_h, depth_or_array_layers: 1 }
        );

        let snapshot_view = snapshot.create_view(&Default::default());

        let filtered = self.pipelines.postprocess.apply_prepadded(
            self.device,
            self.queue,
            self.encoder,
            &snapshot_view,
            cap_w,
            cap_h,
            chain,
            self.scale_factor
        );

        let dest_rect = (dst_x, dst_y, dst_w, dst_h);
        let source_uv_rect = backdrop_crop_uv_rect(
            left_pad,
            top_pad,
            dst_w,
            dst_h,
            cap_w as f32,
            cap_h as f32
        );

        log::trace!(
            "draw_backdrop_filtered dest_rect={dest_rect:?} source_uv_rect={source_uv_rect:?}"
        );

        self.pipelines.postprocess.composite(
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

fn box_shadow_overflow(cmds: &[DrawCommand], bounds: (f32, f32, f32, f32)) -> (f32, f32, f32, f32) {
    let (bx, by, bw, bh) = bounds;
    let mut overflow = (0.0f32, 0.0f32, 0.0f32, 0.0f32); // left, top, right, bottom

    fn visit(
        cmds: &[DrawCommand],
        bx: f32,
        by: f32,
        bw: f32,
        bh: f32,
        overflow: &mut (f32, f32, f32, f32)
    ) {
        for cmd in cmds {
            match cmd {
                DrawCommand::BoxShadow(c) if !c.inset => {
                    let full = c.blur * 3.0 + 4.0;
                    let (pl, pt, pr, pb) = directional_shadow_padding(c.direction, full);
                    let sx0 = c.shadow_position.0 - pl;
                    let sy0 = c.shadow_position.1 - pt;
                    let sx1 = c.shadow_position.0 + c.shadow_size.0 + pr;
                    let sy1 = c.shadow_position.1 + c.shadow_size.1 + pb;
                    overflow.0 = overflow.0.max(bx - sx0);
                    overflow.1 = overflow.1.max(by - sy0);
                    overflow.2 = overflow.2.max(sx1 - (bx + bw));
                    overflow.3 = overflow.3.max(sy1 - (by + bh));
                }
                DrawCommand::Filtered(nested) => {
                    visit(&nested.commands, bx, by, bw, bh, overflow);
                }
                _ => {}
            }
        }
    }

    visit(cmds, bx, by, bw, bh, &mut overflow);
    (overflow.0.max(0.0), overflow.1.max(0.0), overflow.2.max(0.0), overflow.3.max(0.0))
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
        DrawCommand::Stroke(c) => {
            let mut c = c.clone();
            c.p0.0 -= ox;
            c.p0.1 -= oy;
            c.p1.0 -= ox;
            c.p1.1 -= oy;
            c.clip_rect = shift_clip(c.clip_rect);
            DrawCommand::Stroke(c)
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

// Computes the physical-pixel rect to snapshot for a backdrop-filter
// pass: `bounds` (intersected with `clip_rect` first, i.e. the widget's
// real visible box) expanded by `padding_px` on every side, then clamped
// to the `0..screen_w x 0..screen_h` surface - there's no real scene
// content to sample past the screen's own edges either way. Also returns
// how much padding actually survived clamping on each edge (left, top,
// right, bottom), since a widget flush against the clip/screen edge
// won't have the full padding available on that side.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn backdrop_capture_rect(
    bounds: (f32, f32, f32, f32),
    clip_rect: Option<(f32, f32, f32, f32)>,
    padding_px: f32,
    screen_w: f32,
    screen_h: f32
) -> Option<(u32, u32, u32, u32, f32, f32, f32, f32)> {
    let (bx, by, bw, bh) = bounds;
    let (cx, cy, cw, ch) = clip_rect.unwrap_or((0.0, 0.0, screen_w, screen_h));

    let bound_left = bx.max(cx).max(0.0);
    let bound_top = by.max(cy).max(0.0);
    let bound_right = (bx + bw).min(cx + cw).min(screen_w);
    let bound_bottom = (by + bh).min(cy + ch).min(screen_h);

    if bound_right <= bound_left || bound_bottom <= bound_top {
        return None;
    }

    let cap_left = (bound_left - padding_px).max(0.0);
    let cap_top = (bound_top - padding_px).max(0.0);
    let cap_right = (bound_right + padding_px).min(screen_w);
    let cap_bottom = (bound_bottom + padding_px).min(screen_h);

    if cap_right <= cap_left || cap_bottom <= cap_top {
        return None;
    }

    // Edges are rounded to whole texels first, and every derived value
    // (capture size, padding) comes from those same rounded edges instead
    // of mixing rounded and unrounded numbers - otherwise dst_w/dst_h in
    // the caller drift by up to a texel from the widget's real size
    // whenever sub-pixel scroll offsets nudge rounding across a boundary,
    // which reads as the blurred content behind a scrolling sticky
    // element subtly growing/shrinking frame to frame.
    let cap_left_px = cap_left.round();
    let cap_top_px = cap_top.round();
    let cap_right_px = cap_right.round();
    let cap_bottom_px = cap_bottom.round();

    let cap_w = cap_right_px - cap_left_px;
    let cap_h = cap_bottom_px - cap_top_px;
    if cap_w <= 0.0 || cap_h <= 0.0 {
        return None;
    }

    let left_pad = bound_left - cap_left_px;
    let top_pad = bound_top - cap_top_px;
    let right_pad = cap_right_px - bound_right;
    let bottom_pad = cap_bottom_px - bound_bottom;

    Some((
        cap_left_px as u32,
        cap_top_px as u32,
        cap_w as u32,
        cap_h as u32,
        left_pad,
        top_pad,
        right_pad,
        bottom_pad,
    ))
}

fn backdrop_crop_uv_rect(
    left_pad: f32,
    top_pad: f32,
    dst_w: f32,
    dst_h: f32,
    cap_w: f32,
    cap_h: f32
) -> (f32, f32, f32, f32) {
    (
        left_pad / cap_w.max(1.0),
        top_pad / cap_h.max(1.0),
        dst_w / cap_w.max(1.0),
        dst_h / cap_h.max(1.0),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capture_rect_pads_evenly_when_far_from_every_edge() {
        let bounds = (100.0, 100.0, 200.0, 50.0);
        let (cx, cy, cw, ch, l, t, r, b) = backdrop_capture_rect(
            bounds,
            None,
            16.0,
            1000.0,
            1000.0
        ).expect("capture rect should exist");
        assert_eq!((cx, cy, cw, ch), (84, 84, 232, 82));
        assert_eq!((l, t, r, b), (16.0, 16.0, 16.0, 16.0));
    }

    #[test]
    fn capture_rect_clamps_padding_against_screen_top() {
        // A sticky header pinned at y=0 has no real content above it to
        // pad into - the top padding must be clamped to whatever room is
        // actually available (zero here), not silently assumed.
        let bounds = (50.0, 0.0, 500.0, 55.0);
        let (_cx, cy, cw, ch, l, t, r, _b) = backdrop_capture_rect(
            bounds,
            None,
            16.0,
            1000.0,
            1000.0
        ).expect("capture rect should exist");
        assert_eq!(cy, 0);
        assert_eq!(t, 0.0);
        assert_eq!(ch, 55 + 16);
        assert_eq!(l, 16.0);
        assert_eq!(r, 16.0);
        assert_eq!(cw, 500 + 16 + 16);
    }

    #[test]
    fn capture_rect_shrinks_to_ancestor_clip_before_padding() {
        let bounds = (50.0, 0.0, 400.0, 55.0);
        let clip = Some((0.0, 0.0, 500.0, 40.0));
        let (_cx, cy, _cw, ch, _l, t, _r, b) = backdrop_capture_rect(
            bounds,
            clip,
            16.0,
            1000.0,
            1000.0
        ).expect("capture rect should exist");
        assert_eq!(cy, 0);
        assert_eq!(t, 0.0);
        assert_eq!(ch, 40 + 16);
        assert_eq!(b, 16.0);
    }

    #[test]
    fn capture_rect_is_none_when_fully_clipped_away() {
        let bounds = (0.0, 0.0, 500.0, 55.0);
        let clip = Some((0.0, 200.0, 500.0, 40.0));
        assert!(backdrop_capture_rect(bounds, clip, 16.0, 1000.0, 1000.0).is_none());
    }

    #[test]
    fn crop_uv_rect_recovers_the_original_destination_size() {
        let uv = backdrop_crop_uv_rect(16.0, 16.0, 200.0, 50.0, 232.0, 82.0);
        assert!((uv.0 - 16.0 / 232.0).abs() < 1e-5);
        assert!((uv.1 - 16.0 / 82.0).abs() < 1e-5);
        assert!((uv.2 - 200.0 / 232.0).abs() < 1e-5);
        assert!((uv.3 - 50.0 / 82.0).abs() < 1e-5);
    }
}
