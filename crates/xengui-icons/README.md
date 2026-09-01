# xengui-icons

[![Crates.io](https://img.shields.io/crates/v/xengui-icons.svg)](https://crates.io/crates/xengui-icons)
[![Documentation](https://docs.rs/xengui-icons/badge.svg)](https://docs.rs/xengui-icons)
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

`xengui-icons` packages the Material Symbols Rounded variable font, its generated codepoints, and strongly defined variation axes for XenGui's `VariableIcon` widget.

## Features

- Font bytes embedded at compile time with no runtime asset lookup.
- Generated icon codepoints in the `codepoints` module.
- Continuous fill, weight, grade, and optical-size axes.
- Stable variation cache keys for downstream glyph renderers.

## Installation

```toml
[dependencies]
xengui-icons = "0.1.0"
```

## Usage

```rust
use xengui::{VariableIcon, Widget};
use xengui_icons::{codepoints, IconAxes};

fn icon() -> impl Widget {
    VariableIcon::new(codepoints::PLAY_ARROW)
        .size(24)
        .axes(IconAxes::default().fill(1.0).weight(500.0))
}
```

See the generated `codepoints` module for available constant names. Axis values are clamped to the ranges supported by the bundled font.

## Attribution

Material Symbols are designed by Google and distributed under the Apache License 2.0. The generated Rust bindings and crate source are also Apache-2.0 licensed.

## License

Licensed under the [Apache License 2.0](LICENSE).
