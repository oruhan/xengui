// SPDX-License-Identifier: Apache-2.0
use xengui::{ StrokeCommand, paint };

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 2],
    local_pos: [f32; 2],
    half_length: f32,
    half_thickness: f32,
    color: [f32; 4],
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
                    format: wgpu::VertexFormat::Float32,
                },
                wgpu::VertexAttribute {
                    shader_location: 3,
                    offset: 20,
                    format: wgpu::VertexFormat::Float32,
                },
                wgpu::VertexAttribute {
                    shader_location: 4,
                    offset: 24,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

pub struct StrokePipeline {
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    vertex_capacity: usize,
    write_offset: usize,
}

const VERTICES_PER_STROKE: usize = 6;
const DEFAULT_STROKE_CAPACITY: usize = 64;

impl StrokePipeline {
    pub fn new(
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
        sample_count: u32
    ) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Stroke Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/stroke.wgsl").into()),
        });

        let layout = device.create_pipeline_layout(
            &(wgpu::PipelineLayoutDescriptor {
                label: Some("Stroke Pipeline Layout"),
                bind_group_layouts: &[],
                immediate_size: 0,
            })
        );

        let pipeline = device.create_render_pipeline(
            &(wgpu::RenderPipelineDescriptor {
                label: Some("Stroke Pipeline"),
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
                            blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                            write_mask: wgpu::ColorWrites::ALL,
                        }),
                    ],
                }),
                multiview_mask: None,
                cache: None,
            })
        );

        let vertex_capacity = DEFAULT_STROKE_CAPACITY * VERTICES_PER_STROKE;
        let vertex_buffer = device.create_buffer(
            &(wgpu::BufferDescriptor {
                label: Some("Stroke Vertex Buffer"),
                size: (vertex_capacity * std::mem::size_of::<Vertex>()) as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })
        );

        Self { pipeline, vertex_buffer, vertex_capacity, write_offset: 0 }
    }

    pub fn reset_frame(&mut self) {
        self.write_offset = 0;
    }

    pub fn draw_batch(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        render_pass: &mut wgpu::RenderPass<'_>,
        surface_width: u32,
        surface_height: u32,
        cmds: &[StrokeCommand]
    ) {
        if cmds.is_empty() {
            return;
        }

        let inv_w = 2.0 / (surface_width.max(1) as f32);
        let inv_h = 2.0 / (surface_height.max(1) as f32);
        let ndc = |px: f32, py: f32| -> [f32; 2] { [px * inv_w - 1.0, 1.0 - py * inv_h] };

        let mut vertices = Vec::with_capacity(cmds.len() * VERTICES_PER_STROKE);

        for cmd in cmds {
            let (x0, y0) = cmd.p0;
            let (x1, y1) = cmd.p1;
            let dx = x1 - x0;
            let dy = y1 - y0;
            let len = (dx * dx + dy * dy).sqrt().max(0.0001);
            let (ux, uy) = (dx / len, dy / len);
            let (nx, ny) = (-uy, ux);

            let radius = cmd.thickness * 0.5;
            // Room for the SDF fragment shader to smooth the edge instead
            // of clipping it hard at the quad boundary.
            let pad = radius + 2.0;
            let half_length = len * 0.5;
            let along = half_length + pad;
            let across = radius + pad;
            let (cx, cy) = ((x0 + x1) * 0.5, (y0 + y1) * 0.5);
            let color = cmd.color.to_f32_array();

            let world = |along_sign: f32, across_sign: f32| -> (f32, f32) {
                (
                    cx + ux * along * along_sign + nx * across * across_sign,
                    cy + uy * along * along_sign + ny * across * across_sign,
                )
            };

            let mk = |along_sign: f32, across_sign: f32| {
                let (wx, wy) = world(along_sign, across_sign);
                Vertex {
                    position: ndc(wx, wy),
                    local_pos: [along * along_sign, across * across_sign],
                    half_length,
                    half_thickness: radius,
                    color,
                }
            };

            vertices.extend_from_slice(
                &[
                    mk(-1.0, -1.0),
                    mk(1.0, -1.0),
                    mk(-1.0, 1.0),
                    mk(-1.0, 1.0),
                    mk(1.0, -1.0),
                    mk(1.0, 1.0),
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
            (base_vertex + start * VERTICES_PER_STROKE) as u32..(base_vertex +
                end * VERTICES_PER_STROKE) as u32,
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
                label: Some("Stroke Vertex Buffer"),
                size: (self.vertex_capacity * std::mem::size_of::<Vertex>()) as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })
        );
    }
}
