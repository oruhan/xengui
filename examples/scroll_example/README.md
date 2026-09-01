# Scroll example

[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

A focused example for scroll containers, overflow behavior, and scrollbar styling in XenGui.

## Demonstrates

- Independent `overflow_x` and `overflow_y` behavior.
- Automatic scrollbars for content larger than its viewport.
- Track, thumb, and arrow color customization.
- Scrolling a dynamically generated widget list.

## Run

From the workspace root:

```bash
cargo run -p scroll-example
```

For WebAssembly:

```bash
cd examples/scroll_example
trunk serve --open
```

Verify mouse-wheel or touchpad scrolling, thumb dragging, clipping, and resizing at several window sizes.

## Related code

The complete example is in [`src/main.rs`](src/main.rs).

## License

Licensed under the [Apache License 2.0](LICENSE).
