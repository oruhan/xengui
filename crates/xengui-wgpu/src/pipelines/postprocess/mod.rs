// SPDX-License-Identifier: Apache-2.0
//! GPU post-process engine: turns a [`xengui::FilterChain`] into a
//! sequence of offscreen render passes over pooled intermediate textures.
//!
//! Architecture: the chain is split into contiguous segments — a run of
//! pointwise color ops becomes one [`ColorFilterPass`], and each
//! [`xengui::Filter::Blur`]/[`xengui::Filter::DropShadow`] becomes its own
//! [`KawasePass`] (a Dual Kawase down/up chain, plus for drop shadow a
//! composite via [`BlitPass`]). Segments ping-pong through a small pool of
//! reusable textures sized to the source, so a filtered widget never
//! allocates a new GPU texture on frames where its size hasn't changed.
mod color_pass;
mod kawase_pass;
mod blit_pass;
mod texture_pool;

pub use color_pass::ColorFilterPass;
pub use kawase_pass::KawasePass;
pub use blit_pass::BlitPass;
use texture_pool::TexturePool;

use xengui::{ Filter, FilterChain };

/// Physical-pixel padding needed around a filtered subtree so blur can
/// sample past its own edges without clipping. `chain.max_blur_radius()`
/// already represents the kernel's full reach, so no extra multiplier
/// belongs here.
pub(crate) fn padding_for_chain(chain: &FilterChain, scale_factor: f32) -> f32 {
    (chain.max_blur_radius() * scale_factor).ceil()
}

fn padded_dims(src_w: u32, src_h: u32, padding_px: f32) -> (u32, u32) {
    let pad = padding_px as u32;
    (src_w + pad * 2, src_h + pad * 2)
}

// UV offset/scale that maps a padded texture's own [0,1] UV space onto a
// smaller src_w x src_h source centered within it with padding_px on
// every side.
fn centered_uv_offset_scale(src_w: u32, src_h: u32, padding_px: f32) -> ((f32, f32), (f32, f32)) {
    let (out_w, out_h) = padded_dims(src_w, src_h, padding_px);
    let scale_u = (out_w as f32) / (src_w as f32);
    let scale_v = (out_h as f32) / (src_h as f32);
    let offset_u = -padding_px / (src_w as f32);
    let offset_v = -padding_px / (src_h as f32);
    ((offset_u, offset_v), (scale_u, scale_v))
}

/// Result of running a [`PostProcessEngine`] over a source texture: the
/// final filtered texture plus how far its content extends past the
/// widget's own logical bounds (blur/drop-shadow grow the visible
/// footprint).
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
/// and reused across frames; call [`PostProcessEngine::reset_frame`] once
/// per frame to release textures that weren't reused.
pub struct PostProcessEngine {
    color: ColorFilterPass,
    kawase: KawasePass,
    blit: BlitPass,
    pool: TexturePool,
}

impl PostProcessEngine {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        Self {
            color: ColorFilterPass::new(device, format),
            kawase: KawasePass::new(device, format),
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
        let padding_px = padding_for_chain(chain, scale_factor);
        let (out_w, out_h) = padded_dims(src_w, src_h, padding_px);

        log::trace!(
            "PostProcessEngine::apply src={src_w}x{src_h} scale_factor={scale_factor} padding_px={padding_px} out={out_w}x{out_h}"
        );

        // Composites `source` centered into a padded working texture so
        // every subsequent pass has headroom for blur without clipping.
        let mut current = self.pool.acquire(device, out_w, out_h);
        let (offset_uv, scale_uv) = centered_uv_offset_scale(src_w, src_h, padding_px);
        self.blit.run(
            device,
            queue,
            encoder,
            source,
            &current.view,
            out_w,
            out_h,
            offset_uv,
            scale_uv,
            None
        );

        current = self.run_chain(
            device,
            queue,
            encoder,
            current,
            out_w,
            out_h,
            chain,
            scale_factor
        );

        FilterOutput {
            view: current.view.clone(),
            width: out_w,
            height: out_h,
            padding: padding_px,
        }
    }

    /// Like `apply`, but for a `source` that already contains real
    /// padding content around the area the caller cares about (e.g. a
    /// backdrop-filter capture that grabbed extra surrounding scene
    /// pixels instead of letting blur fade into synthetic transparency).
    /// Runs the chain directly over `source` at its own size - nothing is
    /// re-padded, and the caller is responsible for cropping the result
    /// back down itself.
    #[allow(clippy::too_many_arguments)]
    pub fn apply_prepadded(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        source: &wgpu::TextureView,
        width: u32,
        height: u32,
        chain: &FilterChain,
        scale_factor: f32
    ) -> FilterOutput {
        log::trace!(
            "PostProcessEngine::apply_prepadded size={width}x{height} scale_factor={scale_factor}"
        );

        let mut current = self.pool.acquire(device, width, height);
        self.blit.run(
            device,
            queue,
            encoder,
            source,
            &current.view,
            width,
            height,
            (0.0, 0.0),
            (1.0, 1.0),
            None
        );

        current = self.run_chain(
            device,
            queue,
            encoder,
            current,
            width,
            height,
            chain,
            scale_factor
        );

        FilterOutput {
            view: current.view.clone(),
            width,
            height,
            padding: 0.0,
        }
    }

