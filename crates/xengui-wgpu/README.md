# xengui-wgpu

[![Crates.io](https://img.shields.io/crates/v/xengui-wgpu.svg)](https://crates.io/crates/xengui-wgpu)
[![Documentation](https://docs.rs/xengui-wgpu/badge.svg)](https://docs.rs/xengui-wgpu)
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

`xengui-wgpu` is the `wgpu` render backend for XenGui. It converts platform-independent paint commands from `xengui` into ordered GPU passes and can either own a window surface or render into a host-provided target.

## Features

- `RenderBackend` implementation for rectangles, images, SVG triangles, and text.
- Instanced rectangle drawing and batched pipelines with scissor clipping and stable paint order.
- Reusable CPU staging buffers and one contiguous upload per compatible command run.
- Rounded rectangles, borders, anti-aliasing, gradients, filters, and shadows.
- Text shaping and glyph-atlas management through `glyphon`.
- User font loading and WebAssembly fallback-font support.
- Adapter-clamped MSAA for tessellated SVG geometry on native and browser targets.
- Configurable GPU backends, power preference, frame latency, and presentation mode.
- Compatibility-first WebGL2 default on WebAssembly; applications can explicitly opt into browser WebGPU.
- Typed initialization/frame errors, bounded surface recovery, and device-loss reporting.

## Installation

```toml
[dependencies]
xengui-wgpu = "0.1.2"
```

Most applications should use this crate indirectly through [`xenframe`](../xenframe). Hosts that already own a `wgpu::Device`, command encoder, and render target can construct `WgpuPipelines` and call `begin_frame` directly.

## Main entry points

| Type | Use case |
| --- | --- |
| `WgpuWindowRenderer` | Own a device and surface for a XenGui window. |
| `WgpuPipelines` | Integrate XenGui into an existing `wgpu` renderer. |
| `WgpuFrame` | Submit paint commands during a host-managed frame. |
| `SampleCount` | Configure multisample anti-aliasing. |
| `RendererOptions` | Configure backend, power, presentation, latency, and MSAA policy. |
| `RendererError` / `FrameOutcome` | Handle GPU failures and intentionally skipped frames. |

## Validation

Rendering changes should be checked on both native and WebAssembly targets because WebGPU applies stricter shader and buffer-layout validation on some adapters.

```bash
cargo check -p xengui-wgpu
cargo test -p xengui-wgpu
cargo check -p xengui_website --target wasm32-unknown-unknown
```

## Documentation and support

- [API reference](https://docs.rs/xengui-wgpu)
- [Project documentation](https://xengui.vercel.app/docs/xengui-wgpu)
- [Workspace guide](../../README.md)

## License

Licensed under the [Apache License 2.0](LICENSE).
