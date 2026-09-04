// SPDX-License-Identifier: Apache-2.0
use crate::{
    Align as XAlign, BoxSizing as XBoxSizing, Display as XDisplay, FlexDirection as XFlexDir,
    FlexWrap as XFlexWrap, JustifyContent as XJustify, Overflow as XOverflow,
    Position as XPosition, Style,
};
use taffy::prelude::*;
use taffy::style::Style as TaffyStyle;

fn dim<T>(l: crate::Length, scale_factor: f32) -> T
where
    T: taffy::style_helpers::FromLength + taffy::style_helpers::FromPercent,
{
    match l {
        crate::Length::Px(v) => length(v * scale_factor),
        crate::Length::Percent(v) => percent(v / 100.0),
        crate::Length::ViewportWidth(_) | crate::Length::ViewportHeight(_) => {
            length(l.to_physical(scale_factor))
        }
    }
}

/// Returns or updates the `style_to_taffy` value.
pub fn style_to_taffy(style: &Style, scale_factor: f32, has_children: bool) -> TaffyStyle {
    let mut t = TaffyStyle {
        display: match style.display.unwrap_or_default() {
            XDisplay::Flex => taffy::style::Display::Flex,
            XDisplay::Grid => taffy::style::Display::Grid,
            XDisplay::Block => taffy::style::Display::Block,
            XDisplay::None => taffy::style::Display::None,
        },
        position: match style.position.unwrap_or_default() {
            XPosition::Static | XPosition::Relative | XPosition::Sticky => {
                taffy::style::Position::Relative
            }
            XPosition::Absolute | XPosition::Fixed => taffy::style::Position::Absolute,
        },
        box_sizing: match style.box_sizing {
            XBoxSizing::BorderBox => taffy::style::BoxSizing::BorderBox,
            XBoxSizing::ContentBox => taffy::style::BoxSizing::ContentBox,
        },
        ..Default::default()
    };

    if let Some(ox) = style.overflow_x {
        t.overflow.x = map_overflow(ox);
    }
    if let Some(oy) = style.overflow_y {
        t.overflow.y = map_overflow(oy);
    }

    // Sticky stays in-flow and is clamped after layout instead (see
    // LayoutEngine), so it must not also receive taffy's own relative
    // offset shift the way Absolute/Fixed insets do.
    if matches!(
        style.position.unwrap_or_default(),
        XPosition::Absolute | XPosition::Fixed
    ) {
        if let Some(top) = style.top {
            t.inset.top = dim(top, scale_factor);
        }
        if let Some(right) = style.right {
            t.inset.right = dim(right, scale_factor);
        }
        if let Some(bottom) = style.bottom {
            t.inset.bottom = dim(bottom, scale_factor);
        }
        if let Some(left) = style.left {
            t.inset.left = dim(left, scale_factor);
        }
    }

    if let Some(dir) = style.flex_direction {
        t.flex_direction = match dir {
            XFlexDir::Row => taffy::style::FlexDirection::Row,
            XFlexDir::RowReverse => taffy::style::FlexDirection::RowReverse,
            XFlexDir::Column => taffy::style::FlexDirection::Column,
            XFlexDir::ColumnReverse => taffy::style::FlexDirection::ColumnReverse,
        };
    }

    if let Some(wrap) = style.flex_wrap {
        t.flex_wrap = match wrap {
            XFlexWrap::NoWrap => taffy::style::FlexWrap::NoWrap,
            XFlexWrap::Wrap => taffy::style::FlexWrap::Wrap,
            XFlexWrap::WrapReverse => taffy::style::FlexWrap::WrapReverse,
        };
    }

    if let Some(v) = style.flex_grow {
        t.flex_grow = v;
    }
    #[allow(unused_parens)]
    if let Some(v) = style.flex_shrink {
        t.flex_shrink = v;
    } else if style
        .min_size
        .is_some_and(|s| (s.width.is_some() || s.height.is_some()))
    {
        // Taffy's own default (flex-shrink:1, matching web CSS) still lets
        // the flex algorithm compress a content-sized item back down to its
        // min_size whenever available space is tight, making an explicit
        // min_size behave like a max instead of a floor. Only widgets that
        // opted into min_size get this override, so ordinary shrink-to-fit
        // layouts elsewhere are untouched.
        t.flex_shrink = 0.0;
    }
    if let Some(v) = style.flex_basis {
        t.flex_basis = dim(v, scale_factor);
    }

    if let Some(align) = style.align_items {
        t.align_items = Some(map_align(align));
    }
    if let Some(align) = style.align_self {
        t.align_self = Some(map_align(align));
    }
    if let Some(j) = style.justify_content {
        t.justify_content = Some(map_justify(j));
    }
    if let Some(j) = style.align_content {
        t.align_content = Some(map_justify(j));
    }

    if let Some((gx, gy)) = style.gap {
        t.gap = Size {
            width: dim(gx, scale_factor),
            height: dim(gy, scale_factor),
        };
    }

    if let Some(size) = &style.size {
        if let Some(w) = size.width {
            let px = dim(w, scale_factor);
            t.size.width = px;
            // A hard pixel width acts as a floor; a percentage width must
            // stay shrinkable, or a paired max-width can never clamp it
            // (min-width always wins over max-width in flex sizing).
            if style.min_size.and_then(|s| s.width).is_none() && matches!(w, crate::Length::Px(_)) {
                t.min_size.width = px;
                if style.flex_shrink.is_none() {
                    t.flex_shrink = 0.0;
                }
            }
        }
        if let Some(h) = size.height {
            let px = dim(h, scale_factor);
            t.size.height = px;
            if style.min_size.and_then(|s| s.height).is_none() && matches!(h, crate::Length::Px(_))
            {
                t.min_size.height = px;
                if style.flex_shrink.is_none() {
                    t.flex_shrink = 0.0;
                }
            }
        }
    }
    if let Some(size) = &style.min_size {
        if let Some(w) = size.width {
            // Same leaf-vs-parent split as min-height below: a leaf's
            // percentage min-width needs the real parent width (resolved
            // after layout in apply_layout), not taffy's own indeterminate
            // measurement pass, or it can resolve against the wrong space
            // and overflow past its actual container.
            if has_children || !matches!(w, crate::Length::Percent(_)) {
                t.min_size.width = dim(w, scale_factor);
            }
        }
        if let Some(h) = size.height {
            // Percentage min-height only resolves correctly through taffy's
            // own flex algorithm (it needs to interact with sibling
            // flex-grow/shrink and overflow), so it's forwarded here for
            // widgets that have children. Leaf widgets go through their own
            // intrinsic measurement pass instead (see layout_engine's
            // build_taffy_node), where resolving this here would lock their
            // auto width to whatever space is available instead of their
            // natural content size, so it's excluded there and resolved
            // manually afterwards.
            if has_children || !matches!(h, crate::Length::Percent(_)) {
                t.min_size.height = dim(h, scale_factor);
            }
        }
    }
    if let Some(size) = &style.max_size {
        if let Some(w) = size.width {
            t.max_size.width = dim(w, scale_factor);
        }
        if let Some(h) = size.height {
            t.max_size.height = dim(h, scale_factor);
        }
    }

    if let Some(p) = &style.padding {
        t.padding = Rect {
            left: dim(p.left, scale_factor),
            right: dim(p.right, scale_factor),
            top: dim(p.top, scale_factor),
            bottom: dim(p.bottom, scale_factor),
        };
    }

    if let Some(m) = &style.margin {
        t.margin = Rect {
            left: dim(m.left, scale_factor),
            right: dim(m.right, scale_factor),
            top: dim(m.top, scale_factor),
            bottom: dim(m.bottom, scale_factor),
        };
    }

    if let Some(b) = &style.border {
        t.border = Rect {
            left: dim(b.left, scale_factor),
            right: dim(b.right, scale_factor),
            top: dim(b.top, scale_factor),
            bottom: dim(b.bottom, scale_factor),
        };
    }

    if let Some(cols) = &style.grid_template_columns {
        t.grid_template_columns = cols
            .iter()
            .map(|track| map_grid_track(track, scale_factor))
            .collect();
    }
    if let Some(rows) = &style.grid_template_rows {
        t.grid_template_rows = rows
            .iter()
            .map(|track| map_grid_track(track, scale_factor))
            .collect();
    }
    if let Some(p) = style.grid_column {
        t.grid_column = Line {
            start: line(p.start),
            end: line(p.end),
        };
    }
    if let Some(p) = style.grid_row {
        t.grid_row = Line {
            start: line(p.start),
            end: line(p.end),
        };
    }

    t
}

