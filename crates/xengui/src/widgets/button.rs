// SPDX-License-Identifier: Apache-2.0
use crate::{
    Align, AnimationManager, Color, Constraints, Easing, EventCtx, EventStatus, InputEvent,
    Interaction, JustifyContent, LayoutBox, Length, MeasureContext, MeasureResult, PaintContext,
    RectCommand, Style, StyleBuilder, TextCommand, Transition, TransitionProperty, TriangleCommand,
    VariableIconCommand, Widget, WidgetBase, WidgetContent, WidgetId,
    constants::{DEFAULT_CURSOR_ICON, DEFAULT_FONT_SIZE, DEFAULT_POINTER_CURSOR_ICON},
};
use smol_str::SmolStr;
use std::cell::Cell;
use std::sync::Arc;
use web_time::Duration;
use xen_svg::{SvgDocument, SvgTriangle, parse_svg, tessellate_document};
use xengui_icons::{IconAxes, MaterialSymbolsVariable};

/// Where the icon sits relative to the label along the content's main axis.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum IconPosition {
    #[default]
    /// The `Start` variant.
    Start,
    /// The `End` variant.
    End,
}

/// A clickable widget that performs an action when activated.
///
/// A `Button` displays a text label and responds to user interactions such as
/// pointer clicks and keyboard activation. Its appearance can be customized
/// through the styling API, including typography, colors, borders, spacing,
/// and sizing.
///
/// Buttons can be enabled or disabled. When disabled, they remain visible but
/// do not respond to user input.
///
/// ## Example
///
/// ```no_run
/// use xengui::Button;
///
/// let button = Button::new()
///     .label("Click me");
/// ```
pub struct Button {
    base: WidgetBase,
    content: SmolStr,
    layout_box: LayoutBox,
    content_size: Cell<(f32, f32)>,
    anim_id: WidgetId,
    icon_document: Option<Arc<SvgDocument>>,
    icon_triangles: Arc<Vec<SvgTriangle>>,
    icon_render_size: Cell<(f32, f32)>,
    icon_size: Option<(f32, f32)>,
    icon_gap: f32,
    icon_position: IconPosition,
    icon_tint: Option<Color>,
    material_icon: Option<char>,
    material_icon_axes: IconAxes,
}

impl Button {
    /// Creates a value with its default configuration.
    pub fn new() -> Self {
        let mut interaction = Interaction::new();
        interaction.focusable = true;
        interaction.hover_cursor = Some(DEFAULT_POINTER_CURSOR_ICON);

        let mut base = WidgetBase::new(interaction);
        // Default alignment for the label/icon content inside the button box;
        // overridable via the normal StyleBuilder::justify_content/align_items.
        base.style.justify_content = Some(JustifyContent::Center);
        base.style.align_items = Some(Align::Center);

        Self {
            base,
            content: SmolStr::new(""),
            layout_box: LayoutBox::default(),
            content_size: Cell::new((0.0, 0.0)),
            anim_id: WidgetId::new_unique(),
            icon_document: None,
            icon_triangles: Arc::new(Vec::new()),
            icon_render_size: Cell::new((0.0, 0.0)),
            icon_size: None,
            icon_gap: 8.0,
            icon_position: IconPosition::Start,
            icon_tint: None,
            material_icon: None,
            material_icon_axes: IconAxes::default(),
        }
    }

    /// Sets the text displayed by this widget.
    pub fn label(mut self, content: impl Into<SmolStr>) -> Self {
        self.content = content.into();
        self.mark_dirty();
        self
    }

    /// Sets the icon's SVG source.
    pub fn icon(mut self, svg_source: &str) -> Result<Self, crate::AssetError> {
        let document = parse_svg(svg_source).map_err(|message| crate::AssetError::Parse {
            kind: "SVG icon",
            message,
        })?;
        self.icon_triangles = Arc::new(tessellate_document(&document));
        self.icon_document = Some(Arc::new(document));
        self.mark_dirty();
        Ok(self)
    }

