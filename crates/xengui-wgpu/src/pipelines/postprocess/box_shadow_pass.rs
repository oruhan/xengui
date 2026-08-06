// SPDX-License-Identifier: Apache-2.0
//! Renders CSS-parity box shadows by rasterizing a rounded-rect coverage
//! mask and blurring it through the same Dual Kawase pipeline used for
//! `Filter::Blur`, instead of an analytic single-pass approximation.
use super::kawase_pass::KawasePass;
use super::texture_pool::{ PooledTexture, TexturePool };
use xengui::{ BoxShadowCommand, ShadowDirection, paint };

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct MaskVertex {
    position: [f32; 2],
    local_pos: [f32; 2],
    half_size: [f32; 2],
    radius: [f32; 4],
}

impl MaskVertex {
    fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<MaskVertex>() as u64,
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
                },
            ],
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct CompositeVertex {
    position: [f32; 2],
    local_pos: [f32; 2],
    half_size: [f32; 2],
    box_radius: f32,
    mask_uv: [f32; 2],
    color: [f32; 4],
    inset: f32,
}

impl CompositeVertex {
    fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<CompositeVertex>() as u64,
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
                    format: wgpu::VertexFormat::Float32,
                },
                wgpu::VertexAttribute {
                    shader_location: 4,
                    offset: 28,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    shader_location: 5,
                    offset: 36,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    shader_location: 6,
                    offset: 52,
                    format: wgpu::VertexFormat::Float32,
                },
            ],
        }
    }
}

const VERTICES_PER_QUAD: usize = 6;

pub struct BoxShadowEngine {
    mask_pipeline: wgpu::RenderPipeline,
    composite_pipeline: wgpu::RenderPipeline,
    composite_bind_group_layout: wgpu::BindGroupLayout,
    mask_sampler: wgpu::Sampler,
}

