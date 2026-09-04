// SPDX-License-Identifier: Apache-2.0
use crate::{
    AnimationManager, Color, Constraints, Cursor, FontStyle, FontWeight, Interaction, LayoutBox,
    MeasureContext, MeasureResult, PaintContext, Style, StyleBuilder, TextCommand, TextDecoration,
    Widget, WidgetBase, WidgetContent, WidgetId, constants::DEFAULT_FONT_SIZE,
};
use smol_str::SmolStr;
use std::cell::{Cell, RefCell};

/// One run of text within a [`RichText`] widget, styled independently of
/// its siblings. Unset fields fall back to the widget's own resolved
/// style, the same way a hover/pressed style patch overlays a base style.
#[derive(Clone, Debug, PartialEq)]
pub struct TextSpan {
    /// The `text` value carried by this type.
    pub text: SmolStr,
    /// The `color` value carried by this type.
    pub color: Option<Color>,
    /// The `weight` value carried by this type.
    pub weight: Option<FontWeight>,
    /// The `style` value carried by this type.
    pub style: Option<FontStyle>,
    /// The `decoration` value carried by this type.
    pub decoration: Option<TextDecoration>,
}

impl TextSpan {
    /// Creates a value with its default configuration.
    pub fn new(text: impl Into<SmolStr>) -> Self {
        Self {
            text: text.into(),
            color: None,
            weight: None,
            style: None,
            decoration: None,
        }
    }

    /// Returns or updates the `color` value.
    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    /// Returns or updates the `weight` value.
    pub fn weight(mut self, weight: FontWeight) -> Self {
        self.weight = Some(weight);
        self
    }

    /// Returns or updates the `style` value.
    pub fn style(mut self, style: FontStyle) -> Self {
        self.style = Some(style);
        self
    }

    /// Returns or updates the `decoration` value.
    pub fn decoration(mut self, decoration: TextDecoration) -> Self {
        self.decoration = Some(decoration);
        self
    }
}

impl From<&str> for TextSpan {
    fn from(text: &str) -> Self {
        Self::new(text)
    }
}

impl From<String> for TextSpan {
    fn from(text: String) -> Self {
        Self::new(text)
    }
}

impl From<SmolStr> for TextSpan {
    fn from(text: SmolStr) -> Self {
        Self::new(text)
    }
}

#[derive(Clone, Copy)]
struct PlacedToken {
    span_index: usize,
    start_byte: usize,
    end_byte: usize,
    x: f32,
    line: u32,
}

// Splits `text` into alternating whitespace/non-whitespace runs so line
// wrapping can break between words without needing to reshape a whole
// paragraph as a single buffer.
fn tokenize(text: &str, span_index: usize, out: &mut Vec<(usize, usize, usize, bool)>) {
    if text.is_empty() {
        return;
    }

    let mut start = 0;
    let mut current_is_space = text.chars().next().is_some_and(char::is_whitespace);

    for (i, c) in text.char_indices() {
        let is_space = c.is_whitespace();
        if is_space != current_is_space {
            out.push((span_index, start, i, current_is_space));
            start = i;
            current_is_space = is_space;
        }
    }
    out.push((span_index, start, text.len(), current_is_space));
}

/// Paints multiple independently-styled [`TextSpan`]s flowing on the same
/// line(s), wrapping across span boundaries like ordinary paragraph text.
/// Every span shares this widget's font/size/line-height; only color,
/// weight, style, and decoration can differ per span.
pub struct RichText {
    base: WidgetBase,
    anim_id: WidgetId,
    spans: Vec<TextSpan>,
    layout_box: LayoutBox,

    placed: RefCell<Vec<PlacedToken>>,
    content_size: Cell<(f32, f32)>,
    measured_max_width: Cell<Option<f32>>,
    line_height: Cell<f32>,
}

impl RichText {
    /// Creates a value with its default configuration.
    pub fn new() -> Self {
        let mut interaction = Interaction::new();
        interaction.focusable = false;
        interaction.hover_cursor = Some(Cursor::Default);

        let mut rich_text = Self {
            base: WidgetBase::new(interaction),
            anim_id: WidgetId::new_unique(),
            spans: Vec::new(),
            layout_box: LayoutBox::default(),

            placed: RefCell::new(Vec::new()),
            content_size: Cell::new((0.0, 0.0)),
            measured_max_width: Cell::new(None),
            line_height: Cell::new(0.0),
        };

        rich_text.recompute_style();
        rich_text
    }

    /// Replaces every span in this widget.
    pub fn spans(mut self, spans: impl Into<Vec<TextSpan>>) -> Self {
        self.spans = spans.into();
        self.mark_dirty();
        self
    }

    /// Appends one more span after any already set.
    pub fn span(mut self, span: impl Into<TextSpan>) -> Self {
        self.spans.push(span.into());
        self.mark_dirty();
        self
    }

    fn recompute_style(&mut self) {
        self.base.recompute_style();
        self.base.interaction.hover_cursor = self.base.computed_style.cursor;
    }
}

impl Default for RichText {
    fn default() -> Self {
        Self::new()
    }
}

impl StyleBuilder for RichText {
    fn style_mut(&mut self) -> &mut Style {
        &mut self.base.style
    }

    fn mark_dirty(&mut self) {
        self.base.dirty = true;
        self.recompute_style();
    }
}

impl WidgetContent for RichText {
    fn with_content(self, content: impl Into<SmolStr>) -> Self {
        self.span(TextSpan::new(content.into()))
    }
}

crate::impl_common_style_builders!(base RichText);
crate::impl_themed_style_builders!(base RichText; hover_style => hover_style, pressed_style => pressed_style, disabled_style => disabled_style, focus_style => focus_style, focused_hover_style => focused_hover_style, focused_pressed_style => focused_pressed_style);

