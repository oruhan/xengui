// SPDX-License-Identifier: Apache-2.0
use super::{
    Align, Background, Border, Color, Display, Edges, FlexDirection, FlexWrap, FontStyle,
    FontWeight, GridPlacement, GridTrack, JustifyContent, Length, LetterSpacing, LineHeight,
    Outline, Overflow, Position, ScrollbarStyle, Size, TextAlign, TextDecoration,
};
use crate::{
    BoxShadow, BoxSizing, Cursor, FilterChain, Overscroll, ScrollbarGutter, TransformOrigin,
    TransitionProperty,
};
use smol_str::SmolStr;

#[derive(Default, Clone, Debug, PartialEq)]
/// Available `StyleValue` choices.
pub enum StyleValue<T> {
    #[default]
    /// The `Default` variant.
    Default,
    /// The `Value` variant.
    Value(T),
    /// The `None` variant.
    None,
}

impl<T> From<T> for StyleValue<T> {
    fn from(value: T) -> Self {
        Self::Value(value)
    }
}

impl<T: Clone> StyleValue<T> {
    /// Returns or updates the `overlay` value.
    pub fn overlay(&self, parent: &Self) -> Self {
        match self {
            Self::Default => parent.clone(),
            Self::Value(value) => Self::Value(value.clone()),
            Self::None => Self::None,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
/// Data and behavior represented by `Style`.
pub struct Style {
    // Typography
    /// The `color` value carried by this type.
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

    /// The `cursor` value carried by this type.
    pub cursor: Option<Cursor>,
    /// The `background` value carried by this type.
    pub background: Option<Background>,
    /// The `font` value carried by this type.
    pub font: Option<SmolStr>,
    /// The `font_size` value carried by this type.
    pub font_size: Option<Length>,
    /// The `font_weight` value carried by this type.
    pub font_weight: Option<FontWeight>,
    /// The `font_style` value carried by this type.
    pub font_style: Option<FontStyle>,
    /// The `text_align` value carried by this type.
    pub text_align: Option<TextAlign>,
    /// The `text_decoration` value carried by this type.
    pub text_decoration: Option<TextDecoration>,
    /// The `letter_spacing` value carried by this type.
    pub letter_spacing: Option<LetterSpacing>,
    /// The `line_height` value carried by this type.
    pub line_height: Option<LineHeight>,

    // Box model
    /// The `padding` value carried by this type.
    pub padding: Option<Edges>,
    /// The `margin` value carried by this type.
    pub margin: Option<Edges>,
    /// The `border` value carried by this type.
    pub border: Option<Border>,
    /// The `outline` value carried by this type.
    pub outline: StyleValue<Outline>,
    /// The `focus_outline` value carried by this type.
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
    /// The `size` value carried by this type.
    pub size: Option<Size>,
    /// The `min_size` value carried by this type.
    pub min_size: Option<Size>,
    /// The `max_size` value carried by this type.
    pub max_size: Option<Size>,
    /// The `box_sizing` value carried by this type.
    pub box_sizing: BoxSizing,

    // Layout
    /// The `display` value carried by this type.
    pub display: Option<Display>,
    /// The `position` value carried by this type.
    pub position: Option<Position>,
    /// The `top` value carried by this type.
    pub top: Option<Length>,
    /// The `right` value carried by this type.
    pub right: Option<Length>,
    /// The `bottom` value carried by this type.
    pub bottom: Option<Length>,
    /// The `left` value carried by this type.
    pub left: Option<Length>,
    /// The `overflow_x` value carried by this type.
    pub overflow_x: Option<Overflow>,
    /// The `overflow_y` value carried by this type.
    pub overflow_y: Option<Overflow>,
    /// The `overscroll` value carried by this type.
    pub overscroll: Option<Overscroll>,

    /// Paint order relative to siblings; higher values paint later, on top. Mirrors CSS z-index.
    pub z_index: Option<i32>,

    // Flexbox
    /// The `flex_direction` value carried by this type.
    pub flex_direction: Option<FlexDirection>,
    /// The `flex_wrap` value carried by this type.
    pub flex_wrap: Option<FlexWrap>,
    /// The `flex_grow` value carried by this type.
    pub flex_grow: Option<f32>,
    /// The `flex_shrink` value carried by this type.
    pub flex_shrink: Option<f32>,
    /// The `flex_basis` value carried by this type.
    pub flex_basis: Option<Length>,
    /// The `align_items` value carried by this type.
    pub align_items: Option<Align>,
    /// The `align_self` value carried by this type.
    pub align_self: Option<Align>,
    /// The `justify_content` value carried by this type.
    pub justify_content: Option<JustifyContent>,
    /// The `align_content` value carried by this type.
    pub align_content: Option<JustifyContent>,
    /// The `gap` value carried by this type.
    pub gap: Option<(Length, Length)>,

    // Grid
    /// The `grid_template_columns` value carried by this type.
    pub grid_template_columns: Option<Vec<GridTrack>>,
    /// The `grid_template_rows` value carried by this type.
    pub grid_template_rows: Option<Vec<GridTrack>>,
    /// The `grid_column` value carried by this type.
    pub grid_column: Option<GridPlacement>,
    /// The `grid_row` value carried by this type.
    pub grid_row: Option<GridPlacement>,

    // Scrollbar
    /// The `scrollbar` value carried by this type.
    pub scrollbar: Option<ScrollbarStyle>,
    /// The `scrollbar_hover` value carried by this type.
    pub scrollbar_hover: Option<ScrollbarStyle>,
    /// The `scrollbar_pressed` value carried by this type.
    pub scrollbar_pressed: Option<ScrollbarStyle>,
    /// The `scrollbar_gutter` value carried by this type.
    pub scrollbar_gutter: Option<ScrollbarGutter>,
    /// The `scrollbar_auto_hide` value carried by this type.
    pub scrollbar_auto_hide: Option<bool>,

    /// The point around which CSS transforms are applied.
    /// Defaults to the widget center (`50% 50%`) when `None`.
    pub transform_origin: Option<TransformOrigin>,
    /// Overrides `scale` for the content layer only; `None` means the
    /// content follows the same scale as the rest of the widget.
    pub scale: Option<f32>,
    /// The `content_scale` value carried by this type.
    pub content_scale: Option<f32>,
    /// The `transition` value carried by this type.
    pub transition: Option<crate::Transition>,
    /// The `transition_properties` value carried by this type.
    pub transition_properties: Option<TransitionProperty>,
    /// The `transition_overrides` value carried by this type.
    pub transition_overrides: crate::TransitionOverrides,
}

impl Style {
    /// Returns or updates the `overlay` value.
    pub fn overlay(&self, patch: &Style) -> Style {
        Style {
            color: patch.color.or(self.color),
            selection_color: patch.selection_color.or(self.selection_color),
            selection_background: patch.selection_background.or(self.selection_background),
            caret_color: patch.caret_color.or(self.caret_color),
            selection_border_width: patch.selection_border_width.or(self.selection_border_width),
            selection_border_color: patch.selection_border_color.or(self.selection_border_color),
            selection_border_radius: patch
                .selection_border_radius
                .or(self.selection_border_radius),
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
            backdrop_filter: patch
                .backdrop_filter
                .clone()
                .or(self.backdrop_filter.clone()),

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

            grid_template_columns: patch
                .grid_template_columns
                .clone()
                .or(self.grid_template_columns.clone()),
            grid_template_rows: patch
                .grid_template_rows
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
            scrollbar_auto_hide: patch.scrollbar_auto_hide.or(self.scrollbar_auto_hide),

            transform_origin: patch.transform_origin.or(self.transform_origin),
            scale: patch.scale.or(self.scale),
            content_scale: patch.content_scale.or(self.content_scale),
            transition: patch.transition.or(self.transition),
            transition_properties: patch.transition_properties.or(self.transition_properties),
            transition_overrides: self
                .transition_overrides
                .overlay(&patch.transition_overrides),
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
        out.selection_border_radius = patch
            .selection_border_radius
            .or(self.selection_border_radius);
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

    /// Whether `other` differs from `self` in a way that would change this
    /// widget's own taffy layout node - size, box model, flex/grid
    /// placement, or text metrics feeding intrinsic measurement - as
    /// opposed to purely visual properties (colors, shadows, cursor,
    /// outline, ...).
    pub fn layout_affecting_diff(&self, other: &Style) -> bool {
        self.display != other.display
            || self.position != other.position
            || self.top != other.top
            || self.right != other.right
            || self.bottom != other.bottom
            || self.left != other.left
            || self.size != other.size
            || self.min_size != other.min_size
            || self.max_size != other.max_size
            || self.box_sizing != other.box_sizing
            || self.padding != other.padding
            || self.margin != other.margin
            || border_width_diff(self.border, other.border)
            || self.overflow_x != other.overflow_x
            || self.overflow_y != other.overflow_y
            || self.flex_direction != other.flex_direction
            || self.flex_wrap != other.flex_wrap
            || self.flex_grow != other.flex_grow
            || self.flex_shrink != other.flex_shrink
            || self.flex_basis != other.flex_basis
            || self.align_items != other.align_items
            || self.align_self != other.align_self
            || self.justify_content != other.justify_content
            || self.align_content != other.align_content
            || self.gap != other.gap
            || self.grid_template_columns != other.grid_template_columns
            || self.grid_template_rows != other.grid_template_rows
            || self.grid_column != other.grid_column
            || self.grid_row != other.grid_row
            || self.font != other.font
            || self.font_size != other.font_size
            || self.font_weight != other.font_weight
            || self.font_style != other.font_style
            || self.letter_spacing != other.letter_spacing
            || self.line_height != other.line_height
    }
}

fn border_width_diff(a: Option<Border>, b: Option<Border>) -> bool {
    let widths = |border: Option<Border>| border.map(|b| (b.top, b.right, b.bottom, b.left));
    widths(a) != widths(b)
}
