// SPDX-License-Identifier: Apache-2.0
use super::texture_pool::{ PooledTexture, TexturePool };

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct GpuBlurParams {
    direction: [f32; 2],
    radius: f32,
    _pad: f32,
}

/// Two-pass separable Gaussian blur: one horizontal pass followed by one
/// vertical pass over its result, giving the same visual output as a full
/// 2D Gaussian convolution at `O(2 * radius)` texture samples instead of
/// `O(radius^2)`.
///
/// Each pass renders into a texture acquired from the shared
/// [`TexturePool`], so repeated blurs (e.g. inside [`super::FilterEngine::run_drop_shadow`])
/// reuse GPU memory across a frame instead of allocating a fresh
/// intermediate every time.
pub struct BlurPass {
    pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    /// Two ping-ponged uniform buffers (horizontal, then vertical) so the
    /// second pass's `write_buffer` can't race the first pass's read on
    /// backends that don't serialize uniform writes within a frame.
    uniform_buffers: [wgpu::Buffer; 2],
}

impl BlurPass {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let vs = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Blur Fullscreen VS"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/fullscreen.wgsl").into()),
        });
        let fs = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Blur FS"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/blur.wgsl").into()),
        });

        let bind_group_layout = device.create_bind_group_layout(
            &(wgpu::BindGroupLayoutDescriptor {
                label: Some("Blur Bind Group Layout"),
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
                label: Some("Blur Pipeline Layout"),
                bind_group_layouts: &[&bind_group_layout],
                immediate_size: 0,
            })
        );

        let pipeline = device.create_render_pipeline(
            &(wgpu::RenderPipelineDescriptor {
                label: Some("Blur Pipeline"),
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
                            // Blur passes write a fully opaque-coverage
                            // intermediate result (the whole padded texture was
                            // cleared+blitted before this runs), so no blending
                            // is needed - a straight overwrite is both correct
                            // and cheaper than blending.
                            blend: None,
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
                label: Some("xengui blur sampler"),
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                ..Default::default()
            })
        );

        let make_uniform = || {
            device.create_buffer(
                &(wgpu::BufferDescriptor {
                    label: Some("Blur Params Uniform"),
                    size: std::mem::size_of::<GpuBlurParams>() as u64,
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                })
            )
        };

        Self {
            pipeline,
            bind_group_layout,
            sampler,
            uniform_buffers: [make_uniform(), make_uniform()],
        }
    }

    /// Runs the full two-pass blur over `source` and returns a new pooled
    /// texture holding the result. `radius` is in physical px; values
    /// `<= 0.0` are treated as a no-op that still copies `source` through
    /// (via a single degenerate pass) so callers don't need a special
    /// case for "blur radius zero".
    #[allow(clippy::too_many_arguments)]
    pub fn run(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        source: &PooledTexture,
        width: u32,
        height: u32,
        radius: f32,
        pool: &mut TexturePool
    ) -> PooledTexture {
        let radius = radius.max(0.0);

        let horizontal = pool.acquire(device, width, height);
        self.run_single_pass(
            device,
            queue,
            encoder,
            &source.view,
            &horizontal.view,
            width,
            height,
            [1.0, 0.0],
            radius,
            0
        );

        let vertical = pool.acquire(device, width, height);
        self.run_single_pass(
            device,
            queue,
            encoder,
            &horizontal.view,
            &vertical.view,
            width,
            height,
            [0.0, 1.0],
            radius,
            1
        );

        vertical
    }

    #[allow(clippy::too_many_arguments)]
    fn run_single_pass(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        source: &wgpu::TextureView,
        target: &wgpu::TextureView,
        width: u32,
        height: u32,
        direction: [f32; 2],
        radius: f32,
        buffer_index: usize
    ) {
        let params = GpuBlurParams { direction, radius, _pad: 0.0 };
        let uniform_buffer = &self.uniform_buffers[buffer_index];
        queue.write_buffer(uniform_buffer, 0, bytemuck::bytes_of(&params));

        let bind_group = device.create_bind_group(
            &(wgpu::BindGroupDescriptor {
                label: Some("Blur Bind Group"),
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
                        resource: uniform_buffer.as_entire_binding(),
                    },
                ],
            })
        );

        let mut pass = encoder.begin_render_pass(
            &(wgpu::RenderPassDescriptor {
                label: Some("Blur Pass"),
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
