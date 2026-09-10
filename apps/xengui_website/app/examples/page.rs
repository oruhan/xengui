use std::time::Duration;
use xen_router::RouteParams;
use xengui::*;

fn example_card(kind: &str, title: &str, desc: &str, preview: impl Widget + 'static) -> View {
    let viewport_width = viewport_size().0;
    let text_width = if responsive_bool(Breakpoint::Large, true) {
        ((viewport_width - 272.0) * 0.31 - 36.0).clamp(200.0, 420.0)
    } else if responsive_bool(Breakpoint::Expanded, true) {
        ((viewport_width - 144.0) * 0.48 - 36.0).clamp(200.0, 420.0)
    } else {
        (viewport_width - 76.0).clamp(200.0, 420.0)
    };

    Column::new()
        .flex_basis(Responsive::new(pct!(100.0)).md(pct!(48.0)).lg(pct!(31.0)))
        .min_width(px!(0.0))
        .background(|theme: &Theme| theme.surface)
        .border(|theme: &Theme| Border::all(1.0, theme.outline_variant).radius(14.0))
        .overflow_x(Overflow::Hidden)
        .transition_all(Transition::new(Duration::from_millis(160)).easing(Easing::EaseOut))
        .hover_style(|style: StylePatch, theme: &Theme| {
            style.background(theme.surface_container_lowest)
        })
        .child(
            View::new()
                .display(Display::Flex)
                .align_items(Align::Center)
                .justify_content(JustifyContent::Center)
                .height(Responsive::new(px!(150.0)).md(px!(176.0)))
                .padding(Edges::all(20.0))
                .background(|theme: &Theme| theme.surface_container_low)
                .border(|theme: &Theme| Border::bottom(1.0, theme.outline_variant))
                .child(preview),
        )
        .child(
            Column::new()
                .gap(0.0, 8.0)
                .padding(Edges::all(18.0))
                .child(
                    Label::new()
                        .label(kind)
                        .font_size(10.0)
                        .font_weight(FontWeight::SemiBold)
                        .letter_spacing(px!(1.0))
                        .color(|theme: &Theme| theme.primary),
                )
                .child(
                    Label::new()
                        .label(title)
                        .font_weight(FontWeight::SemiBold)
                        .font_size(17.0)
                        .color(|theme: &Theme| theme.on_background),
                )
                .child(
                    RichText::new()
                        .with_content(desc)
                        .width(pct!(100.0))
                        .max_width(px!(text_width))
                        .font_size(13.0)
                        .line_height(px!(20.0))
                        .color(|theme: &Theme| theme.on_surface_variant),
                ),
        )
}

pub fn page(_params: &RouteParams) -> Box<dyn Widget> {
    let content_width = (viewport_size().0 - 40.0).clamp(280.0, 760.0);

    let hero = Column::new()
        .align_items(Align::Start)
        .gap(0.0, 16.0)
        .padding(
            Responsive::new(Edges::only(20.0, 48.0, 20.0, 36.0))
                .md(Edges::only(64.0, 72.0, 64.0, 48.0))
                .lg(Edges::only(120.0, 80.0, 120.0, 56.0)),
        )
        .child(
            Label::new()
                .label("WIDGET GALERİSİ")
                .font_size(11.0)
                .font_weight(FontWeight::SemiBold)
                .letter_spacing(px!(1.1))
                .color(|theme: &Theme| theme.primary),
        )
        .child(
            RichText::new()
                .with_content("Temel parçalar, gerçek davranışlarıyla.")
                .width(pct!(100.0))
                .max_width(px!(content_width))
                .font_size(Responsive::new(px!(36.0)).md(px!(52.0)))
                .line_height(Responsive::new(px!(41.0)).md(px!(58.0)).resolve())
                .font_weight(FontWeight::SemiBold)
                .letter_spacing(px!(-1.8))
                .color(|theme: &Theme| theme.on_background),
        )
        .child(
            RichText::new()
                .with_content("XenGui widget'larını tema, input ve layout davranışlarıyla birlikte inceleyin. Bunlar statik çizimler değil; aşağıdaki kontroller etkileşimlidir.")
                .width(pct!(100.0))
                .max_width(px!(content_width.min(680.0)))
                .font_size(15.0)
                .line_height(px!(24.0))
                .color(|theme: &Theme| theme.on_surface_variant),
        );

    let grid = View::new()
        .display(Display::Flex)
        .flex_wrap(FlexWrap::Wrap)
        .gap(16.0, 16.0)
        .padding(
            Responsive::new(Edges::only(20.0, 0.0, 20.0, 72.0))
                .md(Edges::only(64.0, 0.0, 64.0, 88.0))
                .lg(Edges::only(120.0, 0.0, 120.0, 104.0)),
        )
        .child(example_card(
            "ACTION",
            "Button",
            "Hover, pressed ve focus durumlarını aynı stil zincirinde yönetin.",
            Button::new()
                .label("Değişiklikleri kaydet")
                .background(|theme: &Theme| theme.primary)
                .color(|theme: &Theme| theme.on_primary)
                .padding(Edges::symmetric(16.0, 10.0))
                .border(Border::all(0.0, Color::TRANSPARENT).radius(9.0)),
        ))
        .child(example_card(
            "BOOLEAN",
            "Switch",
            "Kontrollü açık/kapalı durumu ve değişim callback'i.",
            Row::new()
                .align_items(Align::Center)
                .gap(12.0, 0.0)
                .child(Switch::new().checked(true))
                .child(Label::new().label("Bildirimler").font_size(14.0)),
        ))
        .child(example_card(
            "SELECTION",
            "RadioButton",
            "Tek seçimli gruplar için klavye ve pointer girdisi.",
            Row::new()
                .align_items(Align::Center)
                .gap(12.0, 0.0)
                .child(RadioButton::new().selected(true))
                .child(Label::new().label("Kararlı sürüm").font_size(14.0)),
        ))
        .child(example_card(
            "INPUT",
            "TextBox",
            "Seçim, placeholder, IME ve submit akışı tek kontrolde.",
            TextBox::new()
                .placeholder("proje-adi")
                .width(Responsive::new(pct!(100.0)).md(px!(220.0))),
        ))
        .child(example_card(
            "FEEDBACK",
            "ProgressBar",
            "Belirli veya uygulama state'ine bağlı ilerleme göstergesi.",
            Column::new()
                .width(Responsive::new(pct!(100.0)).md(px!(220.0)))
                .gap(0.0, 10.0)
                .child(ProgressBar::new().value(0.68))
                .child(
                    Label::new()
                        .label("Derleniyor · %68")
                        .font_size(12.0)
                        .color(|theme: &Theme| theme.on_surface_variant),
                ),
        ))
        .child(example_card(
            "STATUS",
            "Badge + Kbd",
            "Yoğun arayüzlerde kısa durum ve klavye ipucu.",
            Row::new()
                .align_items(Align::Center)
                .gap(12.0, 0.0)
                .child(Badge::new().label("Kararlı"))
                .child(Kbd::new().label("Ctrl K")),
        ));

    Box::new(Column::new().width(pct!(100.0)).child(hero).child(grid))
}
