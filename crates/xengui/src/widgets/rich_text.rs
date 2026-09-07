// SPDX-License-Identifier: Apache-2.0
use crate::{
    AnimationManager, Background, Color, Constraints, Cursor, ElementState, EventCtx, EventStatus,
    FontStyle, FontWeight, InputEvent, Interaction, LayoutBox, MULTI_CLICK_DISTANCE_DP,
    MULTI_CLICK_INTERVAL, MeasureContext, MeasureResult, MouseButton, PaintContext, RectCommand,
    Style, StyleBuilder, TextCommand, TextDecoration, Widget, WidgetBase, WidgetContent, WidgetId,
    constants::DEFAULT_FONT_SIZE,
};
use smol_str::SmolStr;
use std::cell::{Cell, RefCell};
use web_time::Instant;

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
    char_start: usize,
    char_end: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TokenKind {
    Text,
    Space,
    Newline,
}

#[derive(Clone, Copy)]
struct RawToken {
    span_index: usize,
    start_byte: usize,
    end_byte: usize,
    char_start: usize,
    char_end: usize,
    kind: TokenKind,
}

#[derive(Clone, Default)]
struct PlacedLine {
    start_char: usize,
    end_char: usize,
    width: f32,
    boundaries: Vec<(usize, f32)>,
}

// Splits text into words, horizontal whitespace, and explicit hard breaks.
fn tokenize(text: &str, span_index: usize, global_char_start: usize, out: &mut Vec<RawToken>) {
    if text.is_empty() {
        return;
    }

    let classify = |c: char| {
        if c == '\n' {
            TokenKind::Newline
        } else if c.is_whitespace() {
            TokenKind::Space
        } else {
            TokenKind::Text
        }
    };

    let mut chars = text.char_indices().peekable();
    let mut char_index = global_char_start;
    while let Some((start_byte, first)) = chars.next() {
        let kind = classify(first);
        let start_char = char_index;
        char_index += 1;
        let mut end_byte = start_byte + first.len_utf8();

        if kind != TokenKind::Newline {
            while let Some(&(byte, c)) = chars.peek() {
                if classify(c) != kind {
                    break;
                }
                chars.next();
                char_index += 1;
                end_byte = byte + c.len_utf8();
            }
        }

        out.push(RawToken {
            span_index,
            start_byte,
            end_byte,
            char_start: start_char,
            char_end: char_index,
            kind,
        });
    }
}

/// Paints multiple independently-styled [`TextSpan`]s flowing on the same
/// line(s), wrapping across span boundaries like ordinary paragraph text.
/// Every span shares this widget's font/size/line-height; only color,
/// weight, style, and decoration can differ per span.
pub struct RichText {
    base: WidgetBase,
    anim_id: WidgetId,
    spans: Vec<TextSpan>,
    plain_text: SmolStr,
    selectable: bool,
    preserve_whitespace: bool,
    wrap: bool,
    layout_box: LayoutBox,

    placed: RefCell<Vec<PlacedToken>>,
    lines: RefCell<Vec<PlacedLine>>,
    content_size: Cell<(f32, f32)>,
    measured_max_width: Cell<Option<f32>>,
    line_height: Cell<f32>,
    scale_factor: Cell<f32>,
    selection_anchor: Cell<Option<usize>>,
    selection_cursor: Cell<Option<usize>>,
    dragging: Cell<bool>,
    click_count: Cell<u8>,
    last_click_time: Cell<Option<Instant>>,
    last_click_pos: Cell<(f32, f32)>,
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
            plain_text: SmolStr::new(""),
            selectable: false,
            preserve_whitespace: false,
            wrap: true,
            layout_box: LayoutBox::default(),

