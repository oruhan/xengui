use xen_router::RouteParams;
use xengui::*;

fn example_card(title: &str, desc: &str, preview: impl Widget + 'static) -> View {
    View::new()
        .display(Display::Flex)
        .flex_direction(FlexDirection::Column)
        .flex_basis(Responsive::new(pct!(100.0)).md(pct!(48.0)).lg(pct!(31.0)))
        .border(|theme: &Theme| Border::all(1, theme.outline_variant).radius(theme.radius_lg))
        .background(|theme: &Theme| theme.surface)
        .overflow_x(Overflow::Hidden)
        .child(
            View::new()
                .display(Display::Flex)
                .align_items(Align::Center)
                .justify_content(JustifyContent::Center)
                .height(px!(140))
                .background(|theme: &Theme| theme.surface_container)
                .child(preview)
        )
        .child(
            View::new()
                .display(Display::Flex)
                .flex_direction(FlexDirection::Column)
                .gap(0, 4)
                .padding(px!(16))
                .child(
                    Label::new()
                        .label(title)
                        .font_weight(FontWeight::SemiBold)
                        .font_size(15)
                        .color(|theme: &Theme| theme.on_background)
                )
                .child(
                    Label::new()
                        .label(desc)
                        .font_size(13)
                        .color(|theme: &Theme| theme.on_surface_variant)
                )
        )
}

pub fn page(_params: &RouteParams) -> Box<dyn Widget> {
    let hero = View::new()
        .display(Display::Flex)
        .flex_direction(FlexDirection::Column)
        .align_items(Align::Start)
        .justify_content(JustifyContent::Start)
        .padding(Responsive::new(Edges::only(24, 160, 24, 0)).md(Edges::only(120, 160, 120, 0)))
        .child(
            View::new()
                .display(Display::Flex)
                .flex_direction(FlexDirection::Column)
                .font_size(Responsive::new(px!(38.0)).md(px!(60.0)))
                .font_weight(FontWeight::Medium)
                .line_height(pct!(64.0))
                .letter_spacing(px!(-2.25))
                .color(|theme: &Theme| theme.on_background)
                .child(Label::new().label("Examples"))
                .child(Label::new().label("page"))
        );

    let grid = View::new()
        .display(Display::Flex)
        .flex_wrap(FlexWrap::Wrap)
        .gap(16, 16)
        .padding(Responsive::new(Edges::symmetric(24, 0)).md(Edges::symmetric(120, 0)))
        .margin(Edges::only(0, 0, 0, 60))
        .child(
            example_card(
                "Düğmeler",
                "Birincil, ikincil ve dışa hatlı buton varyantları.",
                Button::new()
                    .label("Buton")
                    .background(Color::BLUE_500)
                    .color(Color::WHITE)
                    .padding(Edges::only(16, 8, 16, 8))
                    .border(Border::all(1, Color::BLUE_400).radius(10))
            )
        )
        .child(
            example_card(
                "Anahtarlar",
                "Switch ile açık/kapalı durum kontrolü.",
                Switch::new().checked(true)
            )
        )
        .child(
            example_card(
                "Radyo Butonları",
                "Tek seçimli gruplar için RadioButton.",
                RadioButton::new().selected(true)
            )
        )
        .child(
            example_card(
                "Metin Girişi",
                "Placeholder ve klavye kısayolları destekli TextBox.",
                TextBox::new().placeholder("İsim girin...")
            )
        )
        .child(
            example_card(
                "Kart Düzeni",
                "Border, gölge ve border-radius ile kart bileşenleri.",
                View::new()
                    .width(px!(80))
                    .height(px!(48))
                    .background(|theme: &Theme| theme.surface_container_high)
                    .border(|theme: &Theme| Border::all(1, theme.outline_variant).radius(8))
            )
        )
        .child(
            example_card(
                "Bağlam Menüsü",
                "Sağ tık ile açılan, alt menü destekli ContextMenu.",
                Label::new().label("Sağ tıkla").font_size(13)
            )
        );

    Box::new(
        View::new()
            .display(Display::Flex)
            .flex_direction(FlexDirection::Column)
            .child(hero)
            .child(grid)
    )
}
