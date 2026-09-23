// SPDX-License-Identifier: Apache-2.0
//! Host-provided services used by otherwise platform-independent widgets.

use crate::{PlatformError, SemanticsNode};
use std::{cell::RefCell, rc::Rc, sync::Mutex};

/// Callback used by asynchronous clipboard reads.
pub type ClipboardReadCallback =
    Box<dyn FnOnce(Result<Option<String>, PlatformError>) + Send + 'static>;

/// Clipboard access supplied by the embedding host.
pub trait ClipboardService {
    /// Reads text, invoking `callback` when the host has completed the request.
    fn read_text(&self, callback: ClipboardReadCallback);

    /// Replaces the text currently stored by the host clipboard.
    fn write_text(&self, text: String) -> Result<(), PlatformError>;
}

/// A host-independent URI passed to [`PlatformServices::open_uri`].
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Uri {
    value: String,
    new_context: bool,
}

impl Uri {
    /// Creates a URI without interpreting its scheme.
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            new_context: false,
        }
    }

    /// Requests a separate browser tab/window when the host supports it.
    pub fn with_new_context(mut self, value: bool) -> Self {
        self.new_context = value;
        self
    }

    /// Returns the URI string.
    pub fn as_str(&self) -> &str {
        &self.value
    }

    /// Whether opening should prefer a separate context.
    pub fn opens_in_new_context(&self) -> bool {
        self.new_context
    }
}

impl From<&str> for Uri {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for Uri {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

/// Opaque window identity understood by a host.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct WindowId(pub u64);

/// Snapshot mirrored by hosts that expose a native text-input control.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeTextInputSnapshot {
    /// Current editable value.
    pub value: String,
    /// Placeholder shown for an empty value.
    pub placeholder: String,
    /// Optional maximum character count.
    pub max_length: Option<usize>,
    /// Whether editing is disabled while selection remains possible.
    pub read_only: bool,
}

/// Native text-input bridge supplied by a host.
pub trait TextInputService {
    /// Mirrors the focused widget state; `None` hides the host input.
    fn update(&self, snapshot: Option<&NativeTextInputSnapshot>) -> Result<(), PlatformError>;
}

/// Binary asset loading supplied by a host or application.
pub trait AssetLoader {
    /// Loads all bytes addressed by `uri`.
    fn load(&self, uri: &Uri) -> Result<Vec<u8>, PlatformError>;
}

/// Accessibility bridge supplied by a host.
pub trait AccessibilityAdapter {
    /// Publishes the latest platform-independent semantics tree.
    fn update(&self, roots: &[SemanticsNode]) -> Result<(), PlatformError>;
}

/// Complete set of operations that cross the core/host boundary.
pub trait PlatformServices {
    /// Returns the host clipboard adapter.
    fn clipboard(&self) -> &dyn ClipboardService;
    /// Asks the host to open `uri`.
    fn open_uri(&self, uri: &Uri) -> Result<(), PlatformError>;
    /// Returns the native text-input adapter.
    fn text_input(&self) -> &dyn TextInputService;
    /// Returns the host asset loader.
    fn asset_loader(&self) -> &dyn AssetLoader;
    /// Returns the accessibility adapter.
    fn accessibility(&self) -> &dyn AccessibilityAdapter;
    /// Wakes the host window so it can render another frame.
    fn request_redraw(&self, window: WindowId);
}

/// Deterministic in-memory services for tests and renderer-less hosts.
#[derive(Default)]
pub struct HeadlessPlatformServices {
    clipboard: Mutex<Option<String>>,
    opened_uris: Mutex<Vec<Uri>>,
    text_input: Mutex<Option<NativeTextInputSnapshot>>,
    semantics: Mutex<Vec<SemanticsNode>>,
    redraws: Mutex<Vec<WindowId>>,
}

impl HeadlessPlatformServices {
    /// Returns the current in-memory clipboard value.
    pub fn clipboard_text(&self) -> Option<String> {
        self.clipboard.lock().ok().and_then(|value| value.clone())
    }

    /// Returns every URI opened since construction.
    pub fn opened_uris(&self) -> Vec<Uri> {
        self.opened_uris
            .lock()
            .map_or_else(|_| Vec::new(), |v| v.clone())
    }

