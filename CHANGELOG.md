# Changelog

All notable changes to the XenGui workspace are documented here. The project follows
[Semantic Versioning](https://semver.org/spec/v2.0.0.html) while its crates remain pre-1.0.

## Unreleased

### Rendering performance

- Reused frame-lifetime paint, batching, decoration, and widget-path storage after its high-water mark.
- Replaced per-frame render-cache liveness sets with generation tracking.
- Converted rectangle rendering from six expanded vertices per rectangle to one instanced record, reducing rectangle upload data from 552 to 92 bytes per item.
- Reused triangle, stroke, image, gradient, and rectangle CPU staging buffers between frames.
- Kept image textures live across every batch in a frame, preventing cache churn between non-contiguous image runs.
- Fixed top-layer box-shadow run tracking while consolidating the batching path.

### Behavior fixes

- Prevented wheel, touch, momentum, AutoScroll, and overscroll effects on axes without both an explicit scroll overflow mode and real content overflow.
- Made vertical wheel input bubble past horizontal-only scrollers unless Shift requests horizontal scrolling.
- Kept wheel input clamped to scroll bounds while reserving bounce/stretch for direct touch manipulation and momentum.
- Removed duplicate touch-pan dispatch and redundant scroll-state transfer work.
- Made reconciliation compare every authored pseudo-state style, including focus-within and focused-pressed layers, before reusing cached widget output.
- Limited scrollbar thumb-width hover animation to the thumb itself; hovering empty track no longer expands it.
- Upgraded middle-click AutoScroll with smooth acceleration/deceleration, a precision dead zone, diagonal speed limiting, stalled-frame protection, and an animated directional origin marker.

### Documentation

- Added an English/Turkish language selector to the default English workspace README and introduced a complete Turkish translation.

## 2026-09-03

### xengui 0.2.8

- Added `Badge`, `ProgressBar`, and horizontal/vertical `Separator` widgets.
- Added complete public API rustdoc coverage and made missing or broken documentation fail the build.
- Made progress bars sanitize non-finite values and clamp progress to `0.0..=1.0`.
- Hardened hook scheduling and input handling edge cases.

### xengui-wgpu 0.1.2

- Fixed Kawase blur uniforms for WebGPU devices that require 16-byte-aligned buffer bindings.
- Made WebGL2 the compatibility-first WebAssembly default while retaining explicit WebGPU opt-in.
- Completed renderer option and pending GPU error initialization.
- Improved WebGPU error reporting and renderer recovery behavior.

### xenframe 0.1.2

- Updated the native and WebAssembly runtime integration for the current XenGui renderer contracts.
- Improved event, text-input, window, and platform handling.

### Documentation website

- Rebuilt `/docs` as a responsive, task-oriented guide with live widget examples.
- Added a clear split between curated website guides and automatically generated docs.rs API reference.
