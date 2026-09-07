use std::time::Duration;
use xen_router::RouteParams;
use xengui::*;

fn eyebrow(label: &str) -> View {
    Row::new()
        .align_items(Align::Center)
        .gap(8.0, 0.0)
        .child(
            View::new()
                .width(px!(7.0))
                .height(px!(7.0))
                .background(Color::BLUE_500)
                .border(Border::all(0.0, Color::TRANSPARENT).radius(7.0)),
        )
        .child(
            Label::new()
                .label(label)
                .font_size(12.0)
                .font_weight(FontWeight::SemiBold)
                .letter_spacing(px!(0.8))
                .color(|theme: &Theme| theme.on_surface_variant),
        )
}

fn dot(color: Color) -> View {
    View::new()
        .width(px!(8.0))
        .height(px!(8.0))
        .background(color)
        .border(Border::all(0.0, Color::TRANSPARENT).radius(8.0))
}

fn code_window() -> View {
    let toolbar = Row::new()
        .align_items(Align::Center)
        .justify_content(JustifyContent::SpaceBetween)
        .padding(Edges::symmetric(16.0, 12.0))
        .border(Border::bottom(1.0, Color::NEUTRAL_800))
        .child(
            Row::new()
                .gap(6.0, 0.0)
                .child(dot(Color::hex("#ff6b66")))
                .child(dot(Color::hex("#e9b949")))
                .child(dot(Color::hex("#45b97c"))),
        )
        .child(
            Label::new()
                .label("src/main.rs")
                .font_size(11.0)
                .color(Color::NEUTRAL_500),
        );

    Column::new()
        .width(Responsive::new(pct!(100.0)).lg(pct!(45.0)))
        .min_width(px!(0.0))
        .background(Color::NEUTRAL_950)
        .border(Border::all(1.0, Color::NEUTRAL_800).radius(16.0))
        .overflow_x(Overflow::Hidden)
        .box_shadow(
            BoxShadow::new(0.0, 18.0, 48.0, Color::BLACK.with_alpha(42))
                .direction(ShadowDirection::Bottom),
        )
        .child(toolbar)
        .child(
            Label::new()
                .label(
                    "use xengui::*;\n\nfn view() -> View {\n    Column::new()\n        .gap(0.0, 12.0)\n        .child(Label::new()\n            .label(\"Merhaba, XenGui\"))\n        .child(Button::new()\n            .label(\"Devam et\"))\n}",
                )
                .selectable(true)
                .padding(Responsive::new(Edges::all(18.0)).md(Edges::all(24.0)))
                .font_size(13.0)
                .line_height(px!(21.0))
                .color(Color::NEUTRAL_200),
        )
        .child(
            Row::new()
                .align_items(Align::Center)
                .justify_content(JustifyContent::SpaceBetween)
                .padding(Edges::symmetric(18.0, 13.0))
                .background(Color::hex("#111827"))
                .border(Border::top(1.0, Color::NEUTRAL_800))
                .child(
                    Label::new()
                        .label("native / wasm32")
                        .font_size(11.0)
                        .color(Color::NEUTRAL_400),
                )
                .child(
                    Label::new()
                        .label("cargo run")
                        .font_size(11.0)
                        .font_weight(FontWeight::SemiBold)
                        .color(Color::BLUE_300),
                ),
        )
}

fn stat(value: &str, label: &str) -> View {
    Column::new()
        .flex_basis(Responsive::new(pct!(48.0)).lg(pct!(23.0)))
        .gap(0.0, 5.0)
        .padding(Edges::only(0.0, 20.0, 0.0, 20.0))
        .border(|theme: &Theme| Border::top(1.0, theme.outline_variant))
        .child(
            Label::new()
                .label(value)
                .font_size(19.0)
                .font_weight(FontWeight::SemiBold)
                .color(|theme: &Theme| theme.on_background),
        )
        .child(
            Label::new()
                .label(label)
                .font_size(12.0)
                .color(|theme: &Theme| theme.on_surface_variant),
        )
}

fn feature(index: &str, title: &str, text: &str) -> View {
    Column::new()
        .flex_basis(Responsive::new(pct!(100.0)).md(pct!(31.0)))
        .min_width(px!(0.0))
        .gap(0.0, 14.0)
        .padding(Edges::only(0.0, 22.0, 0.0, 8.0))
        .border(|theme: &Theme| Border::top(1.0, theme.outline_variant))
        .child(
            Label::new()
                .label(index)
                .font_size(11.0)
                .font_weight(FontWeight::SemiBold)
                .letter_spacing(px!(1.0))
                .color(|theme: &Theme| theme.primary),
        )
        .child(
            Label::new()
                .label(title)
                .font_size(19.0)
                .font_weight(FontWeight::SemiBold)
                .color(|theme: &Theme| theme.on_background),
        )
        .child(
            RichText::new()
                .with_content(text)
                .width(pct!(100.0))
                .font_size(14.0)
                .line_height(px!(22.0))
                .color(|theme: &Theme| theme.on_surface_variant),
        )
}

