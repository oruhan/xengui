# android_demo

Pixel Settings-inspired Material 3 Expressive demo for XenGui. Android and Linux
use the same UI source; navigation is handled by `xen-router`.

## Build the debug APK

```bash
ANDROID_HOME="$HOME/Android/Sdk" \
ANDROID_NDK_HOME="$HOME/Android/Sdk/ndk/27.3.13750724" \
NDK_HOME="$HOME/Android/Sdk/ndk/27.3.13750724" \
cargo apk build -p android_demo
```

The APK is written to `target/debug/apk/android_demo.apk`.

## Install on a connected device

```bash
adb install --no-incremental -r target/debug/apk/android_demo.apk
```

## Run on Linux

The native launcher is a separate workspace package because `cargo-apk` only
accepts a pure `cdylib` package. It includes the exact same application source.

```bash
cargo run -p android_demo_native --bin android_demo
```
