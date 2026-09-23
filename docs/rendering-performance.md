# Rendering performance

XenGui keeps widget identity and hook state in a retained tree. Retained widgets therefore outlive a frame and are intentionally not allocated from a frame arena. The frame arena is reserved for transient paint and batching data whose lifetime ends after presentation.

## Frame flow

1. Layout runs only when geometry-affecting state is dirty. Scroll-only frames reuse layout and reflow positions.
2. The paint walk writes commands, focus chrome, top-layer commands, batch buffers, text decorations, and widget paths into reusable storage owned by `FrameRenderer`.
3. Commands are checked for monotonic z-order. The stable sort runs only when an explicit z-index introduced an inversion, then commands are drained into typed batches without releasing their capacity.
4. Rectangle commands become one `RectInstance` each. The vertex shader synthesizes the six quad corners from `vertex_index`.
5. Each compatible run is copied once from its contiguous CPU staging slice into the growable GPU buffer, then drawn as one instanced range per clip run.

## Allocation contract

After buffers have reached the high-water mark, an unchanged common-path frame reuses its transient `Vec`, `String`, and cache-liveness storage. Allocations can still occur when:

- the widget tree or cached paint output changes;
- a larger frame exceeds a staging buffer's previous capacity;
- text shaping, glyph atlases, images, fonts, filters, or GPU buffers need new resources;
- a full Taffy layout rebuild is required.

This is a steady-state allocation policy, not an unsafe claim that arbitrary dynamic application code can never allocate. It preserves widget state and the backend-independent `RenderBackend` boundary.

## Upload layout

Rectangle instances use a 92-byte GPU record containing screen bounds, local half-size, corner radii, border width, fill and border colors, and gradient metadata. Previously, every rectangle expanded the same payload into six 92-byte vertices (552 bytes). Triangle, stroke, and image pipelines retain flattened staging vectors and upload each compatible run as one contiguous slice.

Taffy types do not cross into the GPU backend. Layout boxes are lowered to backend-neutral paint commands first, then flattened into pipeline-specific records. This keeps alternative render backends possible while ensuring only one contiguous staging-to-buffer copy per run.

## CPU fast paths

- Clean, non-layout frames skip the full style cascade unless a style animation, dirty paint state, or theme-generation change requires it. Scroll and ripple reflow keep their existing behavior without paying for an unrelated whole-tree style walk.
- Text measurement and drawing share the same shaped glyph buffer cache. Caret hit-testing, layout measurement, and the following draw therefore do not reshape an identical run independently.
- Variable-icon staging vectors, frame paint arenas, and pipeline staging vectors retain their high-water capacities.

## GPU fast paths

- A single-sample frame with no backdrop filter renders directly into the swapchain. It avoids the full-screen scene texture write, read, and presentation blit. MSAA and backdrop-filter frames retain the offscreen path required by their existing semantics.
- Text runs use a retained pool of `glyphon` renderers, so independently ordered text runs can remain in one frame command submission without later `prepare` calls overwriting earlier vertex data.
- Filter/composite capture textures come from the post-process texture pool. Box-shadow mask/composite vertices and variable-icon vertices use growable retained buffers instead of creating fresh GPU buffers for every draw.

## Verification

Rendering changes should pass native tests and linting, the `wasm32-unknown-unknown` check, and a browser smoke test on the documentation and widget-heavy example pages. WebGPU validation must remain clean; WebGL2 remains the compatibility-first browser default.
