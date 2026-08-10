// SPDX-License-Identifier: Apache-2.0
//! Rasterizes Material Symbols (or any other variable font) glyphs
//! straight through `swash`, independent of the glyphon/cosmic-text text
//! pipeline - cosmic-text's own font selection only resolves the
//! standard weight/style/stretch axes, not the custom GRAD/opsz/FILL
//! axes Material Symbols needs blended continuously.
use std::collections::HashMap;
use swash::scale::{ Render, ScaleContext, Source };
use swash::zeno::Format;
use swash::FontRef;
use xengui::{ paint, VariableIconCommand };
use std::sync::Arc;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 2],
    uv: [f32; 2],
    tint: [f32; 4],
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
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct GlyphKey {
    font_ptr: usize,
    codepoint: u32,
    size_bits: u32,
    axes_hash: u64,
}

struct CachedGlyph {
    bind_group: wgpu::BindGroup,
    size: (f32, f32),
}

pub struct VariableIconPipeline {
    pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    vertex_buffer: wgpu::Buffer,
    vertex_capacity: usize,
    write_offset: usize,
    glyphs: HashMap<GlyphKey, CachedGlyph>,
    scale_context: ScaleContext,
    // WOFF2 containers must be unpacked into raw TTF/OTF bytes before
    // swash can parse them; decoded once per font pointer and reused.
    decoded_fonts: HashMap<usize, Arc<Vec<u8>>>,
}
const VERTICES_PER_ICON: usize = 6;
const DEFAULT_ICON_CAPACITY: usize = 64;