    /// Uses an icon from the project's Material Symbols font.
    pub fn material_icon(mut self, codepoint: char) -> Self {
        self.material_icon = Some(codepoint);
        self.icon_document = None;
        self.icon_triangles = Arc::new(Vec::new());
        self.mark_dirty();
        self
    }

    /// Overrides the icon's rendered size; otherwise the SVG's own viewBox size is used.
    pub fn icon_size(mut self, width: f32, height: f32) -> Self {
        self.icon_size = Some((width, height));
        self.mark_dirty();
        self
    }

    /// Returns or updates the `icon_gap` value.
    pub fn icon_gap(mut self, gap: f32) -> Self {
        self.icon_gap = gap;
        self.mark_dirty();
        self
    }

    /// Returns or updates the `icon_position` value.
    pub fn icon_position(mut self, position: IconPosition) -> Self {
        self.icon_position = position;
        self.mark_dirty();
        self
    }

    /// Overrides the icon's currentColor fallback (defaults to the label's text color).
    pub fn icon_color(mut self, color: Color) -> Self {
        self.icon_tint = Some(color);
        self.mark_dirty();
        self
    }

    fn icon_natural_size(&self) -> (f32, f32) {
        if let Some(size) = self.icon_size {
            return size;
        }
        if self.material_icon.is_some() {
            return self.icon_size.unwrap_or((18.0, 18.0));
        }
        match &self.icon_document {
            Some(doc) => {
                let (_, _, w, h) = doc.view_box;
                (w, h)
            }
            None => (0.0, 0.0),
        }
    }

    // Widget-specific extra step (hover cursor) stays local; the shared
    // style-overlay logic lives in WidgetBase::recompute_style.
    fn recompute_style(&mut self) {
        self.base.recompute_style();

        // M3 state layers are part of the component default, so a button
        // still communicates hover/press when the caller only supplies its
        // resting container color. Explicit state styles continue to win.
        let theme = crate::current_theme();
        let pressed_has_background = self
            .base
            .pressed_style
            .as_ref()
            .is_some_and(|style| style.background.is_some());
        let hover_has_background = self
            .base
            .hover_style
            .as_ref()
            .is_some_and(|style| style.background.is_some());
        let state_amount = if self.base.interaction.pressed && !pressed_has_background {
            Some(0.10)
        } else if self.base.interaction.hovered && !hover_has_background {
            Some(0.08)
        } else {
            None
        };
        if let Some(amount) = state_amount
            && let Some(background) = self.base.computed_style.background.as_ref()
        {
            let base = background.representative_color();
            let layer = theme.on_surface;
            self.base.computed_style.background = Some(crate::Background::Color(Color::rgba_f32(
                base.r() * (1.0 - amount) + layer.r() * amount,
                base.g() * (1.0 - amount) + layer.g() * amount,
                base.b() * (1.0 - amount) + layer.b() * amount,
                base.a(),
            )));
        }
        let pressed_has_scale = self
            .base
            .pressed_style
            .as_ref()
            .is_some_and(|style| style.scale.is_some());
        if !pressed_has_scale {
            let resting_scale = self.base.computed_style.scale.unwrap_or(1.0);
            self.base.computed_style.scale = Some(if self.base.interaction.pressed {
                resting_scale * 0.97
            } else {
                resting_scale
            });
            // Keep the label/icon at its natural size while the container
            // supplies the pressed geometry feedback. This also prevents a
            // transient glyph-atlas layer from disappearing during rebuilds.
            self.base.computed_style.content_scale = Some(1.0);
            self.base
                .computed_style
                .transition_overrides
                .transform
                .get_or_insert(
                    Transition::new(Duration::from_millis(350))
                        .easing(Easing::cubic_bezier(0.42, 1.67, 0.21, 0.90)),
                );
            let properties = self
                .base
                .computed_style
                .transition_properties
                .unwrap_or(TransitionProperty::NONE);
            self.base.computed_style.transition_properties =
                Some(properties.union(TransitionProperty::TRANSFORM));
        }
        self.base.interaction.hover_cursor =
            self.base
                .computed_style
                .cursor
                .or(Some(if self.base.interaction.enabled {
                    DEFAULT_POINTER_CURSOR_ICON
                } else {
                    DEFAULT_CURSOR_ICON
                }));
    }
}

