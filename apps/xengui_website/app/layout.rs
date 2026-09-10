use std::time::Duration;
use xen_router::RouteParams;
use xengui::*;

pub fn layout(_params: &RouteParams, child: Box<dyn Widget>) -> Box<dyn Widget> {
    let desktop = responsive_bool(Breakpoint::Expanded, true);

    Box::new(
        View::new()
            .font("Inter")
            .display(Display::Flex)
            .flex_direction(FlexDirection::Column)
            .gap(0, 0)
            .background(|theme: &Theme| theme.background)
            .overflow_y(Overflow::Scroll)
            .scrollbar_gutter(ScrollbarGutter::Stable)
            .height(pct!(100.0))
            .child(
                View::new()
                    .top(0)
                    .position(Position::Fixed)
                    .z_index(10)
                    .display(Display::Flex)
                    .flex_direction(FlexDirection::Row)
                    .align_items(Align::Center)
                    .justify_content(JustifyContent::SpaceBetween)
                    .width(pct!(100.0))
                    .height(px!(64.0))
                    .backdrop_filter(Filter::Blur(px!(18.0)))
                    .background(|theme: &Theme| theme.background.with_alpha(232))
                    .border(|theme: &Theme| Border::bottom(1.0, theme.outline_variant))
                    .padding(
                        Responsive::new(Edges::symmetric(20.0, 0.0))
                            .md(Edges::symmetric(64.0, 0.0))
                            .lg(Edges::symmetric(120.0, 0.0)),
                    )
                    .child(
                        Button::new()
                            .icon(include_str!(concat!(
                                env!("CARGO_MANIFEST_DIR"),
                                "/assets/XenGui_header.svg"
                            )))
                            .icon_size(if desktop { 104.0 } else { 92.0 }, 32.0)
                            .background(Color::TRANSPARENT)
                            .padding(Edges::all(0.0))
                            .pressed_style(|style: StylePatch, _theme: &Theme| style.scale(0.97))
                            .on_click(|_ctx| xen_router::push("/")),
                    )
                    .child(
                        View::new()
                            .display(if desktop {
                                Display::Flex
                            } else {
                                Display::None
                            })
                            .flex_direction(FlexDirection::Row)
                            .align_items(Align::Center)
                            .gap(4.0, 0.0)
                            .child(nav_link("/docs", "Dokümantasyon"))
                            .child(nav_link("/examples", "Örnekler"))
                            .child(nav_link("/playground", "Playground"))
                            .child(external_nav_link(
                                "GitHub",
                                "https://github.com/randseas/xengui",
                            )),
                    )
                    .child(
                        Button::new()
                            .label(if desktop {
                                "Başlangıç rehberi"
                            } else {
                                "Başla"
                            })
                            .font_size(13.0)
                            .font_weight(FontWeight::SemiBold)
                            .background(|theme: &Theme| theme.on_background)
                            .color(|theme: &Theme| theme.background)
                            .border(Border::all(0.0, Color::TRANSPARENT).radius(10.0))
                            .padding(Edges::symmetric(if desktop { 16.0 } else { 13.0 }, 9.0))
                            .transition_all(
                                Transition::new(Duration::from_millis(140)).easing(Easing::EaseOut),
                            )
                            .pressed_style(|style: StylePatch, _theme: &Theme| style.scale(0.97))
                            .on_click(|_ctx| xen_router::push("/docs")),
                    ),
            )
            .child(View::new().height(px!(64.0)).flex_shrink(0.0))
            .child_boxed(child)
            .child(*footer()),
    )
}

fn nav_link(path: &str, label: &str) -> Button {
    xen_router::link(path)
        .label(label)
        .font_size(13.0)
        .font_weight(FontWeight::Medium)
        .color(|theme: &Theme| theme.on_surface_variant)
        .background(Color::TRANSPARENT)
        .padding(Edges::symmetric(12.0, 8.0))
        .border(Border::all(0.0, Color::TRANSPARENT).radius(8.0))
        .transition_colors(Transition::new(Duration::from_millis(140)).easing(Easing::EaseOut))
        .hover_style(|style: StylePatch, theme: &Theme| {
            style
                .color(theme.on_background)
                .background(theme.surface_container)
        })
}

fn external_nav_link(label: &str, href: &str) -> Link {
    Link::new()
        .label(label)
        .href(href)
        .target_blank(true)
        .font_size(13.0)
        .font_weight(FontWeight::Medium)
        .color(|theme: &Theme| theme.on_surface_variant)
        .background(Color::TRANSPARENT)
        .padding(Edges::symmetric(12.0, 8.0))
}

fn footer_link(path: &str, label: &str) -> Button {
    xen_router::link(path)
        .label(label)
        .transition_colors(Transition::new(Duration::from_millis(150)).easing(Easing::EaseInOut))
        .color(|theme: &Theme| theme.on_surface)
        .hover_style(|ctx: StylePatch, theme: &Theme| ctx.color(theme.on_background))
}

fn external_link(label: &str, href: &str) -> Link {
    Link::new()
        .label(label)
        .href(href)
        .target_blank(true)
        .color(|theme: &Theme| theme.on_surface)
}

