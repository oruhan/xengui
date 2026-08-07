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
            .height(pct!(100.0))
            .children_vec(
                vec![
                    Box::new(
                        /* Header */
                        View::new()
                            .top(0)
                            .position(Position::Fixed)
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
                            .background(|theme: &Theme| theme.surface)
                            .border(|theme: &Theme| Border::top(1, theme.border))
                            .padding(Edges::only(120, 48, 120, 32))
                            .gap(0, 32)

                            // Top
                            .child(
                                View::new()
                                    .display(Display::Flex)
                                    .justify_content(JustifyContent::SpaceBetween)
                                    .align_items(Align::Start)

                                    // Left
                                    .child(
                                        View::new()
                                            .display(Display::Flex)
                                            .flex_direction(FlexDirection::Column)
                                            .gap(0, 12)

                                            .child(
                                                Svg::from_bytes(
                                                    include_bytes!(
                                                        concat!(
                                                            env!("CARGO_MANIFEST_DIR"),
                                                            "/assets/XenGui_header.svg"
                                                        )
                                                    )
                                                )
                                                    .width(110)
                                                    .height(110)
                                                    .background(Color::TRANSPARENT)
                                            )

                                            .child(
                                                Label::new()
                                                    .label(
                                                        "Modern Rust UI framework for embedded, desktop and web."
                                                    )
                                                    .font_size(15)
                                                    .color(Color::NEUTRAL_400)
                                                    .max_width(px!(320))
                                            )
                                    )

                                    // Resources
                                    .child(
                                        View::new()
                                            .display(Display::Flex)
                                            .flex_direction(FlexDirection::Column)
                                            .gap(0, 10)

                                            .child(
                                                Label::new()
                                                    .label("Resources")
                                                    .font_weight(FontWeight::SemiBold)
                                            )

                                            .child(xen_router::link("/docs").label("Docs"))
                                            .child(xen_router::link("/examples").label("Examples"))
                                            .child(
                                                xen_router::link("/playground").label("Playground")
                                            )
                                    )

                                    // Community
                                    .child(
                                        View::new()
                                            .display(Display::Flex)
                                            .flex_direction(FlexDirection::Column)
                                            .gap(0, 10)

                                            .child(
                                                Label::new()
                                                    .label("Community")
                                                    .font_weight(FontWeight::SemiBold)
                                            )

                                            .child(
                                                Link::new()
                                                    .label("GitHub")
                                                    .href("https://github.com/randseas/xengui")
                                                    .target_blank(true)
                                            )

                                            .child(
                                                Link::new()
                                                    .label("Crates.io")
                                                    .href("https://crates.io/crates/xengui")
                                                    .target_blank(true)
                                            )

                                            .child(Link::new().label("Discord").href("#"))
                                    )
                            )

                            // Divider
                            .child(View::new().height(1).background(Color::NEUTRAL_800))

                            // Bottom
                            .child(
                                View::new()
                                    .display(Display::Flex)
                                    .justify_content(JustifyContent::SpaceBetween)
                                    .align_items(Align::Center)

                                    .child(
                                        Label::new()
                                            .label("© 2026 Xengui • Apache-2.0")
                                            .font_size(14)
                                            .color(Color::NEUTRAL_500)
                                    )

                                    .child(
                                        View::new()
                                            .display(Display::Flex)
                                            .gap(8, 0)

                                            .child(
                                                Label::new()
                                                    .label("Rust")
                                                    .padding(Edges::only(10, 5, 10, 5))
                                                    .background(Color::NEUTRAL_900)
                                                    .border(
                                                        Border::all(1, Color::NEUTRAL_800).radius(
                                                            999
                                                        )
                                                    )
                                            )

                                            .child(
                                                Label::new()
                                                    .label("wgpu")
                                                    .padding(Edges::only(10, 5, 10, 5))
                                                    .background(Color::NEUTRAL_900)
                                                    .border(
                                                        Border::all(1, Color::NEUTRAL_800).radius(
                                                            999
                                                        )
                                                    )
                                            )

                                            .child(
                                                Label::new()
                                                    .label("Taffy")
                                                    .padding(Edges::only(10, 5, 10, 5))
                                                    .background(Color::NEUTRAL_900)
                                                    .border(
                                                        Border::all(1, Color::NEUTRAL_800).radius(
                                                            999
                                                        )
                                                    )
                                            )
                                    )
                            )
                    ) as Box<dyn Widget>
                ]
            )
    )
}
