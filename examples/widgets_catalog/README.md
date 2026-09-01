# Widgets catalog

[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

The widgets catalog is XenGui's primary interactive component showcase and manual regression surface. It gathers controls, layout primitives, styling effects, overlays, and developer tools into one application.

## Coverage

- Buttons, checkboxes, switches, radio buttons, text boxes, links, tooltips, and keyboard hints.
- Images, SVG, Material Symbols variable icons, rich text, and tables.
- Flexbox, CSS Grid, scrolling, split panes, portals, and clipping.
- Gradients, borders, per-corner radii, outlines, shadows, filter chains, and backdrop filters.
- Controlled state, disabled states, pointer cursors, transitions, and composite widgets.
- XenGui DevTools, toggled with <kbd>F12</kbd>.

## Run

From the workspace root:

```bash
cargo run -p widgets-catalog
```

For WebAssembly:

```bash
cd examples/widgets_catalog
trunk serve --open
```

## Regression checklist

After widget, layout, input, or rendering changes:

1. Exercise each interactive and disabled state with mouse and keyboard.
2. Resize the window across responsive breakpoints.
3. Verify scrolling, clipping, portals, and split-pane dragging.
4. Compare filters, shadows, gradients, text, SVG, and icons for visual artifacts.
5. Open DevTools and confirm the inspected tree updates.
6. Repeat the relevant checks in a browser build for WebGPU parity.

## Project structure

- [`src/main.rs`](src/main.rs) defines the catalog sections and application shell.
- [`components`](components) contains custom composite widgets used by the catalog.

## License

Licensed under the [Apache License 2.0](LICENSE).
