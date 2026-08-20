// SPDX-License-Identifier: Apache-2.0
//! Framework-agnostic local audio playback, kept separate from xengui core
//! the same way xen-animation is - no xengui dependency, reusable by any app.
mod error;

#[cfg(not(target_arch = "wasm32"))]
mod rodio_backend;

pub use error::AudioError;
#[cfg(not(target_arch = "wasm32"))]
pub use rodio_backend::RodioBackend;

use std::path::Path;
use web_time::Duration;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum PlaybackState {
    #[default]
    Idle,
    Playing,
    Paused,
    Ended,
}

pub trait AudioBackend {
    fn load_from_path(&mut self, path: &Path) -> Result<(), AudioError>;
    fn load_from_bytes(&mut self, bytes: Vec<u8>) -> Result<(), AudioError>;
    fn play(&mut self);
    fn pause(&mut self);
    fn stop(&mut self);
    fn seek(&mut self, position: Duration) -> Result<(), AudioError>;
    fn set_volume(&mut self, volume: f32);
    fn volume(&self) -> f32;
    fn position(&self) -> Duration;
    fn duration(&self) -> Option<Duration>;
    fn state(&self) -> PlaybackState;
}
