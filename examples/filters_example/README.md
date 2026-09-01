# Filters example

[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

A visual reference for XenGui's GPU-backed filter pipeline.

## Demonstrates

- Blur, brightness, contrast, saturation, grayscale, hue rotation, invert, opacity, and gamma filters.
- Drop shadows rendered as filter operations.
- Multiple operations composed with `FilterChain`.
- Filtered content inside rounded views.

## Run

From the workspace root:

```bash
cargo run -p filters-example
```

For WebAssembly:

```bash
cd examples/filters_example
trunk serve --open
```

Each card labels the effect and parameters applied to the same source content, making rendering regressions easy to compare visually.

## Related code

The complete example is in [`src/main.rs`](src/main.rs).

## License

Licensed under the [Apache License 2.0](LICENSE).
