# Animation example

[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

A focused XenGui example showing independent color, border, and transform transitions driven by hover and pressed interaction states.

## Demonstrates

- `transition_colors` and `transition_transform` with different durations.
- CSS-style easing through `Transition` and `Easing`.
- Hover and pressed style patches.
- Smooth scale, color, and border interpolation.

## Run

From the workspace root:

```bash
cargo run -p animation-example
```

For WebAssembly:

```bash
cd examples/animation_example
trunk serve --open
```

Move the pointer over the controls and press them to compare independently timed properties.

## Related code

- [`src/main.rs`](src/main.rs) contains the complete example.
- [`xen-animation`](../../crates/xen-animation) documents the underlying animation engine.

## License

Licensed under the [Apache License 2.0](LICENSE).
