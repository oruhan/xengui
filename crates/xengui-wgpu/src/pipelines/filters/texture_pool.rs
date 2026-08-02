// SPDX-License-Identifier: Apache-2.0
//! Reuses same-sized offscreen textures across a chain's intermediate
//! passes and across frames, instead of allocating fresh GPU memory on
//! every filter pass. Textures not reused within a frame are dropped at
//! the next `reset_frame`.

pub struct PooledTexture {
    pub texture: std::rc::Rc<wgpu::Texture>,
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

pub struct TexturePool {
    format: wgpu::TextureFormat,
    free: Vec<PooledTexture>,
    used_this_frame: Vec<PooledTexture>,
}

impl TexturePool {
    pub fn new(format: wgpu::TextureFormat) -> Self {
        Self { format, free: Vec::new(), used_this_frame: Vec::new() }
    }

    pub fn reset_frame(&mut self) {
        self.free.extend(self.used_this_frame.drain(..));
    }

    pub fn acquire(&mut self, device: &wgpu::Device, width: u32, height: u32) -> PooledTexture {
        if let Some(idx) = self.free.iter().position(|t| t.width == width && t.height == height) {
            let tex = self.free.remove(idx);
            self.used_this_frame.push(tex.clone());
            return tex;
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
        let pooled = PooledTexture { texture: std::rc::Rc::new(texture), view, width, height };
        self.used_this_frame.push(pooled.clone());
        pooled
    }
}