impl Default for Button {
    fn default() -> Self {
        Self::new()
    }
}

impl StyleBuilder for Button {
    fn style_mut(&mut self) -> &mut Style {
        &mut self.base.style
    }

    fn mark_dirty(&mut self) {
        self.base.dirty = true;
        self.recompute_style();
    }
}

impl WidgetContent for Button {
    fn with_content(self, content: impl Into<SmolStr>) -> Self {
        self.label(content)
    }
}

crate::impl_interaction_builders!(base Button);
crate::impl_common_style_builders!(base Button);
crate::impl_themed_style_builders!(base Button; hover_style => hover_style, pressed_style => pressed_style, disabled_style => disabled_style, focus_style => focus_style, focused_hover_style => focused_hover_style, focused_pressed_style => focused_pressed_style);

impl Widget for Button {
    fn semantics(&self) -> Option<crate::Semantics> {
        let label = self
            .base
            .accessible_label
            .as_ref()
            .unwrap_or(&self.content)
            .to_string();
        let mut semantics = crate::Semantics::new(crate::SemanticRole::Button)
            .label(label)
            .action(crate::SemanticAction::Focus);
        semantics.disabled = !self.base.interaction.enabled;
        if !semantics.disabled {
            semantics = semantics.action(crate::SemanticAction::Activate);
        }
        Some(semantics)
    }

    crate::impl_widget_boilerplate!();

