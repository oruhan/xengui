// SPDX-License-Identifier: Apache-2.0
use xengui::{ Background, GradientStop, RectCommand, paint };

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 2],
    local_pos: [f32; 2],
    half_size: [f32; 2],
    radius: [f32; 4],
    border_width: f32,
    fill_color: [f32; 4],
    border_color: [f32; 4],
    // x: kind (0 = solid, 1 = linear, 2 = radial), y: linear angle (rad),
    // z: stop count, w: offset into the shared gradient stop buffers.
    gradient_meta: [f32; 4],
}

impl Vertex {
    fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    shader_location: 0,
                    offset: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    shader_location: 1,
                    offset: 8,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    shader_location: 2,
                    offset: 16,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    shader_location: 3,
                    offset: 24,
                    format: wgpu::VertexFormat::Float32x4,
                }, // radius
                wgpu::VertexAttribute {
                    shader_location: 4,
                    offset: 40,
                    format: wgpu::VertexFormat::Float32,
                }, // border_width
                wgpu::VertexAttribute {
                    shader_location: 5,
                    offset: 44,
                    format: wgpu::VertexFormat::Float32x4,
                }, // fill_color
                wgpu::VertexAttribute {
                    shader_location: 6,
                    offset: 60,
                    format: wgpu::VertexFormat::Float32x4,
                }, // border_color
                wgpu::VertexAttribute {
                    shader_location: 7,
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
}

