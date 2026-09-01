# xen-animation

[![Crates.io](https://img.shields.io/crates/v/xen-animation.svg)](https://crates.io/crates/xen-animation)
[![Documentation](https://docs.rs/xen-animation/badge.svg)](https://docs.rs/xen-animation)
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

`xen-animation` is a small, framework-independent transition engine. Callers report target values; `AnimationManager` owns timing, easing, retargeting, and completion for each keyed animation.

## Features

- CSS-compatible cubic Bézier easing and common presets.
- Independently keyed concurrent animations.
- Duration, delay, and per-property transition overrides.
- Mid-flight retargeting without caller-managed timers.
- No rendering, windowing, or XenGui dependency.

## Installation

```toml
[dependencies]
xen-animation = "0.1.4"
```

## Usage

```rust
use web_time::Duration;
use xen_animation::{AnimValue, AnimationManager, Easing, Transition};

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
enum Key {
    Opacity,
}

let mut animations = AnimationManager::<Key>::new();
let transition = Transition::new(Duration::from_millis(250)).easing(Easing::EaseOut);

animations.set_target(Key::Opacity, AnimValue([1.0, 0.0, 0.0, 0.0]), Some(transition));
animations.tick(Duration::from_millis(16));

if let Some(value) = animations.value(Key::Opacity) {
    println!("opacity: {}", value.0[0]);
}
```

Call `tick` once per frame and continue scheduling frames while `is_animating()` returns `true`.

## Compatibility

The minimum supported Rust version is 1.92. Timing uses `web-time`, allowing the same API on native and WebAssembly targets.

## License

Licensed under the [Apache License 2.0](LICENSE).