fn footer_column(title: &str, links: &[(&str, &str)]) -> View {
    let mut list = View::new()
        .gap(0, 8)
        .display(Display::Flex)
        .flex_direction(FlexDirection::Column)
        .align_items(Align::Start);

    for (path, label) in links {
        list = list.child(footer_link(path, label));
    }

    View::new()
        .display(Display::Flex)
        .flex_direction(FlexDirection::Column)
        .align_items(Align::Start)
        .gap(0, 12)
        .font_size(14)
        .child(
            Label::new()
                .label(title)
                .font_weight(FontWeight::SemiBold)
                .font_size(15)
                .color(|theme: &Theme| theme.on_background),
        )
        .child(list)
}

fn footer() -> Box<View> {
    let stacked = !responsive_bool(Breakpoint::Large, true);

    let brand = View::new()
        .display(Display::Flex)
        .flex_direction(FlexDirection::Column)
        .justify_content(JustifyContent::Start)
        .align_items(Align::Start)
        .gap(0, 12)
        .child(
            Button::new()
                .icon(include_str!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/assets/XenGui_header.svg"
                )))
                .icon_size(100.0, 100.0)
                .transition_all(
                    Transition::new(Duration::from_millis(150)).easing(Easing::EaseInOut),
                )
                .hover_style(|ctx: StylePatch, _theme: &Theme| ctx.color(Color::BLUE_400))
                .pressed_style(|ctx: StylePatch, _theme: &Theme| ctx.scale(0.96))
                .on_click(|_ctx| xen_router::push("/")),
        )
        .child(
            Label::new()
                .label("Rust ile masaüstü ve web için aynı arayüz modeli.")
                .font_size(14)
                .line_height(px!(22.0))
                .color(|theme: &Theme| theme.on_surface_variant)
                .max_width(px!(300)),
        );

    let columns = View::new()
        .display(Display::Flex)
        .flex_direction(FlexDirection::Row)
        .flex_wrap(FlexWrap::Wrap)
        .align_items(Align::Start)
        .justify_content(if stacked {
            JustifyContent::Start
        } else {
            JustifyContent::End
        })
        .gap(Responsive::new(px!(32.0)).lg(px!(56.0)), px!(28.0))
        .child(footer_column(
            "Ürün",
            &[
                ("/docs", "Dokümantasyon"),
                ("/examples", "Örnekler"),
                ("/playground", "Playground"),
            ],
        ))
        .child(footer_column(
            "Keşfet",
            &[
                ("/showcase", "Canlı uygulama"),
                ("/docs", "Başlangıç rehberi"),
            ],
        ))
        .child(
            View::new()
                .display(Display::Flex)
                .flex_direction(FlexDirection::Column)
                .align_items(Align::Start)
                .gap(0.0, 8.0)
                .font_size(14.0)
                .child(
                    Label::new()
                        .label("Kaynak")
                        .font_weight(FontWeight::SemiBold)
                        .font_size(15.0)
                        .color(|theme: &Theme| theme.on_background),
                )
                .child(external_link(
                    "GitHub",
                    "https://github.com/randseas/xengui",
                ))
                .child(external_link(
                    "crates.io",
                    "https://crates.io/crates/xengui",
                )),
        );

    let top = View::new()
        .display(Display::Flex)
        .flex_direction(if stacked {
            FlexDirection::Column
        } else {
            FlexDirection::Row
        })
        .justify_content(JustifyContent::SpaceBetween)
        .align_items(Align::Start)
        .gap(px!(40.0), px!(32.0))
        .child(brand)
        .child(columns);

    let bottom = View::new()
        .display(Display::Flex)
        .flex_direction(if stacked {
            FlexDirection::Column
        } else {
            FlexDirection::Row
        })
        .justify_content(JustifyContent::SpaceBetween)
        .align_items(if stacked { Align::Start } else { Align::Center })
        .gap(px!(8), px!(8))
        .child(
            Label::new()
                .label("© 2026 XenGui · Apache 2.0")
                .font_size(13)
                .color(|theme: &Theme| theme.on_surface_variant),
        )
        .child(
            Label::new()
                .label("Rust · wgpu · WebGPU")
                .font_size(13)
                .color(|theme: &Theme| theme.on_surface_variant),
        );

    Box::new(
        View::new()
            .display(Display::Flex)
            .flex_direction(FlexDirection::Column)
            .background(|theme: &Theme| theme.surface_container_lowest)
            .border(|theme: &Theme| Border::top(1, theme.outline_variant))
            .padding(
                Responsive::new(Edges::only(20.0, 40.0, 20.0, 24.0))
                    .md(Edges::only(64.0, 56.0, 64.0, 28.0))
                    .lg(Edges::only(120.0, 64.0, 120.0, 28.0)),
            )
            .gap(0, 28)
            .child(top)
            .child(
                View::new()
                    .height(1)
                    .background(|theme: &Theme| theme.outline.with_alpha(200)),
            )
            .child(bottom),
    )
}