const VERTICES_PER_RECT: usize = 6;
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
        sample_count: u32
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
            })
        );

        let layout = device.create_pipeline_layout(
            &(wgpu::PipelineLayoutDescriptor {
                label: Some("Rect Pipeline Layout"),
                bind_group_layouts: &[Some(&gradient_bind_group_layout)],
                immediate_size: 0,
            })
        );

        let pipeline = device.create_render_pipeline(
            &(wgpu::RenderPipelineDescriptor {
                label: Some("Rect Pipeline"),
                layout: Some(&layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"),
                    compilation_options: Default::default(),
                    buffers: &[Some(Vertex::layout())],
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
                    targets: &[
                        Some(wgpu::ColorTargetState {
                            format: surface_format,
                            blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                            write_mask: wgpu::ColorWrites::ALL,
                        }),
                    ],
                }),
                multiview_mask: None,
                cache: None,
            })
        );

        let vertex_capacity = DEFAULT_RECT_CAPACITY * VERTICES_PER_RECT;
        let vertex_buffer = device.create_buffer(
            &(wgpu::BufferDescriptor {
                label: Some("Rect Vertex Buffer"),
                size: (vertex_capacity * std::mem::size_of::<Vertex>()) as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })
        );

        let gradient_positions_buffer = device.create_buffer(
            &(wgpu::BufferDescriptor {
                label: Some("Rect Gradient Positions Buffer"),
                size: (MAX_GRADIENT_POSITION_VEC4S * 16) as u64,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })
        );

        let gradient_colors_buffer = device.create_buffer(
            &(wgpu::BufferDescriptor {
                label: Some("Rect Gradient Colors Buffer"),
                size: (MAX_GRADIENT_STOPS_TOTAL * 16) as u64,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })
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
            })
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
        cmds: &[RectCommand]
    ) {
        if cmds.is_empty() {
            return;
        }

        let mut new_positions: Vec<f32> = Vec::new();
        let mut new_colors: Vec<[f32; 4]> = Vec::new();
        let mut metas: Vec<[f32; 4]> = Vec::with_capacity(cmds.len());

        for cmd in cmds {
            let (kind, angle, stops): (f32, f32, &[GradientStop]) = match cmd.background.as_ref() {
                Some(Background::LinearGradient(g)) =>
                    (1.0, g.angle_deg.to_radians(), g.stops.as_slice()),
                Some(Background::RadialGradient(g)) => (2.0, 0.0, g.stops.as_slice()),
                _ => (0.0, 0.0, &[]),
            };

            if stops.is_empty() {
                metas.push([kind, angle, 0.0, 0.0]);
                continue;
            }

            let already_used = self.stops_used + new_colors.len();
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
                new_positions.push(stop.position);
                new_colors.push(stop.color.to_f32_array());
            }

            metas.push([kind, angle, take as f32, already_used as f32]);
        }

        if !new_colors.is_empty() {
            let base_offset = self.stops_used;
            queue.write_buffer(
                &self.gradient_positions_buffer,
                (base_offset * 4) as u64,
                bytemuck::cast_slice(&new_positions)
            );
            queue.write_buffer(
                &self.gradient_colors_buffer,
                (base_offset * 16) as u64,
                bytemuck::cast_slice(&new_colors)
            );
            self.stops_used += new_colors.len();
        }

        let inv_w = 2.0 / (surface_width.max(1) as f32);
        let inv_h = 2.0 / (surface_height.max(1) as f32);
        let ndc = |px: f32, py: f32| -> [f32; 2] { [px * inv_w - 1.0, 1.0 - py * inv_h] };

        let mut vertices = Vec::with_capacity(cmds.len() * VERTICES_PER_RECT);

        // Extra headroom (physical px) rasterized beyond each rect's true
        // edge, so a shape whose antialiasing fringe would otherwise land
        // exactly on the quad boundary - most visibly a full circle inscribed
        // in its own box, e.g. the Switch thumb - has room to fade to
        // transparent instead of hard-clipping into a flat facet.
        const AA_PAD: f32 = 1.5;

        for (i, cmd) in cmds.iter().enumerate() {
            let fill_color = cmd.background
                .as_ref()
                .map(|bg| bg.representative_color().to_f32_array())
                .unwrap_or([0.0, 0.0, 0.0, 0.0]);

            let gradient_meta = metas[i];

            let (x, y) = cmd.position;
            let (w, h) = cmd.size;
            let half_w = w * 0.5;
            let half_h = h * 0.5;

            let radius = cmd.border_radius
                .map(|r| r.to_physical_array(1.0, w, h))
                .unwrap_or([0.0; 4]);

            let border_width = cmd.border_width.map(|bw| bw.value()).unwrap_or(0.0);
            let border_color = cmd.border_color
                .map(|c| c.to_f32_array())
                .unwrap_or([0.0, 0.0, 0.0, 0.0]);

            // half_size stays the shape's real (unpadded) bounds - only the
            // rasterized quad grows by AA_PAD, so the SDF is unaffected and
            // just gets evaluated a bit past its own edge.
            let half_size = [half_w, half_h];
            let outer_w = half_w + AA_PAD;
            let outer_h = half_h + AA_PAD;
            let p0 = ndc(x - AA_PAD, y - AA_PAD);
            let p1 = ndc(x + w + AA_PAD, y - AA_PAD);
            let p2 = ndc(x - AA_PAD, y + h + AA_PAD);
            let p3 = ndc(x + w + AA_PAD, y + h + AA_PAD);

            let local = |lx: f32, ly: f32| [lx, ly];

            let mk = |screen: [f32; 2], local_pos: [f32; 2]| Vertex {
                position: screen,
                local_pos,
                half_size,
                radius,
                border_width,
                fill_color,
                border_color,
                gradient_meta,
            };

            vertices.extend_from_slice(
                &[
                    mk(p0, local(-outer_w, -outer_h)),
                    mk(p1, local(outer_w, -outer_h)),
                    mk(p2, local(-outer_w, outer_h)),
                    mk(p2, local(-outer_w, outer_h)),
                    mk(p1, local(outer_w, -outer_h)),
                    mk(p3, local(outer_w, outer_h)),
                ]
            );
        }

        let base_vertex = self.write_offset;
        self.ensure_capacity(device, base_vertex + vertices.len());
        queue.write_buffer(
            &self.vertex_buffer,
            (base_vertex * std::mem::size_of::<Vertex>()) as u64,
            bytemuck::cast_slice(&vertices)
        );
        self.write_offset += vertices.len();

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.gradient_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_viewport(0.0, 0.0, surface_width as f32, surface_height as f32, 0.0, 1.0);

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
                    surface_height
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
            surface_height
        );
    }

    fn draw_run(
        render_pass: &mut wgpu::RenderPass<'_>,
        base_vertex: usize,
        start: usize,
        end: usize,
        clip: Option<(f32, f32, f32, f32)>,
        surface_width: u32,
        surface_height: u32
    ) {
        let (sx, sy, sw, sh) = paint::draw_command::scissor_for_clip(
            clip,
            surface_width,
            surface_height
        );
        if sw == 0 || sh == 0 {
            return;
        }
        render_pass.set_scissor_rect(sx, sy, sw, sh);
        render_pass.draw(
            (base_vertex + start * VERTICES_PER_RECT) as u32..(base_vertex +
                end * VERTICES_PER_RECT) as u32,
            0..1
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
                size: (self.vertex_capacity * std::mem::size_of::<Vertex>()) as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })
        );
    }
}
