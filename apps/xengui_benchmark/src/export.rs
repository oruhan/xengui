// SPDX-License-Identifier: Apache-2.0

#[cfg(target_os = "android")]
use std::path::PathBuf;

#[cfg(target_os = "android")]
static ANDROID_DATA_DIR: std::sync::OnceLock<PathBuf> = std::sync::OnceLock::new();

#[cfg(target_os = "android")]
pub fn set_android_data_dir(path: PathBuf) {
    let _ = ANDROID_DATA_DIR.set(path);
}

#[cfg(not(target_arch = "wasm32"))]
pub fn export(json: &str) -> Result<String, String> {
    #[cfg(target_os = "android")]
    let path = ANDROID_DATA_DIR
        .get()
        .cloned()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("xengui-benchmark-trace.json");

    #[cfg(not(target_os = "android"))]
    let path = std::env::current_dir()
        .map_err(|error| error.to_string())?
        .join("xengui-benchmark-trace.json");

    std::fs::write(&path, json).map_err(|error| error.to_string())?;
    Ok(path.display().to_string())
}

#[cfg(target_arch = "wasm32")]
pub fn export(json: &str) -> Result<String, String> {
    use wasm_bindgen::JsCast;

    let parts = js_sys::Array::new();
    parts.push(&wasm_bindgen::JsValue::from_str(json));
    let options = web_sys::BlobPropertyBag::new();
    options.set_type("application/json");
    let blob = web_sys::Blob::new_with_str_sequence_and_options(&parts, &options)
        .map_err(|error| format!("Blob oluşturulamadı: {error:?}"))?;
    let url = web_sys::Url::create_object_url_with_blob(&blob)
        .map_err(|error| format!("URL oluşturulamadı: {error:?}"))?;
    let document = web_sys::window()
        .and_then(|window| window.document())
        .ok_or_else(|| "document bulunamadı".to_string())?;
    let anchor = document
        .create_element("a")
        .map_err(|error| format!("anchor oluşturulamadı: {error:?}"))?
        .dyn_into::<web_sys::HtmlAnchorElement>()
        .map_err(|_| "anchor dönüştürülemedi".to_string())?;
    anchor.set_href(&url);
    anchor.set_download("xengui-benchmark-trace.json");
    anchor.click();
    web_sys::Url::revoke_object_url(&url)
        .map_err(|error| format!("geçici URL bırakılamadı: {error:?}"))?;
    Ok("xengui-benchmark-trace.json indirildi".to_string())
}
