# xenframe

[![Crates.io](https://img.shields.io/crates/v/xenframe.svg)](https://crates.io/crates/xenframe)
[![Documentation](https://docs.rs/xenframe/badge.svg)](https://docs.rs/xenframe)
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

`xenframe` is XenGui's application runtime. It connects `xengui` and `xengui-wgpu` to `winit`, creates windows, drives redraws, and translates operating-system or browser events into XenGui input events.

## Features

- Configurable native windows and WebAssembly canvases.
- Mouse, keyboard, touch, focus, and IME event translation.
- Clipboard, cursor, system theme, and window-control integration.
- Multi-click, selection, long-press, and mobile text-input handling.
- Efficient redraw and caret scheduling through the `winit` event loop.
- Automatic surface recovery and renderer recreation after recoverable GPU device loss.

## Installation

```toml
[dependencies]
xengui = "0.2.8"
xenframe = "0.1.2"
xengui-wgpu = "0.1.2"
```

## Usage

```rust
use xenframe::{App, AppConfig};
use xengui::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = App::new(AppConfig {
        title: "XenGui application".into(),
        width: 800,
        height: 600,
        ..Default::default()
    });

    app.render(|| {
        Box::new(
            View::new()
                .width(pct!(100))
                .height(pct!(100))
                .child(Label::new().label("Hello from XenGui")),
        )
    });

    app.run()?;
    Ok(())
}
```

## Platform notes

Native targets use `winit` windows. Browser targets attach to the canvas declared by the application's `index.html`; use Trunk to build and serve the application. Mobile-browser IME support uses a hidden HTML input managed by the runtime.

GPU policy is available through `AppConfig::renderer`. Its defaults select broadly compatible backends, vsync presentation, conservative device limits, and adapter-clamped 4× MSAA for tessellated geometry.

## Documentation and support

- [API reference](https://docs.rs/xenframe)
- [Project documentation](https://xengui.vercel.app/docs/xenframe)
- [Workspace guide](../../README.md)

## License

Licensed under the [Apache License 2.0](LICENSE).