pub fn page(_params: &RouteParams) -> Box<dyn Widget> {
    let stacked = !responsive_bool(Breakpoint::Lg, true);
    let mobile_content_width = (viewport_size().0 - 40.0).clamp(280.0, 720.0);

    let hero_copy = Column::new()
        .width(Responsive::new(pct!(100.0)).lg(pct!(49.0)))
        .min_width(px!(0.0))
        .align_items(Align::Start)
        .gap(0.0, 22.0)
        .child(eyebrow("CROSS-PLATFORM UI TOOLKIT"))
        .child(
            RichText::new()
                .with_content("Rust arayüzleri, platform farkı olmadan.")
                .width(pct!(100.0))
                .max_width(px!(mobile_content_width))
                .font_size(
                    Responsive::new(px!(42.0))
                        .md(px!(58.0))
                        .lg(px!(68.0)),
                )
                .line_height(
                    Responsive::new(px!(47.0))
                        .md(px!(63.0))
                        .lg(px!(72.0))
                        .resolve(),
                )
                .font_weight(FontWeight::SemiBold)
                .letter_spacing(px!(-2.2))
                .color(|theme: &Theme| theme.on_background),
        )
        .child(
            RichText::new()
                .with_content("Tek bir retained widget ağacıyla native masaüstü ve web uygulamaları geliştirin. Layout, input ve GPU renderer aynı kod tabanında.")
                .width(pct!(100.0))
                .max_width(px!(mobile_content_width.min(610.0)))
                .font_size(Responsive::new(px!(15.0)).md(px!(17.0)))
                .line_height(px!(26.0))
                .color(|theme: &Theme| theme.on_surface_variant),
        )
        .child(
            View::new()
                .display(Display::Flex)
                .flex_direction(FlexDirection::Row)
                .flex_wrap(FlexWrap::Wrap)
                .gap(10.0, 10.0)
                .child(
                    xen_router::link("/docs")
                        .label("Başlangıç rehberi  →")
                        .font_weight(FontWeight::SemiBold)
                        .background(|theme: &Theme| theme.on_background)
                        .color(|theme: &Theme| theme.background)
                        .padding(Edges::symmetric(18.0, 12.0))
                        .border(Border::all(0.0, Color::TRANSPARENT).radius(10.0))
                        .transition_all(
                            Transition::new(Duration::from_millis(140)).easing(Easing::EaseOut),
                        )
                        .pressed_style(|style: StylePatch, _theme: &Theme| style.scale(0.98)),
                )
                .child(
                    Link::new()
                        .href("https://github.com/randseas/xengui")
                        .target_blank(true)
                        .label("GitHub'da incele")
                        .font_weight(FontWeight::Medium)
                        .background(Color::TRANSPARENT)
                        .color(|theme: &Theme| theme.on_background)
                        .padding(Edges::symmetric(18.0, 12.0))
                        .border(|theme: &Theme| {
                            Border::all(1.0, theme.outline_variant).radius(10.0)
                        }),
                ),
        )
        .child(
            Label::new()
                .label("Apache 2.0 · Rust 1.92+ · v0.2.8")
                .font_size(12.0)
                .color(|theme: &Theme| theme.on_surface_variant),
        );

    let hero = View::new()
        .display(Display::Flex)
        .flex_direction(if stacked {
            FlexDirection::Column
        } else {
            FlexDirection::Row
        })
        .align_items(if stacked {
            Align::Stretch
        } else {
            Align::Center
        })
        .justify_content(JustifyContent::SpaceBetween)
        .gap(Responsive::new(px!(44.0)).lg(px!(72.0)), px!(44.0))
        .padding(
            Responsive::new(Edges::only(20.0, 56.0, 20.0, 48.0))
                .md(Edges::only(64.0, 80.0, 64.0, 64.0))
                .lg(Edges::only(120.0, 104.0, 120.0, 88.0)),
        )
        .child(hero_copy)
        .child(code_window());

    let stats = View::new()
        .display(Display::Flex)
        .flex_wrap(FlexWrap::Wrap)
        .gap(16.0, 16.0)
        .padding(
            Responsive::new(Edges::only(20.0, 0.0, 20.0, 56.0))
                .md(Edges::only(64.0, 0.0, 64.0, 72.0))
                .lg(Edges::only(120.0, 0.0, 120.0, 88.0)),
        )
        .child(stat("1", "widget modeli"))
        .child(stat("2", "render hedefi"))
        .child(stat("wgpu", "GPU renderer"))
        .child(stat("0 JS", "uygulama mantığı"));

    let principles = Column::new()
        .gap(0.0, 32.0)
        .padding(
            Responsive::new(Edges::only(20.0, 56.0, 20.0, 80.0))
                .md(Edges::only(64.0, 72.0, 64.0, 96.0))
                .lg(Edges::only(120.0, 88.0, 120.0, 120.0)),
        )
        .child(
            Column::new()
                .gap(0.0, 10.0)
                .child(eyebrow("NEDEN XENGUI"))
                .child(
                    RichText::new()
                        .with_content("Arayüz katmanı sade, kontrol sizde.")
                        .width(pct!(100.0))
                        .font_size(Responsive::new(px!(28.0)).md(px!(38.0)))
                        .line_height(px!(44.0))
                        .font_weight(FontWeight::SemiBold)
                        .letter_spacing(px!(-1.2))
                        .color(|theme: &Theme| theme.on_background),
                ),
        )
        .child(
            View::new()
                .display(Display::Flex)
                .flex_wrap(FlexWrap::Wrap)
                .gap(18.0, 18.0)
                .child(feature(
                    "01 / MODEL",
                    "Bildirimsel ve retained",
                    "Builder API ile okunabilir ağaçlar kurun; state güncellemelerinde kimlik ve etkileşim durumu korunsun.",
                ))
                .child(feature(
                    "02 / RENDER",
                    "GPU odaklı",
                    "Metin, SVG ve yüzeyler wgpu üzerinden çizilir. Native ve WebGPU aynı paint komutlarını tüketir.",
                ))
                .child(feature(
                    "03 / SCALE",
                    "Responsive temelden",
                    "Breakpoint, tema tokenı ve esnek layout araçları ek bir stil katmanı olmadan builder zincirinde birleşir.",
                )),
        );

    Box::new(
        Column::new()
            .width(pct!(100.0))
            .min_width(px!(0.0))
            .child(hero)
            .child(stats)
            .child(principles),
    )
}
