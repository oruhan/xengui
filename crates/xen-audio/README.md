# xen-audio

[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](../../LICENSE)

`xen-audio` is a framework-independent abstraction for local audio playback. Its native `RodioBackend` supports loading, playback, pausing, seeking, volume control, and playback-state queries.

## Features

- Backend-neutral `AudioBackend` trait.
- Load audio from a filesystem path or in-memory bytes.
- Play, pause, stop, seek, position, duration, and volume controls.
- Native decoding and playback through `rodio` with Symphonia codecs.
- No dependency on `xengui`.

Linux builds require ALSA development files discoverable through `pkg-config`. Install the appropriate ALSA development package for your distribution before building this crate.

## Installation

Within this workspace:

```toml
[dependencies]
xen-audio = { path = "../xen-workspace/crates/xen-audio" }
```

## Usage

```rust,no_run
use std::path::Path;
use xen_audio::{AudioBackend, RodioBackend};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut audio = RodioBackend::new()?;
    audio.load_from_path(Path::new("track.mp3"))?;
    audio.set_volume(0.8);
    audio.play();
    Ok(())
}
```

`RodioBackend` is available only on non-WebAssembly targets. The public trait remains available on WebAssembly so applications can provide a browser-specific backend.

## License

Licensed under the workspace [Apache License 2.0](../../LICENSE).
