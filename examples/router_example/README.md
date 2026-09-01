# Router example

[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

A minimal file-based routing example for `xen-router` and `xen-router-build`.

## Routes

| Source | Route |
| --- | --- |
| `app/page.rs` | `/` |
| `app/blog/[id]/page.rs` | `/blog/:id` |
| `app/layout.rs` | Shared layout for all pages. |
| `app/notfound.rs` | Fallback for unmatched paths. |

## Run

From the workspace root:

```bash
cargo run -p router-example
```

For browser History API behavior:

```bash
cd examples/router_example
trunk serve --open
```

Open `/blog/42` to verify dynamic parameter extraction. Browser refreshes on nested paths require the static host to fall back to `index.html`.

## How generation works

`build.rs` calls `xen_router_build::generate("app")`. The generated source is included from `OUT_DIR` in [`src/main.rs`](src/main.rs), where `build_router().build()` resolves the current path.

See [`xen-router`](../../crates/xen-router) and [`xen-router-build`](../../crates/xen-router-build) for the runtime and directory conventions.

## License

Licensed under the [Apache License 2.0](LICENSE).