    /// Returns the most recently mirrored native text input.
    pub fn text_input_snapshot(&self) -> Option<NativeTextInputSnapshot> {
        self.text_input.lock().ok().and_then(|value| value.clone())
    }

    /// Returns the most recently published accessibility roots.
    pub fn semantics(&self) -> Vec<SemanticsNode> {
        self.semantics
            .lock()
            .map_or_else(|_| Vec::new(), |v| v.clone())
    }

    /// Returns redraw requests in arrival order.
    pub fn redraw_requests(&self) -> Vec<WindowId> {
        self.redraws
            .lock()
            .map_or_else(|_| Vec::new(), |v| v.clone())
    }
}

impl ClipboardService for HeadlessPlatformServices {
    fn read_text(&self, callback: ClipboardReadCallback) {
        let value = self
            .clipboard
            .lock()
            .map(|value| value.clone())
            .map_err(|_| PlatformError::Operation("headless clipboard lock is poisoned".into()));
        callback(value);
    }

    fn write_text(&self, text: String) -> Result<(), PlatformError> {
        *self.clipboard.lock().map_err(|_| {
            PlatformError::Operation("headless clipboard lock is poisoned".into())
        })? = Some(text);
        Ok(())
    }
}

impl TextInputService for HeadlessPlatformServices {
    fn update(&self, snapshot: Option<&NativeTextInputSnapshot>) -> Result<(), PlatformError> {
        *self.text_input.lock().map_err(|_| {
            PlatformError::Operation("headless text-input lock is poisoned".into())
        })? = snapshot.cloned();
        Ok(())
    }
}

impl AssetLoader for HeadlessPlatformServices {
    fn load(&self, _uri: &Uri) -> Result<Vec<u8>, PlatformError> {
        Err(PlatformError::Unavailable("asset loader"))
    }
}

impl AccessibilityAdapter for HeadlessPlatformServices {
    fn update(&self, roots: &[SemanticsNode]) -> Result<(), PlatformError> {
        *self.semantics.lock().map_err(|_| {
            PlatformError::Operation("headless accessibility lock is poisoned".into())
        })? = roots.to_vec();
        Ok(())
    }
}

impl PlatformServices for HeadlessPlatformServices {
    fn clipboard(&self) -> &dyn ClipboardService {
        self
    }

    fn open_uri(&self, uri: &Uri) -> Result<(), PlatformError> {
        self.opened_uris
            .lock()
            .map_err(|_| PlatformError::Operation("headless URI lock is poisoned".into()))?
            .push(uri.clone());
        Ok(())
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

    fn request_redraw(&self, window: WindowId) {
        if let Ok(mut redraws) = self.redraws.lock() {
            redraws.push(window);
        }
    }
}

thread_local! {
    static DEFAULT_PLATFORM_SERVICES: RefCell<Rc<dyn PlatformServices>> =
        RefCell::new(Rc::new(HeadlessPlatformServices::default()));
}

/// Replaces the services inherited by subsequently created [`crate::EventCtx`] values.
///
/// Embedding runtimes should call this on their UI thread before dispatching events.
/// A host may instead use [`crate::EventCtx::with_platform_services`] when it owns
/// event-context construction directly.
pub fn set_platform_services(services: Rc<dyn PlatformServices>) {
    DEFAULT_PLATFORM_SERVICES.with(|current| *current.borrow_mut() = services);
}

pub(crate) fn default_platform_services() -> Rc<dyn PlatformServices> {
    DEFAULT_PLATFORM_SERVICES.with(|current| Rc::clone(&current.borrow()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn headless_services_cover_clipboard_uri_and_redraw() {
        let services = HeadlessPlatformServices::default();
        services.clipboard().write_text("hello".into()).unwrap();
        let read = Arc::new(Mutex::new(None));
        let returned = Arc::clone(&read);
        services.clipboard().read_text(Box::new(move |value| {
            *returned.lock().unwrap() = Some(value.unwrap());
        }));
        services
            .open_uri(&Uri::new("https://example.test"))
            .unwrap();
        services.request_redraw(WindowId(7));

        assert_eq!(*read.lock().unwrap(), Some(Some("hello".into())));
        assert_eq!(services.opened_uris().len(), 1);
        assert_eq!(services.redraw_requests(), vec![WindowId(7)]);
    }
}
