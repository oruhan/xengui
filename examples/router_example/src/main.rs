// SPDX-License-Identifier: Apache-2.0
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(not(target_arch = "wasm32"))]
use xenframe::WindowPosition;
use xenframe::{ App, AppConfig };

include!(concat!(env!("OUT_DIR"), "/xen_router_generated.rs"));

// write debug messages directly into the screen
#[cfg(target_arch = "wasm32")]
fn show_debug_overlay(message: &str) {
    let Some(document) = web_sys::window().and_then(|w| w.document()) else {
        return;
    };
    let Some(body) = document.body() else {
        return;
    };
    let Ok(overlay) = document.create_element("pre") else {
        return;
    };
    let _ = overlay.set_attribute(
        "style",
        "position:fixed;inset:0;margin:0;background:rgba(0,0,0,0);color:#ff8080;\
         font:12px/1.5 monospace;padding:16px;white-space:pre-wrap;\
         z-index:2147483647;overflow:auto;pointer-events:none;"
    );
    overlay.set_text_content(Some(message));
    let _ = body.append_child(&overlay);
}

// write debug messages directly into the screen
#[cfg(target_arch = "wasm32")]
fn install_panic_hook() {
    std::panic::set_hook(
        Box::new(|info| {
            console_error_panic_hook::hook(info);
            show_debug_overlay(&format!("xengui panicked:\n\n{info}"));
        })
    );
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(target_arch = "wasm32")]
    {
        // console_error_panic_hook::set_once();
        install_panic_hook();
        let _ = console_log::init_with_level(log::Level::Info);
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = env_logger::Builder
            ::new()
            .filter_module("xenframe", log::LevelFilter::Info)
            .filter_module("xengui", log::LevelFilter::Debug)
            .filter_module("xengui_wgpu", log::LevelFilter::Trace)
            .filter_level(log::LevelFilter::Warn) 
            .format_timestamp(None)
            .try_init();
    }

    let config = AppConfig {
        title: "XenGui | Cross-platform UI in Rust".into(),
        reload_shortcut: true,

        #[cfg(not(target_arch = "wasm32"))]
        width: 640,
        #[cfg(not(target_arch = "wasm32"))]
        height: 480,
        #[cfg(not(target_arch = "wasm32"))]
        position: WindowPosition::Center,
        #[cfg(not(target_arch = "wasm32"))]
        decorations: true,

        ..Default::default()
    };

    let mut app = App::new(config);

    app.with_font(
        "Noto Sans",
        include_bytes!(
            concat!(env!("CARGO_MANIFEST_DIR"), "/fonts/NotoSans-VariableFont.ttf")
        ).to_vec()
    );

    app.render(|| { build_router().build() });

    if let Err(e) = app.run() {
        eprintln!("[router-example] Error running app: {:?}", e);
    }

    Ok(())
}
