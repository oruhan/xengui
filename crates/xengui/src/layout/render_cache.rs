// SPDX-License-Identifier: Apache-2.0
use crate::{Constraints, DrawCommand, LayoutBox, MeasureResult, WidgetPath};
use std::collections::HashMap;

struct CachedEntry {
    layout_box: LayoutBox,
    commands: Vec<DrawCommand>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct MeasurementConstraints {
    known_width: Option<u32>,
    known_height: Option<u32>,
    max_width: Option<u32>,
    max_height: Option<u32>,
}

impl From<Constraints> for MeasurementConstraints {
    fn from(constraints: Constraints) -> Self {
        fn bits(value: Option<f32>) -> Option<u32> {
            value.map(|value| {
                // Layout constraints are expected to be finite. Canonicalising
                // zero still prevents equivalent `0.0` and `-0.0` constraints
                // from occupying separate cache entries.
                if value == 0.0 {
                    0.0f32.to_bits()
                } else {
                    value.to_bits()
                }
            })
        }

        Self {
            known_width: bits(constraints.known_width),
            known_height: bits(constraints.known_height),
            max_width: bits(constraints.max_width),
            max_height: bits(constraints.max_height),
        }
    }
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
    entries: HashMap<WidgetPath, CachedEntry>,
    measured: HashMap<WidgetPath, HashMap<MeasurementConstraints, MeasureResult>>,
    measurement_environment: Option<MeasurementEnvironment>,
    live_generation: HashMap<WidgetPath, u64>,
    generation: u64,
}

impl RenderCache {
    /// Creates a value with its default configuration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns or updates the `cached_size` value.
    pub fn cached_size(&self, key: &WidgetPath) -> Option<(f32, f32)> {
        self.entries
            .get(key)
            .map(|e| (e.layout_box.width, e.layout_box.height))
    }

    /// Returns or updates the `try_reuse` value.
    pub fn try_reuse(
        &self,
        key: &WidgetPath,
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
        key: &WidgetPath,
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
    pub fn store(&mut self, key: &WidgetPath, layout_box: LayoutBox, commands: Vec<DrawCommand>) {
        self.entries.insert(
            key.clone(),
            CachedEntry {
                layout_box,
                commands,
            },
        );
    }

    /// Returns a measurement made under exactly the same layout constraints.
    pub(crate) fn cached_measure(
        &self,
        key: &WidgetPath,
        constraints: Constraints,
    ) -> Option<MeasureResult> {
        self.measured
            .get(key)
            .and_then(|measurements| measurements.get(&constraints.into()))
            .copied()
    }

    /// Stores a measurement for one exact set of layout constraints.
    pub(crate) fn store_measure(
        &mut self,
        key: &WidgetPath,
        constraints: Constraints,
        size: MeasureResult,
    ) {
        self.measured
            .entry(key.clone())
            .or_default()
            .insert(constraints.into(), size);
    }

    /// Invalidates every constraint variant cached for a widget.
    pub(crate) fn invalidate_measure(&mut self, key: &WidgetPath) {
        self.measured.remove(key);
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
    pub fn mark_live(&mut self, key: &WidgetPath) {
        if let Some(generation) = self.live_generation.get_mut(key) {
            *generation = self.generation;
        } else {
            self.live_generation.insert(key.clone(), self.generation);
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
    use crate::{DrawCommand, LayoutBox, MeasureResult, RectCommand, View, WidgetPath};

    fn path(index: usize) -> WidgetPath {
        WidgetPath::from_widget(&View::new(), index)
    }

    #[test]
    fn measurement_environment_change_expires_only_measurements() {
        let mut cache = RenderCache::new();
        let initial = MeasurementEnvironment::new(1, 2, 1.0);
        cache.sync_measurement_environment(initial);
        let text = path(0);
        cache.store_measure(
            &text,
            crate::Constraints::UNBOUNDED,
            MeasureResult::new(10.0, 20.0),
        );

        cache.sync_measurement_environment(initial);
        assert!(
            cache
                .cached_measure(&text, crate::Constraints::UNBOUNDED)
                .is_some()
        );

        cache.sync_measurement_environment(MeasurementEnvironment::new(1, 3, 1.0));
        assert!(
            cache
                .cached_measure(&text, crate::Constraints::UNBOUNDED)
                .is_none()
        );
    }

    #[test]
    fn structurally_different_paths_cannot_alias_in_the_cache() {
        let flat_widget = View::new().key("a.kb");
        let parent = View::new().key("a");
        let child = View::new().key("b");
        let flat = WidgetPath::from_widget(&flat_widget, 0);
        let mut nested = WidgetPath::from_widget(&parent, 0);
        nested.push(&child, 0);
        assert_ne!(flat, nested);

        let mut cache = RenderCache::new();
        cache.store_measure(
            &flat,
            crate::Constraints::UNBOUNDED,
            MeasureResult::new(10.0, 10.0),
        );
        cache.store_measure(
            &nested,
            crate::Constraints::UNBOUNDED,
            MeasureResult::new(20.0, 20.0),
        );

        assert_eq!(
            cache
                .cached_measure(&flat, crate::Constraints::UNBOUNDED)
                .unwrap()
                .width,
            10.0
        );
        assert_eq!(
            cache
                .cached_measure(&nested, crate::Constraints::UNBOUNDED)
                .unwrap()
                .width,
            20.0
        );
    }

    #[test]
    fn measurements_are_partitioned_by_constraints() {
        let mut cache = RenderCache::new();
        let widget = path(0);
        let narrow = crate::Constraints::UNBOUNDED.with_max_width(100.0);
        let wide = crate::Constraints::UNBOUNDED.with_max_width(200.0);

        cache.store_measure(&widget, narrow, MeasureResult::new(100.0, 20.0));
        cache.store_measure(&widget, wide, MeasureResult::new(200.0, 10.0));

        assert_eq!(cache.cached_measure(&widget, narrow).unwrap().height, 20.0);
        assert_eq!(cache.cached_measure(&widget, wide).unwrap().height, 10.0);
        assert!(
            cache
                .cached_measure(&widget, crate::Constraints::UNBOUNDED)
                .is_none()
        );
    }

    #[test]
    fn liveness_generations_reuse_path_storage_and_expire_old_paths() {
        let mut cache = RenderCache::new();
        cache.begin_frame();
        let widget = path(0);
        cache.mark_live(&widget);
        cache.finish_frame();
        let capacity = cache.live_generation.capacity();

        cache.begin_frame();
        cache.mark_live(&widget);
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
        let text = path(0);
        cache.store(
            &text,
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
            .try_reuse_moved(&text, moved, false)
            .expect("pure translation should reuse cached commands");
        assert_eq!(offset, (0.25, -0.5));

        assert!(
            cache
                .try_reuse_moved(
                    &text,
                    LayoutBox {
                        width: 101.0,
                        ..moved
                    },
                    false,
                )
                .is_none()
        );
        assert!(cache.try_reuse_moved(&text, moved, true).is_none());
    }
}
