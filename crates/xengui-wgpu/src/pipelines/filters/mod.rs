// SPDX-License-Identifier: Apache-2.0
//! GPU filter engine: turns a [`xengui::FilterChain`] into a sequence of
//! offscreen render passes over pooled intermediate textures.
//!
//! Architecture: the chain is split into contiguous segments — a run of
//! pointwise color ops becomes one [`ColorFilterPass`], and each
//! [`xengui::Filter::Blur`]/[`xengui::Filter::DropShadow`] becomes its own
//! two-pass [`BlurPass`] (plus, for drop shadow, a composite via
//! [`BlitPass`]). Segments ping-pong through a small pool of reusable
//! textures sized to the source, so a filtered widget never allocates a
//! new GPU texture on frames where its size hasn't changed.
mod color_pass;
mod blur_pass;
mod blit_pass;
mod texture_pool;

pub use color_pass::ColorFilterPass;
pub use blur_pass::BlurPass;
pub use blit_pass::BlitPass;
use texture_pool::TexturePool;

use xengui::{ Filter, FilterChain };

/// Result of running a [`FilterEngine`] over a source texture: the final
/// filtered texture plus how far its content extends past the widget's
/// own logical bounds (blur/drop-shadow grow the visible footprint).
pub struct FilterOutput {
    pub view: wgpu::TextureView,
    pub width: u32,
    pub height: u32,
    /// Extra padding (in physical px) added on every side beyond the
    /// widget's own bounds, so the caller can offset when compositing.
    pub padding: f32,
}

/// Owns every GPU resource a [`FilterChain`] needs and orchestrates
/// running one over a source texture. Created once per [`crate::WgpuPipelines`]
/// and reused across frames; call [`FilterEngine::reset_frame`] once per
/// frame to release textures that weren't reused.
pub struct FilterEngine {
    color: ColorFilterPass,
    blur: BlurPass,
    blit: BlitPass,
    pool: TexturePool,
}

impl FilterEngine {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        Self {
            color: ColorFilterPass::new(device, format),
            blur: BlurPass::new(device, format),
            blit: BlitPass::new(device, format),
            pool: TexturePool::new(format),
        }
    }

    pub fn reset_frame(&mut self) {
        self.pool.reset_frame();
    }

    /// Runs `chain` over `source`, returning the filtered result. `source`
    /// must already contain the widget's straight-alpha rendered content
    /// at `(src_w, src_h)`; the returned texture is premultiplied alpha,
    /// ready to be composited with standard alpha blending.
    #[allow(clippy::too_many_arguments)]
    pub fn apply(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        source: &wgpu::TextureView,
        src_w: u32,
        src_h: u32,
        chain: &FilterChain,
        scale_factor: f32
    ) -> FilterOutput {
        let padding_px = (chain.max_blur_radius() * scale_factor * 3.0).ceil();
        let out_w = src_w + (padding_px as u32) * 2;
        let out_h = src_h + (padding_px as u32) * 2;

        // Composites `source` centered into a padded working texture so
        // every subsequent pass has headroom for blur without clipping.
        let mut current = self.pool.acquire(device, out_w, out_h);
        self.blit.run(
            device,
            queue,
            encoder,
            source,
            &current.view,
            out_w,
            out_h,
            (padding_px / (out_w as f32), padding_px / (out_h as f32)),
            (src_w, src_h),
            None
        );

        let mut segment: Vec<&Filter> = Vec::new();

        for filter in chain.iter() {
            if filter.requires_blur_pass() {
                if !segment.is_empty() {
                    current = self.run_color_segment(
                        device,
                        queue,
                        encoder,
                        &current,
                        &segment,
                        out_w,
                        out_h
                    );
                    segment.clear();
                }
                current = match filter {
                    Filter::Blur(radius) => {
                        self.blur.run(
                            device,
                            queue,
                            encoder,
                            &current,
                            out_w,
                            out_h,
                            radius.value() * scale_factor,
                            &mut self.pool
                        )
                    }
                    Filter::DropShadow(shadow) => {
                        self.run_drop_shadow(
                            device,
                            queue,
                            encoder,
                            &current,
                            out_w,
                            out_h,
                            shadow,
                            scale_factor
                        )
                    }
                    _ => unreachable!("requires_blur_pass() only true for Blur/DropShadow"),
                };
            } else {
                segment.push(filter);
            }
        }

        if !segment.is_empty() {
            current = self.run_color_segment(
                device,
                queue,
                encoder,
                &current,
                &segment,
                out_w,
                out_h
            );
        }

        FilterOutput {
            view: current.view.clone(),
            width: out_w,
            height: out_h,
            padding: padding_px,
        }
    }

    fn run_color_segment(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        source: &texture_pool::PooledTexture,
        segment: &[&Filter],
        w: u32,
        h: u32
    ) -> texture_pool::PooledTexture {
        let target = self.pool.acquire(device, w, h);
        self.color.run(device, queue, encoder, &source.view, &target.view, w, h, segment);
        target
    }

    #[allow(clippy::too_many_arguments)]
    fn run_drop_shadow(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        source: &texture_pool::PooledTexture,
        w: u32,
        h: u32,
        shadow: &xengui::DropShadow,
        scale_factor: f32
    ) -> texture_pool::PooledTexture {
        // 1. Extract a tinted silhouette from the source's alpha channel.
        let silhouette = self.pool.acquire(device, w, h);
        self.blit.run(
            device,
            queue,
            encoder,
            &source.view,
            &silhouette.view,
            w,
            h,
            (0.0, 0.0),
            (w, h),
            Some(shadow.color)
        );

        // 2. Blur the silhouette.
        let blurred = self.blur.run(
            device,
            queue,
            encoder,
            &silhouette,
            w,
            h,
            shadow.blur_radius.value() * scale_factor,
            &mut self.pool
        );

        // 3. Composite: blurred silhouette offset, then original on top.
        let composited = self.pool.acquire(device, w, h);
        let offset_uv = (
            -shadow.offset_x.to_physical(scale_factor) / (w as f32),
            -shadow.offset_y.to_physical(scale_factor) / (h as f32),
        );
        self.blit.run(
            device,
            queue,
            encoder,
            &blurred.view,
            &composited.view,
            w,
            h,
            offset_uv,
            (w, h),
            None
        );
        self.blit.run_over(device, queue, encoder, &source.view, &composited.view, w, h);

        composited
    }
}
