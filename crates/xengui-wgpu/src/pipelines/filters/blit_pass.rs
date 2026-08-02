// SPDX-License-Identifier: Apache-2.0
use xengui::{ Color, paint };

// Explicit padding mirrors WGSL's std140-style alignment rules (vec4
// aligns to 16 bytes, vec2 to 8 bytes, and the whole struct rounds up to
// its largest member's alignment) - Rust's own repr(C) layout only uses
// each field's natural 4-byte alignment, so without this padding the
// uniform buffer ends up smaller than what the shader's BlitParams expects.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct GpuBlitParams {
    tint: [f32; 4],
    tint_mix: f32,
    _pad0: f32,
    offset: [f32; 2],
    _pad1: f32,
    _pad2: [f32; 3],
}

/// A fullscreen-triangle "copy with optional offset/tint" pass, the
/// general-purpose building block every filter segment uses to move a
/// texture into a differently-sized/positioned target - centering a
/// source into a padded working texture, extracting a tinted alpha
/// silhouette for [`xengui::Filter::DropShadow`], and compositing one
/// texture over another at an arbitrary screen position.
pub struct BlitPass {
    pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    uniform_buffer: wgpu::Buffer,
    /// Same pipeline, but with standard (non-premultiplied-source) alpha
    /// blending enabled - used by [`Self::run_over`] to composite one
    /// pass's output on top of another instead of overwriting it.
    pipeline_blend: wgpu::RenderPipeline,
}