impl Widget for RichText {
    crate::impl_widget_boilerplate!();

    fn debug_name(&self) -> &'static str {
        "Widget#RichText"
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
        let base_weight = style.font_weight.unwrap_or_default();
        let base_style = style.font_style.unwrap_or_default();

        // Kept logical (0.0 lets measure() auto-resolve the default ratio
        // internally); resolved to a concrete physical value separately
        // below, since this widget needs the real number for its own
        // line-stepping math in paint().
        let line_height_logical = style
            .line_height
            .map(|lh| lh.value().value())
            .unwrap_or(0.0);
        let line_height = if line_height_logical > 0.0 {
            line_height_logical * scale_factor
        } else {
            ctx.text.line_height(
                style.font.as_deref(),
                font_size,
                base_weight,
                base_style,
                scale_factor,
            )
        };

        self.measured_max_width.set(constraints.max_width);

        let mut tokens: Vec<(usize, usize, usize, bool)> = Vec::new();
        for (span_index, span) in self.spans.iter().enumerate() {
            tokenize(&span.text, span_index, &mut tokens);
        }

        let mut placed = Vec::with_capacity(tokens.len());
        let mut line = 0u32;
        let mut cursor_x = 0.0f32;
        let mut max_line_width = 0.0f32;

        for &(span_index, start, end, is_space) in &tokens {
            let span = &self.spans[span_index];
            let text = &span.text[start..end];
            let weight = span.weight.unwrap_or(base_weight);
            let font_style = span.style.unwrap_or(base_style);

            let width = ctx
                .text
                .measure(
                    text,
                    style.font.as_deref(),
                    font_size,
                    weight,
                    font_style,
                    letter_spacing,
                    line_height_logical,
                    None,
                    scale_factor,
                )
                .width;

            if let Some(max_w) = constraints.max_width
                && !is_space
                && cursor_x > 0.0
                && cursor_x + width > max_w
            {
                line += 1;
                cursor_x = 0.0;
            }

            // A space landing at the very start of a wrapped line carries
            // no visible width worth keeping (matches normal text reflow).
            if is_space && cursor_x == 0.0 {
                continue;
            }

            placed.push(PlacedToken {
                span_index,
                start_byte: start,
                end_byte: end,
                x: cursor_x,
                line,
            });
            cursor_x += width;
            max_line_width = max_line_width.max(cursor_x);
        }

        let line_count = (line + 1) as f32;
        self.line_height.set(line_height);
        *self.placed.borrow_mut() = placed;
        self.content_size
            .set((max_line_width, line_count * line_height));

        let padding = style.padding.unwrap_or_default();
        let width = max_line_width
            + padding.left.to_physical(scale_factor)
            + padding.right.to_physical(scale_factor);
        let height = line_count * line_height
            + padding.top.to_physical(scale_factor)
            + padding.bottom.to_physical(scale_factor);

        let (width, height) = constraints.constrain_size(width, height);
        MeasureResult::new(width, height)
    }

    fn paint(&self, ctx: &mut PaintContext) {
        self.paint_box(ctx);
        self.paint_outline(ctx);

        let style = &self.base.computed_style;
        let padding = style.padding.unwrap_or_default();
        let sf = ctx.scale_factor;

        let origin_x = self.layout_box.x + padding.left.to_physical(sf);
        let origin_y = self.layout_box.y + padding.top.to_physical(sf);
        let line_height = self.line_height.get();

        for token in self.placed.borrow().iter() {
            let span = &self.spans[token.span_index];
            let text = &span.text[token.start_byte..token.end_byte];
            if text.chars().all(char::is_whitespace) {
                continue;
            }

            let mut span_style = style.clone();
            span_style.color = span.color.or(style.color);
            span_style.font_weight =
                Some(span.weight.unwrap_or(style.font_weight.unwrap_or_default()));
            span_style.font_style =
                Some(span.style.unwrap_or(style.font_style.unwrap_or_default()));
            span_style.text_decoration = span.decoration.or(style.text_decoration);
            span_style.font_size.get_or_insert(DEFAULT_FONT_SIZE);

            ctx.draw_text(TextCommand {
                text: SmolStr::new(text),
                position: (
                    origin_x + token.x,
                    origin_y + (token.line as f32) * line_height,
                ),
                style: span_style,
                max_width: None,
                clip_rect: None,
            });
        }
    }

    fn content_eq(&self, other: &dyn Widget) -> bool {
        let Some(other) = other.as_any().downcast_ref::<RichText>() else {
            return false;
        };

        self.spans == other.spans && self.base.authored_styles_eq(&other.base)
    }

    fn cascade_style(&mut self, parent: &Style, anim: &mut AnimationManager) {
        self.base.inherited_style = parent.clone();
        self.recompute_style();
        if crate::animate_computed_style(self.anim_id, &mut self.base.computed_style, anim) {
            self.base.dirty = true;
        }
    }

    fn transfer_measured_state(&mut self, old: &dyn Widget) {
        if let Some(old) = old.as_any().downcast_ref::<RichText>() {
            self.content_size.set(old.content_size.get());
            self.measured_max_width.set(old.measured_max_width.get());
            self.line_height.set(old.line_height.get());
            self.placed.replace(old.placed.borrow().clone());
        }
    }

    fn after_interaction_transfer(&mut self) {
        self.recompute_style();
    }

    fn transfer_interaction_state(&mut self, old: &dyn Widget) {
        if let (Some(new), Some(old_i)) = (self.interaction_mut(), old.interaction()) {
            new.transfer_from(old_i);
        }
        if let Some(old) = old.as_any().downcast_ref::<RichText>() {
            self.anim_id = old.anim_id;
        }
    }

    fn anim_id(&self) -> WidgetId {
        self.anim_id
    }
}
