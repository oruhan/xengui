# xen-router

[![Crates.io](https://img.shields.io/crates/v/xen-router.svg)](https://crates.io/crates/xen-router)
[![Documentation](https://docs.rs/xen-router/badge.svg)](https://docs.rs/xen-router)
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

`xen-router` is a lightweight client-side router for XenGui. It selects a widget tree from the current path and synchronizes navigation with the browser History API on WebAssembly.

## Features

- Ordered route matching with literal, `:parameter`, and trailing `*catch_all` segments.
- Programmatic `push`, `replace`, `back`, and `forward` navigation.
- Query-string access through `search_params`.
- Route-aware button helper through `link`.
- Browser `pushState` and `popstate` synchronization on WebAssembly.
- In-memory paths on native targets for shared application code.

## Installation

```toml
[dependencies]
xen-router = "0.1.0"
```

## Usage

```rust
use xen_router::Router;
use xengui::{Label, Widget};

let router = Router::new()
    .route("/", |_| Box::new(Label::new().label("Home")) as Box<dyn Widget>)
    .route("/users/:id", |params| {
        let id = params.get("id").unwrap_or("unknown");
        Box::new(Label::new().label(format!("User {id}"))) as Box<dyn Widget>
    })
    .not_found(|| Box::new(Label::new().label("Not found")) as Box<dyn Widget>);

let page = router.build();
```

Construct and build the router inside the application's render closure so navigation-triggered redraws resolve the new route. Routes are checked in registration order; register specific patterns before broad wildcards.

For filesystem conventions and generated routes, see [`xen-router-build`](../xen-router-build).

## License

Licensed under the [Apache License 2.0](LICENSE).
