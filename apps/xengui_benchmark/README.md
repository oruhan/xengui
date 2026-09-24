# XenGui Benchmark Lab

Cross-platform visual, interaction and stress harness for XenGui. The same Rust widget tree runs on desktop, WebAssembly and Android.

## Run

```bash
cargo run -p xengui-benchmark
trunk serve apps/xengui_benchmark/index.html
ANDROID_NDK_ROOT="$ANDROID_NDK_HOME" cargo apk run -p xengui-benchmark --lib
```

Use each lab, mark anything that looks wrong, then open **Trace** and export. The generated `xengui-benchmark-trace.json` is self-contained and intended to be attached directly to an AI debugging request.

On Android the file is written to the app's internal data directory. It can be retrieved from a debuggable build with:

```bash
adb exec-out run-as dev.xengui.benchmark cat files/xengui-benchmark-trace.json > xengui-benchmark-trace.json
```

The JSON is also copied to the platform clipboard when export is requested.
