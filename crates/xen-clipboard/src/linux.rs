use std::sync::Mutex;

use super::ClipboardBackend;
use crate::ClipboardError;

pub struct LinuxClipboard {
    // Linux clipboards use selection ownership: the owner must remain alive to
    // serve paste requests. Keeping the arboard handle here preserves ownership
    // between calls, while the mutex serializes access to the native clipboard.
    clipboard: Mutex<Option<arboard::Clipboard>>,
}

impl LinuxClipboard {
    #[inline]
    pub fn new() -> Self {
        Self {
            clipboard: Mutex::new(None),
        }
    }

    fn with_clipboard<T>(
        &self,
        operation: impl FnOnce(&mut arboard::Clipboard) -> Result<T, arboard::Error>,
        map_error: fn(arboard::Error) -> ClipboardError,
    ) -> Result<T, ClipboardError> {
        let mut clipboard = self.clipboard.lock().map_err(|_| {
            ClipboardError::PlatformError("Linux clipboard lock is poisoned".into())
        })?;

        if clipboard.is_none() {
            *clipboard = Some(arboard::Clipboard::new().map_err(map_open_error)?);
        }

        operation(
            clipboard
                .as_mut()
                .expect("clipboard is initialized immediately above"),
        )
        .map_err(map_error)
    }
}

impl ClipboardBackend for LinuxClipboard {
    fn get_text(&self, callback: Box<dyn FnOnce(Result<Option<String>, ClipboardError>) + Send>) {
        let result = self
            .with_clipboard(arboard::Clipboard::get_text, map_read_error)
            .map(Some);
        callback(match result {
            Err(ClipboardError::FormatUnavailable) => Ok(None),
            other => other,
        });
    }

    fn set_text(&self, text: String, callback: Box<dyn FnOnce(Result<(), ClipboardError>) + Send>) {
        callback(self.with_clipboard(|clipboard| clipboard.set_text(text), map_write_error));
    }

    fn has_text(&self, callback: Box<dyn FnOnce(Result<bool, ClipboardError>) + Send>) {
        let result = self
            .with_clipboard(arboard::Clipboard::get_text, map_read_error)
            .map(|_| true);
        callback(match result {
            Err(ClipboardError::FormatUnavailable) => Ok(false),
            other => other,
        });
    }
}

fn map_open_error(error: arboard::Error) -> ClipboardError {
    match error {
        arboard::Error::ClipboardNotSupported => ClipboardError::Unsupported,
        arboard::Error::ClipboardOccupied => ClipboardError::OpenFailed,
        other => ClipboardError::PlatformError(other.to_string()),
    }
}

fn map_read_error(error: arboard::Error) -> ClipboardError {
    match error {
        arboard::Error::ContentNotAvailable => ClipboardError::FormatUnavailable,
        arboard::Error::ClipboardNotSupported => ClipboardError::Unsupported,
        arboard::Error::ClipboardOccupied => ClipboardError::OpenFailed,
        arboard::Error::ConversionFailure => ClipboardError::ReadFailed,
        other => ClipboardError::PlatformError(other.to_string()),
    }
}

fn map_write_error(error: arboard::Error) -> ClipboardError {
    match error {
        arboard::Error::ClipboardNotSupported => ClipboardError::Unsupported,
        arboard::Error::ClipboardOccupied => ClipboardError::OpenFailed,
        arboard::Error::ConversionFailure | arboard::Error::ContentNotAvailable => {
            ClipboardError::WriteFailed
        }
        other => ClipboardError::PlatformError(other.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_missing_text_to_format_unavailable() {
        assert_eq!(
            map_read_error(arboard::Error::ContentNotAvailable),
            ClipboardError::FormatUnavailable
        );
    }

    #[test]
    fn maps_write_conversion_failure() {
        assert_eq!(
            map_write_error(arboard::Error::ConversionFailure),
            ClipboardError::WriteFailed
        );
    }

    #[test]
    fn maps_unsupported_clipboard() {
        assert_eq!(
            map_open_error(arboard::Error::ClipboardNotSupported),
            ClipboardError::Unsupported
        );
    }
}
