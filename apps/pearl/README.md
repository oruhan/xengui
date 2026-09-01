# Pearl

[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

Pearl is a desktop music-library application built with XenGui. It serves as a realistic integration test for routing, large scrollable views, background tasks, filesystem watching, metadata extraction, cover art, and local audio playback.

## Features

- Recursively scans configured folders for MP3, M4A, FLAC, WAV, Ogg, Opus, and WebM files.
- Reads track metadata, duration, and embedded cover art with `lofty`.
- Plays local media through `xen-audio` and `rodio`.
- Watches library folders and performs debounced background rescans.
- Provides search, playlists, routing, volume controls, and a responsive Material-style interface.
- Persists library and playback settings in the platform configuration directory.

## Requirements

- Rust 1.92 or newer.
- A working native audio output device.
- Platform development libraries required by `winit`, `wgpu`, and `rodio`.

## Run

From the workspace root:

```bash
cargo run -p pearl
```

On first launch, Pearl creates `config.toml` in the platform-specific application configuration directory. If the platform exposes a standard Music directory, it is used as the initial scan path.

Example configuration:

```toml
[library]
scan_paths = ["/path/to/music"]
watch_for_changes = true

[playback]
default_volume = 0.7
muted = false
```

Restart the application after editing scan paths. Large libraries are scanned on a background thread.

## Current limitations

- The production playback backend is native-only; browser file selection and playback are not implemented.
- Playlist-to-track membership is currently demonstrative and not a complete persistent media database.
- The application is an active workspace project, not a separately versioned stable product.

## Project structure

- `src/main.rs` defines the application shell, routes, views, and playback state.
- `src/library.rs` owns configuration, scanning, metadata extraction, and file watching.
- `src/components.rs` contains reusable presentation components.
- `assets/` and `fonts/` contain packaged application resources.

## License

Licensed under the [Apache License 2.0](LICENSE).
