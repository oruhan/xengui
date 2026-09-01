# Layout example

[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

A resizable application layout that demonstrates XenGui split panes and nested Flexbox composition.

## Demonstrates

- Left, right, and bottom `SplitPanel` regions.
- Minimum and maximum panel sizes.
- Pointer-driven resizing with persistent in-memory dimensions.
- Nested rows and columns, context menus, themes, and bundled fonts.

## Run

From the workspace root:

```bash
cargo run -p layout-example
```

For WebAssembly:

```bash
cd examples/layout_example
trunk serve --open
```

Drag each divider and verify that adjacent panels respect their configured constraints.

## Project structure

- [`src/main.rs`](src/main.rs) assembles the layout.
- [`components`](components) contains reusable example components.

## License

Licensed under the [Apache License 2.0](LICENSE).
