// SPDX-License-Identifier: Apache-2.0
use super::texture_pool::{ PooledTexture, TexturePool };

/// Upper bound on how many downsample/upsample levels a single blur can
/// use. Higher radii just get a larger per-level offset instead of more
/// levels past this point, so worst-case GPU work per blur stays bounded.
const MAX_ITERATIONS: usize = 5;
const MAX_PASSES: usize = MAX_ITERATIONS * 2;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct GpuKawaseParams {
    offset: f32,
    _pad: [f32; 3],
}

/// Dual Kawase blur: alternates downsampling (5-tap box average) and
/// upsampling (8-tap tent) passes instead of a single-axis Gaussian
/// convolution. Each level halves/doubles resolution, so a wide blur
/// costs a handful of small-resolution passes rather than O(radius)
/// samples at full resolution - the standard technique behind most
/// real-time engine bloom/blur passes (ARM's "dual filtering").
pub struct KawasePass {
    down_pipeline: wgpu::RenderPipeline,
    up_pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    // Every iteration's offset for one `run` call is written into this
    // buffer in one shot, and each pass reads its own slice via a dynamic
    // offset - avoids the write/read race a single reused uniform slot
    // would hit across passes recorded into the same not-yet-submitted
    // encoder.
    params_buffer: wgpu::Buffer,
    params_stride: u64,
}

