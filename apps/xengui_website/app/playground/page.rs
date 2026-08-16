use xen_router::RouteParams;
use xengui::*;

pub fn page(_params: &RouteParams) -> Box<dyn Widget> {
    let stacked = !responsive_bool(Breakpoint::Lg, true);

    let hero = View::new()
        .display(Display::Flex)
        .flex_direction(FlexDirection::Column)
        .align_items(Align::Start)
        .justify_content(JustifyContent::Start)
        .padding(Responsive::new(Edges::only(24, 100, 24, 0)).md(Edges::only(120, 100, 120, 0)))
        .child(
            View::new()
                .display(Display::Flex)
                .flex_direction(FlexDirection::Column)
                .font_size(Responsive::new(px!(38.0)).md(px!(60.0)))
                .font_weight(FontWeight::Medium)
                .line_height(pct!(64.0))
                .letter_spacing(px!(-2.25))
                .color(|theme: &Theme| theme.on_background)
                .child(Label::new().label("Playground"))
                .child(Label::new().label("page"))
                .child(
                    Label::new()
                        .color(|theme: &Theme| theme.on_surface_variant)
                        .font_size(16)
                        .line_height(px!(24.0))
                        .margin(Edges::only(0, 16, 0, 8))
                        .max_width(px!(560))
                        .label("Örnek bir bileşeni canlı önizlemesiyle birlikte inceleyin.")
                )
        );

    let code_panel = View::new()
        .display(Display::Flex)
        .flex_direction(FlexDirection::Column)
        .flex_grow(1.0)
        .min_height(px!(280))
        .padding(px!(20))
        .background(Color::NEUTRAL_950)
        .border(Border::all(1, Color::NEUTRAL_800).radius(12))
        .overflow_x(Overflow::Auto)
        .child(
            Label::new()
                .selectable(true)
                .label(
                    "Button::new()\n    .label(\"Gönder\")\n    .background(Color::BLUE_500)\n    .color(Color::WHITE)\n    .padding(Edges::only(20, 10, 20, 10))\n    .border(Border::all(1, Color::BLUE_400).radius(10))\n    .hover_style(|ctx, _t| ctx.background(Color::BLUE_600))\n    .on_click(|_ctx| { /* ... */ })"
                )
                .font_size(13)
                .line_height(px!(20.0))
                .color(Color::NEUTRAL_100)
        );

    let preview_panel = View::new()
        .display(Display::Flex)
        .flex_direction(FlexDirection::Column)
        .flex_grow(1.0)
        .min_height(px!(280))
        .align_items(Align::Center)
        .justify_content(JustifyContent::Center)
        .background(|theme: &Theme| theme.surface_container)
        .border(|theme: &Theme| Border::all(1, theme.outline_variant).radius(12))
        .child(
            Button::new()
                .label("Gönder")
                .background(Color::BLUE_500)
                .color(Color::WHITE)
                .padding(Edges::only(20, 10, 20, 10))
                .border(Border::all(1, Color::BLUE_400).radius(10))
                .hover_style(|ctx: StylePatch, _theme: &Theme| ctx.background(Color::BLUE_600))
        );

    let split = View::new()
        .display(Display::Flex)
        .flex_direction(if stacked { FlexDirection::Column } else { FlexDirection::Row })
        .gap(16, 16)
        .width(pct!(100))
        .padding(Responsive::new(Edges::symmetric(24, 0)).md(Edges::symmetric(120, 0)))
        .margin(Edges::only(0, 0, 0, 60))
        .child(code_panel)
        .child(preview_panel);

    Box::new(
        View::new()
            .display(Display::Flex)
            .flex_direction(FlexDirection::Column)
            .child(hero)
            .child(split)
    )
}