impl VariableIconPipeline {
    pub fn new(
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
        sample_count: u32
    ) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Variable Icon Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/variable_icon.wgsl").into()),
        });

        let bind_group_layout = device.create_bind_group_layout(
            &(wgpu::BindGroupLayoutDescriptor {
                label: Some("Variable Icon Bind Group Layout"),
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

        let layout = device.create_pipeline_layout(
            &(wgpu::PipelineLayoutDescriptor {
                label: Some("Variable Icon Pipeline Layout"),
                bind_group_layouts: &[Some(&bind_group_layout)],
                immediate_size: 0,
            })
        );

        let pipeline = device.create_render_pipeline(
            &(wgpu::RenderPipelineDescriptor {
                label: Some("Variable Icon Pipeline"),
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

        let sampler = device.create_sampler(
            &(wgpu::SamplerDescriptor {
                label: Some("xengui variable icon sampler"),
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                ..Default::default()
            })
        );

        let vertex_capacity = DEFAULT_ICON_CAPACITY * VERTICES_PER_ICON;
        let vertex_buffer = device.create_buffer(
            &(wgpu::BufferDescriptor {
                label: Some("Variable Icon Vertex Buffer"),
                size: (vertex_capacity * std::mem::size_of::<Vertex>()) as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })
        );

        Self {
            pipeline,
            bind_group_layout,
            sampler,
            vertex_buffer,
            vertex_capacity,
            write_offset: 0,
            glyphs: HashMap::new(),
            scale_context: ScaleContext::new(),
            decoded_fonts: HashMap::new(),
        }
    }

    pub fn reset_frame(&mut self) {
        self.write_offset = 0;
    }

    // swash only parses raw TTF/OTF outline tables, not WOFF2's own
    // compressed container - unpacks (and caches by font pointer) once
    // per distinct font instead of on every glyph rasterization.
    fn decoded_font_bytes(&mut self, font: &'static [u8]) -> Arc<Vec<u8>> {
        let key = font.as_ptr() as usize;
        if let Some(bytes) = self.decoded_fonts.get(&key) {
            return bytes.clone();
        }

        let decoded = if woff2_patched::decode::is_woff2(font) {
            match woff2_patched::decode::convert_woff2_to_ttf(&mut std::io::Cursor::new(font)) {
                Ok(ttf) => ttf,
                Err(err) => {
                    log::error!("xengui-wgpu: failed to decode woff2 font: {err:?}");
                    font.to_vec()
                }
            }
        } else {
            font.to_vec()
        };

        let decoded = Arc::new(decoded);
        self.decoded_fonts.insert(key, decoded.clone());
        decoded
    }

    // Rasterizes (or reuses a cached rasterization of) the glyph a
    // command asks for, uploading a fresh alpha-mask texture only the
    // first time a given (font, codepoint, size, axes) combination is seen.
    fn ensure_glyph(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        cmd: &VariableIconCommand
    ) -> Option<GlyphKey> {
        let physical_size = cmd.size.1.max(cmd.size.0);
        let key = GlyphKey {
            font_ptr: cmd.font.as_ptr() as usize,
            codepoint: cmd.codepoint as u32,
            size_bits: physical_size.to_bits(),
            axes_hash: cmd.axes.cache_key(),
        };

        if self.glyphs.contains_key(&key) {
            return Some(key);
        }

        let font_bytes = self.decoded_font_bytes(cmd.font);
        let font = FontRef::from_index(&font_bytes, 0)?;
        let glyph_id = font.charmap().map(cmd.codepoint);

        if glyph_id == 0 {
            log::warn!(
                "xengui-wgpu: VariableIcon codepoint U+{:04X} not found in font",
                cmd.codepoint as u32
            );
            return None;
        }

        let variation_settings: Vec<(swash::Tag, f32)> = cmd.axes
            .to_variations()
            .into_iter()
            .map(|(tag, value)| { (u32::from_be_bytes(tag), value) })
            .collect();

        let mut scaler = self.scale_context
            .builder(font)
            .size(physical_size)
            .hint(true)
            .variations(variation_settings)
            .build();

        let image = Render::new(&[Source::Outline])
            .format(Format::Alpha)
            .render(&mut scaler, glyph_id)?;

        if image.placement.width == 0 || image.placement.height == 0 {
            return None;
        }

        let width = image.placement.width;
        let height = image.placement.height;

        let texture = device.create_texture(
            &(wgpu::TextureDescriptor {
                label: Some("xengui variable icon glyph"),
                size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::R8Unorm,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            })
        );

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &image.data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(width),
                rows_per_image: Some(height),
            },
            wgpu::Extent3d { width, height, depth_or_array_layers: 1 }
        );

        let view = texture.create_view(&Default::default());
        let bind_group = device.create_bind_group(
            &(wgpu::BindGroupDescriptor {
                label: Some("xengui variable icon bind group"),
                layout: &self.bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&self.sampler),
                    },
                ],
            })
        );

        self.glyphs.insert(key, CachedGlyph {
            bind_group,
            size: (width as f32, height as f32),
        });

        Some(key)
    }

    pub fn draw_batch(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        render_pass: &mut wgpu::RenderPass<'_>,
        surface_width: u32,
        surface_height: u32,
        cmds: &[VariableIconCommand]
    ) {
        if cmds.is_empty() {
            return;
        }

        let inv_w = 2.0 / (surface_width.max(1) as f32);
        let inv_h = 2.0 / (surface_height.max(1) as f32);
        let ndc = |px: f32, py: f32| -> [f32; 2] { [px * inv_w - 1.0, 1.0 - py * inv_h] };

        let mut vertices = Vec::with_capacity(cmds.len() * VERTICES_PER_ICON);
        let mut keys: Vec<Option<GlyphKey>> = Vec::with_capacity(cmds.len());

        for cmd in cmds {
            let Some(key) = self.ensure_glyph(device, queue, cmd) else {
                keys.push(None);
                continue;
            };
            let glyph = &self.glyphs[&key];

            // Centers the rasterized glyph's own bounding box within the
            // requested icon box - font bearing cancels out algebraically
            // for box-centering, so it's not needed here (unlike normal
            // baseline-aligned text layout).
            let cx = cmd.position.0 + cmd.size.0 * 0.5;
            let cy = cmd.position.1 + cmd.size.1 * 0.5;
            let gx = (cx - glyph.size.0 * 0.5).round();
            let gy = (cy + glyph.size.1 * 0.5).round();

            let tint = cmd.color.to_f32_array();
            let p0 = ndc(gx, gy - glyph.size.1);
            let p1 = ndc(gx + glyph.size.0, gy - glyph.size.1);
            let p2 = ndc(gx, gy);
            let p3 = ndc(gx + glyph.size.0, gy);

            let mk = |screen: [f32; 2], uv: [f32; 2]| Vertex { position: screen, uv, tint };
            vertices.extend_from_slice(
                &[
                    mk(p0, [0.0, 0.0]),
                    mk(p1, [1.0, 0.0]),
                    mk(p2, [0.0, 1.0]),
                    mk(p2, [0.0, 1.0]),
                    mk(p1, [1.0, 0.0]),
                    mk(p3, [1.0, 1.0]),
                ]
            );
            keys.push(Some(key));
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

        let mut vertex_cursor = base_vertex;
        for (cmd, key) in cmds.iter().zip(keys.iter()) {
            let Some(key) = key else {
                continue;
            };
            let glyph = &self.glyphs[key];

            let (sx, sy, sw, sh) = paint::draw_command::scissor_for_clip(
                cmd.clip_rect,
                surface_width,
                surface_height
            );
            if sw == 0 || sh == 0 {
                vertex_cursor += VERTICES_PER_ICON;
                continue;
            }
            render_pass.set_scissor_rect(sx, sy, sw, sh);
            render_pass.set_bind_group(0, &glyph.bind_group, &[]);
            render_pass.draw(
                vertex_cursor as u32..(vertex_cursor + VERTICES_PER_ICON) as u32,
                0..1
            );
            vertex_cursor += VERTICES_PER_ICON;
        }
    }

    fn ensure_capacity(&mut self, device: &wgpu::Device, required: usize) {
        if required <= self.vertex_capacity {
            return;
        }
        self.vertex_capacity = required.next_power_of_two();
        self.vertex_buffer = device.create_buffer(
            &(wgpu::BufferDescriptor {
                label: Some("Variable Icon Vertex Buffer"),
                size: (self.vertex_capacity * std::mem::size_of::<Vertex>()) as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })
        );
    }
}