            placed: RefCell::new(Vec::new()),
            lines: RefCell::new(Vec::new()),
            content_size: Cell::new((0.0, 0.0)),
            measured_max_width: Cell::new(None),
            line_height: Cell::new(0.0),
            scale_factor: Cell::new(1.0),
            selection_anchor: Cell::new(None),
            selection_cursor: Cell::new(None),
            dragging: Cell::new(false),
            click_count: Cell::new(0),
            last_click_time: Cell::new(None),
            last_click_pos: Cell::new((0.0, 0.0)),
        };

        rich_text.recompute_style();
        rich_text
    }

    /// Replaces every span in this widget.
    pub fn spans(mut self, spans: impl Into<Vec<TextSpan>>) -> Self {
        self.spans = spans.into();
        self.rebuild_plain_text();
        self.mark_dirty();
        self
    }

    /// Appends one more span after any already set.
    pub fn span(mut self, span: impl Into<TextSpan>) -> Self {
        self.spans.push(span.into());
        self.rebuild_plain_text();
        self.mark_dirty();
        self
    }

    /// Enables or disables pointer selection and platform copy support.
    pub fn selectable(mut self, value: bool) -> Self {
        self.selectable = value;
        self.mark_dirty();
        self
    }

    /// Preserves leading whitespace instead of collapsing it at line starts.
    pub fn preserve_whitespace(mut self, value: bool) -> Self {
        self.preserve_whitespace = value;
        self.mark_dirty();
        self
    }

    /// Enables or disables automatic word wrapping at the available width.
    pub fn wrap(mut self, value: bool) -> Self {
        self.wrap = value;
        self.mark_dirty();
        self
    }

    fn rebuild_plain_text(&mut self) {
        let mut text = String::new();
        for span in &self.spans {
            text.push_str(&span.text);
        }
        self.plain_text = SmolStr::new(text);
    }

    fn char_class(c: char) -> u8 {
        if c.is_whitespace() {
            0
        } else if c.is_alphanumeric() || c == '_' {
            1
        } else {
            2
        }
    }

    fn word_bounds_at(&self, idx: usize) -> (usize, usize) {
        let chars: Vec<char> = self.plain_text.chars().collect();
        if chars.is_empty() {
            return (0, 0);
        }
        let probe = idx.min(chars.len() - 1);
        let class = Self::char_class(chars[probe]);
        let mut start = probe;
        while start > 0 && Self::char_class(chars[start - 1]) == class {
            start -= 1;
        }
        let mut end = probe + 1;
        while end < chars.len() && Self::char_class(chars[end]) == class {
            end += 1;
        }
        (start, end)
    }

    fn line_x_at(line: &PlacedLine, index: usize) -> f32 {
        line.boundaries
            .iter()
            .find_map(|&(i, x)| (i == index).then_some(x))
            .unwrap_or({
                if index <= line.start_char {
                    0.0
                } else {
                    line.width
                }
            })
    }

    fn index_for_point(&self, point: (f32, f32)) -> usize {
        let style = &self.base.computed_style;
        let sf = self.scale_factor.get();
        let padding = style.padding.unwrap_or_default();
        let local_x = point.0 - self.layout_box.x - padding.left.to_physical(sf);
        let local_y = point.1 - self.layout_box.y - padding.top.to_physical(sf);
        let lines = self.lines.borrow();
        if lines.is_empty() {
            return 0;
        }
        let height = self.line_height.get().max(1.0);
        let line_index = (local_y / height).floor().max(0.0) as usize;
        let line = &lines[line_index.min(lines.len() - 1)];
        line.boundaries
            .iter()
            .min_by(|a, b| (a.1 - local_x).abs().total_cmp(&(b.1 - local_x).abs()))
            .map(|&(index, _)| index)
            .unwrap_or(line.start_char)
    }

    fn recompute_style(&mut self) {
        self.base.recompute_style();
        self.base.interaction.hover_cursor = self
            .base
            .computed_style
            .cursor
            .or(self.selectable.then_some(Cursor::Text));
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
        self.scale_factor.set(scale_factor);
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

        let mut tokens = Vec::new();
        let mut global_char_start = 0;
        for (span_index, span) in self.spans.iter().enumerate() {
            tokenize(&span.text, span_index, global_char_start, &mut tokens);
            global_char_start += span.text.chars().count();
        }

        let mut placed = Vec::with_capacity(tokens.len());
        let mut lines = vec![PlacedLine {
            start_char: 0,
            end_char: 0,
            width: 0.0,
            boundaries: vec![(0, 0.0)],
        }];
        let mut cursor_x = 0.0f32;
        let mut max_line_width = 0.0f32;

        for token in &tokens {
            if token.kind == TokenKind::Newline {
                let current = lines.last_mut().expect("a text layout always has one line");
                current.end_char = token.char_end;
                current.width = cursor_x;
                if current
                    .boundaries
                    .last()
                    .is_none_or(|&(index, _)| index != token.char_end)
                {
                    current.boundaries.push((token.char_end, cursor_x));
                }
                max_line_width = max_line_width.max(cursor_x);
                lines.push(PlacedLine {
                    start_char: token.char_end,
                    end_char: token.char_end,
                    width: 0.0,
                    boundaries: vec![(token.char_end, 0.0)],
                });
                cursor_x = 0.0;
                continue;
            }

            let span = &self.spans[token.span_index];
            let text = &span.text[token.start_byte..token.end_byte];
            let weight = span.weight.unwrap_or(base_weight);
            let font_style = span.style.unwrap_or(base_style);

            let offsets = ctx.text.character_offsets(
                text,
                style.font.as_deref(),
                font_size,
                weight,
                font_style,
                letter_spacing,
                line_height_logical,
                scale_factor,
            );
            let width = offsets.last().copied().unwrap_or(0.0);

            if self.wrap
                && let Some(max_w) = constraints.max_width
                && token.kind != TokenKind::Space
                && cursor_x > 0.0
                && cursor_x + width > max_w
            {
                let current = lines.last_mut().expect("a text layout always has one line");
                current.end_char = token.char_start;
                current.width = cursor_x;
                max_line_width = max_line_width.max(cursor_x);
                lines.push(PlacedLine {
                    start_char: token.char_start,
                    end_char: token.char_start,
                    width: 0.0,
                    boundaries: vec![(token.char_start, 0.0)],
                });
                cursor_x = 0.0;
            }

            // A space landing at the very start of a wrapped line carries
            // no visible width worth keeping (matches normal text reflow).
            if !self.preserve_whitespace && token.kind == TokenKind::Space && cursor_x == 0.0 {
                let current = lines.last_mut().expect("a text layout always has one line");
                for index in token.char_start..=token.char_end {
                    if current.boundaries.last().map(|item| item.0) != Some(index) {
                        current.boundaries.push((index, 0.0));
                    }
                }
                current.end_char = token.char_end;
                continue;
            }

            let line = (lines.len() - 1) as u32;
            placed.push(PlacedToken {
                span_index: token.span_index,
                start_byte: token.start_byte,
                end_byte: token.end_byte,
                x: cursor_x,
                line,
                char_start: token.char_start,
                char_end: token.char_end,
            });

            let current = lines.last_mut().expect("a text layout always has one line");
            for (offset_index, &offset) in offsets.iter().enumerate() {
                let index = token.char_start + offset_index;
                if current.boundaries.last().map(|item| item.0) == Some(index) {
                    if let Some(last) = current.boundaries.last_mut() {
                        last.1 = cursor_x + offset;
                    }
                } else {
                    current.boundaries.push((index, cursor_x + offset));
                }
            }
            cursor_x += width;
            current.end_char = token.char_end;
            current.width = cursor_x;
            max_line_width = max_line_width.max(cursor_x);
        }

        if let Some(current) = lines.last_mut() {
            current.end_char = global_char_start.max(current.end_char);
            current.width = cursor_x;
            if current.boundaries.last().map(|item| item.0) != Some(current.end_char) {
                current.boundaries.push((current.end_char, cursor_x));
            }
        }

        let line_count = lines.len() as f32;
        self.line_height.set(line_height);
        *self.placed.borrow_mut() = placed;
        *self.lines.borrow_mut() = lines;
        self.content_size
            .set((max_line_width, line_count * line_height));

        let padding = style.padding.unwrap_or_default();
        let width = max_line_width
            + padding.left.to_physical(scale_factor)
            + padding.right.to_physical(scale_factor);
        let height = line_count * line_height
            + padding.top.to_physical(scale_factor)
            + padding.bottom.to_physical(scale_factor);

        let (width, height) = if self.wrap {
            constraints.constrain_size(width, height)
        } else {
            (
                constraints.known_width.unwrap_or(width),
                constraints.constrain_height(height),
            )
        };
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

        let selection = self.selectable.then(|| self.text_selection()).flatten();
        if let Some((selection_start, selection_end)) = selection {
            for (line_index, line) in self.lines.borrow().iter().enumerate() {
                let start = selection_start.max(line.start_char);
                let end = selection_end.min(line.end_char);
                if start >= end {
                    continue;
                }
                let start_x = Self::line_x_at(line, start);
                let end_x = Self::line_x_at(line, end);
                ctx.draw_rect(RectCommand {
                    position: (
                        origin_x + start_x,
                        origin_y + line_index as f32 * line_height,
                    ),
                    size: ((end_x - start_x).max(2.0 * sf), line_height.max(1.0)),
                    background: Some(Background::Color(
                        style
                            .selection_background
                            .unwrap_or(Color::rgba(90, 140, 230, 100)),
                    )),
                    border_radius: style.selection_border_radius.map(Into::into),
                    border_width: style.selection_border_width,
                    border_color: style.selection_border_color,
                    clip_rect: None,
                });
            }
        }

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
                style: span_style.clone(),
                max_width: None,
                clip_rect: None,
            });

            if let (Some((selection_start, selection_end)), Some(selection_color)) =
                (selection, style.selection_color)
            {
                let start = selection_start.max(token.char_start);
                let end = selection_end.min(token.char_end);
                if start < end {
                    let lines = self.lines.borrow();
                    let line = &lines[token.line as usize];
                    let start_x = Self::line_x_at(line, start);
                    let end_x = Self::line_x_at(line, end);
                    let mut selected_style = span_style;
                    selected_style.color = Some(selection_color);
                    ctx.draw_text(TextCommand {
                        text: SmolStr::new(text),
                        position: (
                            origin_x + token.x,
                            origin_y + token.line as f32 * line_height,
                        ),
                        style: selected_style,
                        max_width: None,
                        clip_rect: Some((
                            origin_x + start_x,
                            origin_y + token.line as f32 * line_height,
                            (end_x - start_x).max(0.0),
                            line_height.max(1.0),
                        )),
                    });
                }
            }
        }
    }

    fn event(&mut self, event: &InputEvent, ctx: &mut EventCtx) -> EventStatus {
        if self.selectable
            && let InputEvent::MouseInput {
                state,
                button: MouseButton::Left,
                position,
            } = event
        {
            let idx = self.index_for_point(*position);
            match state {
                ElementState::Pressed => {
                    let now = Instant::now();
                    let (last_x, last_y) = self.last_click_pos.get();
                    let click_distance = MULTI_CLICK_DISTANCE_DP * self.scale_factor.get();
                    let same_spot = (position.0 - last_x).abs() < click_distance
                        && (position.1 - last_y).abs() < click_distance;
                    let is_repeat = same_spot
                        && self
                            .last_click_time
                            .get()
                            .is_some_and(|time| now.duration_since(time) < MULTI_CLICK_INTERVAL);
                    let click_count = if is_repeat {
                        (self.click_count.get() + 1).min(3)
                    } else {
                        1
                    };
                    self.click_count.set(click_count);
                    self.last_click_time.set(Some(now));
                    self.last_click_pos.set(*position);

                    match click_count {
                        1 => {
                            self.selection_anchor.set(Some(idx));
                            self.selection_cursor.set(Some(idx));
                            self.dragging.set(true);
                        }
                        2 => {
                            let (start, end) = self.word_bounds_at(idx);
                            self.selection_anchor.set(Some(start));
                            self.selection_cursor.set(Some(end));
                            self.dragging.set(false);
                            ctx.suppress_text_drag();
                        }
                        _ => {
                            self.select_all_text();
                            self.dragging.set(false);
                            ctx.suppress_text_drag();
                        }
                    }
                }
                ElementState::Released => self.dragging.set(false),
            }
            self.base.dirty = true;
            ctx.request_redraw();
        }

        if !self.base.interaction.is_active() {
            return EventStatus::Ignored;
        }

        let before_style = self.base.computed_style.clone();
        let status = self.base.interaction.handle(event, ctx);
        if matches!(status, EventStatus::Handled) {
            self.recompute_style();
            if self.base.computed_style != before_style {
                self.base.dirty = true;
                ctx.request_redraw();
            }
        }
        status
    }

    fn selectable_text(&self) -> Option<&str> {
        self.selectable.then_some(self.plain_text.as_str())
    }

    fn text_selection(&self) -> Option<(usize, usize)> {
        let anchor = self.selection_anchor.get()?;
        let cursor = self.selection_cursor.get()?;
        (anchor != cursor).then(|| (anchor.min(cursor), anchor.max(cursor)))
    }

    fn set_text_selection(&mut self, range: Option<(usize, usize)>) {
        let length = self.plain_text.chars().count();
        let (anchor, cursor) = range.map_or((None, None), |(start, end)| {
            (Some(start.min(length)), Some(end.min(length)))
        });
        if self.selection_anchor.get() == anchor && self.selection_cursor.get() == cursor {
            return;
        }
        self.selection_anchor.set(anchor);
        self.selection_cursor.set(cursor);
        self.base.dirty = true;
    }

    fn cancel_text_selection(&mut self) {
        self.selection_anchor.set(None);
        self.selection_cursor.set(None);
        self.dragging.set(false);
        self.base.dirty = true;
    }

    fn text_index_at(&self, point: (f32, f32)) -> usize {
        self.index_for_point(point)
    }

    fn select_all_text(&mut self) {
        if !self.selectable {
            return;
        }
        self.selection_anchor.set(Some(0));
        self.selection_cursor
            .set(Some(self.plain_text.chars().count()));
        self.base.dirty = true;
    }

    fn content_eq(&self, other: &dyn Widget) -> bool {
        let Some(other) = other.as_any().downcast_ref::<RichText>() else {
            return false;
        };

        self.spans == other.spans
            && self.base.authored_styles_eq(&other.base)
            && self.selectable == other.selectable
            && self.preserve_whitespace == other.preserve_whitespace
            && self.wrap == other.wrap
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
            self.lines.replace(old.lines.borrow().clone());
            self.scale_factor.set(old.scale_factor.get());
            self.selection_anchor.set(old.selection_anchor.get());
            self.selection_cursor.set(old.selection_cursor.get());
            self.click_count.set(old.click_count.get());
            self.last_click_time.set(old.last_click_time.get());
            self.last_click_pos.set(old.last_click_pos.get());
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
            self.dragging.set(old.dragging.get());
        }
    }

    fn anim_id(&self) -> WidgetId {
        self.anim_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TextMeasurer;

    struct FixedTextMeasurer;

    impl TextMeasurer for FixedTextMeasurer {
        fn measure(
            &mut self,
            text: &str,
            _font: Option<&str>,
            _font_size: f32,
            _font_weight: FontWeight,
            _font_style: FontStyle,
            _letter_spacing: f32,
            _line_height: f32,
            _max_width: Option<f32>,
            _scale_factor: f32,
        ) -> MeasureResult {
            MeasureResult::new(text.chars().count() as f32 * 10.0, 20.0)
        }

        fn character_offsets(
            &mut self,
            text: &str,
            _font: Option<&str>,
            _font_size: f32,
            _font_weight: FontWeight,
            _font_style: FontStyle,
            _letter_spacing: f32,
            _line_height: f32,
            _scale_factor: f32,
        ) -> Vec<f32> {
            (0..=text.chars().count())
                .map(|index| index as f32 * 10.0)
                .collect()
        }

        fn ascent(
            &mut self,
            _font: Option<&str>,
            _font_size: f32,
            _font_weight: FontWeight,
            _font_style: FontStyle,
            _scale_factor: f32,
        ) -> f32 {
            15.0
        }

        fn descent(
            &mut self,
            _font: Option<&str>,
            _font_size: f32,
            _font_weight: FontWeight,
            _font_style: FontStyle,
            _scale_factor: f32,
        ) -> f32 {
            5.0
        }

        fn line_height(
            &mut self,
            _font: Option<&str>,
            _font_size: f32,
            _font_weight: FontWeight,
            _font_style: FontStyle,
            _scale_factor: f32,
        ) -> f32 {
            20.0
        }
    }

    #[test]
    fn multiline_selection_hit_testing_uses_both_axes() {
        let mut rich_text = RichText::new()
            .span(TextSpan::new("ab\n").color(Color::BLUE_400))
            .span("  cd")
            .selectable(true)
            .preserve_whitespace(true)
            .wrap(false);
        let mut measurer = FixedTextMeasurer;
        let mut context = MeasureContext::new(&mut measurer, 1.0);
        let result = rich_text.measure(&mut context, Constraints::new().with_max_width(20.0));
        rich_text.layout(LayoutBox {
            x: 0.0,
            y: 0.0,
            width: result.width,
            height: result.height,
        });

        assert_eq!(result.height, 40.0);
        assert_eq!(result.width, 40.0);
        assert_eq!(rich_text.text_index_at((21.0, 25.0)), 5);
        rich_text.set_text_selection(Some((1, 6)));
        assert_eq!(rich_text.text_selection(), Some((1, 6)));
        assert_eq!(rich_text.selectable_text(), Some("ab\n  cd"));
    }
}
