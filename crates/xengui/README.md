# xengui

[![Crates.io](https://img.shields.io/crates/v/xengui.svg)](https://crates.io/crates/xengui)
[![Documentation](https://docs.rs/xengui/badge.svg)](https://docs.rs/xengui)
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

`xengui` is the platform-independent core of the XenGui toolkit. It owns the retained widget tree, hooks, layout, styling, text model, input dispatch, and reconciliation. Window management and GPU rendering are intentionally delegated to separate crates.

> [!NOTE]
> The project is pre-1.0 and its public API is still evolving.

## Features

- Retained components with state, effects, resources, and context.
- Flexbox and CSS Grid layout powered by `taffy`.
- Responsive values, themes, transitions, filters, shadows, and interaction states.
- Built-in views, labels, buttons, badges, progress bars, separators, form controls, text boxes, images, SVG, menus, portals, and tables.
- Backend-independent paint commands through the `RenderBackend` abstraction.
- Native and WebAssembly-compatible task and platform abstractions.

## Installation

```toml
[dependencies]
xengui = "0.2.8"
```

Most applications also need [`xenframe`](../xenframe) for the event loop and [`xengui-wgpu`](../xengui-wgpu) for rendering.

## Usage

```rust
use xengui::*;

fn counter() -> impl Widget {
    let (count, set_count) = use_state(0_i32);

    Column::new()
        .gap(0, 8)
        .child(Label::new().label(format!("Count: {count}")))
        .child(
            Button::new()
                .label("Increment")
                .on_click(move |_| set_count.update(|value| *value += 1)),
        )
}
```

See the [workspace quick start](../../README.md#quick-start) for a complete runnable application.

## Compatibility

The minimum supported Rust version is 1.92. The core crate supports native and `wasm32-unknown-unknown` targets; actual platform availability depends on the selected runtime and render backend.

## Documentation and support

- [API reference](https://docs.rs/xengui)
- [Guides and live examples](https://xengui.vercel.app/docs)
- [Issues](https://github.com/randseas/xengui/issues)

## License

Licensed under the [Apache License 2.0](LICENSE).
