// SPDX-License-Identifier: Apache-2.0
use xengui::Filter;

const MAX_OPS: usize = 8;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct GpuFilterOp {
    kind: u32,
    _pad0: u32,
    _pad1: u32,
    _pad2: u32,
    params: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct GpuFilterOps {
    count: u32,
    _pad: [u32; 3],
    ops: [GpuFilterOp; MAX_OPS],
}

/// Runs the fused, single-pass shader covering every pointwise CSS-style
/// color filter (brightness, contrast, saturate, grayscale, hue-rotate,
/// invert, opacity, gamma). A whole contiguous run of these filters in a
/// [`xengui::FilterChain`] costs exactly one texture sample and one pass.
pub struct ColorFilterPass {
    pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    uniform_buffer: wgpu::Buffer,
}

impl ColorFilterPass {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let vs = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Filter Fullscreen VS"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/fullscreen.wgsl").into()),
        });
        let fs = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Color Filter FS"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/color_filter.wgsl").into()),
        });

        let bind_group_layout = device.create_bind_group_layout(
            &(wgpu::BindGroupLayoutDescriptor {
                label: Some("Color Filter Bind Group Layout"),
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
                label: Some("Color Filter Pipeline Layout"),
                bind_group_layouts: &[Some(&bind_group_layout)],
                immediate_size: 0,
            })
        );

        let pipeline = device.create_render_pipeline(
            &(wgpu::RenderPipelineDescriptor {
                label: Some("Color Filter Pipeline"),
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
                            blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                            write_mask: wgpu::ColorWrites::ALL,
                        }),
                    ],
                }),
                multiview_mask: None,
                cache: None,
            })
        );

        let sampler = device.create_sampler(
            &(wgpu::SamplerDescriptor {
                label: Some("xengui filter sampler"),
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                ..Default::default()
            })
        );

        let uniform_buffer = device.create_buffer(
            &(wgpu::BufferDescriptor {
                label: Some("Color Filter Ops Uniform"),
                size: std::mem::size_of::<GpuFilterOps>() as u64,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })
        );

        Self { pipeline, bind_group_layout, sampler, uniform_buffer }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn run(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        source: &wgpu::TextureView,
        target: &wgpu::TextureView,
        width: u32,
        height: u32,
        filters: &[&Filter]
    ) {
        let mut ops = GpuFilterOps {
            count: 0,
            _pad: [0; 3],
            ops: [GpuFilterOp { kind: 0, _pad0: 0, _pad1: 0, _pad2: 0, params: [0.0; 4] }; MAX_OPS],
        };

        for filter in filters.iter().take(MAX_OPS) {
            let (kind, params) = match filter {
                Filter::Brightness(v) => (0u32, [*v, 0.0, 0.0, 0.0]),
                Filter::Contrast(v) => (1, [*v, 0.0, 0.0, 0.0]),
                Filter::Saturate(v) => (2, [*v, 0.0, 0.0, 0.0]),
                Filter::Grayscale(v) => (3, [*v, 0.0, 0.0, 0.0]),
                Filter::HueRotate(deg) => (4, [*deg, 0.0, 0.0, 0.0]),
                Filter::Invert(v) => (5, [*v, 0.0, 0.0, 0.0]),
                Filter::Opacity(v) => (6, [*v, 0.0, 0.0, 0.0]),
                Filter::Gamma(v) => (7, [*v, 0.0, 0.0, 0.0]),
                Filter::Blur(_) | Filter::DropShadow(_) => {
                    continue;
                } // handled by a different pass
            };
            ops.ops[ops.count as usize] = GpuFilterOp {
                kind,
                _pad0: 0,
                _pad1: 0,
                _pad2: 0,
                params,
            };
            ops.count += 1;
        }

        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&ops));

        let bind_group = device.create_bind_group(
            &(wgpu::BindGroupDescriptor {
                label: Some("Color Filter Bind Group"),
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

        let mut pass = encoder.begin_render_pass(
            &(wgpu::RenderPassDescriptor {
                label: Some("Color Filter Pass"),
                color_attachments: &[
                    Some(wgpu::RenderPassColorAttachment {
                        view: target,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
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
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.set_viewport(0.0, 0.0, width as f32, height as f32, 0.0, 1.0);
        pass.draw(0..3, 0..1);
    }
}
