// SPDX-License-Identifier: Apache-2.0
use crate::{DrawCommand, LayoutBox, MeasureResult};
use std::collections::HashMap;

struct CachedEntry {
    layout_box: LayoutBox,
    commands: Vec<DrawCommand>,
}

#[derive(Default)]
/// Data and behavior represented by `RenderCache`.
pub struct RenderCache {
    entries: HashMap<String, CachedEntry>,
    measured: HashMap<String, MeasureResult>,
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
    use super::RenderCache;

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
}
