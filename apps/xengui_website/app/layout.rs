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
                            /*.backdrop_filter(Filter::Blur(Length::px(8.0)))*/
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
                            .padding(
                                Responsive::new(Edges::symmetric(16, 0)).md(
                                    Edges::symmetric(120, 0)
                                )
                            )
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
                                    .display(
                                        if responsive_bool(Breakpoint::Md, true) {
                                            Display::Flex
                                        } else {
                                            Display::None
                                        }
                                    )
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
                    footer() as Box<dyn Widget>
                ]
            )
    )
}

fn footer_link(path: &str, label: &str) -> Button {
    xen_router
        ::link(path)
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
                .color(|theme: &Theme| theme.on_background)
        )
        .child(list)
}

fn footer_ecosystem_column() -> View {
    View::new()
        .display(Display::Flex)
        .flex_direction(FlexDirection::Column)
        .align_items(Align::Start)
        .gap(0, 8)
        .font_size(14)
        .child(
            Label::new()
                .label("Ekosistem")
                .font_size(15)
                .font_weight(FontWeight::SemiBold)
                .color(|theme: &Theme| theme.on_background)
        )
        .child(external_link("xengui", "https://crates.io/crates/xengui"))
        .child(external_link("xenframe", "https://crates.io/crates/xenframe"))
        .child(external_link("xengui-wgpu", "https://crates.io/crates/xengui-wgpu"))
        .child(external_link("xengui-icons", "https://crates.io/crates/xengui-icons"))
        .child(external_link("xen-svg", "https://crates.io/crates/xen-svg"))
        .child(external_link("xen-router", "https://crates.io/crates/xen-router"))
        .child(external_link("xen-animation", "https://crates.io/crates/xen-animation"))
        .child(external_link("xen-clipboard", "https://crates.io/crates/xen-clipboard"))
}

fn footer_community_column() -> View {
    View::new()
        .display(Display::Flex)
        .flex_direction(FlexDirection::Column)
        .align_items(Align::Start)
        .gap(0, 8)
        .font_size(14)
        .child(
            Label::new()
                .label("Topluluk")
                .font_size(15)
                .font_weight(FontWeight::SemiBold)
                .color(|theme: &Theme| theme.on_background)
        )
        .child(external_link("GitHub", "https://github.com/randseas/xengui"))
        .child(external_link("Discord", "https://discord.com/invite/"))
        .child(
            external_link("Katkıda Bulunma", "https://github.com/randseas/xengui/CONTRIBUTING.md")
        )
}

fn footer() -> Box<View> {
    let stacked = !responsive_bool(Breakpoint::Lg, true);

    let brand = View::new()
        .display(Display::Flex)
        .flex_direction(FlexDirection::Column)
        .justify_content(JustifyContent::Start)
        .align_items(Align::Start)
        .gap(0, 12)
        .child(
            Button::new()
                .icon(
                    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/XenGui_header.svg"))
                )
                .icon_size(100.0, 100.0)
                .transition_all(
                    Transition::new(Duration::from_millis(150)).easing(Easing::EaseInOut)
                )
                .hover_style(|ctx: StylePatch, _theme: &Theme| ctx.color(Color::BLUE_400))
                .pressed_style(|ctx: StylePatch, _theme: &Theme| ctx.scale(0.96))
                .on_click(|_ctx| xen_router::push("/"))
        )
        .child(
            Label::new()
                .label("Rust için tek kod tabanıyla masaüstü, web ve gömülü uygulamalar.")
                .font_size(15)
                .color(|theme: &Theme| theme.on_background)
                .max_width(px!(320))
        );

    let columns = View::new()
        .display(Display::Flex)
        .flex_direction(FlexDirection::Row)
        .flex_wrap(FlexWrap::Wrap)
        .align_items(Align::Start)
        .justify_content(if stacked { JustifyContent::Start } else { JustifyContent::End })
        .gap(Responsive::new(px!(32.0)).lg(px!(64.0)), px!(32.0))
        .child(
            footer_column(
                "XenGui",
                &[
                    ("/about", "Hakkında"),
                    ("/releases", "Sürümler"),
                    ("/playground", "Playground"),
                    ("/changelog", "Değişiklik Günlüğü"),
                ]
            )
        )
        .child(
            footer_column(
                "Dokümantasyon",
                &[
                    ("/docs/xengui#getting-started", "Başlarken"),
                    ("/docs/xengui/guides", "Rehberler"),
                    ("/examples", "Örnekler"),
                ]
            )
        )
        .child(
            footer_column(
                "Kaynaklar",
                &[
                    ("/license", "Lisans"),
                    ("/security", "Güvenlik"),
                ]
            )
        )
        .child(footer_ecosystem_column())
        .child(footer_community_column());

    let top = View::new()
        .display(Display::Flex)
        .flex_direction(if stacked { FlexDirection::Column } else { FlexDirection::Row })
        .justify_content(JustifyContent::SpaceBetween)
        .align_items(Align::Start)
        .gap(px!(24), px!(32))
        .child(brand)
        .child(columns);

    let bottom = View::new()
        .display(Display::Flex)
        .flex_direction(if stacked { FlexDirection::Column } else { FlexDirection::Row })
        .justify_content(JustifyContent::SpaceBetween)
        .align_items(if stacked { Align::Start } else { Align::Center })
        .gap(px!(8), px!(8))
        .child(
            Label::new()
                .label("© 2026 XenGui — Apache License 2.0")
                .font_size(14)
                .color(|theme: &Theme| theme.on_background)
        )
        .child(
            Label::new()
                .label("Rust ile yapıldı")
                .font_size(13)
                .color(|theme: &Theme| theme.on_surface_variant)
        );

    Box::new(
        View::new()
            .display(Display::Flex)
            .flex_direction(FlexDirection::Column)
            .background(|theme: &Theme| theme.surface_container_lowest.with_alpha(200))
            .border(|theme: &Theme| Border::top(1, theme.outline.with_alpha(200)))
            .padding(Responsive::new(Edges::symmetric(24, 26)).md(Edges::only(120, 26, 120, 18)))
            .gap(0, 24)
            .child(top)
            .child(
                View::new()
                    .height(1)
                    .background(|theme: &Theme| theme.outline.with_alpha(200))
            )
            .child(bottom)
    )
}
