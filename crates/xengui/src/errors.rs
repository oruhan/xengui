// SPDX-License-Identifier: Apache-2.0
use std::fmt;

/// Asset loading failures that callers can inspect and recover from.
#[derive(Clone, Debug, PartialEq, Eq)]
#[allow(missing_docs)]
pub enum AssetError {
    Io { path: String, message: String },
    Load { uri: String, message: String },
    Decode { kind: &'static str, message: String },
    InvalidBuffer { expected: usize, actual: usize },
    InvalidUtf8(String),
    Parse { kind: &'static str, message: String },
}

impl fmt::Display for AssetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { path, message } => write!(f, "cannot read asset '{path}': {message}"),
            Self::Load { uri, message } => write!(f, "cannot load asset '{uri}': {message}"),
            Self::Decode { kind, message } => write!(f, "cannot decode {kind}: {message}"),
            Self::InvalidBuffer { expected, actual } => write!(
                f,
                "invalid RGBA buffer length: expected {expected}, got {actual}"
            ),
            Self::InvalidUtf8(message) => write!(f, "asset is not valid UTF-8: {message}"),
            Self::Parse { kind, message } => write!(f, "cannot parse {kind}: {message}"),
        }
    }
}
impl std::error::Error for AssetError {}

#[derive(Clone, Debug, PartialEq, Eq)]
#[allow(missing_docs)]
pub enum FontError {
    InvalidData(String),
    Atlas(String),
    Render(String),
}
impl fmt::Display for FontError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidData(v) => write!(f, "invalid font data: {v}"),
            Self::Atlas(v) => write!(f, "font atlas failure: {v}"),
            Self::Render(v) => write!(f, "font render failure: {v}"),
        }
    }
}
impl std::error::Error for FontError {}

#[derive(Clone, Debug, PartialEq, Eq)]
#[allow(missing_docs)]
pub enum PlatformError {
    Initialization(String),
    Unavailable(&'static str),
    Operation(String),
}
impl fmt::Display for PlatformError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Initialization(v) => write!(f, "platform initialization failed: {v}"),
            Self::Unavailable(v) => write!(f, "platform service unavailable: {v}"),
            Self::Operation(v) => write!(f, "platform operation failed: {v}"),
        }
    }
}
impl std::error::Error for PlatformError {}
