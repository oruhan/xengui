use std::time::Duration;
use xen_router::RouteParams;
use xengui::*;

pub fn layout(_params: &RouteParams, child: Box<dyn Widget>) -> Box<dyn Widget> {
    Box::new(
        /* Main */
        View::new()
            .font("Inter")
            .display(Display::Flex)
            .flex_direction(FlexDirection::Column)
            .gap(0, 4)
            .background(|theme: &Theme| theme.background)
            .overflow_y(Overflow::Scroll)
            .scrollbar_gutter(ScrollbarGutter::Stable)
            .overscroll(Overscroll::Stretch)
            .children_vec(
                vec![
                    Box::new(
                        /* Header */
                        View::new()
                            .top(0)
                            .position(Position::Sticky)
                            .z_index(10)
                            .display(Display::Flex)
                            .flex_direction(FlexDirection::Row)
                            .align_items(Align::Center)
                            .justify_content(JustifyContent::SpaceBetween)
                            .width(pct!(100))
                            .height(px!(55))
                            .backdrop_filter(Filter::Blur(Length::px(8.0)))
                            .background(|theme: &Theme| theme.surface.with_alpha(200))
                            .box_shadow(
                                BoxShadow::new(
                                    0.0,
                                    4.0,
                                    12.0,
                                    Color::NEUTRAL_500.with_alpha(16)
                                ).direction(ShadowDirection::Bottom)
                            )
                            .border(|theme: &Theme| Border::bottom(1, theme.border))
                            .padding(Edges::symmetric(120, 0))
                            .child(
                                Svg::from_bytes(
                                    include_bytes!(
                                        concat!(
                                            env!("CARGO_MANIFEST_DIR"),
                                            "/assets/XenGui_header.svg"
                                        )
                                    )
                                )
                                    .width(100)
                                    .height(100)
                                    .background(Color::TRANSPARENT)
                                    .on_click(|_f| xen_router::push("/"))
                            )
                            .child(
                                View::new()
                                    .display(Display::Flex)
                                    .flex_direction(FlexDirection::Row)
                                    .background(Color::TRANSPARENT)
                                    .gap(8, 0)
                                    .child(
                                        xen_router
                                            ::link("/docs")
                                            .label("Docs")
                                            .background(Color::TRANSPARENT)
                                            .transition_all(
                                                Transition::new(Duration::from_millis(200)).easing(
                                                    Easing::EaseInOut
                                                )
                                            )
                                            .padding(Edges::only(10, 6, 10, 6))
                                            .border(
                                                Border::new()
                                                    .width(1)
                                                    .color(Color::NEUTRAL_800)
                                                    .radius(64)
                                            )
                                            .hover_style(|ctx, theme| ctx.background(theme.hover))
                                    )
                                    .child(
                                        xen_router
                                            ::link("/examples")
                                            .label("Examples")
                                            .background(Color::TRANSPARENT)
                                            .transition_all(
                                                Transition::new(Duration::from_millis(200)).easing(
                                                    Easing::EaseInOut
                                                )
                                            )
                                            .padding(Edges::only(10, 6, 10, 6))
                                            .border(
                                                Border::new()
                                                    .width(1)
                                                    .color(Color::NEUTRAL_800)
                                                    .radius(64)
                                            )
                                            .hover_style(|ctx, theme| ctx.background(theme.hover))
                                    )
                                    .child(
                                        xen_router
                                            ::link("/playground")
                                            .label("Playground")
                                            .background(Color::TRANSPARENT)
                                            .transition_all(
                                                Transition::new(Duration::from_millis(200)).easing(
                                                    Easing::EaseInOut
                                                )
                                            )
                                            .padding(Edges::only(10, 6, 10, 6))
                                            .border(
                                                Border::new()
                                                    .width(1)
                                                    .color(Color::NEUTRAL_800)
                                                    .radius(64)
                                            )
                                            .hover_style(|ctx, theme| ctx.background(theme.hover))
                                    )
                                    .child(
                                        Link::new()
                                            .href("https://github.com/randseas/xengui")
                                            .target_blank(true)
                                            .label("GitHub")
                                            .background(Color::TRANSPARENT)
                                            .transition_all(
                                                Transition::new(Duration::from_millis(200)).easing(
                                                    Easing::EaseInOut
                                                )
                                            )
                                            .padding(Edges::only(10, 6, 10, 6))
                                            .border(
                                                Border::new()
                                                    .width(1)
                                                    .color(Color::NEUTRAL_800)
                                                    .radius(64)
                                            )
                                            .hover_style(|ctx, theme| ctx.background(theme.hover))
                                    )
                            )
                            .child(
                                View::new()
                                    .display(Display::Flex)
                                    .flex_direction(FlexDirection::Row)
                                    .gap(4, 0)
                                    .child(
                                        Button::new()
                                            .background(Color::BLUE_500)
                                            .font_size(14)
                                            .font_weight(FontWeight::Medium)
                                            .transition_all(
                                                Transition::new(Duration::from_millis(200)).easing(
                                                    Easing::EaseInOut
                                                )
                                            )
                                            .border(Border::all(1, Color::BLUE_400).radius(64))
                                            .hover_style(|ctx: StylePatch, _theme: &Theme|
                                                ctx
                                                    .background(Color::BLUE_600)
                                                    .border(
                                                        Border::all(1, Color::BLUE_500).radius(64)
                                                    )
                                            )
                                            .pressed_style(|ctx: StylePatch, _theme: &Theme|
                                                ctx
                                                    .background(Color::BLUE_700)
                                                    .border(
                                                        Border::all(1, Color::BLUE_600).radius(64)
                                                    )
                                                    .scale(0.97)
                                            )
                                            .padding(Edges::only(14, 7, 14, 7))
                                            .label("Get started")
                                    )
                            )
                    ) as Box<dyn Widget>,
                    child,
                    Box::new(
                        View::new()
                            .display(Display::Flex)
                            .flex_direction(FlexDirection::Column)
                            .align_items(Align::Start)
                            .justify_content(JustifyContent::Center)
                            .padding(Edges::symmetric(120, 0))
                            .child(Label::new().label("Footer"))
                    ) as Box<dyn Widget>
                ]
            )
    )
}