    // Runs every non-padding segment of `chain` over an already-sized
    // `current` texture in place - shared by `apply` (which pads first)
    // and `apply_prepadded` (whose caller already captured its own padding).
    #[allow(clippy::too_many_arguments)]
    fn run_chain(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        mut current: texture_pool::PooledTexture,
        w: u32,
        h: u32,
        chain: &FilterChain,
        scale_factor: f32
    ) -> texture_pool::PooledTexture {
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
                        w,
                        h
                    );
                    segment.clear();
                }
                current = match filter {
                    Filter::Blur(radius) => {
                        let physical_radius = radius.value() * scale_factor;
                        log::trace!(
                            "PostProcessEngine::run_chain blur radius_physical={physical_radius}"
                        );
                        self.kawase.run(
                            device,
                            queue,
                            encoder,
                            &current,
                            w,
                            h,
                            physical_radius,
                            &mut self.pool
                        )
                    }
                    Filter::DropShadow(shadow) => {
                        self.run_drop_shadow(
                            device,
                            queue,
                            encoder,
                            &current,
                            w,
                            h,
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
            current = self.run_color_segment(device, queue, encoder, &current, &segment, w, h);
        }

        current
    }

    /// Composites a filtered subtree's output onto `target` at `dest_rect`
    /// (physical px), blending over whatever `target` already contains.
    #[allow(clippy::too_many_arguments)]
    pub fn composite(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        source: &wgpu::TextureView,
        target: &wgpu::TextureView,
        dest_rect: (f32, f32, f32, f32),
        clip_rect: Option<(f32, f32, f32, f32)>,
        target_width: u32,
        target_height: u32,
        source_uv_rect: (f32, f32, f32, f32)
    ) {
        self.blit.run_over(
            device,
            queue,
            encoder,
            source,
            target,
            dest_rect,
            clip_rect,
            target_width,
            target_height,
            source_uv_rect
        );
    }

    /// Copies `source` into `target`, overwriting it entirely - used to
    /// present the accumulated scene target onto the real swapchain view
    /// once a frame is done.
    #[allow(clippy::too_many_arguments)]
    pub fn blit_full(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        source: &wgpu::TextureView,
        target: &wgpu::TextureView,
        target_width: u32,
        target_height: u32
    ) {
        self.blit.run(
            device,
            queue,
            encoder,
            source,
            target,
            target_width,
            target_height,
            (0.0, 0.0),
            (1.0, 1.0),
            None
        );
    }

    #[allow(clippy::too_many_arguments)]
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
            (1.0, 1.0),
            Some(shadow.color)
        );

        // 2. Blur the silhouette.
        let blurred = self.kawase.run(
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
            (1.0, 1.0),
            None
        );
        self.blit.run_over(
            device,
            queue,
            encoder,
            &source.view,
            &composited.view,
            (0.0, 0.0, w as f32, h as f32),
            None,
            w,
            h,
            (0.0, 0.0, 1.0, 1.0)
        );

        composited
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use xengui::Length;

    #[test]
    fn padding_matches_blur_radius_without_extra_multiplier() {
        let chain = FilterChain::new().push(Filter::Blur(Length::px(16.0)));
        assert_eq!(padding_for_chain(&chain, 1.0), 16.0);
        assert_eq!(padding_for_chain(&chain, 2.0), 32.0);
    }

    #[test]
    fn zero_blur_radius_produces_zero_padding() {
        let chain = FilterChain::new().push(Filter::Brightness(1.2));
        assert_eq!(padding_for_chain(&chain, 2.0), 0.0);
    }

    #[test]
    fn padded_dims_adds_padding_on_every_side() {
        assert_eq!(padded_dims(100, 55, 16.0), (132, 87));
    }

    #[test]
    fn centered_uv_offset_scale_maps_padding_symmetrically() {
        let (offset, scale) = centered_uv_offset_scale(100, 55, 16.0);
        let sample_at = |uv: f32, o: f32, s: f32| uv * s + o;
        assert!((sample_at(0.0, offset.0, scale.0) - -16.0 / 100.0).abs() < 1e-5);
        assert!((sample_at(1.0, offset.0, scale.0) - (1.0 + 16.0 / 100.0)).abs() < 1e-5);
        assert!((sample_at(0.0, offset.1, scale.1) - -16.0 / 55.0).abs() < 1e-5);
        assert!((sample_at(1.0, offset.1, scale.1) - (1.0 + 16.0 / 55.0)).abs() < 1e-5);
    }
}
