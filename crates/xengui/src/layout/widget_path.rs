// SPDX-License-Identifier: Apache-2.0
use crate::Widget;
use smol_str::SmolStr;
use std::fmt;

pub(crate) type PathCheckpoint = usize;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) enum WidgetPathSegment {
    Key(SmolStr),
    Index(usize),
}

/// Collision-free identity of a widget's position in a rendered tree.
///
/// Keys and positional indices are stored as typed segments. Formatting is
/// intentionally presentation-only; runtime lookup never reparses it.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct WidgetPath {
    pub(crate) segments: Vec<WidgetPathSegment>,
}

impl WidgetPath {
    /// Creates an empty root path.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns whether this path has no widget segments.
    pub fn is_empty(&self) -> bool {
        self.segments.is_empty()
    }

    pub(crate) fn from_widget(widget: &dyn Widget, index: usize) -> Self {
        let mut path = Self::new();
        path.push(widget, index);
        path
    }

    pub(crate) fn ancestors(&self) -> Vec<Self> {
        (1..=self.segments.len())
            .map(|len| Self {
                segments: self.segments[..len].to_vec(),
            })
            .collect()
    }

    pub(crate) fn is_within(&self, ancestor: &Self) -> bool {
        self.segments.starts_with(&ancestor.segments)
    }

    #[inline]
    pub(crate) fn checkpoint(&self) -> PathCheckpoint {
        self.segments.len()
    }

    #[inline]
    pub(crate) fn restore(&mut self, checkpoint: PathCheckpoint) {
        self.segments.truncate(checkpoint);
    }

    pub(crate) fn push(&mut self, widget: &dyn Widget, index: usize) {
        self.segments.push(match widget.get_key() {
            Some(key) => WidgetPathSegment::Key(key.clone()),
            None => WidgetPathSegment::Index(index),
        });
    }
}

impl fmt::Display for WidgetPath {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (index, segment) in self.segments.iter().enumerate() {
            if index != 0 {
                formatter.write_str("/")?;
            }
            match segment {
                WidgetPathSegment::Key(key) => write!(formatter, "key({key:?})")?,
                WidgetPathSegment::Index(index) => write!(formatter, "index({index})")?,
            }
        }
        Ok(())
    }
}
