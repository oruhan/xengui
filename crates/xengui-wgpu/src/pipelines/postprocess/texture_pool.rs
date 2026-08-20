// SPDX-License-Identifier: Apache-2.0
//! Reuses same-sized offscreen textures across a chain's intermediate
//! passes and across frames, instead of allocating fresh GPU memory on
//! every filter pass. Textures not reused within a frame are dropped at
//! the next `reset_frame`.

pub struct PooledTexture {
    pub texture: std::sync::Arc<wgpu::Texture>,
    pub view: wgpu::TextureView,
    pub width: u32,
    pub height: u32,
}

impl Clone for PooledTexture {
    fn clone(&self) -> Self {
        Self {
            texture: self.texture.clone(),
            view: self.view.clone(),
            width: self.width,
            height: self.height,
        }
    }
}

// Free-list cap and idle timeout keep the pool from growing forever when
// many distinct sizes pass through it (e.g. continuous window resizing).
const MAX_FREE_TEXTURES: usize = 32;
const MAX_IDLE_FRAMES: u32 = 120;

pub struct TexturePool {
    format: wgpu::TextureFormat,
    // Each entry's second field counts frames since it was last used;
    // reset to 0 whenever it's handed out again via `acquire`.
    free: Vec<(PooledTexture, u32)>,
    used_this_frame: Vec<PooledTexture>,
}

impl TexturePool {
    pub fn new(format: wgpu::TextureFormat) -> Self {
        Self { format, free: Vec::new(), used_this_frame: Vec::new() }
    }

    pub fn reset_frame(&mut self) {
        for texture in self.used_this_frame.drain(..) {
            self.free.push((texture, 0));
        }

        for (_, idle_frames) in self.free.iter_mut() {
            *idle_frames += 1;
        }
        self.free.retain(|(_, idle_frames)| *idle_frames <= MAX_IDLE_FRAMES);

        if self.free.len() > MAX_FREE_TEXTURES {
            self.free.sort_by_key(|(_, idle_frames)| std::cmp::Reverse(*idle_frames));
            self.free.truncate(MAX_FREE_TEXTURES);
        }
    }

    pub fn acquire(&mut self, device: &wgpu::Device, width: u32, height: u32) -> PooledTexture {
        if
            let Some(idx) = self.free
                .iter()
                .position(|(t, _)| t.width == width && t.height == height)
        {
            let (texture, _) = self.free.remove(idx);
            self.used_this_frame.push(texture.clone());
            return texture;
        }

        let texture = device.create_texture(
            &(wgpu::TextureDescriptor {
                label: Some("xengui filter intermediate texture"),
                size: wgpu::Extent3d {
                    width: width.max(1),
                    height: height.max(1),
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: self.format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT |
                wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            })
        );
        let view = texture.create_view(&Default::default());
        let pooled = PooledTexture { texture: std::sync::Arc::new(texture), view, width, height };
        self.used_this_frame.push(pooled.clone());
        pooled
    }
}
