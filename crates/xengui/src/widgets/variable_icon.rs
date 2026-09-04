// SPDX-License-Identifier: Apache-2.0
use crate::{
    AnimationManager, Color, Constraints, EventCtx, EventStatus, InputEvent, Interaction,
    LayoutBox, MeasureContext, MeasureResult, PaintContext, Style, StyleBuilder,
    VariableIconCommand, Widget, WidgetBase, WidgetId,
};
use xengui_icons::material_symbols::{IconAxes, MaterialSymbolsVariable};

/// A single Material Symbols glyph rendered straight from the variable
/// font, with `weight`/`grade`/`optical_size`/`fill` blended continuously
/// through the wgpu backend's own rasterizer instead of picking between
/// a handful of pre-baked SVG variants (see `crate::Svg`/`IconSlot`).
pub struct VariableIcon {
    base: WidgetBase,
    anim_id: WidgetId,
    layout_box: LayoutBox,
    font: &'static [u8],
    codepoint: char,
    axes: IconAxes,
    size: f32,
    color: Option<Color>,
}

impl VariableIcon {
    /// Creates a value with its default configuration.
    pub fn new(codepoint: char) -> Self {
        Self {
            base: WidgetBase::new(Interaction::new()),
            anim_id: WidgetId::new_unique(),
            layout_box: LayoutBox::default(),
            font: MaterialSymbolsVariable::FONT,
            codepoint,
            axes: IconAxes::default(),
            size: 24.0,
            color: None,
        }
    }

    /// Overrides the underlying variable font. `codepoint` is looked up
    /// through this font's own `cmap`, so it only makes sense together
    /// with a matching codepoint.
    pub fn font_bytes(mut self, font: &'static [u8]) -> Self {
        self.font = font;
        self.mark_dirty();
        self
    }

    /// Returns or updates the `axes` value.
    pub fn axes(mut self, axes: IconAxes) -> Self {
        self.axes = axes;
        self.mark_dirty();
        self
    }

    /// Returns or updates the `size` value.
    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self.mark_dirty();
        self
    }

    /// Returns or updates the `color` value.
    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self.mark_dirty();
        self
    }

    fn recompute_style(&mut self) {
        self.base.recompute_style();
    }
}

impl Default for VariableIcon {
    fn default() -> Self {
        Self::new('\u{e000}')
    }
}

impl StyleBuilder for VariableIcon {
    fn style_mut(&mut self) -> &mut Style {
        &mut self.base.style
    }

    fn mark_dirty(&mut self) {
        self.base.dirty = true;
        self.recompute_style();
    }
}

crate::impl_common_style_builders!(base VariableIcon);

impl Widget for VariableIcon {
    crate::impl_widget_boilerplate!();

    fn debug_name(&self) -> &'static str {
        "Widget#VariableIcon"
    }

    fn measure(&self, ctx: &mut MeasureContext, constraints: Constraints) -> MeasureResult {
        let px = self.size * ctx.scale_factor;
        let (w, h) = constraints.constrain_size(px, px);
        MeasureResult::new(w, h)
    }

    fn hit_test(&self, _point: (f32, f32)) -> bool {
        // Purely decorative - pointer events must always fall through to
        // whatever interactive ancestor (a button, an icon button) wraps
        // this icon, never stop here.
        false
    }

    fn paint(&self, ctx: &mut PaintContext) {
        self.paint_box(ctx);
        self.paint_outline(ctx);

        let color = self
            .color
            .unwrap_or(self.base.computed_style.color.unwrap_or(Color::BLACK));

        ctx.draw_variable_icon(VariableIconCommand {
            position: (self.layout_box.x, self.layout_box.y),
            size: (self.layout_box.width, self.layout_box.height),
            codepoint: self.codepoint,
            font: self.font,
            axes: self.axes,
            color,
            clip_rect: None,
        });
    }

    fn event(&mut self, event: &InputEvent, ctx: &mut EventCtx) -> EventStatus {
        if !self.base.interaction.is_active() {
            return EventStatus::Ignored;
        }
        let status = self.base.interaction.handle(event, ctx);
        if matches!(status, EventStatus::Handled) {
            self.base.dirty = true;
            ctx.request_redraw();
        }
        status
    }

    fn content_eq(&self, other: &dyn Widget) -> bool {
        let Some(other) = other.as_any().downcast_ref::<VariableIcon>() else {
            return false;
        };
        self.codepoint == other.codepoint
            && self.font.as_ptr() == other.font.as_ptr()
            && self.font.len() == other.font.len()
            && self.axes == other.axes
            && self.size == other.size
            && self.color == other.color
            && self.base.authored_styles_eq(&other.base)
    }

    fn cascade_style(&mut self, parent: &Style, anim: &mut AnimationManager) {
        self.base.inherited_style = parent.clone();
        self.recompute_style();
        if crate::animate_computed_style(self.anim_id, &mut self.base.computed_style, anim) {
            self.base.dirty = true;
        }
    }

    fn transfer_interaction_state(&mut self, old: &dyn Widget) {
        if let (Some(new), Some(old_i)) = (self.interaction_mut(), old.interaction()) {
            new.transfer_from(old_i);
        }
        if let Some(old) = old.as_any().downcast_ref::<VariableIcon>() {
            self.anim_id = old.anim_id;
        }
    }

    fn anim_id(&self) -> WidgetId {
        self.anim_id
    }
}
