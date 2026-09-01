# xen-svg

[![Crates.io](https://img.shields.io/crates/v/xen-svg.svg)](https://crates.io/crates/xen-svg)
[![Documentation](https://docs.rs/xen-svg/badge.svg)](https://docs.rs/xen-svg)
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

`xen-svg` is a renderer-independent SVG parser and tessellator. It converts supported SVG documents into triangle draw operations and raster-image references suitable for a custom rendering pipeline.

## Features

- Lightweight SVG document and element model.
- Path, `viewBox`, transform, fill, stroke, and color parsing.
- Fill and stroke tessellation through `lyon`.
- Ordered vector and raster draw operations.
- No GPU or window-system dependency.

## Installation

```toml
[dependencies]
xen-svg = "0.1.1"
```

## Usage

```rust
use xen_svg::{parse_svg, tessellate_document};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let source = r#"<svg viewBox="0 0 24 24"><path d="M12 2L2 22h20Z"/></svg>"#;
    let document = parse_svg(source)?;
    let triangles = tessellate_document(&document);

    println!("generated {} triangles", triangles.len());
    Ok(())
}
```

Upload the resulting vertices to the graphics API of your choice. Use `collect_draw_ops` when source paint order or embedded raster images must be preserved.

## Scope

This crate intentionally implements the SVG subset needed by XenGui. It is not a browser-complete SVG engine; validate required elements and attributes before adopting it for arbitrary third-party documents.

## License

Licensed under the [Apache License 2.0](LICENSE).
