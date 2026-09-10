// SPDX-License-Identifier: Apache-2.0
use xengui::{RippleCommand, paint};

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct RippleInstance {
    screen_rect: [f32; 4],
    half_size: [f32; 2],
    radius: [f32; 4],
    origin_radius: [f32; 4],
    progress_noise: [f32; 2],
    color: [f32; 4],
}

impl RippleInstance {
    fn layout() -> wgpu::VertexBufferLayout<'static> {
        const ATTRS: [wgpu::VertexAttribute; 6] = wgpu::vertex_attr_array![
            0 => Float32x4, 1 => Float32x2, 2 => Float32x4,
            3 => Float32x4, 4 => Float32x2, 5 => Float32x4
        ];
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as u64,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &ATTRS,
        }
    }
}

/// GPU implementation of Android's patterned ripple: one instanced quad per
/// active interaction and no CPU-generated particles or per-frame geometry.
pub struct RipplePipeline {
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    capacity: usize,
    write_offset: usize,
    instances: Vec<RippleInstance>,
}

impl RipplePipeline {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Patterned Ripple Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/ripple.wgsl").into()),
        });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Patterned Ripple Pipeline Layout"),
            bind_group_layouts: &[],
            immediate_size: 0,
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Patterned Ripple Pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[Some(RippleInstance::layout())],
            },
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: None,
        });
        let capacity = 64;
        let vertex_buffer = Self::buffer(device, capacity);
        Self {
            pipeline,
            vertex_buffer,
            capacity,
            write_offset: 0,
            instances: Vec::with_capacity(capacity),
        }
    }

    fn buffer(device: &wgpu::Device, capacity: usize) -> wgpu::Buffer {
        device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Patterned Ripple Instance Buffer"),
            size: (capacity * std::mem::size_of::<RippleInstance>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    }

    pub fn reset_frame(&mut self) {
        self.write_offset = 0;
    }

    pub fn draw_batch(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        pass: &mut wgpu::RenderPass<'_>,
        width: u32,
        height: u32,
        commands: &[RippleCommand],
    ) {
        if commands.is_empty() {
            return;
        }
        let inv_w = 2.0 / width.max(1) as f32;
        let inv_h = 2.0 / height.max(1) as f32;
        self.instances.clear();
        for command in commands {
            let (x, y, w, h) = command.bounds;
            let local_origin = (command.origin.0 - x, command.origin.1 - y);
            let max_radius = farthest_corner_radius(local_origin, w, h);
            self.instances.push(RippleInstance {
                screen_rect: [
                    x * inv_w - 1.0,
                    1.0 - y * inv_h,
                    (x + w) * inv_w - 1.0,
                    1.0 - (y + h) * inv_h,
                ],
                half_size: [w * 0.5, h * 0.5],
                radius: command.radius,
                origin_radius: [
                    local_origin.0,
                    local_origin.1,
                    max_radius,
                    command.opacity.clamp(0.0, 1.0),
                ],
                progress_noise: [command.progress.clamp(0.0, 1.0), command.noise_phase],
                color: command.color.to_f32_array(),
            });
        }
        let base = self.write_offset;
        let required = base + self.instances.len();
        if required > self.capacity {
            self.capacity = required.next_power_of_two();
            self.vertex_buffer = Self::buffer(device, self.capacity);
        }
        queue.write_buffer(
            &self.vertex_buffer,
            (base * std::mem::size_of::<RippleInstance>()) as u64,
            bytemuck::cast_slice(&self.instances),
        );
        self.write_offset = required;

        pass.set_pipeline(&self.pipeline);
        pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        pass.set_viewport(0.0, 0.0, width as f32, height as f32, 0.0, 1.0);
        let mut start = 0;
        let mut clip = commands[0].clip_rect;
        for (index, command) in commands.iter().enumerate().skip(1) {
            if command.clip_rect != clip {
                Self::draw_run(pass, base, start, index, clip, width, height);
                start = index;
                clip = command.clip_rect;
            }
        }
        Self::draw_run(pass, base, start, commands.len(), clip, width, height);
    }

    fn draw_run(
        pass: &mut wgpu::RenderPass<'_>,
        base: usize,
        start: usize,
        end: usize,
        clip: Option<(f32, f32, f32, f32)>,
        width: u32,
        height: u32,
    ) {
        let (x, y, w, h) = paint::draw_command::scissor_for_clip(clip, width, height);
        if w == 0 || h == 0 {
            return;
        }
        pass.set_scissor_rect(x, y, w, h);
        pass.draw(0..6, (base + start) as u32..(base + end) as u32);
    }
}

fn farthest_corner_radius(origin: (f32, f32), width: f32, height: f32) -> f32 {
    let far_x = origin.0.abs().max((width - origin.0).abs());
    let far_y = origin.1.abs().max((height - origin.1).abs());
    far_x.hypot(far_y)
}

#[cfg(test)]
mod tests {
    use super::{RippleInstance, farthest_corner_radius};

    #[test]
    fn patterned_ripple_is_one_compact_gpu_instance() {
        assert_eq!(std::mem::size_of::<RippleInstance>(), 80);
    }

    #[test]
    fn hotspot_radius_reaches_the_farthest_view_corner() {
        let radius = farthest_corner_radius((10.0, 20.0), 100.0, 60.0);
        assert!((radius - 98.488_58).abs() < 0.001);
    }
}
