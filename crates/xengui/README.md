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

### Runtime ownership

Each `xenframe::App` owns a `RuntimeContext` and enters it for rendering and
platform events. Custom hosts should keep one context per application/window
tree and enter it while building, reconciling, laying out, painting, or dispatching
input:

```rust
let runtime = xengui::RuntimeContext::new();
let _guard = runtime.enter();
xengui::style::theme::set_current_theme(xengui::Theme::dark());
xengui::task::spawn(async { /* application work */ });
runtime.tasks().poll();
```

Use `app.runtime().enter()` for service configuration outside App callbacks.
`RuntimeContext::bind` captures a weak owner for callbacks invoked by external
systems. Hook setters and `EventCtx::spawn` retain their original runtime identity;
calling them while another runtime is active does not redirect their work.
Standalone widget construction without an entered context uses transient defaults;
service writes in that mode are not inherited by a future App.

Resource reload, invalidation and component unmount cancel the previous task and
drop its future without waiting for another wakeup. Dropping the context drops
pending tasks/effects and runs mounted effect cleanups. Already-running
`spawn_blocking` closures cannot be forcibly interrupted; their abandoned results
are not applied to disposed hook state.
