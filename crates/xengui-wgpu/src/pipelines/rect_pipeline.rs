// SPDX-License-Identifier: Apache-2.0
use xengui::{Background, GradientStop, RectCommand, paint};

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct RectInstance {
    // left, top, right, bottom in normalized device coordinates.
    screen_rect: [f32; 4],
    half_size: [f32; 2],
    radius: [f32; 4],
    border_width: f32,
    fill_color: [f32; 4],
    border_color: [f32; 4],
    // x: kind (0 = solid, 1 = linear, 2 = radial), y: linear angle (rad),
    // z: stop count, w: offset into the shared gradient stop buffers.
    gradient_meta: [f32; 4],
}

impl RectInstance {
    fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<RectInstance>() as u64,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    shader_location: 0,
                    offset: 0,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    shader_location: 1,
                    offset: 16,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    shader_location: 2,
                    offset: 24,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    shader_location: 3,
                    offset: 40,
                    format: wgpu::VertexFormat::Float32,
                }, // border_width
                wgpu::VertexAttribute {
                    shader_location: 4,
                    offset: 44,
                    format: wgpu::VertexFormat::Float32x4,
                }, // fill_color
                wgpu::VertexAttribute {
                    shader_location: 5,
                    offset: 60,
                    format: wgpu::VertexFormat::Float32x4,
                }, // border_color
                wgpu::VertexAttribute {
                    shader_location: 6,
                    offset: 76,
                    format: wgpu::VertexFormat::Float32x4,
                }, // gradient_meta
            ],
        }
    }
}

pub struct RectPipeline {
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    vertex_capacity: usize,
    write_offset: usize,

    gradient_positions_buffer: wgpu::Buffer,
    gradient_colors_buffer: wgpu::Buffer,
    gradient_bind_group: wgpu::BindGroup,
    stops_used: usize,
    instances: Vec<RectInstance>,
    gradient_positions: Vec<f32>,
    gradient_colors: Vec<[f32; 4]>,
}

const DEFAULT_RECT_CAPACITY: usize = 256;

// Total gradient stops shared across every gradient-filled rect drawn in
// one frame (possibly across multiple draw_batch calls, e.g. main pass +
// top layer) - see xengui's Background::MAX_GRADIENT_STOPS for the
// matching per-gradient headroom within this budget.
const MAX_GRADIENT_STOPS_TOTAL: usize = 512;
const MAX_GRADIENT_POSITION_VEC4S: usize = MAX_GRADIENT_STOPS_TOTAL / 4;