impl KawasePass {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let vs = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Kawase Fullscreen VS"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/fullscreen.wgsl").into()),
        });
        let down_fs = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Kawase Down FS"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/kawase_down.wgsl").into()),
        });
        let up_fs = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Kawase Up FS"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/kawase_up.wgsl").into()),
        });

        let bind_group_layout = device.create_bind_group_layout(
            &(wgpu::BindGroupLayoutDescriptor {
                label: Some("Kawase Bind Group Layout"),
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
                            has_dynamic_offset: true,
                            min_binding_size: std::num::NonZeroU64::new(
                                std::mem::size_of::<GpuKawaseParams>() as u64
                            ),
                        },
                        count: None,
                    },
                ],
            })
        );

        let layout = device.create_pipeline_layout(
            &(wgpu::PipelineLayoutDescriptor {
                label: Some("Kawase Pipeline Layout"),
                bind_group_layouts: &[Some(&bind_group_layout)],
                immediate_size: 0,
            })
        );

        let build_pipeline = |fs: &wgpu::ShaderModule, label: &str| {
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
                        module: fs,
                        entry_point: Some("fs_main"),
                        compilation_options: Default::default(),
                        targets: &[
                            Some(wgpu::ColorTargetState {
                                format,
                                blend: None,
                                write_mask: wgpu::ColorWrites::ALL,
                            }),
                        ],
                    }),
                    multiview_mask: None,
                    cache: None,
                })
            )
        };

        let down_pipeline = build_pipeline(&down_fs, "Kawase Down Pipeline");
        let up_pipeline = build_pipeline(&up_fs, "Kawase Up Pipeline");

        let sampler = device.create_sampler(
            &(wgpu::SamplerDescriptor {
                label: Some("xengui kawase sampler"),
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                ..Default::default()
            })
        );

        let alignment = device.limits().min_uniform_buffer_offset_alignment as u64;
        let params_stride = align_up(std::mem::size_of::<GpuKawaseParams>() as u64, alignment);
        let params_buffer = device.create_buffer(
            &(wgpu::BufferDescriptor {
                label: Some("Kawase Params Uniform"),
                size: params_stride * (MAX_PASSES as u64),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })
        );

        Self {
            down_pipeline,
            up_pipeline,
            bind_group_layout,
            sampler,
            params_buffer,
            params_stride,
        }
    }

    /// Runs the full down/up chain over `source` and returns a new pooled
    /// texture at the same `(width, height)`. `radius` is in physical px;
    /// values below half a texel are a no-op that just hands `source`
    /// back.
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
        let (iterations, offset) = plan_iterations(radius.max(0.0));
        if iterations == 0 {
            return source.clone();
        }

        let params = vec![GpuKawaseParams { offset, _pad: [0.0; 3] }; iterations * 2];
        self.write_params(queue, &params);

        let mut levels: Vec<PooledTexture> = Vec::with_capacity(iterations + 1);
        levels.push(source.clone());

        let mut pass_index = 0usize;
        let mut level_w = width;
        let mut level_h = height;
        for _ in 0..iterations {
            level_w = (level_w / 2).max(1);
            level_h = (level_h / 2).max(1);
            let target = pool.acquire(device, level_w, level_h);
            self.dispatch(
                device,
                encoder,
                &self.down_pipeline,
                &levels.last().expect("levels always has at least the source").view,
                &target.view,
                level_w,
                level_h,
                pass_index
            );
            levels.push(target);
            pass_index += 1;
        }

        let mut current = levels.pop().expect("at least one downsample level was pushed");
        while let Some(level) = levels.pop() {
            let target = pool.acquire(device, level.width, level.height);
            self.dispatch(
                device,
                encoder,
                &self.up_pipeline,
                &current.view,
                &target.view,
                level.width,
                level.height,
                pass_index
            );
            current = target;
            pass_index += 1;
        }

        current
    }

    fn write_params(&self, queue: &wgpu::Queue, params: &[GpuKawaseParams]) {
        let stride = self.params_stride as usize;
        let mut bytes = vec![0u8; stride * params.len()];
        for (i, p) in params.iter().enumerate() {
            let start = i * stride;
            let raw = bytemuck::bytes_of(p);
            bytes[start..start + raw.len()].copy_from_slice(raw);
        }
        queue.write_buffer(&self.params_buffer, 0, &bytes);
    }

    #[allow(clippy::too_many_arguments)]
    fn dispatch(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        pipeline: &wgpu::RenderPipeline,
        source: &wgpu::TextureView,
        target: &wgpu::TextureView,
        width: u32,
        height: u32,
        pass_index: usize
    ) {
        let bind_group = device.create_bind_group(
            &(wgpu::BindGroupDescriptor {
                label: Some("Kawase Bind Group"),
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
                        resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                            buffer: &self.params_buffer,
                            offset: 0,
                            size: std::num::NonZeroU64::new(
                                std::mem::size_of::<GpuKawaseParams>() as u64
                            ),
                        }),
                    },
                ],
            })
        );

        let mut pass = encoder.begin_render_pass(
            &(wgpu::RenderPassDescriptor {
                label: Some("Kawase Pass"),
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
        pass.set_pipeline(pipeline);
        pass.set_bind_group(0, &bind_group, &[((pass_index as u64) * self.params_stride) as u32]);
        pass.set_viewport(0.0, 0.0, width as f32, height as f32, 0.0, 1.0);
        pass.draw(0..3, 0..1);
    }
}

fn align_up(value: u64, alignment: u64) -> u64 {
    value.div_ceil(alignment) * alignment
}

// Picks a level count and a shared per-level sample offset for `radius`
// (physical px). Each additional level roughly doubles the blur a fixed
// offset produces (it operates at half the resolution of the level
// before it), so this walks up levels until a modest base offset would
// already cover the requested radius, then folds the remainder into the
// offset itself.
fn plan_iterations(radius: f32) -> (usize, f32) {
    const BASE_OFFSET: f32 = 1.5;
    if radius < 0.5 {
        return (0, 0.0);
    }

    let mut iterations = 1usize;
    while iterations < MAX_ITERATIONS && BASE_OFFSET * ((1u32 << iterations) as f32) < radius {
        iterations += 1;
    }

    let reach = (1u32 << iterations) as f32;
    let offset = (radius / reach).max(0.5);
    (iterations, offset)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_radius_uses_no_iterations() {
        assert_eq!(plan_iterations(0.0), (0, 0.0));
    }

    #[test]
    fn small_radius_uses_one_iteration() {
        let (iterations, offset) = plan_iterations(2.0);
        assert_eq!(iterations, 1);
        assert!(offset >= 0.5);
    }

    #[test]
    fn large_radius_caps_at_max_iterations() {
        let (iterations, _offset) = plan_iterations(10_000.0);
        assert_eq!(iterations, MAX_ITERATIONS);
    }
}
