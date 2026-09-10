use crate::{Color, DEFAULT_SCROLLBAR_THUMB_THICKNESS, current_theme};

#[derive(Clone, Copy, Debug, PartialEq, Default)]
/// Data and behavior represented by `ScrollbarStyle`.
pub struct ScrollbarStyle {
    /// The `thickness` value carried by this type.
    pub thickness: Option<f32>,
    /// The `thumb_color` value carried by this type.
    pub thumb_color: Option<Color>,
    /// The `track_color` value carried by this type.
    pub track_color: Option<Color>,
    /// The `button_color` value carried by this type.
    pub button_color: Option<Color>,
    /// The `arrow_color` value carried by this type.
    pub arrow_color: Option<Color>,
    /// The `min_thumb_length` value carried by this type.
    pub min_thumb_length: Option<f32>,
    /// The `thumb_radius` value carried by this type.
    pub thumb_radius: Option<f32>,
    /// The `thumb_border_width` value carried by this type.
    pub thumb_border_width: Option<f32>,
    /// The `thumb_border_color` value carried by this type.
    pub thumb_border_color: Option<Color>,
    /// The `track_border_width` value carried by this type.
    pub track_border_width: Option<f32>,
    /// The `track_border_color` value carried by this type.
    pub track_border_color: Option<Color>,
    /// The `show_arrows` value carried by this type.
    pub show_arrows: Option<bool>,
}

impl ScrollbarStyle {
    /// Returns or updates the `overlay` value.
    pub fn overlay(&self, patch: &Self) -> Self {
        Self {
            thickness: patch.thickness.or(self.thickness),
            thumb_color: patch.thumb_color.or(self.thumb_color),
            track_color: patch.track_color.or(self.track_color),
            button_color: patch.button_color.or(self.button_color),
            arrow_color: patch.arrow_color.or(self.arrow_color),
            min_thumb_length: patch.min_thumb_length.or(self.min_thumb_length),
            thumb_radius: patch.thumb_radius.or(self.thumb_radius),
            thumb_border_width: patch.thumb_border_width.or(self.thumb_border_width),
            thumb_border_color: patch.thumb_border_color.or(self.thumb_border_color),
            track_border_width: patch.track_border_width.or(self.track_border_width),
            track_border_color: patch.track_border_color.or(self.track_border_color),
            show_arrows: patch.show_arrows.or(self.show_arrows),
        }
    }

    /// Returns or updates the `resolve` value.
    pub fn resolve(&self) -> ResolvedScrollbar {
        let theme = current_theme();
        let thickness = self.thickness.unwrap_or(DEFAULT_SCROLLBAR_THUMB_THICKNESS);
        let thumb_color = self.thumb_color.unwrap_or(theme.scrollbar_thumb);
        ResolvedScrollbar {
            thickness,
            thumb_color,
            track_color: self.track_color.unwrap_or(theme.scrollbar_track),
            button_color: self.button_color.unwrap_or(theme.scrollbar_button),
            arrow_color: self.arrow_color.unwrap_or(theme.scrollbar_arrow),
            min_thumb_length: self.min_thumb_length.unwrap_or(thickness * 1.5),
            thumb_radius: self.thumb_radius.unwrap_or(thickness * 2.0),
            thumb_border_width: self.thumb_border_width.unwrap_or(0.0),
            thumb_border_color: self
                .thumb_border_color
                .unwrap_or(theme.scrollbar_thumb_border),
            track_border_width: self.track_border_width.unwrap_or(0.0),
            track_border_color: self
                .track_border_color
                .unwrap_or(theme.scrollbar_track_border),
            // Modern overlay scrollbars omit step buttons on every platform.
            // `scrollbar_show_arrows(true)` remains an explicit opt-in.
            show_arrows: self.show_arrows.unwrap_or(false),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
/// Data and behavior represented by `ResolvedScrollbar`.
pub struct ResolvedScrollbar {
    /// The `thickness` value carried by this type.
    pub thickness: f32,
    /// The `thumb_color` value carried by this type.
    pub thumb_color: Color,
    /// The `track_color` value carried by this type.
    pub track_color: Color,
    /// The `button_color` value carried by this type.
    pub button_color: Color,
    /// The `arrow_color` value carried by this type.
    pub arrow_color: Color,
    /// The `min_thumb_length` value carried by this type.
    pub min_thumb_length: f32,
    /// The `thumb_radius` value carried by this type.
    pub thumb_radius: f32,
    /// The `thumb_border_width` value carried by this type.
    pub thumb_border_width: f32,
    /// The `thumb_border_color` value carried by this type.
    pub thumb_border_color: Color,
    /// The `track_border_width` value carried by this type.
    pub track_border_width: f32,
    /// The `track_border_color` value carried by this type.
    pub track_border_color: Color,
    /// The `show_arrows` value carried by this type.
    pub show_arrows: bool,
}

impl ResolvedScrollbar {
    /// Returns or updates the `patched` value.
    pub fn patched(&self, patch: &ScrollbarStyle, default_thickness: f32) -> Self {
        Self {
            thickness: patch.thickness.unwrap_or(default_thickness),
            thumb_color: patch.thumb_color.unwrap_or(self.thumb_color),
            track_color: patch.track_color.unwrap_or(self.track_color),
            button_color: patch.button_color.unwrap_or(self.button_color),
            arrow_color: patch.arrow_color.unwrap_or(self.arrow_color),
            min_thumb_length: patch.min_thumb_length.unwrap_or(self.min_thumb_length),
            thumb_radius: patch.thumb_radius.unwrap_or(self.thumb_radius),
            thumb_border_width: patch.thumb_border_width.unwrap_or(self.thumb_border_width),
            thumb_border_color: patch.thumb_border_color.unwrap_or(self.thumb_border_color),
            track_border_width: patch.track_border_width.unwrap_or(self.track_border_width),
            track_border_color: patch.track_border_color.unwrap_or(self.track_border_color),
            show_arrows: patch.show_arrows.unwrap_or(self.show_arrows),
        }
    }
}

impl Default for ResolvedScrollbar {
    fn default() -> Self {
        ScrollbarStyle::default().resolve()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn modern_defaults_hide_arrows_and_use_a_transparent_track() {
        let resolved = ScrollbarStyle::default().resolve();
        assert!(!resolved.show_arrows);
        assert_eq!(resolved.track_color, Color::TRANSPARENT);
    }

    #[test]
    fn arrows_remain_explicitly_customizable() {
        let resolved = ScrollbarStyle {
            show_arrows: Some(true),
            ..Default::default()
        }
        .resolve();
        assert!(resolved.show_arrows);
    }
}
