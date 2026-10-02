use crate::site_tokens::{fast_effect, fast_spatial, radius, space, type_scale};
use xen_router::RouteParams;
use xengui::*;

fn focus_ring(style: StylePatch, theme: &Theme) -> StylePatch {
    style.border(Border::all(2.0, theme.primary).radius(radius::FULL))
}

fn nav_link(path: &'static str, label: &'static str) -> Button {
    let selected = xen_router::current_path() == path;
    xen_router::link(path)
        .label(label)
        .font_size(type_scale::LABEL_LG)
        .font_weight(if selected { FontWeight::Bold } else { FontWeight::Medium })
        .color(move |theme: &Theme| if selected { theme.on_surface } else { theme.on_surface_variant })
        .background(move |theme: &Theme| if selected { theme.surface_container_high } else { Color::TRANSPARENT })
        .height(px!(40.0))
        .padding(Edges::symmetric(16.0, 0.0))
        .border(Border::all(0.0, Color::TRANSPARENT).radius(radius::FULL))
        .transform_origin(TransformOrigin::CENTER)
        .transition_colors(fast_effect())
        .transition_transform(fast_spatial())
        .hover_style(|style: StylePatch, theme: &Theme| style.color(theme.on_surface).background(theme.surface_container_high))
        .focus_style(focus_ring)
        .pressed_style(|style: StylePatch, _| style.scale(0.97))
}

fn external_nav_link(label: &'static str, href: &'static str) -> Link {
    Link::new()
        .label(label)
        .href(href)
        .target_blank(true)
        .font_size(type_scale::LABEL_LG)
        .font_weight(FontWeight::Medium)
        .color(|theme: &Theme| theme.on_surface_variant)
        .height(px!(40.0))
        .padding(Edges::symmetric(16.0, 0.0))
        .border(Border::all(0.0, Color::TRANSPARENT).radius(radius::FULL))
        .transform_origin(TransformOrigin::CENTER)
        .transition_colors(fast_effect())
        .transition_transform(fast_spatial())
        .hover_style(|style: StylePatch, theme: &Theme| style.color(theme.on_surface).background(theme.surface_container_high))
        .focus_style(focus_ring)
        .pressed_style(|style: StylePatch, _| style.scale(0.97))
}

