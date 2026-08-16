// SPDX-License-Identifier: Apache-2.0
use crate::{ Interaction, Style };
use smol_str::SmolStr;

pub struct WidgetBase {
    pub key: Option<SmolStr>,
    /// Global identifier, usable with `xengui::dom` to trigger this widget from anywhere
    pub id: Option<SmolStr>,
    pub dirty: bool,
    /// Set when a style recompute actually changes something affecting
    /// this widget's own taffy layout node, tracked separately from
    /// `dirty` so a purely visual hover/press style swap never forces a
    /// full tree relayout.
    pub layout_dirty: bool,

    pub style: Style,
    pub inherited_style: Style,
    pub computed_style: Style,
    pub hover_style: Option<Style>,
    pub pressed_style: Option<Style>,
    pub disabled_style: Option<Style>,
    pub focus_style: Option<Style>,
    pub focused_hover_style: Option<Style>,
    pub focused_pressed_style: Option<Style>,

    pub interaction: Interaction,
}

impl WidgetBase {
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
            focused_hover_style: None,
            focused_pressed_style: None,

            interaction,
        }
    }

    // Layers each active interaction-state patch on top of the base style,
    // from least to most specific (hover -> pressed -> focus -> combined),
    // so an unset field at any level simply falls through to the previous
    // layer instead of the whole state reverting to the base style.
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
        } else if focused && hovered && let Some(patch) = &self.focused_hover_style {
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

    pub fn mark_dirty(&mut self) {
        self.dirty = true;
        self.recompute_style();
    }
}
