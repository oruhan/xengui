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
                            .background(|theme: &Theme|
                                theme.surface_container_lowest.with_alpha(200)
                            )
                            .box_shadow(
                                BoxShadow::new(
                                    0.0,
                                    4.0,
                                    12.0,
                                    Color::NEUTRAL_500.with_alpha(16)
                                ).direction(ShadowDirection::Bottom)
                            )
                            .border(|theme: &Theme|
                                Border::bottom(1, theme.outline.with_alpha(200))
                            )
                            .padding(Edges::symmetric(120, 0))
                            .child(
                                Button::new()
                                    .icon(
                                        include_str!(
                                            concat!(
                                                env!("CARGO_MANIFEST_DIR"),
                                                "/assets/XenGui_header.svg"
                                            )
                                        )
                                    )
                                    .icon_size(100.0, 100.0)
                                    .transition_all(
                                        Transition::new(Duration::from_millis(150)).easing(
                                            Easing::EaseInOut
                                        )
                                    )
                                    .hover_style(|ctx: StylePatch, _theme: &Theme|
                                        ctx.color(Color::BLUE_400)
                                    )
                                    .pressed_style(|ctx: StylePatch, _theme: &Theme|
                                        ctx.scale(0.96)
                                    )
                                    .on_click(|_ctx| xen_router::push("/"))
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
                                            .hover_style(|ctx, theme| ctx.background(theme.surface))
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
                                            .hover_style(|ctx, theme| ctx.background(theme.surface))
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
                                            .hover_style(|ctx, theme| ctx.background(theme.surface))
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
                                            .hover_style(|ctx, theme| ctx.background(theme.surface))
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
                                            .on_click(|_ctx| xen_router::push("/docs"))
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
                            .background(|theme: &Theme|
                                theme.surface_container_lowest.with_alpha(200)
                            )
                            .border(|theme: &Theme| Border::top(1, theme.outline.with_alpha(200)))
                            .padding(Edges::only(120, 26, 120, 18))
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
                                            .justify_content(JustifyContent::Start)
                                            .align_items(Align::Start)
                                            .gap(0, 12)
                                            .child(
                                                Button::new()
                                                    .icon(
                                                        include_str!(
                                                            concat!(
                                                                env!("CARGO_MANIFEST_DIR"),
                                                                "/assets/XenGui_header.svg"
                                                            )
                                                        )
                                                    )
                                                    .icon_size(100.0, 100.0)
                                                    .transition_all(
                                                        Transition::new(
                                                            Duration::from_millis(150)
                                                        ).easing(Easing::EaseInOut)
                                                    )
                                                    .hover_style(|ctx: StylePatch, _theme: &Theme|
                                                        ctx.color(Color::BLUE_400)
                                                    )
                                                    .pressed_style(|ctx: StylePatch, _theme: &Theme|
                                                        ctx.scale(0.96)
                                                    )
                                                    .on_click(|_ctx| xen_router::push("/"))
                                            )
                                            .child(
                                                Label::new()
                                                    .label(
                                                        "Build native desktop, web, mobile, and embedded applications from a single codebase."
                                                    )
                                                    .font_size(15)
                                                    .color(|theme: &Theme| theme.on_background)
                                                    .max_width(px!(320))
                                            )
                                    )

                                    .child(
                                        View::new()
                                            .display(Display::Flex)
                                            .flex_direction(FlexDirection::Row)
                                            .align_items(Align::Start)
                                            .justify_content(JustifyContent::End)
                                            .gap(64, 0)

                                            // XenGui
                                            .child(
                                                View::new()
                                                    .display(Display::Flex)
                                                    .flex_direction(FlexDirection::Column)
                                                    .align_items(Align::Start)
                                                    .justify_content(JustifyContent::Start)
                                                    .gap(0, 12)
                                                    .font_size(14)
                                                    .child(
                                                        Label::new()
                                                            .label("XenGui")
                                                            .font_weight(FontWeight::SemiBold)
                                                            .font_size(15)
                                                            .color(
                                                                |theme: &Theme| theme.on_background
                                                            )
                                                    )
                                                    .child(
                                                        View::new()
                                                            .gap(0, 8)
                                                            .display(Display::Flex)
                                                            .flex_direction(FlexDirection::Column)
                                                            .align_items(Align::Start)
                                                            .justify_content(JustifyContent::Start)
                                                            .child(
                                                                xen_router
                                                                    ::link("/about")
                                                                    .label("About")
                                                                    .transition_colors(
                                                                        Transition::new(
                                                                            Duration::from_millis(
                                                                                150
                                                                            )
                                                                        ).easing(Easing::EaseInOut)
                                                                    )
                                                                    .color(
                                                                        |theme: &Theme|
                                                                            theme.on_surface
                                                                    )
                                                                    .hover_style(
                                                                        |
                                                                            ctx: StylePatch,
                                                                            theme: &Theme
                                                                        |
                                                                            ctx.color(
                                                                                theme.on_background
                                                                            )
                                                                    )
                                                            )
                                                            .child(
                                                                xen_router
                                                                    ::link("/releases")
                                                                    .label("Releases")
                                                                    .transition_colors(
                                                                        Transition::new(
                                                                            Duration::from_millis(
                                                                                150
                                                                            )
                                                                        ).easing(Easing::EaseInOut)
                                                                    )
                                                                    .color(
                                                                        |theme: &Theme|
                                                                            theme.on_surface
                                                                    )
                                                                    .hover_style(
                                                                        |
                                                                            ctx: StylePatch,
                                                                            theme: &Theme
                                                                        |
                                                                            ctx.color(
                                                                                theme.on_background
                                                                            )
                                                                    )
                                                            )
                                                            .child(
                                                                xen_router
                                                                    ::link("/playground")
                                                                    .label("Playground")
                                                                    .transition_colors(
                                                                        Transition::new(
                                                                            Duration::from_millis(
                                                                                150
                                                                            )
                                                                        ).easing(Easing::EaseInOut)
                                                                    )
                                                                    .color(
                                                                        |theme: &Theme|
                                                                            theme.on_surface
                                                                    )
                                                                    .hover_style(
                                                                        |
                                                                            ctx: StylePatch,
                                                                            theme: &Theme
                                                                        |
                                                                            ctx.color(
                                                                                theme.on_background
                                                                            )
                                                                    )
                                                            )
                                                            .child(
                                                                xen_router
                                                                    ::link("/changelog")
                                                                    .label("Changelog")
                                                                    .transition_colors(
                                                                        Transition::new(
                                                                            Duration::from_millis(
                                                                                150
                                                                            )
                                                                        ).easing(Easing::EaseInOut)
                                                                    )
                                                                    .color(
                                                                        |theme: &Theme|
                                                                            theme.on_surface
                                                                    )
                                                                    .hover_style(
                                                                        |
                                                                            ctx: StylePatch,
                                                                            theme: &Theme
                                                                        |
                                                                            ctx.color(
                                                                                theme.on_background
                                                                            )
                                                                    )
                                                            )
                                                    )
                                            )

                                            // Documentation
                                            .child(
                                                View::new()
                                                    .display(Display::Flex)
                                                    .flex_direction(FlexDirection::Column)
                                                    .align_items(Align::Start)
                                                    .justify_content(JustifyContent::Start)
                                                    .gap(0, 12)
                                                    .font_size(14)
                                                    .child(
                                                        Label::new()
                                                            .label("Documentation")
                                                            .font_weight(FontWeight::SemiBold)
                                                            .font_size(15)
                                                            .color(
                                                                |theme: &Theme| theme.on_background
                                                            )
                                                    )
                                                    .child(
                                                        View::new()
                                                            .gap(0, 8)
                                                            .display(Display::Flex)
                                                            .flex_direction(FlexDirection::Column)
                                                            .align_items(Align::Start)
                                                            .justify_content(JustifyContent::Start)
                                                            .child(
                                                                xen_router
                                                                    ::link(
                                                                        "/docs/xengui#getting-started"
                                                                    )
                                                                    .label("Getting Started")
                                                                    .transition_colors(
                                                                        Transition::new(
                                                                            Duration::from_millis(
                                                                                150
                                                                            )
                                                                        ).easing(Easing::EaseInOut)
                                                                    )
                                                                    .color(
                                                                        |theme: &Theme|
                                                                            theme.on_surface
                                                                    )
                                                                    .hover_style(
                                                                        |
                                                                            ctx: StylePatch,
                                                                            theme: &Theme
                                                                        |
                                                                            ctx.color(
                                                                                theme.on_background
                                                                            )
                                                                    )
                                                            )
                                                            .child(
                                                                xen_router
                                                                    ::link("/docs/xengui/guides")
                                                                    .label("Guides")
                                                                    .transition_colors(
                                                                        Transition::new(
                                                                            Duration::from_millis(
                                                                                150
                                                                            )
                                                                        ).easing(Easing::EaseInOut)
                                                                    )
                                                                    .color(
                                                                        |theme: &Theme|
                                                                            theme.on_surface
                                                                    )
                                                                    .hover_style(
                                                                        |
                                                                            ctx: StylePatch,
                                                                            theme: &Theme
                                                                        |
                                                                            ctx.color(
                                                                                theme.on_background
                                                                            )
                                                                    )
                                                            )
                                                            .child(
                                                                xen_router
                                                                    ::link("/examples")
                                                                    .label("Examples")
                                                                    .transition_colors(
                                                                        Transition::new(
                                                                            Duration::from_millis(
                                                                                150
                                                                            )
                                                                        ).easing(Easing::EaseInOut)
                                                                    )
                                                                    .color(
                                                                        |theme: &Theme|
                                                                            theme.on_surface
                                                                    )
                                                                    .hover_style(
                                                                        |
                                                                            ctx: StylePatch,
                                                                            theme: &Theme
                                                                        |
                                                                            ctx.color(
                                                                                theme.on_background
                                                                            )
                                                                    )
                                                            )
                                                    )
                                            )

                                            // Resources
                                            .child(
                                                View::new()
                                                    .display(Display::Flex)
                                                    .flex_direction(FlexDirection::Column)
                                                    .align_items(Align::Start)
                                                    .justify_content(JustifyContent::Start)
                                                    .gap(0, 12)
                                                    .font_size(14)
                                                    .child(
                                                        Label::new()
                                                            .label("Resources")
                                                            .font_weight(FontWeight::SemiBold)
                                                            .font_size(15)
                                                            .color(
                                                                |theme: &Theme| theme.on_background
                                                            )
                                                    )
                                                    .child(
                                                        View::new()
                                                            .gap(0, 8)
                                                            .display(Display::Flex)
                                                            .flex_direction(FlexDirection::Column)
                                                            .align_items(Align::Start)
                                                            .justify_content(JustifyContent::Start)
                                                            .child(
                                                                xen_router
                                                                    ::link("/license")
                                                                    .label("License")
                                                                    .transition_colors(
                                                                        Transition::new(
                                                                            Duration::from_millis(
                                                                                150
                                                                            )
                                                                        ).easing(Easing::EaseInOut)
                                                                    )
                                                                    .color(
                                                                        |theme: &Theme|
                                                                            theme.on_surface
                                                                    )
                                                                    .hover_style(
                                                                        |
                                                                            ctx: StylePatch,
                                                                            theme: &Theme
                                                                        |
                                                                            ctx.color(
                                                                                theme.on_background
                                                                            )
                                                                    )
                                                            )
                                                            .child(
                                                                xen_router
                                                                    ::link("/security")
                                                                    .label("Security")
                                                                    .transition_colors(
                                                                        Transition::new(
                                                                            Duration::from_millis(
                                                                                150
                                                                            )
                                                                        ).easing(Easing::EaseInOut)
                                                                    )
                                                                    .color(
                                                                        |theme: &Theme|
                                                                            theme.on_surface
                                                                    )
                                                                    .hover_style(
                                                                        |
                                                                            ctx: StylePatch,
                                                                            theme: &Theme
                                                                        |
                                                                            ctx.color(
                                                                                theme.on_background
                                                                            )
                                                                    )
                                                            )
                                                    )
                                            )

                                            // Ecosystem
                                            .child(
                                                View::new()
                                                    .display(Display::Flex)
                                                    .flex_direction(FlexDirection::Column)
                                                    .align_items(Align::Start)
                                                    .justify_content(JustifyContent::End)
                                                    .gap(0, 8)
                                                    .font_size(14)
                                                    .child(
                                                        Label::new()
                                                            .label("Ecosystem")
                                                            .font_size(15)
                                                            .font_weight(FontWeight::SemiBold)
                                                            .color(
                                                                |theme: &Theme| theme.on_background
                                                            )
                                                    )
                                                    .child(
                                                        Link::new()
                                                            .label("xengui")
                                                            .href("https://crates.io/crates/xengui")
                                                            .target_blank(true)
                                                            .color(|theme: &Theme| theme.on_surface)
                                                    )
                                                    .child(
                                                        Link::new()
                                                            .label("xenframe")
                                                            .href(
                                                                "https://crates.io/crates/xenframe"
                                                            )
                                                            .target_blank(true)
                                                            .color(|theme: &Theme| theme.on_surface)
                                                    )
                                                    .child(
                                                        Link::new()
                                                            .label("xengui-wgpu")
                                                            .href(
                                                                "https://crates.io/crates/xengui-wgpu"
                                                            )
                                                            .target_blank(true)
                                                            .color(|theme: &Theme| theme.on_surface)
                                                    )
                                                    .child(
                                                        Link::new()
                                                            .label("xengui-icons")
                                                            .href(
                                                                "https://crates.io/crates/xengui-icons"
                                                            )
                                                            .target_blank(true)
                                                            .color(|theme: &Theme| theme.on_surface)
                                                    )
                                                    .child(
                                                        Link::new()
                                                            .label("xen-svg")
                                                            .href(
                                                                "https://crates.io/crates/xen-svg"
                                                            )
                                                            .target_blank(true)
                                                            .color(|theme: &Theme| theme.on_surface)
                                                    )
                                                    .child(
                                                        Link::new()
                                                            .label("xen-router")
                                                            .href(
                                                                "https://crates.io/crates/xen-router"
                                                            )
                                                            .target_blank(true)
                                                            .color(|theme: &Theme| theme.on_surface)
                                                    )
                                                    .child(
                                                        Link::new()
                                                            .label("xen-animation")
                                                            .href(
                                                                "https://crates.io/crates/xen-animation"
                                                            )
                                                            .target_blank(true)
                                                            .color(|theme: &Theme| theme.on_surface)
                                                    )
                                                    .child(
                                                        Link::new()
                                                            .label("xen-clipboard")
                                                            .href(
                                                                "https://crates.io/crates/xen-clipboard"
                                                            )
                                                            .target_blank(true)
                                                            .color(|theme: &Theme| theme.on_surface)
                                                    )
                                            )

                                            // Community
                                            .child(
                                                View::new()
                                                    .display(Display::Flex)
                                                    .flex_direction(FlexDirection::Column)
                                                    .align_items(Align::Start)
                                                    .justify_content(JustifyContent::End)
                                                    .gap(0, 8)
                                                    .font_size(14)
                                                    .child(
                                                        Label::new()
                                                            .label("Community")
                                                            .font_size(15)
                                                            .font_weight(FontWeight::SemiBold)
                                                            .color(
                                                                |theme: &Theme| theme.on_background
                                                            )
                                                    )
                                                    .child(
                                                        Link::new()
                                                            .label("GitHub")
                                                            .href(
                                                                "https://github.com/randseas/xengui"
                                                            )
                                                            .target_blank(true)
                                                            .color(|theme: &Theme| theme.on_surface)
                                                    )
                                                    .child(
                                                        Link::new()
                                                            .label("Discord")
                                                            .href("https://discord.com/invite/")
                                                            .target_blank(true)
                                                            .color(|theme: &Theme| theme.on_surface)
                                                    )
                                                    .child(
                                                        Link::new()
                                                            .label("Contributing")
                                                            .href(
                                                                "https://github.com/randseas/xengui/CONTRIBUTING.md"
                                                            )
                                                            .target_blank(true)
                                                            .color(|theme: &Theme| theme.on_surface)
                                                    )
                                            )
                                    )
                            )
                            // Divider
                            .child(
                                View::new()
                                    .height(1)
                                    .background(|theme: &Theme| theme.outline.with_alpha(200))
                                    .margin(Edges::only(0, 24, 0, 16))
                            )
                            // Bottom
                            .child(
                                View::new()
                                    .display(Display::Flex)
                                    .justify_content(JustifyContent::SpaceBetween)
                                    .align_items(Align::Center)
                                    .child(
                                        Label::new()
                                            .label("© 2026 xengui - Apache License 2.0")
                                            .font_size(14)
                                            .color(|theme: &Theme| theme.on_background)
                                    )
                                    .child(View::new().display(Display::Flex))
                            )
                    ) as Box<dyn Widget>
                ]
            )
    )
}
