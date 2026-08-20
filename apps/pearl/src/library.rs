// SPDX-License-Identifier: Apache-2.0
//! User config (config.toml) and directory scanning. Native-only; the
//! wasm32 build has no filesystem and will pick tracks through the
//! browser's own file picker instead (not implemented yet).
#![cfg(not(target_arch = "wasm32"))]

use serde::{ Deserialize, Serialize };
use std::path::{ Path, PathBuf };
use std::sync::mpsc;
use std::time::Duration;
use xengui::Color;

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct LibraryConfig {
    #[serde(default)]
    pub library: LibrarySection,
    #[serde(default)]
    pub playback: PlaybackSection,
    #[serde(default)]
    pub playlists: PlaylistsSection,
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

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct PlaylistEntry {
    pub id: u32,
    pub name: String,
    #[serde(default = "default_playlist_color")]
    pub color: [u8; 3],
}

fn default_playlist_color() -> [u8; 3] {
    [124, 58, 237]
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct PlaylistsSection {
    #[serde(default)]
    pub next_id: u32,
    #[serde(default)]
    pub items: Vec<PlaylistEntry>,
}

fn default_volume() -> f32 {
    0.7
}

impl Default for PlaybackSection {
    fn default() -> Self {
        Self { default_volume: default_volume() }
    }
}

fn default_scan_paths() -> Vec<String> {
    directories::UserDirs
        ::new()
        .and_then(|d| d.audio_dir().map(|p| p.to_string_lossy().to_string()))
        .into_iter()
        .collect()
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MediaKind {
    Audio,
    Video,
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
    pub media_kind: MediaKind,
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
    let extensions = ["mp3", "m4a", "flac", "wav", "ogg", "opus", "webm"];
    let mut tracks = Vec::new();

    for root in scan_paths {
        for entry in walkdir::WalkDir::new(root).into_iter().filter_map(Result::ok) {
            if !entry.file_type().is_file() {
                continue;
            }
            let path = entry.path();
            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| e.to_lowercase());
            let is_audio = ext
                .as_deref()
                .map(|e| extensions.contains(&e))
                .unwrap_or(false);
            if !is_audio {
                continue;
            }
            let media_kind = media_kind_for_extension(ext.as_deref().unwrap_or(""));
            if let Some(track) = read_track_tags(path, media_kind) {
                tracks.push(track);
            }
        }
    }

    tracks
}

// .webm can carry a video track alongside its audio; every other
// supported extension is audio-only. Playback today always extracts just
// the audio stream, but tagging the source here lets a future player
// switch into a video-capable mode without rescanning the library.
fn media_kind_for_extension(ext: &str) -> MediaKind {
    if ext == "webm" { MediaKind::Video } else { MediaKind::Audio }
}

fn read_track_tags(path: &Path, media_kind: MediaKind) -> Option<ScannedTrack> {
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
        media_kind,
    })
}

fn config_path() -> Option<PathBuf> {
    directories::ProjectDirs::from("", "", "pearl").map(|d| d.config_dir().join("config.toml"))
}

/// Watches `scan_paths` for filesystem changes and re-scans the library
/// each time something changes, debounced so a burst of events (e.g.
/// copying a whole album) triggers a single rescan. The returned receiver
/// yields a fresh track list after every debounced rescan.
pub fn spawn_library_watcher(scan_paths: Vec<String>) -> mpsc::Receiver<Vec<ScannedTrack>> {
    let (tx, rx) = mpsc::channel();

    std::thread::spawn(move || {
        use notify::{ RecursiveMode, Watcher };

        let (fs_tx, fs_rx) = mpsc::channel();
        let mut watcher = match
            notify::recommended_watcher(move |res| {
                let _ = fs_tx.send(res);
            })
        {
            Ok(watcher) => watcher,
            Err(e) => {
                log::error!("pearl: failed to create file watcher: {e}");
                return;
            }
        };

        for path in &scan_paths {
            if let Err(e) = watcher.watch(Path::new(path), RecursiveMode::Recursive) {
                log::warn!("pearl: failed to watch '{path}': {e}");
            }
        }

        loop {
            let Ok(_first) = fs_rx.recv() else {
                break;
            };
            while fs_rx.recv_timeout(Duration::from_millis(500)).is_ok() {}

            if tx.send(scan_library(&scan_paths)).is_err() {
                break;
            }
        }
    });

    rx
}

fn write_config(path: &Path, config: &LibraryConfig) {
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(text) = toml::to_string_pretty(config) {
        let _ = std::fs::write(path, text);
    }
}

pub fn load_or_init_config() -> LibraryConfig {
    let Some(path) = config_path() else {
        return LibraryConfig::default();
    };

    if let Ok(text) = std::fs::read_to_string(&path) {
        if let Ok(config) = toml::from_str::<LibraryConfig>(&text) {
            return config;
        }
        log::warn!("pearl: config.toml malformed, using defaults");
    }

    let config = LibraryConfig {
        library: LibrarySection { scan_paths: default_scan_paths(), watch_for_changes: true },
        playback: PlaybackSection::default(),
        playlists: PlaylistsSection::default(),
    };
    write_config(&path, &config);
    config
}
