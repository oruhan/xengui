// SPDX-License-Identifier: Apache-2.0
use std::fmt;

#[derive(Debug)]
pub enum AudioError {
    Device(String),
    Io(String),
    Decode(String),
    Seek(String),
    NotLoaded,
}

impl fmt::Display for AudioError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Device(m) => write!(f, "audio device error: {m}"),
            Self::Io(m) => write!(f, "io error: {m}"),
            Self::Decode(m) => write!(f, "decode error: {m}"),
            Self::Seek(m) => write!(f, "seek error: {m}"),
            Self::NotLoaded => write!(f, "no track loaded"),
        }
    }
}

impl std::error::Error for AudioError {}
