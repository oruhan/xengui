// SPDX-License-Identifier: Apache-2.0
//! User config (config.toml) and directory scanning. Native-only; the
//! wasm32 build has no filesystem and will pick tracks through the
//! browser's own file picker instead (not implemented yet).
#![cfg(not(target_arch = "wasm32"))]

use serde::{ Deserialize, Serialize };
use std::path::{ Path, PathBuf };
use xengui::Color;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LibraryConfig {
    #[serde(default)]
    pub library: LibrarySection,
    #[serde(default)]
    pub playback: PlaybackSection,
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct LibrarySection {
    #[serde(default)]
    pub scan_paths: Vec<String>,
    #[serde(default)]
    pub watch_for_changes: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PlaybackSection {
    #[serde(default = "default_volume")]
    pub default_volume: f32,
}

fn default_volume() -> f32 {
    0.7
}

impl Default for PlaybackSection {
    fn default() -> Self {
        Self { default_volume: default_volume() }
    }
}

pub fn load_or_init_config() -> LibraryConfig {
    let Some(dirs) = directories::ProjectDirs::from("", "", "pearl") else {
        return LibraryConfig {
            library: LibrarySection::default(),
            playback: PlaybackSection::default(),
        };
    };
    let path = dirs.config_dir().join("config.toml");

    if let Ok(text) = std::fs::read_to_string(&path) {
        if let Ok(config) = toml::from_str::<LibraryConfig>(&text) {
            return config;
        }
        log::warn!("pearl: config.toml malformed, using defaults");
    }

    let config = LibraryConfig {
        library: LibrarySection { scan_paths: default_scan_paths(), watch_for_changes: true },
        playback: PlaybackSection::default(),
    };
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(text) = toml::to_string_pretty(&config) {
        let _ = std::fs::write(&path, text);
    }
    config
}

fn default_scan_paths() -> Vec<String> {
    directories::UserDirs
        ::new()
        .and_then(|d| d.audio_dir().map(|p| p.to_string_lossy().to_string()))
        .into_iter()
        .collect()
}

#[derive(Clone, PartialEq)]
pub struct ScannedTrack {
    pub path: PathBuf,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub duration_secs: u32,
    pub art_color: Color,
    pub cover: Option<(u32, u32, Vec<u8>)>,
}

const PLACEHOLDER_COLORS: [Color; 6] = [
    Color::VIOLET_400,
    Color::AMBER_400,
    Color::CYAN_400,
    Color::ROSE_400,
    Color::TEAL_400,
    Color::PINK_400,
];

/// Blocking; call through `xengui::task::spawn_blocking`.
pub fn scan_library(scan_paths: &[String]) -> Vec<ScannedTrack> {
    let extensions = ["mp3", "m4a", "flac", "wav", "ogg", "opus"];
    let mut tracks = Vec::new();

    for root in scan_paths {
        for entry in walkdir::WalkDir::new(root).into_iter().filter_map(Result::ok) {
            if !entry.file_type().is_file() {
                continue;
            }
            let path = entry.path();
            let is_audio = path
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| extensions.contains(&e.to_lowercase().as_str()))
                .unwrap_or(false);
            if !is_audio {
                continue;
            }
            if let Some(track) = read_track_tags(path) {
                tracks.push(track);
            }
        }
    }

    tracks
}

fn read_track_tags(path: &Path) -> Option<ScannedTrack> {
    use lofty::file::{ AudioFile, TaggedFileExt };
    use lofty::tag::Accessor;

    let tagged = lofty::read_from_path(path).ok()?;
    let tag = tagged.primary_tag().or_else(|| tagged.first_tag());

    let title = tag
        .and_then(|t| t.title())
        .map(|s| s.to_string())
        .unwrap_or_else(||
            path
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default()
        );
    let artist = tag
        .and_then(|t| t.artist())
        .map(|s| s.to_string())
        .unwrap_or_else(|| "Unknown Artist".to_string());
    let album = tag
        .and_then(|t| t.album())
        .map(|s| s.to_string())
        .unwrap_or_else(|| "Unknown Album".to_string());
    let duration_secs = tagged.properties().duration().as_secs() as u32;

    let cover = tag
        .and_then(|t| t.pictures().first())
        .and_then(|pic| {
            image
                ::load_from_memory(pic.data())
                .ok()
                .map(|img| {
                    let rgba = img.to_rgba8();
                    let (w, h) = rgba.dimensions();
                    (w, h, rgba.into_raw())
                })
        });

    let color_index =
        title
            .bytes()
            .map(|b| b as usize)
            .sum::<usize>() % PLACEHOLDER_COLORS.len();

    Some(ScannedTrack {
        path: path.to_path_buf(),
        title,
        artist,
        album,
        duration_secs,
        art_color: PLACEHOLDER_COLORS[color_index],
        cover,
    })
}
