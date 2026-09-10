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

GPU policy is available through `AppConfig::renderer`. Desktop defaults use vsync and 4× MSAA. Android defaults favor the high-performance adapter, one queued frame, and 1× MSAA to reduce mobile bandwidth; applications can override every renderer option. Android activity suspend/resume retains the GPU core and frame cache, reattaches only the native surface, and presents the retained scene before revealing the window.

Material ripple feedback defaults to Android and Linux, while WebAssembly is opt-in. Configure it application-wide through `AppConfig::ripple`:

```rust
use xenframe::AppConfig;
use xengui::{RippleConfig, RipplePlatforms};

let config = AppConfig {
    ripple: RippleConfig {
        enabled: true,
        platforms: RipplePlatforms::ALL,
        strength: 0.8,
        duration_scale: 1.0,
    },
    ..Default::default()
};
```

The default is Google's GPU-procedural patterned ripple: a soft wave plus an animated sparkle field, with 450 ms enter and 375 ms exit phases. Clickable widgets also expose `.ripple(false)`, `.ripple_strength(...)`, `.ripple_duration_scale(...)`, and `.ripple_color(...)` overrides. A global `enabled: false` remains a master off switch.

## Documentation and support

- [API reference](https://docs.rs/xenframe)
- [Project documentation](https://xengui.vercel.app/docs/xenframe)
- [Workspace guide](../../README.md)

## License

Licensed under the [Apache License 2.0](LICENSE).
