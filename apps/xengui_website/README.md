# XenGui website

[![Live site](https://img.shields.io/badge/live-xengui.vercel.app-4daafc.svg)](https://xengui.vercel.app)
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

This package contains the official XenGui website and browser demo. The site is itself a XenGui WebAssembly application and exercises routing, responsive layouts, filters, text input, and complex interactive views.

## Routes

- `/` presents the project landing page.
- `/docs` introduces core XenGui concepts.
- `/examples` previews common widgets.
- `/playground` pairs sample code with a live component.
- `/showcase` demonstrates an application-scale responsive interface.

Routes are generated from the `app/` directory by `xen-router-build`.

## Local development

Install the WebAssembly target and Trunk once:

```bash
rustup target add wasm32-unknown-unknown
cargo install trunk
```

Then serve the site:

```bash
cd apps/xengui_website
trunk serve --open
```

To verify the native application shell from the workspace root:

```bash
cargo run -p xengui_website
```

## Production build

From this package directory:

```bash
trunk build --release
```

The generated static site is written to `dist/`. The repository's `scripts/vercel.sh` installs the pinned Trunk release and performs the Vercel build from the repository root.

## Project structure

- `app/` contains file-based route pages, layouts, and the not-found view.
- `src/main.rs` initializes the runtime and includes generated routes.
- `assets/` contains icons, manifests, crawler metadata, and the service worker.
- `fonts/` contains fonts embedded into the application binary.

## Deployment checks

Before publishing, verify route navigation, browser back/forward behavior, responsive breakpoints, console output, and WebGPU initialization in at least one Chromium-based browser and one additional WebGPU-capable browser.

## License

Licensed under the [Apache License 2.0](LICENSE).