    fn debug_name(&self) -> &'static str {
        "Widget#Button"
    }

    fn measure(&self, ctx: &mut MeasureContext, constraints: Constraints) -> MeasureResult {
        let scale_factor = ctx.scale_factor;
        let style = &self.base.computed_style;

        // Logical metrics; TextMeasurer converts to physical internally.
        let font_size = style.font_size.unwrap_or(DEFAULT_FONT_SIZE).value();
        let letter_spacing = style
            .letter_spacing
            .map(|ls| ls.value().value())
            .unwrap_or(0.0);
        let line_height = style
            .line_height
            .map(|lh| lh.value().value())
            .unwrap_or(0.0);

        let result = ctx.text.measure(
            &self.content,
            style.font.as_deref(),
            font_size,
            style.font_weight.unwrap_or_default(),
            style.font_style.unwrap_or_default(),
            letter_spacing,
            line_height,
            constraints.max_width,
            scale_factor,
        );

        self.content_size.set((result.width, result.height));

        let (icon_w, icon_h) = self.icon_natural_size();
        let (icon_w, icon_h) = (icon_w * scale_factor, icon_h * scale_factor);
        self.icon_render_size.set((icon_w, icon_h));

        let has_icon = icon_w > 0.0 && icon_h > 0.0;
        let gap = if has_icon && !self.content.is_empty() {
            self.icon_gap * scale_factor
        } else {
            0.0
        };
        let combined_w = if has_icon {
            icon_w + gap + result.width
        } else {
            result.width
        };
        let combined_h = result.height.max(icon_h);

        let padding = style.padding.unwrap_or_default();

        let width = combined_w
            + padding.left.to_physical(scale_factor)
            + padding.right.to_physical(scale_factor);
        let height = combined_h
            + padding.top.to_physical(scale_factor)
            + padding.bottom.to_physical(scale_factor);

        let (width, height) = constraints.constrain_size(width, height);

        MeasureResult::new(width, height)
    }

    // Runs even when the button has a fixed (non-auto) size, since
    // `measure` is skipped entirely in that case and paint()'s
    // content-centering math needs an up-to-date content_size/icon size.
    fn on_layout_pass(&self, ctx: &mut MeasureContext) {
        let scale_factor = ctx.scale_factor;
        let style = &self.base.computed_style;

        // Logical metrics; TextMeasurer converts to physical internally.
        let font_size = style.font_size.unwrap_or(DEFAULT_FONT_SIZE).value();
        let letter_spacing = style
            .letter_spacing
            .map(|ls| ls.value().value())
            .unwrap_or(0.0);
        let line_height = style
            .line_height
            .map(|lh| lh.value().value())
            .unwrap_or(0.0);

        let result = ctx.text.measure(
            &self.content,
            style.font.as_deref(),
            font_size,
            style.font_weight.unwrap_or_default(),
            style.font_style.unwrap_or_default(),
            letter_spacing,
            line_height,
            None,
            scale_factor,
        );

        self.content_size.set((result.width, result.height));

        let (icon_w, icon_h) = self.icon_natural_size();
        self.icon_render_size
            .set((icon_w * scale_factor, icon_h * scale_factor));
    }

    fn paint(&self, ctx: &mut PaintContext) {
        let style = &self.base.computed_style;
        let sf = ctx.scale_factor;

        let background_box = self.layout_box;
        let radius = style
            .border
            .as_ref()
            .and_then(|b| b.radius)
            .map(|r| r.to_physical_array(sf, background_box.width, background_box.height))
            .unwrap_or([0.0; 4]);

        if let Some(shadows) = &style.box_shadow {
            for shadow in shadows.iter().rev().filter(|s| !s.inset) {
                self.paint_shadow_layer(ctx, background_box, radius, shadow, sf);
            }
        }

        if style.background.is_some() || style.border.is_some() {
            let border = style.border.as_ref();
            ctx.draw_rect(RectCommand {
                position: (background_box.x, background_box.y),
                size: (background_box.width, background_box.height),
                background: style.background.clone(),
                border_radius: radius
                    .iter()
                    .any(|r| *r > 0.0)
                    .then_some(crate::widget::border_radius_from_physical(radius)),
                border_color: border.map(|b| b.color),
                border_width: border.map(|b| Length::px(b.top.to_physical(sf))),
                clip_rect: None,
            });
        }

        if let Some(shadows) = &style.box_shadow {
            for shadow in shadows.iter().rev().filter(|s| s.inset) {
                self.paint_shadow_layer(ctx, background_box, radius, shadow, sf);
            }
        }

        self.paint_outline(ctx);

        let (text_w, text_h) = self.content_size.get();
        let (icon_w, icon_h) = self.icon_render_size.get();
        let has_icon = icon_w > 0.0 && icon_h > 0.0;

        let padding = style.padding.unwrap_or_default();
        let (pad_l, pad_r, pad_t, pad_b) = (
            padding.left.to_physical(sf),
            padding.right.to_physical(sf),
            padding.top.to_physical(sf),
            padding.bottom.to_physical(sf),
        );
        let available_w = (self.layout_box.width - pad_l - pad_r).max(0.0);
        let available_h = (self.layout_box.height - pad_t - pad_b).max(0.0);

        let gap = if has_icon && !self.content.is_empty() {
            self.icon_gap * sf
        } else {
            0.0
        };
        let combined_w = if has_icon {
            icon_w + gap + text_w
        } else {
            text_w
        };
        let combined_h = text_h.max(icon_h);

        let justify = style.justify_content.unwrap_or(JustifyContent::Center);
        let align = style.align_items.unwrap_or(Align::Center);

        let content_x =
            self.layout_box.x + pad_l + justify_offset(justify, available_w, combined_w);
        let content_y = self.layout_box.y + pad_t + align_offset(align, available_h, combined_h);
        let draw_max_width = available_w.max(text_w);

        let (icon_x, text_x) = match (has_icon, self.icon_position) {
            (true, IconPosition::Start) => (content_x, content_x + icon_w + gap),
            (true, IconPosition::End) => (content_x + text_w + gap, content_x),
            _ => (content_x, content_x),
        };
        let icon_y = content_y + (combined_h - icon_h).max(0.0) * 0.5;
        let text_y = content_y + (combined_h - text_h).max(0.0) * 0.5;

        if has_icon && let Some(doc) = &self.icon_document {
            let (vb_x, vb_y, vb_w, vb_h) = doc.view_box;
            if !self.icon_triangles.is_empty() && vb_w > 0.0 && vb_h > 0.0 {
                let icon_scale = (icon_w / vb_w).min(icon_h / vb_h);
                let icon_offset_x = icon_x + (icon_w - vb_w * icon_scale) * 0.5;
                let icon_offset_y = icon_y + (icon_h - vb_h * icon_scale) * 0.5;

                let inherited_color = self
                    .icon_tint
                    .unwrap_or(style.color.unwrap_or(Color::BLACK));
                let inherited_svg_color = xen_svg::Color::rgba_f32(
                    inherited_color.r(),
                    inherited_color.g(),
                    inherited_color.b(),
                    inherited_color.a(),
                );

                for triangle in self.icon_triangles.iter() {
                    let Some(color) = triangle.paint.resolve(inherited_svg_color) else {
                        continue;
                    };
                    let color = crate::svg_compat::from_svg_color(color);
                    let color = color.with_alpha_f32(color.a() * triangle.opacity);

                    let map = |p: (f32, f32)| -> (f32, f32) {
                        (
                            icon_offset_x + (p.0 - vb_x) * icon_scale,
                            icon_offset_y + (p.1 - vb_y) * icon_scale,
                        )
                    };

                    ctx.draw_content_triangle(TriangleCommand {
                        p0: map(triangle.p0),
                        p1: map(triangle.p1),
                        p2: map(triangle.p2),
                        color,
                        clip_rect: None,
                    });
                }
            }
        }

        if has_icon && let Some(codepoint) = self.material_icon {
            ctx.draw_content_variable_icon(VariableIconCommand {
                position: (icon_x, icon_y),
                size: (icon_w, icon_h),
                codepoint,
                font: MaterialSymbolsVariable::FONT,
                axes: self.material_icon_axes,
                color: self
                    .icon_tint
                    .unwrap_or(style.color.unwrap_or(Color::BLACK)),
                rotation_degrees: 0.0,
                clip_rect: None,
            });
        }

        let content_box = LayoutBox {
            x: text_x,
            y: text_y,
            width: text_w,
            height: text_h,
        };

        let mut text_style = style.clone();
        text_style.font_size.get_or_insert(DEFAULT_FONT_SIZE);

        ctx.draw_content_text(TextCommand {
            text: self.content.clone(),
            position: (content_box.x, content_box.y),
            style: text_style,
            max_width: Some(draw_max_width + 0.5),
            clip_rect: None,
        });
    }

    fn event(&mut self, event: &InputEvent, ctx: &mut EventCtx) -> EventStatus {
        if !self.base.interaction.is_active() {
            return EventStatus::Ignored;
        }

        if let InputEvent::AnimationTick { .. } = event {
            if let Some(id) = &self.base.id {
                for action in crate::dom::take_actions(id) {
                    if matches!(action, crate::dom::DomAction::Click)
                        && let Some(cb) = self.base.interaction.on_click.as_mut()
                    {
                        cb(ctx);
                    }
                }
            }
            return EventStatus::Handled;
        }

        let before_style = self.base.computed_style.clone();
        let before_focus_visible = self.base.interaction.focus_visible;

        let status = self.base.interaction.handle(event, ctx);

        if matches!(status, EventStatus::Handled) {
            self.recompute_style();

            if self.base.computed_style != before_style
                || self.base.interaction.focus_visible != before_focus_visible
            {
                self.base.dirty = true;
                ctx.request_redraw();
            }
        }

        status
    }

    fn content_eq(&self, other: &dyn Widget) -> bool {
        let Some(other) = other.as_any().downcast_ref::<Button>() else {
            return false;
        };

        let icon_eq = match (&self.icon_document, &other.icon_document) {
            (Some(a), Some(b)) => **a == **b,
            (None, None) => true,
            _ => false,
        };

        self.content == other.content
            && self.base.authored_styles_eq(&other.base)
            && icon_eq
            && self.icon_position == other.icon_position
            && self.icon_gap == other.icon_gap
            && self.icon_size == other.icon_size
            && self.icon_tint == other.icon_tint
            && self.material_icon == other.material_icon
            && self.material_icon_axes == other.material_icon_axes
    }

    fn cascade_style(&mut self, parent: &Style, anim: &mut AnimationManager) {
        self.base.inherited_style = parent.clone();
        self.recompute_style();
        if crate::animate_computed_style(self.anim_id, &mut self.base.computed_style, anim) {
            self.base.dirty = true;
        }
    }

    fn after_interaction_transfer(&mut self) {
        self.recompute_style();
    }

    fn transfer_measured_state(&mut self, old: &dyn Widget) {
        if let Some(old) = old.as_any().downcast_ref::<Button>() {
            self.content_size.set(old.content_size.get());
            self.icon_render_size.set(old.icon_render_size.get());
        }
    }

    fn wants_animation_frame(&self) -> bool {
        self.base.interaction.enabled
            && self.base.id.as_deref().is_some_and(crate::dom::has_pending)
    }

    fn transfer_interaction_state(&mut self, old: &dyn Widget) {
        if let (Some(new), Some(old_i)) = (self.interaction_mut(), old.interaction()) {
            new.transfer_from(old_i);
        }
        // Transferred unconditionally (not gated by content_eq) so an
        // in-flight transition keeps animating smoothly even when the
        // button's own content changes between rebuilds.
        if let Some(old) = old.as_any().downcast_ref::<Button>() {
            self.anim_id = old.anim_id;
        }
    }

    fn anim_id(&self) -> WidgetId {
        self.anim_id
    }
}

