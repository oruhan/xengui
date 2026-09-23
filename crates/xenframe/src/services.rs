// SPDX-License-Identifier: Apache-2.0
//! `xengui` platform-service adapters backed by the current OS/browser.

use xengui::{
    AccessibilityAdapter, AssetLoader, ClipboardReadCallback, ClipboardService,
    NativeTextInputSnapshot, PlatformError, PlatformServices, SemanticsNode, TextInputService, Uri,
    WindowId,
};

pub(crate) struct HostPlatformServices {
    clipboard: xen_clipboard::Clipboard,
}

impl HostPlatformServices {
    pub(crate) fn new() -> Self {
        Self {
            clipboard: xen_clipboard::Clipboard::new(),
        }
    }
}

impl ClipboardService for HostPlatformServices {
    fn read_text(&self, callback: ClipboardReadCallback) {
        self.clipboard.get_text(move |result| {
            callback(result.map_err(|error| PlatformError::Operation(error.to_string())));
        });
    }

    fn write_text(&self, text: String) -> Result<(), PlatformError> {
        self.clipboard.set_text(text, |result| {
            if let Err(error) = result {
                log::error!("clipboard write failed: {error}");
            }
        });
        Ok(())
    }
}

impl TextInputService for HostPlatformServices {
    fn update(&self, _snapshot: Option<&NativeTextInputSnapshot>) -> Result<(), PlatformError> {
        Ok(())
    }
}

impl AssetLoader for HostPlatformServices {
    fn load(&self, uri: &Uri) -> Result<Vec<u8>, PlatformError> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            std::fs::read(uri.as_str()).map_err(|error| PlatformError::Operation(error.to_string()))
        }
        #[cfg(target_arch = "wasm32")]
        {
            let _ = uri;
            Err(PlatformError::Unavailable("synchronous web asset loading"))
        }
    }
}

impl AccessibilityAdapter for HostPlatformServices {
    fn update(&self, _roots: &[SemanticsNode]) -> Result<(), PlatformError> {
        Ok(())
    }
}

impl PlatformServices for HostPlatformServices {
    fn clipboard(&self) -> &dyn ClipboardService {
        self
    }

    fn open_uri(&self, uri: &Uri) -> Result<(), PlatformError> {
        #[cfg(target_arch = "wasm32")]
        {
            let window = web_sys::window().ok_or(PlatformError::Unavailable("browser window"))?;
            let target = if uri.opens_in_new_context() {
                "_blank"
            } else {
                "_self"
            };
            window
                .open_with_url_and_target(uri.as_str(), target)
                .map_err(|error| PlatformError::Operation(format!("{error:?}")))?;
            Ok(())
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            open_native_uri(uri.as_str())
        }
    }

    fn text_input(&self) -> &dyn TextInputService {
        self
    }

    fn asset_loader(&self) -> &dyn AssetLoader {
        self
    }

    fn accessibility(&self) -> &dyn AccessibilityAdapter {
        self
    }

    fn request_redraw(&self, _window: WindowId) {}
}

#[cfg(not(target_arch = "wasm32"))]
fn open_native_uri(uri: &str) -> Result<(), PlatformError> {
    #[cfg(target_os = "windows")]
    let result = std::process::Command::new("cmd")
        .args(["/C", "start", "", uri])
        .spawn();
    #[cfg(target_os = "macos")]
    let result = std::process::Command::new("open").arg(uri).spawn();
    #[cfg(any(
        target_os = "linux",
        target_os = "freebsd",
        target_os = "openbsd",
        target_os = "netbsd"
    ))]
    let result = std::process::Command::new("xdg-open").arg(uri).spawn();
    #[cfg(any(target_os = "android", target_os = "ios"))]
    let result: Result<std::process::Child, std::io::Error> = Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "URI opening requires the mobile host adapter",
    ));

    result
        .map(|_| ())
        .map_err(|error| PlatformError::Operation(error.to_string()))
}
