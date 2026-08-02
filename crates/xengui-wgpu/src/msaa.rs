// crates/xengui-wgpu/src/msaa.rs (yeni)
// SPDX-License-Identifier: Apache-2.0

/// Hardware multisample count for anti-aliasing triangle geometry (used
/// by `xen-svg` icon/path rendering via the triangle pipeline).
///
/// Not FXAA/SMAA (post-process) or supersampling (render-at-N×-and-downscale) -
/// this drives `wgpu::MultisampleState::count` directly, so the GPU
/// rasterizer itself produces per-sample coverage that's resolved once
/// per frame. Costs proportionally more VRAM/bandwidth per sample; `X4`
/// is a good default, `X8` only where the adapter actually supports it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum SampleCount {
    X1,
    X2,
    #[default]
    X4,
    X8,
}

impl SampleCount {
    pub const fn as_u32(self) -> u32 {
        match self {
            Self::X1 => 1,
            Self::X2 => 2,
            Self::X4 => 4,
            Self::X8 => 8,
        }
    }

    /// Clamps down to the nearest count the adapter actually supports for
    /// `format`, falling back to `X1` (no MSAA) if even 2× isn't available -
    /// this is what makes MSAA "automatic fallback" instead of a hard panic.
    pub fn clamp_to_adapter(self, adapter: &wgpu::Adapter, format: wgpu::TextureFormat) -> Self {
        let flags = adapter.get_texture_format_features(format).flags;
        let supports = |count: u32| flags.sample_count_supported(count);

        let mut candidate = self;
        loop {
            if supports(candidate.as_u32()) {
                return candidate;
            }
            candidate = match candidate {
                Self::X8 => Self::X4,
                Self::X4 => Self::X2,
                Self::X2 | Self::X1 => Self::X1,
            };
            if candidate == Self::X1 {
                return Self::X1;
            }
        }
    }
}
