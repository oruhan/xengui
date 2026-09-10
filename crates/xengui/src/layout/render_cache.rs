// SPDX-License-Identifier: Apache-2.0
use crate::{DrawCommand, LayoutBox, MeasureResult};
use std::collections::HashMap;

struct CachedEntry {
    layout_box: LayoutBox,
    commands: Vec<DrawCommand>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct MeasurementEnvironment {
    theme_generation: u64,
    font_generation: u64,
    scale_factor_bits: u32,
}

impl MeasurementEnvironment {
    pub(crate) fn new(theme_generation: u64, font_generation: u64, scale_factor: f32) -> Self {
        Self {
            theme_generation,
            font_generation,
            scale_factor_bits: scale_factor.to_bits(),
        }
    }
}

#[derive(Default)]
/// Data and behavior represented by `RenderCache`.
pub struct RenderCache {
    entries: HashMap<String, CachedEntry>,
    measured: HashMap<String, MeasureResult>,
    measurement_environment: Option<MeasurementEnvironment>,
    live_generation: HashMap<String, u64>,
    generation: u64,
}

impl RenderCache {
    /// Creates a value with its default configuration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns or updates the `cached_size` value.
    pub fn cached_size(&self, key: &str) -> Option<(f32, f32)> {
        self.entries
            .get(key)
            .map(|e| (e.layout_box.width, e.layout_box.height))
    }

    /// Returns or updates the `try_reuse` value.
    pub fn try_reuse(
        &self,
        key: &str,
        layout_box: LayoutBox,
        dirty: bool,
    ) -> Option<&[DrawCommand]> {
        if dirty {
            return None;
        }
        self.entries
            .get(key)
            .and_then(|entry| (entry.layout_box == layout_box).then_some(entry.commands.as_slice()))
    }

    /// Reuses paint commands when a widget only moved without changing
    /// size. The returned offset translates commands from their cached
    /// position to the widget's current position.
    pub(crate) fn try_reuse_moved(
        &self,
        key: &str,
        layout_box: LayoutBox,
        dirty: bool,
    ) -> Option<(&[DrawCommand], (f32, f32))> {
        if dirty {
            return None;
        }

        self.entries.get(key).and_then(|entry| {
            (entry.layout_box.width == layout_box.width
                && entry.layout_box.height == layout_box.height)
                .then_some((
                    entry.commands.as_slice(),
                    (
                        layout_box.x - entry.layout_box.x,
                        layout_box.y - entry.layout_box.y,
                    ),
                ))
        })
    }

    /// Returns or updates the `store` value.
    pub fn store(&mut self, key: &str, layout_box: LayoutBox, commands: Vec<DrawCommand>) {
        self.entries.insert(
            key.to_string(),
            CachedEntry {
                layout_box,
                commands,
            },
        );
    }

    /// Returns or updates the `cached_measure` value.
    pub fn cached_measure(&self, key: &str) -> Option<MeasureResult> {
        self.measured.get(key).copied()
    }

    /// Returns or updates the `store_measure` value.
    pub fn store_measure(&mut self, key: &str, size: MeasureResult) {
        self.measured.insert(key.to_string(), size);
    }

    /// Discards measurements produced under a different external layout
    /// environment. Paint entries remain eligible for their own geometry and
    /// dirty-state checks.
    pub(crate) fn sync_measurement_environment(&mut self, environment: MeasurementEnvironment) {
        if self.measurement_environment != Some(environment) {
            self.measured.clear();
            self.measurement_environment = Some(environment);
        }
    }

    /// Starts a new cache-liveness generation.
    pub fn begin_frame(&mut self) {
        self.generation = self.generation.wrapping_add(1);
        if self.generation == 0 {
            self.live_generation.clear();
            self.generation = 1;
        }
    }

    /// Marks a widget path as live without allocating again once the path
    /// has reached the cache.
    pub fn mark_live(&mut self, key: &str) {
        if let Some(generation) = self.live_generation.get_mut(key) {
            *generation = self.generation;
        } else {
            self.live_generation.insert(key.to_owned(), self.generation);
        }
    }

    /// Discards entries not observed during the current frame.
    pub fn finish_frame(&mut self) {
        let generation = self.generation;
        self.entries
            .retain(|key, _| self.live_generation.get(key).copied() == Some(generation));
        self.measured
            .retain(|key, _| self.live_generation.get(key).copied() == Some(generation));
        self.live_generation
            .retain(|_, live_generation| *live_generation == generation);
    }
}

#[cfg(test)]
mod tests {
    use super::{MeasurementEnvironment, RenderCache};
    use crate::{DrawCommand, LayoutBox, MeasureResult, RectCommand};

    #[test]
    fn measurement_environment_change_expires_only_measurements() {
        let mut cache = RenderCache::new();
        let initial = MeasurementEnvironment::new(1, 2, 1.0);
        cache.sync_measurement_environment(initial);
        cache.store_measure("text", MeasureResult::new(10.0, 20.0));

        cache.sync_measurement_environment(initial);
        assert!(cache.cached_measure("text").is_some());

        cache.sync_measurement_environment(MeasurementEnvironment::new(1, 3, 1.0));
        assert!(cache.cached_measure("text").is_none());
    }

    #[test]
    fn liveness_generations_reuse_path_storage_and_expire_old_paths() {
        let mut cache = RenderCache::new();
        cache.begin_frame();
        cache.mark_live("root.panel.button");
        cache.finish_frame();
        let capacity = cache.live_generation.capacity();

        cache.begin_frame();
        cache.mark_live("root.panel.button");
        cache.finish_frame();
        assert_eq!(cache.live_generation.len(), 1);
        assert_eq!(cache.live_generation.capacity(), capacity);

        cache.begin_frame();
        cache.finish_frame();
        assert!(cache.live_generation.is_empty());
        assert_eq!(cache.live_generation.capacity(), capacity);
    }

    #[test]
    fn moved_reuse_requires_an_unchanged_size_and_clean_widget() {
        let mut cache = RenderCache::new();
        let original = LayoutBox {
            x: 10.0,
            y: 20.0,
            width: 100.0,
            height: 40.0,
        };
        cache.store(
            "text",
            original,
            vec![DrawCommand::Rect(RectCommand {
                position: (10.0, 20.0),
                size: (100.0, 40.0),
                background: None,
                border_radius: None,
                border_width: None,
                border_color: None,
                clip_rect: None,
            })],
        );

        let moved = LayoutBox {
            x: 10.25,
            y: 19.5,
            ..original
        };
        let (_, offset) = cache
            .try_reuse_moved("text", moved, false)
            .expect("pure translation should reuse cached commands");
        assert_eq!(offset, (0.25, -0.5));

        assert!(
            cache
                .try_reuse_moved(
                    "text",
                    LayoutBox {
                        width: 101.0,
                        ..moved
                    },
                    false,
                )
                .is_none()
        );
        assert!(cache.try_reuse_moved("text", moved, true).is_none());
    }
}