fn map_align(align: XAlign) -> AlignItems {
    match align {
        XAlign::Stretch => AlignItems::STRETCH,
        XAlign::Start => AlignItems::START,
        XAlign::End => AlignItems::END,
        XAlign::Center => AlignItems::CENTER,
        XAlign::Baseline => AlignItems::BASELINE,
    }
}

fn map_justify(j: XJustify) -> JustifyContent {
    match j {
        XJustify::Start => JustifyContent::START,
        XJustify::End => JustifyContent::END,
        XJustify::Center => JustifyContent::CENTER,
        XJustify::SpaceBetween => JustifyContent::SPACE_BETWEEN,
        XJustify::SpaceAround => JustifyContent::SPACE_AROUND,
        XJustify::SpaceEvenly => JustifyContent::SPACE_EVENLY,
    }
}

fn map_grid_track(
    track: &crate::GridTrack,
    scale_factor: f32,
) -> taffy::style::GridTemplateComponent<String> {
    let sizing_function = match track {
        crate::GridTrack::Px(px) => length(*px * scale_factor),
        crate::GridTrack::Fr(f) => fr(*f),
        crate::GridTrack::Auto => auto(),
    };
    taffy::style::GridTemplateComponent::Single(sizing_function)
}

fn map_overflow(overflow: XOverflow) -> taffy::style::Overflow {
    match overflow {
        XOverflow::Visible => taffy::style::Overflow::Visible,
        XOverflow::Hidden => taffy::style::Overflow::Hidden,
        XOverflow::Scroll => taffy::style::Overflow::Scroll,
        XOverflow::Auto => taffy::style::Overflow::Scroll,
    }
}
