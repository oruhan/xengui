# Settings application

[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

The settings application is a XenGui development fixture for native window chrome, form controls, scrolling, themes, icons, and interaction transitions. It is useful for manual regression testing rather than as an end-user settings service.

## Demonstrates

- A borderless native window with custom drag and window-control regions.
- Text inputs, buttons, disabled states, scrolling, and focus handling.
- Material Symbols variable icons and bundled font loading.
- Hover and pressed transitions.
- Native and WebAssembly application initialization.

## Run

From the workspace root:

```bash
cargo run -p settings-app
```

For a browser development build:

```bash
rustup target add wasm32-unknown-unknown
cargo install trunk
cd apps/settings_app
trunk serve --open
```

Native-only window actions such as minimize, maximize, and close do not have an equivalent browser effect.

## Validation

Use the application to verify pointer, keyboard, focus, scroll, theme, animation, and custom-window behavior after changes to `xengui` or `xenframe`.

## License

Licensed under the [Apache License 2.0](LICENSE).
