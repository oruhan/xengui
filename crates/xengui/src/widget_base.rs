// SPDX-License-Identifier: Apache-2.0
use crate::{Interaction, Style};
use smol_str::SmolStr;

/// Data and behavior represented by `WidgetBase`.
pub struct WidgetBase {
    /// The `key` value carried by this type.
    pub key: Option<SmolStr>,
    /// Global identifier, usable with `xengui::dom` to trigger this widget from anywhere
    pub id: Option<SmolStr>,
    /// The `dirty` value carried by this type.
    pub dirty: bool,
    /// Set when a style recompute actually changes something affecting
    /// this widget's own taffy layout node, tracked separately from
    /// `dirty` so a purely visual hover/press style swap never forces a
    /// full tree relayout.
    pub layout_dirty: bool,

    /// The `style` value carried by this type.
    pub style: Style,
    /// The `inherited_style` value carried by this type.
    pub inherited_style: Style,
    /// The `computed_style` value carried by this type.
    pub computed_style: Style,
    /// The `hover_style` value carried by this type.
    pub hover_style: Option<Style>,
    /// The `pressed_style` value carried by this type.
    pub pressed_style: Option<Style>,
    /// The `disabled_style` value carried by this type.
    pub disabled_style: Option<Style>,
    /// The `focus_style` value carried by this type.
    pub focus_style: Option<Style>,
    /// The `focus_within_style` value carried by this type.
    pub focus_within_style: Option<Style>,
    /// The `focused_hover_style` value carried by this type.
    pub focused_hover_style: Option<Style>,
    /// The `focused_pressed_style` value carried by this type.
    pub focused_pressed_style: Option<Style>,

    /// The `interaction` value carried by this type.
    pub interaction: Interaction,
}

impl WidgetBase {
    /// Creates a value with its default configuration.
    pub fn new(interaction: Interaction) -> Self {
        Self {
            key: None,
            id: None,
            dirty: true,
            layout_dirty: true,

            style: Style::default(),
            inherited_style: Style::default(),
            computed_style: Style::default(),
            hover_style: None,
            pressed_style: None,
            disabled_style: None,
            focus_style: None,
            focus_within_style: None,
            focused_hover_style: None,
            focused_pressed_style: None,

            interaction,
        }
    }

    // Layers each active interaction-state patch on top of the base style,
    // from least to most specific (hover -> pressed -> focus -> combined),
    // so an unset field at any level simply falls through to the previous
    // layer instead of the whole state reverting to the base style.
    /// Returns or updates the `recompute_style` value.
    pub fn recompute_style(&mut self) {
        let base = self.inherited_style.inherit_style(&self.style);

        if !self.interaction.enabled {
            let computed = match &self.disabled_style {
                Some(patch) => base.overlay(patch),
                None => base,
            };
            self.commit_computed_style(computed);
            return;
        }

        let hovered = self.interaction.hovered;
        let pressed = self.interaction.pressed;
        let focused = self.interaction.focused;

        let mut computed = base;

        // Priority (lowest -> highest): hover, focus, pressed, then the
        // most-specific combined patch. Pressed sits above focus so a
        // held-down control never loses its press feedback to a focus
        // ring style; the combined patches still win over either single
        // state since they're the most specific match.
        if hovered && let Some(patch) = &self.hover_style {
            computed = computed.overlay(patch);
        }

        if self.interaction.focus_within
            && let Some(patch) = &self.focus_within_style
        {
            computed = computed.overlay(patch);
        }

        if focused && let Some(patch) = &self.focus_style {
            computed = computed.overlay(patch);
        }

        if pressed && let Some(patch) = &self.pressed_style {
            computed = computed.overlay(patch);
        }

        if focused && pressed {
            if let Some(patch) = &self.focused_pressed_style {
                computed = computed.overlay(patch);
            }
        } else if focused
            && hovered
            && let Some(patch) = &self.focused_hover_style
        {
            computed = computed.overlay(patch);
        }

        self.commit_computed_style(computed);
    }

    fn commit_computed_style(&mut self, computed: Style) {
        if self.computed_style.layout_affecting_diff(&computed) {
            self.layout_dirty = true;
        }
        self.computed_style = computed;
    }

    /// Returns or updates the `mark_dirty` value.
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
        self.recompute_style();
    }

    /// Compares every author-provided style layer that can affect the
    /// reconciled widget's paint or layout output. Runtime-only inherited
    /// and computed styles are deliberately excluded.
    pub(crate) fn authored_styles_eq(&self, other: &Self) -> bool {
        self.style == other.style
            && self.hover_style == other.hover_style
            && self.pressed_style == other.pressed_style
            && self.disabled_style == other.disabled_style
            && self.focus_style == other.focus_style
            && self.focus_within_style == other.focus_within_style
            && self.focused_hover_style == other.focused_hover_style
            && self.focused_pressed_style == other.focused_pressed_style
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn authored_style_comparison_includes_combined_focus_states() {
        let first = WidgetBase::new(Interaction::new());
        let mut second = WidgetBase::new(Interaction::new());
        assert!(first.authored_styles_eq(&second));

        let focus_within = Style {
            background: Some(crate::Background::Color(crate::Color::RED_500)),
            ..Default::default()
        };
        second.focus_within_style = Some(focus_within);
        assert!(!first.authored_styles_eq(&second));

        second.focus_within_style = None;
        let focused_pressed = Style {
            background: Some(crate::Background::Color(crate::Color::BLUE_500)),
            ..Default::default()
        };
        second.focused_pressed_style = Some(focused_pressed);
        assert!(!first.authored_styles_eq(&second));
    }
}
