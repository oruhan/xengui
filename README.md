# XenGui

**English** | [Türkçe](README.tr.md)

[![Crates.io](https://img.shields.io/crates/v/xengui.svg)](https://crates.io/crates/xengui)
[![Documentation](https://docs.rs/xengui/badge.svg)](https://docs.rs/xengui)
[![Rust 1.92+](https://img.shields.io/badge/rust-1.92%2B-blue.svg)](https://www.rust-lang.org)
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

XenGui is a retained-mode GUI toolkit for Rust. It combines a hooks-based component model,
Flexbox and Grid layout through [`taffy`](https://github.com/DioxusLabs/taffy), and a
[`wgpu`](https://github.com/gfx-rs/wgpu) render backend. The same widget and state APIs target
native desktop applications and WebAssembly.

[Live showcase](https://xengui.vercel.app/showcase) · [Guides](https://xengui.vercel.app/docs) · [API reference](https://docs.rs/xengui) · [Issues](https://github.com/randseas/xengui/issues)

> [!WARNING]
> XenGui is under active development. Public APIs may change before 1.0; pin crate versions and
> review release notes before upgrading production applications.

## Showcase

The [live route](https://xengui.vercel.app/showcase) is interactive and lets you inspect
navigation, text input, focusable controls, responsive layout, theme roles, and state updates. It
contains no placeholder performance telemetry. The native reference screen is available in
[`apps/showcase`](apps/showcase).

```bash
cargo run -p xengui-showcase
```

## Quick start

Create a binary crate and add the runtime, widget, and renderer crates:

```toml
[dependencies]
xengui = "0.2.8"
xenframe = "0.1.2"
xengui-wgpu = "0.1.2"
```

This small focus board demonstrates layout, state, interaction, styling, text input, a checkbox,
progress, and a button while remaining easy to run:

```rust
use xenframe::{App, AppConfig};
use xengui::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = App::new(AppConfig {
        title: "Focus Board".into(),
        width: 760,
        height: 520,
        ..Default::default()
    });

    app.render(|| {
        let (done, set_done) = use_state(false);
        Box::new(
            Column::new()
                .padding(Edges::all(24.0))
                .gap(0.0, 16.0)
                .child(Label::new().label("Today").font_size(28.0))
                .child(
                    Row::new()
                        .align_items(Align::Center)
                        .gap(12.0, 0.0)
                        .child(
                            Checkbox::new()
                                .checked(done)
                                .on_change(move |value, _| set_done.set(value)),
                        )
                        .child(Label::new().label("Ship a polished XenGui screen")),
                )
                .child(ProgressBar::new().value(if done { 1.0 } else { 0.5 })),
        )
    });

    app.run()?;
    Ok(())
}
```

Run it with `cargo run`. The repository version is available as:

```bash
cargo run -p xengui-quickstart
```

## What XenGui provides

- Retained widget identity with components, hooks, effects, resources, and context.
- Flexbox, CSS Grid, responsive values, scrolling, and split-pane layouts.
- Theme roles and interaction styles for hover, focus, pressed, and disabled states.
- Controls for text, forms, images, SVG, navigation, menus, tables, and overlays.
- Native windowing and input through `winit`, with a WebAssembly browser target.
- Focused crates for runtime, rendering, routing, animation, clipboard, audio, SVG, and icons.

The renderer's verified allocation and upload behavior is documented in
[Rendering performance](docs/rendering-performance.md).

## Performance comparisons

The repository includes a repeatable CPU benchmark for full layout and paint orchestration. The
comparison script builds two Git revisions in detached worktrees, runs the same release workload,
reports median nanoseconds per frame, and fails when the configured regression budget is exceeded.
It does not generate or commit synthetic FPS or frame-time data.

```bash
./scripts/compare-performance.sh HEAD^ HEAD 10
```

Results are written to `artifacts/performance-comparison.md`, `.jsonl`, and a measured `.svg`
chart. The repository includes the [latest measured comparison](artifacts/performance-comparison.md).
Pull requests run the same comparison in CI and upload the results as an artifact.

## Repository map

| Package | Responsibility |
| --- | --- |
| [`xengui`](crates/xengui) | Widget tree, hooks, layout, styling, input, and reconciliation. |
| [`xenframe`](crates/xenframe) | Window lifecycle, event loop, IME, themes, and browser integration. |
| [`xengui-wgpu`](crates/xengui-wgpu) | `wgpu` renderer and window surface integration. |
| [`xen-router`](crates/xen-router) | Client-side routing and browser History API synchronization. |
| [`xengui-icons`](crates/xengui-icons) | Embedded Material Symbols font and codepoints. |
| [`xengui-cli`](crates/xengui-cli) | Workspace checks, versioning, diagnostics, and release tooling. |

Runnable applications live in [`apps`](apps); task-oriented documentation lives at
[xengui.vercel.app/docs](https://xengui.vercel.app/docs), while docs.rs remains the API reference.

## Development

Run the website locally:

```bash
rustup target add wasm32-unknown-unknown
cargo install trunk
cd apps/xengui_website
trunk serve --open
```

Run all workspace quality gates:

```bash
cargo xtask quality
```

Regenerate the native application captures after visual changes:

```bash
./scripts/capture-readme-assets.sh
```

CI runs the same direct-window capture command for relevant UI changes. Pull requests fail when
committed captures are stale; the main branch refreshes changed captures automatically.

## License

Licensed under the [Apache License 2.0](LICENSE).
