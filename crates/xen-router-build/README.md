# xen-router-build

[![Crates.io](https://img.shields.io/crates/v/xen-router-build.svg)](https://crates.io/crates/xen-router-build)
[![Documentation](https://docs.rs/xen-router-build/badge.svg)](https://docs.rs/xen-router-build)
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

`xen-router-build` generates a [`xen-router`](../xen-router) router from an application's directory structure during `build.rs` execution.

## Supported conventions

| Path | Meaning |
| --- | --- |
| `app/page.rs` | Root page (`/`). |
| `app/layout.rs` | Layout wrapped around descendant pages. |
| `app/notfound.rs` | Root not-found page. |
| `app/blog/[id]/page.rs` | Dynamic segment (`/blog/:id`). |
| `app/docs/[...rest]/page.rs` | Trailing catch-all (`/docs/*rest`). |
| `app/(group)/page.rs` | Grouped files without a URL segment. |

Only a root `notfound.rs` is currently supported; nested not-found files produce a build warning.

## Setup

Add runtime and build dependencies:

```toml
[dependencies]
xen-router = "0.1.0"

[build-dependencies]
xen-router-build = "0.1.0"
```

Create `build.rs`:

```rust
fn main() {
    xen_router_build::generate("app");
}
```

Include the generated source from the application:

```rust
include!(concat!(env!("OUT_DIR"), "/xen_router_generated.rs"));

// Inside App::render:
let page = build_router().build();
```

Each `page.rs` exports `page(&RouteParams) -> Box<dyn Widget>`. Each `layout.rs` exports `layout(&RouteParams, Box<dyn Widget>) -> Box<dyn Widget>`. The root `notfound.rs` exports `not_found() -> Box<dyn Widget>`.

## License

Licensed under the [Apache License 2.0](LICENSE).
