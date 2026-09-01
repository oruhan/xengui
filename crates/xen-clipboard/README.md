# xen-clipboard

[![Crates.io](https://img.shields.io/crates/v/xen-clipboard.svg)](https://crates.io/crates/xen-clipboard)
[![Documentation](https://docs.rs/xen-clipboard/badge.svg)](https://docs.rs/xen-clipboard)
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

`xen-clipboard` provides a callback-based text clipboard API for XenGui and other Rust applications.

## Platform support

| Target | Status |
| --- | --- |
| Windows | Read, write, and content checks are implemented. |
| WebAssembly | Implemented through the browser Clipboard API. |
| Linux | Compiles with an explicit unsupported backend; text operations currently return `ClipboardError::Unsupported`. |
| macOS, Android, iOS | Not implemented. |

Browser clipboard access requires a secure context and may require a user gesture or permission, depending on browser policy.

## Installation

```toml
[dependencies]
xen-clipboard = "0.1.7"
```

## Usage

```rust
use xen_clipboard::Clipboard;

let clipboard = Clipboard::new();

clipboard.set_text("Hello", |result| {
    if let Err(error) = result {
        eprintln!("copy failed: {error}");
    }
});

clipboard.get_text(|result| match result {
    Ok(Some(text)) => println!("{text}"),
    Ok(None) => println!("clipboard is empty"),
    Err(error) => eprintln!("paste failed: {error}"),
});
```

Callbacks may execute asynchronously. Do not assume a result is available immediately after invoking an operation.

## Documentation and support

- [API reference](https://docs.rs/xen-clipboard)
- [Project documentation](https://xengui.vercel.app/docs/xen-clipboard)
- [Issues](https://github.com/randseas/xengui/issues)

## License

Licensed under the [Apache License 2.0](LICENSE).