impl BoxShadowEngine {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let mask_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Box Shadow Mask Shader"),
            source: wgpu::ShaderSource::Wgsl(
                include_str!("../shaders/box_shadow_mask.wgsl").into()
            ),
        });
        let mask_layout = device.create_pipeline_layout(
            &(wgpu::PipelineLayoutDescriptor {
                label: Some("Box Shadow Mask Pipeline Layout"),
                bind_group_layouts: &[],
                immediate_size: 0,
            })
        );
        let mask_pipeline = device.create_render_pipeline(
            &(wgpu::RenderPipelineDescriptor {
                label: Some("Box Shadow Mask Pipeline"),
                layout: Some(&mask_layout),
                vertex: wgpu::VertexState {
                    module: &mask_shader,
                    entry_point: Some("vs_main"),
                    compilation_options: Default::default(),
                    buffers: &[Some(MaskVertex::layout())],
                },
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    ..Default::default()
                },
                depth_stencil: None,
                multisample: Default::default(),
                fragment: Some(wgpu::FragmentState {
                    module: &mask_shader,
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
        );

        let composite_bind_group_layout = device.create_bind_group_layout(
            &(wgpu::BindGroupLayoutDescriptor {
                label: Some("Box Shadow Composite Bind Group Layout"),
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
                ],
            })
        );
        let composite_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Box Shadow Composite Shader"),
            source: wgpu::ShaderSource::Wgsl(
                include_str!("../shaders/box_shadow_composite.wgsl").into()
            ),
        });
        let composite_layout = device.create_pipeline_layout(
            &(wgpu::PipelineLayoutDescriptor {
                label: Some("Box Shadow Composite Pipeline Layout"),
                bind_group_layouts: &[Some(&composite_bind_group_layout)],
                immediate_size: 0,
            })
        );
        let composite_pipeline = device.create_render_pipeline(
            &(wgpu::RenderPipelineDescriptor {
                label: Some("Box Shadow Composite Pipeline"),
                layout: Some(&composite_layout),
                vertex: wgpu::VertexState {
                    module: &composite_shader,
                    entry_point: Some("vs_main"),
                    compilation_options: Default::default(),
                    buffers: &[Some(CompositeVertex::layout())],
                },
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    ..Default::default()
                },
                depth_stencil: None,
                multisample: Default::default(),
                fragment: Some(wgpu::FragmentState {
                    module: &composite_shader,
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

        let mask_sampler = device.create_sampler(
            &(wgpu::SamplerDescriptor {
                label: Some("xengui box shadow mask sampler"),
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                ..Default::default()
            })
        );

        Self { mask_pipeline, composite_pipeline, composite_bind_group_layout, mask_sampler }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn draw_batch(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        kawase: &KawasePass,
        pool: &mut TexturePool,
        target_view: &wgpu::TextureView,
        target_width: u32,
        target_height: u32,
        cmds: &[BoxShadowCommand]
    ) {
        for cmd in cmds {
            self.draw_one(
                device,
                queue,
                encoder,
                kawase,
                pool,
                target_view,
                target_width,
                target_height,
                cmd
            );
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn draw_one(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        kawase: &KawasePass,
        pool: &mut TexturePool,
        target_view: &wgpu::TextureView,
        target_width: u32,
        target_height: u32,
        cmd: &BoxShadowCommand
    ) {
        if cmd.color.a() <= 0.0 || cmd.shadow_size.0 <= 0.0 || cmd.shadow_size.1 <= 0.0 {
            return;
        }

        // Same reach the old analytic shader used - enough headroom for
        // the Kawase chain's tent filter to fully resolve near the edges.
        let padding = cmd.blur * 3.0 + 4.0;
        let mask_w = (cmd.shadow_size.0 + padding * 2.0).ceil().max(1.0) as u32;
        let mask_h = (cmd.shadow_size.1 + padding * 2.0).ceil().max(1.0) as u32;
        let mask_origin = (cmd.shadow_position.0 - padding, cmd.shadow_position.1 - padding);

        let mask_tex = pool.acquire(device, mask_w, mask_h);
        self.render_mask(device, queue, encoder, &mask_tex, mask_w, mask_h, cmd);

        let blurred = kawase.run(
            device,
            queue,
            encoder,
            &mask_tex,
            mask_w,
            mask_h,
            cmd.blur * 0.5,
            pool
        );

        // Outset composites the whole padded mask rect directly; inset is
        // only ever visible through the box itself, so it composites over
        // the box's own rect and subtracts the blurred inner shape instead.
        let (quad_pos, quad_size, box_radius) = if cmd.inset {
            (cmd.box_position, cmd.box_size, cmd.box_radius)
        } else {
            (mask_origin, (mask_w as f32, mask_h as f32), 0.0)
        };

        self.render_composite(
            device,
            queue,
            encoder,
            &blurred,
            target_view,
            target_width,
            target_height,
            quad_pos,
            quad_size,
            box_radius,
            mask_origin,
            (mask_w as f32, mask_h as f32),
            cmd
        );
    }

    fn render_mask(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        target: &PooledTexture,
        width: u32,
        height: u32,
        cmd: &BoxShadowCommand
    ) {
        let half_w = (width as f32) * 0.5;
        let half_h = (height as f32) * 0.5;
        let shape_half = [cmd.shadow_size.0 * 0.5, cmd.shadow_size.1 * 0.5];

        let ndc = |x: f32, y: f32| -> [f32; 2] {
            [(x / (width as f32)) * 2.0 - 1.0, 1.0 - (y / (height as f32)) * 2.0]
        };

        let mk = |local: [f32; 2]| MaskVertex {
            position: ndc(local[0] + half_w, local[1] + half_h),
            local_pos: local,
            half_size: shape_half,
            radius: cmd.shadow_radius,
        };

        let vertices = [
            mk([-shape_half[0], -shape_half[1]]),
            mk([shape_half[0], -shape_half[1]]),
            mk([-shape_half[0], shape_half[1]]),
            mk([-shape_half[0], shape_half[1]]),
            mk([shape_half[0], -shape_half[1]]),
            mk([shape_half[0], shape_half[1]]),
        ];

        let vertex_buffer = device.create_buffer(
            &(wgpu::BufferDescriptor {
                label: Some("Box Shadow Mask Vertex Buffer"),
                size: (std::mem::size_of::<MaskVertex>() * VERTICES_PER_QUAD) as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })
        );
        queue.write_buffer(&vertex_buffer, 0, bytemuck::cast_slice(&vertices));

        let mut pass = encoder.begin_render_pass(
            &(wgpu::RenderPassDescriptor {
                label: Some("Box Shadow Mask Pass"),
                color_attachments: &[
                    Some(wgpu::RenderPassColorAttachment {
                        view: &target.view,
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
        pass.set_pipeline(&self.mask_pipeline);
        pass.set_vertex_buffer(0, vertex_buffer.slice(..));
        pass.set_viewport(0.0, 0.0, width as f32, height as f32, 0.0, 1.0);
        pass.draw(0..VERTICES_PER_QUAD as u32, 0..1);
    }

    #[allow(clippy::too_many_arguments)]
    fn render_composite(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        mask: &PooledTexture,
        target_view: &wgpu::TextureView,
        target_width: u32,
        target_height: u32,
        quad_pos: (f32, f32),
        quad_size: (f32, f32),
        box_radius: f32,
        mask_origin: (f32, f32),
        mask_size: (f32, f32),
        cmd: &BoxShadowCommand
    ) {
        if quad_size.0 <= 0.0 || quad_size.1 <= 0.0 {
            return;
        }

        // Inset shadows already stay confined to the box via the shader's
        // own SDF mask, so direction only needs to gate the outset case.
        let effective_clip = if cmd.inset {
            cmd.clip_rect
        } else {
            intersect_rects(
                cmd.clip_rect,
                direction_clip_rect(
                    cmd.direction,
                    cmd.box_position,
                    cmd.box_size,
                    target_width,
                    target_height
                )
            )
        };

        let (sx, sy, sw, sh) = paint::draw_command::scissor_for_clip(
            effective_clip,
            target_width,
            target_height
        );

        if sw == 0 || sh == 0 {
            return;
        }

        let half_size = [quad_size.0 * 0.5, quad_size.1 * 0.5];
        let center = (quad_pos.0 + half_size[0], quad_pos.1 + half_size[1]);
        let color = cmd.color.to_f32_array();
        let inset = if cmd.inset { 1.0 } else { 0.0 };

        let inv_w = 2.0 / (target_width.max(1) as f32);
        let inv_h = 2.0 / (target_height.max(1) as f32);
        let ndc = |x: f32, y: f32| -> [f32; 2] { [x * inv_w - 1.0, 1.0 - y * inv_h] };
        let mask_uv = |wx: f32, wy: f32| -> [f32; 2] {
            [(wx - mask_origin.0) / mask_size.0, (wy - mask_origin.1) / mask_size.1]
        };

        let corners = [
            (quad_pos.0, quad_pos.1),
            (quad_pos.0 + quad_size.0, quad_pos.1),
            (quad_pos.0, quad_pos.1 + quad_size.1),
            (quad_pos.0 + quad_size.0, quad_pos.1 + quad_size.1),
        ];

        let mk = |world: (f32, f32)| CompositeVertex {
            position: ndc(world.0, world.1),
            local_pos: [world.0 - center.0, world.1 - center.1],
            half_size,
            box_radius,
            mask_uv: mask_uv(world.0, world.1),
            color,
            inset,
        };

        let vertices = [
            mk(corners[0]),
            mk(corners[1]),
            mk(corners[2]),
            mk(corners[2]),
            mk(corners[1]),
            mk(corners[3]),
        ];

        let vertex_buffer = device.create_buffer(
            &(wgpu::BufferDescriptor {
                label: Some("Box Shadow Composite Vertex Buffer"),
                size: (std::mem::size_of::<CompositeVertex>() * VERTICES_PER_QUAD) as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })
        );
        queue.write_buffer(&vertex_buffer, 0, bytemuck::cast_slice(&vertices));

        let bind_group = device.create_bind_group(
            &(wgpu::BindGroupDescriptor {
                label: Some("Box Shadow Composite Bind Group"),
                layout: &self.composite_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&mask.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&self.mask_sampler),
                    },
                ],
            })
        );

        let mut pass = encoder.begin_render_pass(
            &(wgpu::RenderPassDescriptor {
                label: Some("Box Shadow Composite Pass"),
                color_attachments: &[
                    Some(wgpu::RenderPassColorAttachment {
                        view: target_view,
                        resolve_target: None,
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
        pass.set_pipeline(&self.composite_pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.set_vertex_buffer(0, vertex_buffer.slice(..));
        pass.set_viewport(0.0, 0.0, target_width as f32, target_height as f32, 0.0, 1.0);
        pass.set_scissor_rect(sx, sy, sw, sh);
        pass.draw(0..VERTICES_PER_QUAD as u32, 0..1);
    }
}

// Padding available on each side (left, top, right, bottom) for the
// blur reach of an outset shadow, given the requested direction. `All`
// keeps the usual symmetric halo; every other value zeroes the side(s)
// the shadow shouldn't extend into, so capture bounds stay tight to
// what's actually visible after the composite-time scissor clip below.
pub(crate) fn directional_shadow_padding(
    direction: ShadowDirection,
    full: f32
) -> (f32, f32, f32, f32) {
    use ShadowDirection::*;
    match direction {
        All => (full, full, full, full),
        Top => (full, full, full, 0.0),
        Bottom => (full, 0.0, full, full),
        Left => (full, full, 0.0, full),
        Right => (0.0, full, full, full),
        TopLeft => (full, full, 0.0, 0.0),
        TopRight => (0.0, full, full, 0.0),
        BottomLeft => (full, 0.0, 0.0, full),
        BottomRight => (0.0, 0.0, full, full),
    }
}

// Restricts an outset shadow's visible composite region to one side (or
// corner quadrant) of the box it belongs to - e.g. `Top` never paints
// below the box's own top edge, regardless of how far the blurred mask
// itself extends.
fn direction_clip_rect(
    direction: ShadowDirection,
    box_position: (f32, f32),
    box_size: (f32, f32),
    target_width: u32,
    target_height: u32
) -> Option<(f32, f32, f32, f32)> {
    use ShadowDirection::*;
    let (bx, by) = box_position;
    let (bw, bh) = box_size;
    let (tw, th) = (target_width as f32, target_height as f32);
    match direction {
        All => None,
        Top => Some((0.0, 0.0, tw, by)),
        Bottom => Some((0.0, by + bh, tw, th - (by + bh))),
        Left => Some((0.0, 0.0, bx, th)),
        Right => Some((bx + bw, 0.0, tw - (bx + bw), th)),
        TopLeft => Some((0.0, 0.0, bx, by)),
        TopRight => Some((bx + bw, 0.0, tw - (bx + bw), by)),
        BottomLeft => Some((0.0, by + bh, bx, th - (by + bh))),
        BottomRight => Some((bx + bw, by + bh, tw - (bx + bw), th - (by + bh))),
    }
}

fn intersect_rects(
    a: Option<(f32, f32, f32, f32)>,
    b: Option<(f32, f32, f32, f32)>
) -> Option<(f32, f32, f32, f32)> {
    match (a, b) {
        (None, None) => None,
        (Some(r), None) | (None, Some(r)) => Some(r),
        (Some((ax, ay, aw, ah)), Some((bx, by, bw, bh))) => {
            let x0 = ax.max(bx);
            let y0 = ay.max(by);
            let x1 = (ax + aw).min(bx + bw);
            let y1 = (ay + ah).min(by + bh);
            Some((x0, y0, (x1 - x0).max(0.0), (y1 - y0).max(0.0)))
        }
    }
}