fn site_header() -> View {
    let desktop = responsive_bool(Breakpoint::Expanded, true);
    View::new()
        .position(Position::Sticky)
        .top(12.0)
        .z_index(20)
        .display(Display::Flex)
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .justify_content(JustifyContent::SpaceBetween)
        .height(px!(64.0))
        .margin(Responsive::new(Edges::symmetric(12.0, 12.0)).md(Edges::symmetric(24.0, 16.0)))
        .padding(Responsive::new(Edges::symmetric(16.0, 0.0)).md(Edges::symmetric(22.0, 0.0)))
        .background(|theme: &Theme| theme.surface_container_lowest.with_alpha(238))
        .backdrop_filter(Filter::Blur(px!(16.0)))
        .border(|theme: &Theme| Border::all(1.0, theme.outline_variant).radius(radius::XL))
        .box_shadow(BoxShadow::new(0.0, 10.0, 28.0, Color::BLACK.with_alpha(24)).direction(ShadowDirection::Bottom))
        .child(
            Button::new()
                .icon(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/XenGui_header.svg")))
                .expect("embedded XenGui logo must be valid SVG")
                .icon_size(if desktop { 104.0 } else { 92.0 }, 32.0)
                .background(Color::TRANSPARENT)
                .padding(Edges::all(0.0))
                .transform_origin(TransformOrigin::CENTER)
                .focus_style(focus_ring)
                .pressed_style(|style: StylePatch, _| style.scale(0.97))
                .on_click(|_| xen_router::push("/")),
        )
        .child(
            Row::new()
                .display(if desktop { Display::Flex } else { Display::None })
                .align_items(Align::Center)
                .gap(space::XS, 0.0)
                .child(nav_link("/docs", "Docs"))
                .child(nav_link("/examples", "Components"))
                .child(nav_link("/showcase", "Showcase"))
                .child(nav_link("/playground", "Playground"))
                .child(external_nav_link("GitHub", "https://github.com/randseas/xengui")),
        )
        .child(
            nav_link("/docs", if desktop { "Get started" } else { "Docs" })
                .background(|theme: &Theme| theme.primary)
                .color(|theme: &Theme| theme.on_primary),
        )
}

fn footer_link(path: &'static str, label: &'static str) -> Button {
    xen_router::link(path)
        .label(label)
        .font_size(type_scale::BODY_MD)
        .color(|theme: &Theme| theme.on_surface_variant)
        .padding(Edges::symmetric(0.0, 4.0))
        .background(Color::TRANSPARENT)
        .border(Border::all(0.0, Color::TRANSPARENT).radius(radius::XS))
        .transition_colors(fast_effect())
        .hover_style(|style: StylePatch, theme: &Theme| style.color(theme.primary))
        .focus_style(|style: StylePatch, theme: &Theme| style.border(Border::all(2.0, theme.primary).radius(radius::XS)))
        .pressed_style(|style: StylePatch, theme: &Theme| style.color(theme.on_surface))
}

fn external_footer_link(label: &'static str, href: &'static str) -> Link {
    Link::new()
        .label(label)
        .href(href)
        .target_blank(true)
        .font_size(type_scale::BODY_MD)
        .color(|theme: &Theme| theme.on_surface_variant)
        .padding(Edges::symmetric(0.0, 4.0))
        .border(Border::all(0.0, Color::TRANSPARENT).radius(radius::XS))
        .transition_colors(fast_effect())
        .hover_style(|style: StylePatch, theme: &Theme| style.color(theme.primary))
        .focus_style(|style: StylePatch, theme: &Theme| style.border(Border::all(2.0, theme.primary).radius(radius::XS)))
        .pressed_style(|style: StylePatch, theme: &Theme| style.color(theme.on_surface))
}

fn footer_column(title: &'static str, links: &[(&'static str, &'static str)]) -> View {
    let mut list = Column::new().align_items(Align::Start).gap(0.0, space::SM);
    for (path, label) in links {
        list = list.child(footer_link(path, label));
    }
    Column::new()
        .align_items(Align::Start)
        .gap(0.0, space::MD)
        .child(Label::new().label(title).font_size(type_scale::TITLE_SM).font_weight(FontWeight::Bold).color(|theme: &Theme| theme.on_surface))
        .child(list)
}

fn site_footer() -> View {
    let stacked = !responsive_bool(Breakpoint::Large, true);
    let brand = Column::new()
        .align_items(Align::Start)
        .gap(0.0, space::MD)
        .child(
            Button::new()
                .icon(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/XenGui_header.svg")))
                .expect("embedded XenGui logo must be valid SVG")
                .icon_size(100.0, 32.0)
                .background(Color::TRANSPARENT)
                .padding(Edges::all(0.0))
                .transform_origin(TransformOrigin::CENTER)
                .focus_style(focus_ring)
                .pressed_style(|style: StylePatch, _| style.scale(0.96))
                .on_click(|_| xen_router::push("/")),
        )
        .child(
            RichText::new()
                .with_content("A retained-mode Rust GUI toolkit for desktop and the web.")
                .width(pct!(100.0))
                .max_width(px!(360.0))
                .font_size(type_scale::BODY_MD)
                .line_height(px!(type_scale::BODY_MD_LINE))
                .color(|theme: &Theme| theme.on_surface_variant),
        );

    let columns = Row::new()
        .flex_wrap(FlexWrap::Wrap)
        .align_items(Align::Start)
        .justify_content(if stacked { JustifyContent::Start } else { JustifyContent::End })
        .gap(Responsive::new(px!(32.0)).lg(px!(56.0)), px!(28.0))
        .child(footer_column("Learn", &[("/docs", "Documentation"), ("/examples", "Components"), ("/playground", "Playground")]))
        .child(footer_column("Explore", &[("/showcase", "Live showcase"), ("/docs", "Quick start")]))
        .child(
            Column::new()
                .align_items(Align::Start)
                .gap(0.0, space::SM)
                .child(Label::new().label("Source").font_size(type_scale::TITLE_SM).font_weight(FontWeight::Bold))
                .child(external_footer_link("GitHub", "https://github.com/randseas/xengui"))
                .child(external_footer_link("crates.io", "https://crates.io/crates/xengui"))
                .child(external_footer_link("API reference", "https://docs.rs/xengui")),
        );

    Column::new()
        .margin(Responsive::new(Edges::all(12.0)).md(Edges::all(24.0)))
        .padding(Responsive::new(Edges::all(24.0)).md(Edges::all(36.0)))
        .gap(0.0, space::XXL)
        .background(|theme: &Theme| theme.surface_container_low)
        .border(|theme: &Theme| Border::all(1.0, theme.outline_variant).radius(radius::XL))
        .child(
            View::new()
                .display(Display::Flex)
                .flex_direction(if stacked { FlexDirection::Column } else { FlexDirection::Row })
                .justify_content(JustifyContent::SpaceBetween)
                .align_items(Align::Start)
                .gap(px!(40.0), px!(32.0))
                .child(brand)
                .child(columns),
        )
        .child(Separator::new())
        .child(
            View::new()
                .display(Display::Flex)
                .flex_direction(if stacked { FlexDirection::Column } else { FlexDirection::Row })
                .justify_content(JustifyContent::SpaceBetween)
                .gap(px!(8.0), px!(8.0))
                .child(Label::new().label("© 2026 XenGui · Apache 2.0").font_size(type_scale::BODY_SM).color(|theme: &Theme| theme.on_surface_variant))
                .child(Label::new().label("Built with XenGui").font_size(type_scale::BODY_SM).color(|theme: &Theme| theme.on_surface_variant)),
        )
}

pub fn layout(_params: &RouteParams, child: Box<dyn Widget>) -> Box<dyn Widget> {
    Box::new(
        Column::new()
            .font("Inter")
            .width(pct!(100.0))
            .height(pct!(100.0))
            .min_width(px!(0.0))
            .background(|theme: &Theme| theme.background)
            .overflow_y(Overflow::Scroll)
            .scrollbar_gutter(ScrollbarGutter::Stable)
            .child(site_header())
            .child_boxed(child)
            .child(site_footer()),
    )
}
