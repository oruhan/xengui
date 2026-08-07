// SPDX-License-Identifier: Apache-2.0
use smol_str::SmolStr;
use crate::{
    BoxShadow,
    BoxSizing,
    Cursor,
    FilterChain,
    Overscroll,
    ScrollbarGutter,
    TransitionProperty,
};
use super::{
    Outline,
    AlignItems,
    Background,
    Border,
    Color,
    Display,
    Edges,
    FlexDirection,
    FlexWrap,
    FontStyle,
    FontWeight,
    GridPlacement,
    GridTrack,
    JustifyContent,
    Length,
    LetterSpacing,
    LineHeight,
    Position,
    ScrollbarStyle,
    Size,
    TextAlign,
    TextDecoration,
    Overflow,
};

#[derive(Default, Clone, Debug, PartialEq)]
pub enum StyleValue<T> {
    #[default]
    Default,
    Value(T),
    None,
}

impl<T> From<T> for StyleValue<T> {
    fn from(value: T) -> Self {
        Self::Value(value)
    }
}

impl<T: Clone> StyleValue<T> {
    pub fn overlay(&self, parent: &Self) -> Self {
        match self {
            Self::Default => parent.clone(),
            Self::Value(value) => Self::Value(value.clone()),
            Self::None => Self::None,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Style {
    // Typography
    pub color: Option<Color>,
    /// Highlight color for selected text; inherited like `color`.
    pub selection_color: Option<Color>,
    /// Background color painted behind selected text; inherited like `color`.
    pub selection_background: Option<Color>,
    /// Color of the text caret; inherited like `color`.
    pub caret_color: Option<Color>,
    /// Border width for the selection highlight rect; inherited like `color`.
    pub selection_border_width: Option<Length>,
    /// Border color for the selection highlight rect; inherited like `color`.
    pub selection_border_color: Option<Color>,
    /// Border radius for the selection highlight rect; inherited like `color`.
    pub selection_border_radius: Option<Length>,

    pub cursor: Option<Cursor>,
    pub background: Option<Background>,
    pub font: Option<SmolStr>,
    pub font_size: Option<Length>,
    pub font_weight: Option<FontWeight>,
    pub font_style: Option<FontStyle>,
    pub text_align: Option<TextAlign>,
    pub text_decoration: Option<TextDecoration>,
    pub letter_spacing: Option<LetterSpacing>,
    pub line_height: Option<LineHeight>,

    // Box model
    pub padding: Option<Edges>,
    pub margin: Option<Edges>,
    pub border: Option<Border>,
    pub outline: StyleValue<Outline>,
    pub focus_outline: StyleValue<Outline>,

    /// CSS-style box shadow(s), painted in list order (first on top).
    /// Non-inherited, like `background`.
    pub box_shadow: Option<Vec<BoxShadow>>,

    /// GPU filter chain applied to this widget's rendered output.
    /// Non-inherited, like `box_shadow`.
    pub filter: Option<FilterChain>,

    /// GPU filter chain applied to whatever has already been painted
    /// behind this widget's own bounds (a live snapshot of the frame so
    /// far), before this widget paints its own background/content on top -
    /// matches CSS `backdrop-filter`. Non-inherited, like `filter`.
    pub backdrop_filter: Option<FilterChain>,

    // Sizing
    pub size: Option<Size>,
    pub min_size: Option<Size>,
    pub max_size: Option<Size>,
    pub box_sizing: BoxSizing,

    // Layout
    pub display: Option<Display>,
    pub position: Option<Position>,
    pub top: Option<Length>,
    pub right: Option<Length>,
    pub bottom: Option<Length>,
    pub left: Option<Length>,
    pub overflow_x: Option<Overflow>,
    pub overflow_y: Option<Overflow>,
    pub overscroll: Option<Overscroll>,

    /// Paint order relative to siblings; higher values paint later, on top. Mirrors CSS z-index.
    pub z_index: Option<i32>,

    // Flexbox
    pub flex_direction: Option<FlexDirection>,
    pub flex_wrap: Option<FlexWrap>,
    pub flex_grow: Option<f32>,
    pub flex_shrink: Option<f32>,
    pub flex_basis: Option<Length>,
    pub align_items: Option<AlignItems>,
    pub align_self: Option<AlignItems>,
    pub justify_content: Option<JustifyContent>,
    pub align_content: Option<JustifyContent>,
    pub gap: Option<(Length, Length)>,

    // Grid
    pub grid_template_columns: Option<Vec<GridTrack>>,
    pub grid_template_rows: Option<Vec<GridTrack>>,
    pub grid_column: Option<GridPlacement>,
    pub grid_row: Option<GridPlacement>,

    // Scrollbar
    pub scrollbar: Option<ScrollbarStyle>,
    pub scrollbar_hover: Option<ScrollbarStyle>,
    pub scrollbar_pressed: Option<ScrollbarStyle>,
    pub scrollbar_gutter: Option<ScrollbarGutter>,

    /// Overrides `scale` for the content layer only; `None` means the
    /// content follows the same scale as the rest of the widget.
    pub scale: Option<f32>,
    pub content_scale: Option<f32>,
    pub transition: Option<crate::Transition>,
    pub transition_properties: Option<TransitionProperty>,
    pub transition_overrides: crate::TransitionOverrides,
}

impl Style {
    pub fn overlay(&self, patch: &Style) -> Style {
        Style {
            color: patch.color.or(self.color),
            selection_color: patch.selection_color.or(self.selection_color),
            selection_background: patch.selection_background.or(self.selection_background),
            caret_color: patch.caret_color.or(self.caret_color),
            selection_border_width: patch.selection_border_width.or(self.selection_border_width),
            selection_border_color: patch.selection_border_color.or(self.selection_border_color),
            selection_border_radius: patch.selection_border_radius.or(self.selection_border_radius),
            cursor: patch.cursor.or(self.cursor),
            background: patch.background.clone().or(self.background.clone()),
            font: patch.font.clone().or(self.font.clone()),
            font_size: patch.font_size.or(self.font_size),
            font_weight: patch.font_weight.or(self.font_weight),
            font_style: patch.font_style.or(self.font_style),
            text_align: patch.text_align.or(self.text_align),
            text_decoration: patch.text_decoration.or(self.text_decoration),
            letter_spacing: patch.letter_spacing.or(self.letter_spacing),
            line_height: patch.line_height.or(self.line_height),

            padding: patch.padding.or(self.padding),
            margin: patch.margin.or(self.margin),
            border: patch.border.or(self.border),
            outline: match &patch.outline {
                StyleValue::Default => self.outline.clone(),
                value => value.clone(),
            },

            focus_outline: match &patch.focus_outline {
                StyleValue::Default => self.focus_outline.clone(),
                value => value.clone(),
            },

            box_shadow: patch.box_shadow.clone().or(self.box_shadow.clone()),

            filter: patch.filter.clone().or(self.filter.clone()),
            backdrop_filter: patch.backdrop_filter.clone().or(self.backdrop_filter.clone()),

            size: patch.size.or(self.size),
            min_size: patch.min_size.or(self.min_size),
            max_size: patch.max_size.or(self.max_size),
            box_sizing: patch.box_sizing,

            display: patch.display.or(self.display),
            position: patch.position.or(self.position),
            top: patch.top.or(self.top),
            right: patch.right.or(self.right),
            bottom: patch.bottom.or(self.bottom),
            left: patch.left.or(self.left),
            overflow_x: patch.overflow_x.or(self.overflow_x),
            overflow_y: patch.overflow_y.or(self.overflow_y),
            overscroll: patch.overscroll.or(self.overscroll),

            z_index: patch.z_index.or(self.z_index),

            flex_direction: patch.flex_direction.or(self.flex_direction),
            flex_wrap: patch.flex_wrap.or(self.flex_wrap),
            flex_grow: patch.flex_grow.or(self.flex_grow),
            flex_shrink: patch.flex_shrink.or(self.flex_shrink),
            flex_basis: patch.flex_basis.or(self.flex_basis),
            align_items: patch.align_items.or(self.align_items),
            align_self: patch.align_self.or(self.align_self),
            justify_content: patch.justify_content.or(self.justify_content),
            align_content: patch.align_content.or(self.align_content),
            gap: patch.gap.or(self.gap),

            grid_template_columns: patch.grid_template_columns
                .clone()
                .or(self.grid_template_columns.clone()),
            grid_template_rows: patch.grid_template_rows
                .clone()
                .or(self.grid_template_rows.clone()),
            grid_column: patch.grid_column.or(self.grid_column),
            grid_row: patch.grid_row.or(self.grid_row),

            scrollbar: match (&self.scrollbar, &patch.scrollbar) {
                (Some(base), Some(p)) => Some(base.overlay(p)),
                (None, Some(p)) => Some(*p),
                (Some(base), None) => Some(*base),
                (None, None) => None,
            },
            scrollbar_hover: match (&self.scrollbar_hover, &patch.scrollbar_hover) {
                (Some(base), Some(p)) => Some(base.overlay(p)),
                (None, Some(p)) => Some(*p),
                (Some(base), None) => Some(*base),
                (None, None) => None,
            },
            scrollbar_pressed: match (&self.scrollbar_pressed, &patch.scrollbar_pressed) {
                (Some(base), Some(p)) => Some(base.overlay(p)),
                (None, Some(p)) => Some(*p),
                (Some(base), None) => Some(*base),
                (None, None) => None,
            },
            scrollbar_gutter: patch.scrollbar_gutter.or(self.scrollbar_gutter),

            scale: patch.scale.or(self.scale),
            content_scale: patch.content_scale.or(self.content_scale),
            transition: patch.transition.or(self.transition),
            transition_properties: patch.transition_properties.or(self.transition_properties),
            transition_overrides: self.transition_overrides.overlay(&patch.transition_overrides),
        }
    }

    /// Fills in `patch`'s unset inheritable CSS properties using `self`
    /// as the parent style. Non-inheritable properties always come from `patch`.
    pub fn inherit_style(&self, patch: &Style) -> Style {
        let mut out = patch.clone();

        out.color = patch.color.or(self.color);
        out.selection_color = patch.selection_color.or(self.selection_color);
        out.selection_background = patch.selection_background.or(self.selection_background);
        out.caret_color = patch.caret_color.or(self.caret_color);
        out.selection_border_width = patch.selection_border_width.or(self.selection_border_width);
        out.selection_border_color = patch.selection_border_color.or(self.selection_border_color);
        out.selection_border_radius = patch.selection_border_radius.or(
            self.selection_border_radius
        );
        out.font = patch.font.clone().or(self.font.clone());
        out.font_size = patch.font_size.or(self.font_size);
        out.font_weight = patch.font_weight.or(self.font_weight);
        out.font_style = patch.font_style.or(self.font_style);
        out.text_align = patch.text_align.or(self.text_align);
        out.text_decoration = patch.text_decoration.or(self.text_decoration);
        out.letter_spacing = patch.letter_spacing.or(self.letter_spacing);
        out.line_height = patch.line_height.or(self.line_height);

        out.outline = patch.outline.overlay(&self.outline);

        out.scrollbar = match (&self.scrollbar, &patch.scrollbar) {
            (Some(base), Some(p)) => Some(base.overlay(p)),
            (None, Some(p)) => Some(*p),
            (Some(base), None) => Some(*base),
            (None, None) => None,
        };
        out.scrollbar_hover = match (&self.scrollbar_hover, &patch.scrollbar_hover) {
            (Some(base), Some(p)) => Some(base.overlay(p)),
            (None, Some(p)) => Some(*p),
            (Some(base), None) => Some(*base),
            (None, None) => None,
        };
        out.scrollbar_pressed = match (&self.scrollbar_pressed, &patch.scrollbar_pressed) {
            (Some(base), Some(p)) => Some(base.overlay(p)),
            (None, Some(p)) => Some(*p),
            (Some(base), None) => Some(*base),
            (None, None) => None,
        };

        out
    }
}
