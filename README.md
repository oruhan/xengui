# XenGui

**English** | [Türkçe](README.tr.md)

[![Crates.io](https://img.shields.io/crates/v/xengui.svg)](https://crates.io/crates/xengui)
[![Documentation](https://docs.rs/xengui/badge.svg)](https://docs.rs/xengui)
[![Rust 1.92+](https://img.shields.io/badge/rust-1.92%2B-blue.svg)](https://www.rust-lang.org)
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

XenGui is a retained-mode GUI toolkit written in Rust. It combines a hooks-based component model, Flexbox and Grid layout through [`taffy`](https://github.com/DioxusLabs/taffy), and GPU rendering through [`wgpu`](https://github.com/gfx-rs/wgpu). The same application code can target desktop and WebAssembly.

[Live demo](https://xengui.vercel.app) | [Documentation](https://xengui.vercel.app/docs) | [API reference](https://docs.rs/xengui) | [Issue tracker](https://github.com/randseas/xengui/issues)

> [!WARNING]
> XenGui is under active development. Public APIs may change before 1.0; pin dependency versions and review release notes before upgrading production applications.

## Highlights

- Retained widget tree with `component`, `use_state`, effects, resources, and context.
- Flexbox, CSS Grid, responsive values, scrolling, and split-pane layouts.
- Declarative themes and interaction-specific styles, including transitions and filters.
- Built-in controls for text, forms, images, SVG, navigation, menus, tables, and overlays.
- Instanced and batched `wgpu` pipelines with reusable frame staging for rectangles, text, images, SVG triangles, filters, and shadows.
- Native windowing and input through `winit`, plus browser support through WebAssembly.
- Rendering, runtime, routing, animation, clipboard, audio, SVG, and icons split into focused crates.

## Architecture

| Package | Role |
| --- | --- |
| [`xengui`](crates/xengui) | Platform-independent widget tree, hooks, layout, styling, and reconciliation. |
| [`xenframe`](crates/xenframe) | Window creation, event loop, input, IME, theme, and browser integration. |
| [`xengui-wgpu`](crates/xengui-wgpu) | GPU render backend and window renderer. |
| [`xen-router`](crates/xen-router) | Client-side routing with browser History API synchronization. |
| [`xen-router-build`](crates/xen-router-build) | Build-time generator for file-based routes. |
| [`xen-animation`](crates/xen-animation) | Framework-independent transitions and easing. |
| [`xen-clipboard`](crates/xen-clipboard) | Asynchronous text clipboard abstraction. |
| [`xen-audio`](crates/xen-audio) | Framework-independent local audio playback abstraction. |
| [`xen-svg`](crates/xen-svg) | SVG parsing and triangle tessellation. |
| [`xengui-icons`](crates/xengui-icons) | Embedded Material Symbols variable icon font and codepoints. |
| [`xengui-cli`](crates/xengui-cli) | Workspace development, versioning, Git, diagnostics, and release tooling. |

Runnable applications live in [`apps`](apps); focused demonstrations live in [`examples`](examples).

The renderer's allocation and upload model is documented in [Rendering performance](docs/rendering-performance.md).

## Requirements

- Rust 1.92 or newer, as declared by the workspace MSRV.
- A graphics adapter and driver supported by `wgpu`.
- [Trunk](https://trunk-rs.github.io/trunk/) and the `wasm32-unknown-unknown` Rust target for browser builds.
- The platform audio development package when building the full workspace; Linux builds of `xen-audio` require ALSA development files discoverable through `pkg-config`.

## Quick start

Create a binary crate and add the application runtime dependencies:

```toml
[dependencies]
xengui = "0.2.8"
xenframe = "0.1.2"
xengui-wgpu = "0.1.2"
```

```rust
use xenframe::{App, AppConfig};
use xengui::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = App::new(AppConfig {
        title: "Counter".into(),
        width: 640,
        height: 480,
        ..Default::default()
    });

    app.render(|| {
        let (count, set_count) = use_state(0_i32);

        Box::new(
            Column::new()
                .width(pct!(100))
                .height(pct!(100))
                .align_items(Align::Center)
                .justify_content(JustifyContent::Center)
                .gap(0, 12)
                .child(Label::new().label(format!("Count: {count}")))
                .child(
                    Button::new()
                        .label("Increment")
                        .on_click(move |_| set_count.update(|value| *value += 1)),
                ),
        )
    });

    app.run()?;
    Ok(())
}
```

Run the application with `cargo run`.

## Run the workspace

From the repository root:

```bash
cargo run -p xengui_website
```

Other useful targets include `settings-app` and `pearl`.

For a browser build:

```bash
rustup target add wasm32-unknown-unknown
cargo install trunk
cd apps/xengui_website
trunk serve --open
```

Trunk serves a local development build and rebuilds it when source files change.

## Development

Install the workspace CLI once, then use it from anywhere inside this checkout:

```bash
cargo install --path crates/xengui-cli
xengui --help
```

`xengui run` starts `xengui_website` by default; pass `--package <NAME>` for another application. The `build`, `check`, and `test` commands wrap their Cargo equivalents for the full workspace. Extra Cargo arguments go after `--`.

Version operations discover every workspace package and keep path-dependency requirements synchronized. Mutating operations are previews unless explicitly confirmed:

```bash
xengui version show
xengui version check
xengui version bump minor          # dry run
xengui version bump minor --write
xengui version suggest             # explains the recommendation
xengui version suggest --apply     # explains, then applies
xengui commit suggest
xengui release check
```

The CLI supersedes the former Python version-bump helper. Setup, platform-specific watch, and source-inventory scripts remain in `scripts/` because they serve separate workflows.

Run the standard quality checks from the repository root:

```bash
cargo fmt --all --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

GPU availability and target-specific dependencies can affect native or WebAssembly checks. When changing rendering code, test both a native example and a browser build.

On Linux, a missing `alsa.pc` error means the distribution's ALSA development package and `pkg-config` must be installed before testing `xen-audio` or `pearl`.

Contributions are welcome through [issues](https://github.com/randseas/xengui/issues) and pull requests. For substantial changes, open an issue first so the design can be discussed. Please include tests or a reproducible example for behavioral changes.

## License

Licensed under the [Apache License 2.0](LICENSE).