impl BlitPass {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let vs = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Blit Fullscreen VS"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/fullscreen.wgsl").into()),
        });
        let fs = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Blit FS"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/blit.wgsl").into()),
        });

        let bind_group_layout = device.create_bind_group_layout(
            &(wgpu::BindGroupLayoutDescriptor {
                label: Some("Blit Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            })
        );

        let layout = device.create_pipeline_layout(
            &(wgpu::PipelineLayoutDescriptor {
                label: Some("Blit Pipeline Layout"),
                bind_group_layouts: &[Some(&bind_group_layout)],
                immediate_size: 0,
            })
        );

        let build_pipeline = |blend: Option<wgpu::BlendState>, label: &str| {
            device.create_render_pipeline(
                &(wgpu::RenderPipelineDescriptor {
                    label: Some(label),
                    layout: Some(&layout),
                    vertex: wgpu::VertexState {
                        module: &vs,
                        entry_point: Some("vs_main"),
                        compilation_options: Default::default(),
                        buffers: &[],
                    },
                    primitive: wgpu::PrimitiveState {
                        topology: wgpu::PrimitiveTopology::TriangleList,
                        ..Default::default()
                    },
                    depth_stencil: None,
                    multisample: Default::default(),
                    fragment: Some(wgpu::FragmentState {
                        module: &fs,
                        entry_point: Some("fs_main"),
                        compilation_options: Default::default(),
                        targets: &[
                            Some(wgpu::ColorTargetState {
                                format,
                                blend,
                                write_mask: wgpu::ColorWrites::ALL,
                            }),
                        ],
                    }),
                    multiview_mask: None,
                    cache: None,
                })
            )
        };

        let pipeline = build_pipeline(None, "Blit Pipeline (overwrite)");
        let pipeline_blend = build_pipeline(
            Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
            "Blit Pipeline (blend)"
        );

        // The shader itself clamps sample_uv to [0,1] and returns transparent
        // black outside that range, so the sampler's own address mode never
        // actually samples out-of-bounds - ClampToEdge avoids depending on
        // the optional ADDRESS_MODE_CLAMP_TO_BORDER device feature.
        let sampler = device.create_sampler(
            &(wgpu::SamplerDescriptor {
                label: Some("xengui blit sampler"),
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                ..Default::default()
            })
        );

        let uniform_buffer = device.create_buffer(
            &(wgpu::BufferDescriptor {
                label: Some("Blit Params Uniform"),
                size: std::mem::size_of::<GpuBlitParams>() as u64,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })
        );

        Self { pipeline, bind_group_layout, sampler, uniform_buffer, pipeline_blend }
    }

    /// Copies `source` into `target`, filling the whole `target_width` x
    /// `target_height` texture, offset by `offset_uv` (fractional UV
    /// units) and optionally tinted into a solid-color alpha silhouette
    /// when `tint` is `Some` (used to build the shadow shape for
    /// [`xengui::Filter::DropShadow`] before it's blurred).
    #[allow(clippy::too_many_arguments)]
    pub fn run(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        source: &wgpu::TextureView,
        target: &wgpu::TextureView,
        target_width: u32,
        target_height: u32,
        offset_uv: (f32, f32),
        _source_size: (u32, u32),
        tint: Option<Color>
    ) {
        self.dispatch(
            device,
            queue,
            encoder,
            source,
            target,
            (0.0, 0.0, target_width as f32, target_height as f32),
            None,
            target_width,
            target_height,
            offset_uv,
            tint,
            false
        );
    }

    /// Composites `source` on top of whatever `target` already contains,
    /// at `dest_rect` (physical px), using standard alpha-over blending
    /// instead of overwriting it. `clip_rect` (if given) restricts the
    /// composite to an ancestor's own clip region via a scissor rect.
    #[allow(clippy::too_many_arguments)]
    pub fn run_over(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        source: &wgpu::TextureView,
        target: &wgpu::TextureView,
        dest_rect: (f32, f32, f32, f32),
        clip_rect: Option<(f32, f32, f32, f32)>,
        target_width: u32,
        target_height: u32
    ) {
        self.dispatch(
            device,
            queue,
            encoder,
            source,
            target,
            dest_rect,
            clip_rect,
            target_width,
            target_height,
            (0.0, 0.0),
            None,
            true
        );
    }

    #[allow(clippy::too_many_arguments)]
    fn dispatch(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        source: &wgpu::TextureView,
        target: &wgpu::TextureView,
        dest_rect: (f32, f32, f32, f32),
        clip_rect: Option<(f32, f32, f32, f32)>,
        target_width: u32,
        target_height: u32,
        offset_uv: (f32, f32),
        tint: Option<Color>,
        blend_over: bool
    ) {
        let (tint_rgba, tint_mix) = match tint {
            Some(c) => (c.to_f32_array(), 1.0),
            None => ([0.0, 0.0, 0.0, 1.0], 0.0),
        };
        let params = GpuBlitParams {
            tint: tint_rgba,
            tint_mix,
            _pad0: 0.0,
            offset: [offset_uv.0, offset_uv.1],
            _pad1: 0.0,
            _pad2: [0.0; 3],
        };
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&params));

        let bind_group = device.create_bind_group(
            &(wgpu::BindGroupDescriptor {
                label: Some("Blit Bind Group"),
                layout: &self.bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(source),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&self.sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: self.uniform_buffer.as_entire_binding(),
                    },
                ],
            })
        );

        let pipeline = if blend_over { &self.pipeline_blend } else { &self.pipeline };

        let mut pass = encoder.begin_render_pass(
            &(wgpu::RenderPassDescriptor {
                label: Some("Blit Pass"),
                color_attachments: &[
                    Some(wgpu::RenderPassColorAttachment {
                        view: target,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: if blend_over {
                                wgpu::LoadOp::Load
                            } else {
                                wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT)
                            },
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
        pass.set_pipeline(pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.set_viewport(
            dest_rect.0,
            dest_rect.1,
            dest_rect.2.max(1.0),
            dest_rect.3.max(1.0),
            0.0,
            1.0
        );

        let (sx, sy, sw, sh) = paint::draw_command::scissor_for_clip(
            clip_rect,
            target_width,
            target_height
        );
        if sw == 0 || sh == 0 {
            return;
        }
        pass.set_scissor_rect(sx, sy, sw, sh);

        pass.draw(0..3, 0..1);
    }
}
