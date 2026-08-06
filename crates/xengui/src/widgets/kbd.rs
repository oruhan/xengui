// SPDX-License-Identifier: Apache-2.0
use crate::{
    AnimationManager,
    Background,
    Border,
    Color,
    Constraints,
    Edges,
    Interaction,
    LayoutBox,
    Length,
    MeasureContext,
    MeasureResult,
    PaintContext,
    Style,
    StyleBuilder,
    TextCommand,
    Widget,
    WidgetBase,
    WidgetContent,
    WidgetId,
    properties::DEFAULT_FONT_SIZE,
};
use smol_str::SmolStr;
use std::cell::Cell;

/// Displays a single keyboard key or shortcut (e.g. "Ctrl", "⌘K"), styled
/// like the HTML `<kbd>` element - a small bordered badge distinct from
/// ordinary body text.
pub struct Kbd {
    base: WidgetBase,
    anim_id: WidgetId,
    content: SmolStr,
    layout_box: LayoutBox,
    content_size: Cell<(f32, f32)>,
}

impl Kbd {
    pub fn new() -> Self {
        let mut base = WidgetBase::new(Interaction::new());

        // Default look: small rounded badge with a keycap-like border.
        base.style.padding = Some(Edges::symmetric(6.0, 2.0));
        base.style.font_size = Some(Length::px(13.0));
        base.style.background = Some(Background::Color(Color::NEUTRAL_100));
        base.style.border = Some(Border::new().width(1.0).color(Color::NEUTRAL_300).radius(5));
        base.style.color = Some(Color::NEUTRAL_600);

        let mut kbd = Self {
            base,
            anim_id: WidgetId::new_unique(),
            content: SmolStr::new(""),
            layout_box: LayoutBox::default(),
            content_size: Cell::new((0.0, 0.0)),
        };
        kbd.recompute_style();
        kbd
    }

    pub fn label(mut self, content: impl Into<SmolStr>) -> Self {
        self.content = content.into();
        self.mark_dirty();
        self
    }

    fn recompute_style(&mut self) {
        self.base.recompute_style();
    }
}

impl Default for Kbd {
    fn default() -> Self {
        Self::new()
    }
}

impl StyleBuilder for Kbd {
    fn style_mut(&mut self) -> &mut Style {
        &mut self.base.style
    }

    fn mark_dirty(&mut self) {
        self.base.dirty = true;
        self.recompute_style();
    }
}

impl WidgetContent for Kbd {
    fn with_content(self, content: impl Into<SmolStr>) -> Self {
        self.label(content)
    }
}

crate::impl_common_style_builders!(base Kbd);

impl Widget for Kbd {
    crate::impl_widget_boilerplate!();

    fn debug_name(&self) -> &'static str {
        "Widget#Kbd"
    }

    fn measure(&self, ctx: &mut MeasureContext, constraints: Constraints) -> MeasureResult {
        let scale_factor = ctx.scale_factor;
        let style = &self.base.computed_style;

        // Logical font size; TextMeasurer converts to physical internally.
        let font_size = style.font_size.unwrap_or(DEFAULT_FONT_SIZE).value();

        let result = ctx.text.measure(
            &self.content,
            style.font.as_deref(),
            font_size,
            style.font_weight.unwrap_or_default(),
            style.font_style.unwrap_or_default(),
            0.0,
            0.0,
            None,
            scale_factor
        );

        self.content_size.set((result.width, result.height));

        let padding = style.padding.unwrap_or_default();
        let width =
            result.width +
            padding.left.to_physical(scale_factor) +
            padding.right.to_physical(scale_factor);
        let height =
            result.height +
            padding.top.to_physical(scale_factor) +
            padding.bottom.to_physical(scale_factor);

        let (width, height) = constraints.constrain_size(width, height);
        MeasureResult::new(width, height)
    }

    fn paint(&self, ctx: &mut PaintContext) {
        self.paint_box(ctx);
        self.paint_outline(ctx);

        let style = &self.base.computed_style;
        let padding = style.padding.unwrap_or_default();
        let sf = ctx.scale_factor;

        let text_x = self.layout_box.x + padding.left.to_physical(sf);
        let text_y = self.layout_box.y + padding.top.to_physical(sf);

        let mut text_style = style.clone();
        text_style.font_size.get_or_insert(Length::px(13.0));

        ctx.draw_text(TextCommand {
            text: self.content.clone(),
            position: (text_x, text_y),
            style: text_style,
            max_width: None,
            clip_rect: None,
        });
    }

    fn content_eq(&self, other: &dyn Widget) -> bool {
        let Some(other) = other.as_any().downcast_ref::<Kbd>() else {
            return false;
        };
        self.content == other.content && self.base.style == other.base.style
    }

    fn cascade_style(&mut self, parent: &Style, anim: &mut AnimationManager) {
        self.base.inherited_style = parent.clone();
        self.recompute_style();
        if crate::animate_computed_style(self.anim_id, &mut self.base.computed_style, anim) {
            self.base.dirty = true;
        }
    }

    fn transfer_measured_state(&mut self, old: &dyn Widget) {
        if let Some(old) = old.as_any().downcast_ref::<Kbd>() {
            self.content_size.set(old.content_size.get());
        }
    }

    fn transfer_interaction_state(&mut self, old: &dyn Widget) {
        if let (Some(new), Some(old_i)) = (self.interaction_mut(), old.interaction()) {
            new.transfer_from(old_i);
        }
        if let Some(old) = old.as_any().downcast_ref::<Kbd>() {
            self.anim_id = old.anim_id;
        }
    }

    fn anim_id(&self) -> WidgetId {
        self.anim_id
    }
}