// helper methods

fn justify_offset(justify: JustifyContent, available: f32, content: f32) -> f32 {
    match justify {
        JustifyContent::Start => 0.0,
        JustifyContent::End => (available - content).max(0.0),
        _ => (available - content).max(0.0) * 0.5,
    }
}

fn align_offset(align: Align, available: f32, content: f32) -> f32 {
    match align {
        Align::Start => 0.0,
        Align::End => (available - content).max(0.0),
        _ => (available - content).max(0.0) * 0.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_pressed_state_supplies_color_and_scale_feedback() {
        let resting = Color::rgba(80, 40, 120, 255);
        let mut button = Button::new().background(resting);
        button.base.interaction.pressed = true;
        button.recompute_style();

        assert_eq!(button.base.computed_style.scale, Some(0.97));
        assert_eq!(button.base.computed_style.content_scale, Some(1.0));
        assert!(
            button
                .base
                .computed_style
                .transition_properties
                .is_some_and(|properties| properties.contains(TransitionProperty::TRANSFORM))
        );
        assert_ne!(
            button
                .base
                .computed_style
                .background
                .as_ref()
                .map(crate::Background::representative_color),
            Some(resting)
        );
    }

    #[test]
    fn default_scale_transition_remains_enabled_after_release() {
        let mut button = Button::new().background(Color::WHITE);
        button.base.interaction.pressed = false;
        button.recompute_style();

        assert_eq!(button.base.computed_style.scale, Some(1.0));
        assert_eq!(button.base.computed_style.content_scale, Some(1.0));
        assert!(
            button
                .base
                .computed_style
                .transition_properties
                .is_some_and(|properties| properties.contains(TransitionProperty::TRANSFORM))
        );
    }
}
