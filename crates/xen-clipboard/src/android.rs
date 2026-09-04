// SPDX-License-Identifier: Apache-2.0

use super::ClipboardBackend;
use crate::ClipboardError;

pub struct AndroidClipboard;

impl AndroidClipboard {
    pub fn new() -> Self {
        Self
    }
}

impl ClipboardBackend for AndroidClipboard {
    fn get_text(&self, callback: Box<dyn FnOnce(Result<Option<String>, ClipboardError>) + Send>) {
        callback(Err(ClipboardError::Unsupported));
    }

    fn set_text(
        &self,
        _text: String,
        callback: Box<dyn FnOnce(Result<(), ClipboardError>) + Send>,
    ) {
        callback(Err(ClipboardError::Unsupported));
    }

    fn has_text(&self, callback: Box<dyn FnOnce(Result<bool, ClipboardError>) + Send>) {
        callback(Ok(false));
    }
}