impl RectPipeline {
    pub fn new(
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
        sample_count: u32,
    ) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Rect Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/rect.wgsl").into()),
        });

        let gradient_bind_group_layout = device.create_bind_group_layout(
            &(wgpu::BindGroupLayoutDescriptor {
                label: Some("Rect Gradient Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            }),
        );

        let layout = device.create_pipeline_layout(
            &(wgpu::PipelineLayoutDescriptor {
                label: Some("Rect Pipeline Layout"),
                bind_group_layouts: &[Some(&gradient_bind_group_layout)],
                immediate_size: 0,
            }),
        );

        let pipeline = device.create_render_pipeline(
            &(wgpu::RenderPipelineDescriptor {
                label: Some("Rect Pipeline"),
                layout: Some(&layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"),
                    compilation_options: Default::default(),
                    buffers: &[Some(RectInstance::layout())],
                },
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    ..Default::default()
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState {
                    count: sample_count,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fs_main"),
                    compilation_options: Default::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: surface_format,
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                multiview_mask: None,
                cache: None,
            }),
        );

        let vertex_capacity = DEFAULT_RECT_CAPACITY;
        let vertex_buffer = device.create_buffer(
            &(wgpu::BufferDescriptor {
                label: Some("Rect Vertex Buffer"),
                size: (vertex_capacity * std::mem::size_of::<RectInstance>()) as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
        );

        let gradient_positions_buffer = device.create_buffer(
            &(wgpu::BufferDescriptor {
                label: Some("Rect Gradient Positions Buffer"),
                size: (MAX_GRADIENT_POSITION_VEC4S * 16) as u64,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
        );

        let gradient_colors_buffer = device.create_buffer(
            &(wgpu::BufferDescriptor {
                label: Some("Rect Gradient Colors Buffer"),
                size: (MAX_GRADIENT_STOPS_TOTAL * 16) as u64,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
        );

        let gradient_bind_group = device.create_bind_group(
            &(wgpu::BindGroupDescriptor {
                label: Some("Rect Gradient Bind Group"),
                layout: &gradient_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: gradient_positions_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: gradient_colors_buffer.as_entire_binding(),
                    },
                ],
            }),
        );

        Self {
            pipeline,
            vertex_buffer,
            vertex_capacity,
            write_offset: 0,
            gradient_positions_buffer,
            gradient_colors_buffer,
            gradient_bind_group,
            stops_used: 0,
            instances: Vec::with_capacity(DEFAULT_RECT_CAPACITY),
            gradient_positions: Vec::with_capacity(MAX_GRADIENT_STOPS_TOTAL),
            gradient_colors: Vec::with_capacity(MAX_GRADIENT_STOPS_TOTAL),
        }
    }

    pub fn reset_frame(&mut self) {
        self.write_offset = 0;
        self.stops_used = 0;
    }

    pub fn draw_batch(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        render_pass: &mut wgpu::RenderPass<'_>,
        surface_width: u32,
        surface_height: u32,
        cmds: &[RectCommand],
    ) {
        if cmds.is_empty() {
            return;
        }

        self.instances.clear();
        self.instances.reserve(cmds.len());
        self.gradient_positions.clear();
        self.gradient_colors.clear();

        let inv_w = 2.0 / (surface_width.max(1) as f32);
        let inv_h = 2.0 / (surface_height.max(1) as f32);
        let ndc = |px: f32, py: f32| -> [f32; 2] { [px * inv_w - 1.0, 1.0 - py * inv_h] };

        // Extra headroom (physical px) rasterized beyond each rect's true
        // edge so the SDF antialiasing fringe is not clipped by the quad.
        const AA_PAD: f32 = 1.5;

        for cmd in cmds {
            let (kind, angle, stops): (f32, f32, &[GradientStop]) = match cmd.background.as_ref() {
                Some(Background::LinearGradient(g)) => {
                    (1.0, g.angle_deg.to_radians(), g.stops.as_slice())
                }
                Some(Background::RadialGradient(g)) => (2.0, 0.0, g.stops.as_slice()),
                _ => (0.0, 0.0, &[]),
            };

            if stops.is_empty() {
                self.push_instance(cmd, [kind, angle, 0.0, 0.0], ndc, AA_PAD);
            } else {
                let already_used = self.stops_used + self.gradient_colors.len();
                let remaining = MAX_GRADIENT_STOPS_TOTAL.saturating_sub(already_used);
                let take = stops.len().min(remaining);

                if take < stops.len() {
                    log::warn!(
                        "rect gradient stop buffer full this frame: dropping {} of {} stops",
                        stops.len() - take,
                        stops.len()
                    );
                }

                for stop in &stops[..take] {
                    self.gradient_positions.push(stop.position);
                    self.gradient_colors.push(stop.color.to_f32_array());
                }

                self.push_instance(
                    cmd,
                    [kind, angle, take as f32, already_used as f32],
                    ndc,
                    AA_PAD,
                );
            }
        }

        if !self.gradient_colors.is_empty() {
            let base_offset = self.stops_used;
            queue.write_buffer(
                &self.gradient_positions_buffer,
                (base_offset * 4) as u64,
                bytemuck::cast_slice(&self.gradient_positions),
            );
            queue.write_buffer(
                &self.gradient_colors_buffer,
                (base_offset * 16) as u64,
                bytemuck::cast_slice(&self.gradient_colors),
            );
            self.stops_used += self.gradient_colors.len();
        }

        let base_vertex = self.write_offset;
        self.ensure_capacity(device, base_vertex + self.instances.len());
        queue.write_buffer(
            &self.vertex_buffer,
            (base_vertex * std::mem::size_of::<RectInstance>()) as u64,
            bytemuck::cast_slice(&self.instances),
        );
        self.write_offset += self.instances.len();

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.gradient_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_viewport(
            0.0,
            0.0,
            surface_width as f32,
            surface_height as f32,
            0.0,
            1.0,
        );

        let mut run_start = 0usize;
        let mut current_clip = cmds[0].clip_rect;

        for (i, cmd) in cmds.iter().enumerate().skip(1) {
            if cmd.clip_rect != current_clip {
                Self::draw_run(
                    render_pass,
                    base_vertex,
                    run_start,
                    i,
                    current_clip,
                    surface_width,
                    surface_height,
                );
                run_start = i;
                current_clip = cmd.clip_rect;
            }
        }
        Self::draw_run(
            render_pass,
            base_vertex,
            run_start,
            cmds.len(),
            current_clip,
            surface_width,
            surface_height,
        );
    }

    fn draw_run(
        render_pass: &mut wgpu::RenderPass<'_>,
        base_vertex: usize,
        start: usize,
        end: usize,
        clip: Option<(f32, f32, f32, f32)>,
        surface_width: u32,
        surface_height: u32,
    ) {
        let (sx, sy, sw, sh) =
            paint::draw_command::scissor_for_clip(clip, surface_width, surface_height);
        if sw == 0 || sh == 0 {
            return;
        }
        render_pass.set_scissor_rect(sx, sy, sw, sh);
        render_pass.draw(
            0..6,
            (base_vertex + start) as u32..(base_vertex + end) as u32,
        );
    }

    fn ensure_capacity(&mut self, device: &wgpu::Device, required: usize) {
        if required <= self.vertex_capacity {
            return;
        }
        self.vertex_capacity = required.next_power_of_two();
        self.vertex_buffer = device.create_buffer(
            &(wgpu::BufferDescriptor {
                label: Some("Rect Vertex Buffer"),
                size: (self.vertex_capacity * std::mem::size_of::<RectInstance>()) as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
        );
    }

    fn push_instance(
        &mut self,
        cmd: &RectCommand,
        gradient_meta: [f32; 4],
        ndc: impl Fn(f32, f32) -> [f32; 2],
        aa_pad: f32,
    ) {
        let (x, y) = cmd.position;
        let (w, h) = cmd.size;
        let top_left = ndc(x - aa_pad, y - aa_pad);
        let bottom_right = ndc(x + w + aa_pad, y + h + aa_pad);
        self.instances.push(RectInstance {
            screen_rect: [top_left[0], top_left[1], bottom_right[0], bottom_right[1]],
            half_size: [w * 0.5, h * 0.5],
            radius: cmd
                .border_radius
                .map(|r| r.to_physical_array(1.0, w, h))
                .unwrap_or([0.0; 4]),
            border_width: cmd.border_width.map(|width| width.value()).unwrap_or(0.0),
            fill_color: cmd
                .background
                .as_ref()
                .map(|background| background.representative_color().to_f32_array())
                .unwrap_or([0.0; 4]),
            border_color: cmd
                .border_color
                .map(|color| color.to_f32_array())
                .unwrap_or([0.0; 4]),
            gradient_meta,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::RectInstance;

    #[test]
    fn instancing_uploads_one_record_instead_of_six_expanded_vertices() {
        // The previous path repeated the complete 92-byte payload for all
        // six quad vertices. Instancing keeps exactly one payload per rect.
        assert_eq!(std::mem::size_of::<RectInstance>(), 92);
        assert!(std::mem::size_of::<RectInstance>() < 92 * 6);
    }
}
