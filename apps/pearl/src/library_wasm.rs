// SPDX-License-Identifier: Apache-2.0
//! Minimal browser-side library model. Browser file picking and metadata
//! extraction are intentionally separate from the native filesystem scanner.

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MediaKind {
    Audio,
    Video,
}
