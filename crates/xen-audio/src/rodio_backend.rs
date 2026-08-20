// SPDX-License-Identifier: Apache-2.0
use std::fs::File;
use std::io::{ BufReader, Cursor };
use std::path::Path;
use web_time::Duration;
use rodio::{ Decoder, OutputStream, OutputStreamHandle, Sink, Source };

use crate::{ AudioBackend, AudioError, PlaybackState };

pub struct RodioBackend {
    // Must stay alive for the whole backend lifetime, or the output
    // device closes and every sink built from `handle` goes silent.
    _stream: OutputStream,
    handle: OutputStreamHandle,
    sink: Option<Sink>,
    duration: Option<Duration>,
    volume: f32,
    state: PlaybackState,
}

impl RodioBackend {
    pub fn new() -> Result<Self, AudioError> {
        let (stream, handle) = OutputStream::try_default().map_err(|e|
            AudioError::Device(e.to_string())
        )?;
        Ok(Self {
            _stream: stream,
            handle,
            sink: None,
            duration: None,
            volume: 1.0,
            state: PlaybackState::Idle,
        })
    }

    fn load_decoder<R>(&mut self, decoder: Decoder<R>) -> Result<(), AudioError>
        where R: std::io::Read + std::io::Seek + Send + Sync + 'static
    {
        let sink = Sink::try_new(&self.handle).map_err(|e| AudioError::Device(e.to_string()))?;
        sink.set_volume(self.volume);
        self.duration = decoder.total_duration();
        sink.append(decoder);
        // Loaded paused; the caller decides whether to start playing
        // immediately (matches how the UI already tracks is_playing).
        sink.pause();
        self.sink = Some(sink);
        self.state = PlaybackState::Paused;
        Ok(())
    }
}

impl AudioBackend for RodioBackend {
    fn load_from_path(&mut self, path: &Path) -> Result<(), AudioError> {
        let file = File::open(path).map_err(|e| AudioError::Io(e.to_string()))?;
        let decoder = Decoder::new(BufReader::new(file)).map_err(|e|
            AudioError::Decode(e.to_string())
        )?;
        self.load_decoder(decoder)
    }

    fn load_from_bytes(&mut self, bytes: Vec<u8>) -> Result<(), AudioError> {
        let decoder = Decoder::new(Cursor::new(bytes)).map_err(|e|
            AudioError::Decode(e.to_string())
        )?;
        self.load_decoder(decoder)
    }

    fn play(&mut self) {
        if let Some(sink) = &self.sink {
            sink.play();
            self.state = PlaybackState::Playing;
        }
    }

    fn pause(&mut self) {
        if let Some(sink) = &self.sink {
            sink.pause();
            self.state = PlaybackState::Paused;
        }
    }

    fn stop(&mut self) {
        if let Some(sink) = self.sink.take() {
            sink.stop();
        }
        self.state = PlaybackState::Idle;
        self.duration = None;
    }

    fn seek(&mut self, position: Duration) -> Result<(), AudioError> {
        let Some(sink) = &self.sink else {
            return Err(AudioError::NotLoaded);
        };
        sink.try_seek(position).map_err(|e| AudioError::Seek(e.to_string()))
    }

    fn set_volume(&mut self, volume: f32) {
        self.volume = volume.clamp(0.0, 1.0);
        if let Some(sink) = &self.sink {
            sink.set_volume(self.volume);
        }
    }

    fn volume(&self) -> f32 {
        self.volume
    }

    fn position(&self) -> Duration {
        self.sink.as_ref().map(Sink::get_pos).unwrap_or_default()
    }

    fn duration(&self) -> Option<Duration> {
        self.duration
    }

    fn state(&self) -> PlaybackState {
        if self.sink.as_ref().is_some_and(Sink::empty) && self.state == PlaybackState::Playing {
            PlaybackState::Ended
        } else {
            self.state
        }
    }
}
