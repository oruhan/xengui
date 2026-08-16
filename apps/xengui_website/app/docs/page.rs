use std::time::Duration;
use xen_router::RouteParams;
use xengui::*;

fn concept_card(title: &str, desc: &str) -> View {
    View::new()
        .display(Display::Flex)
        .flex_direction(FlexDirection::Column)
        .gap(0, 8)
        .flex_basis(Responsive::new(pct!(100.0)).md(pct!(48.0)).lg(pct!(31.0)))
        .padding(px!(20))
        .background(|theme: &Theme| theme.surface)
        .border(|theme: &Theme| Border::all(1, theme.outline_variant).radius(theme.radius_lg))
        .child(
            Label::new()
                .label(title)
                .font_weight(FontWeight::SemiBold)
                .font_size(16)
                .color(|theme: &Theme| theme.on_background)
        )
        .child(
            Label::new()
                .label(desc)
                .font_size(14)
                .color(|theme: &Theme| theme.on_surface_variant)
        )
}

fn code_block(code: &str) -> View {
    View::new()
        .display(Display::Flex)
        .padding(px!(20))
        .background(Color::NEUTRAL_950)
        .border(Border::all(1, Color::NEUTRAL_800).radius(12))
        .overflow_x(Overflow::Auto)
        .child(
            Label::new()
                .label(code)
                .selectable(true)
                .font_size(13)
                .color(Color::NEUTRAL_100)
                .line_height(px!(20.0))
        )
}

fn section_title(text: &str) -> Label {
    Label::new()
        .label(text)
        .font_size(Responsive::new(px!(24.0)).md(px!(28.0)))
        .font_weight(FontWeight::SemiBold)
        .letter_spacing(px!(-0.5))
        .color(|theme: &Theme| theme.on_background)
}

pub fn page(_params: &RouteParams) -> Box<dyn Widget> {
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
                .child(Label::new().label("Docs"))
                .child(Label::new().label("page"))
                .child(
                    Label::new()
                        .color(|theme: &Theme| theme.on_background)
                        .font_size(16)
                        .font_weight(FontWeight::Regular)
                        .line_height(px!(24.0))
                        .letter_spacing(px!(-0.1))
                        .margin(Edges::only(0, 16, 0, 8))
                        .max_width(px!(560))
                        .label(
                            "XenGui, Rust ile tek kod tabanından masaüstü, web ve gömülü arayüzler üretmek için tasarlanmış olay tabanlı bir GUI kütüphanesidir."
                        )
                )
        )
        .child(
            View::new()
                .display(Display::Flex)
                .flex_direction(FlexDirection::Row)
                .margin(Edges::only(0, 16, 0, 16))
                .gap(4, 0)
                .child(
                    Button::new()
                        .background(Color::BLUE_500)
                        .font_weight(FontWeight::Medium)
                        .transition_all(
                            Transition::new(Duration::from_millis(200)).easing(Easing::EaseInOut)
                        )
                        .border(Border::all(1, Color::BLUE_400).radius(10))
                        .hover_style(|ctx: StylePatch, _theme: &Theme|
                            ctx
                                .background(Color::BLUE_600)
                                .border(Border::all(1, Color::BLUE_500).radius(10))
                        )
                        .pressed_style(|ctx: StylePatch, _theme: &Theme|
                            ctx
                                .background(Color::BLUE_700)
                                .border(Border::all(1, Color::BLUE_600).radius(10))
                                .scale(0.97)
                        )
                        .padding(Edges::only(15, 9, 15, 9))
                        .label("Başlarken")
                )
                .child(
                    Button::new()
                        .background(Color::hex("#24292f"))
                        .font_weight(FontWeight::Medium)
                        .transition_all(
                            Transition::new(Duration::from_millis(200)).easing(Easing::EaseInOut)
                        )
                        .border(Border::all(1, Color::hex("#3d444d")).radius(10))
                        .hover_style(|ctx: StylePatch, _theme: &Theme|
                            ctx
                                .background(Color::hex("#30363d"))
                                .border(Border::all(1, Color::hex("#4b5561")).radius(10))
                        )
                        .pressed_style(|ctx: StylePatch, _theme: &Theme|
                            ctx.background(Color::hex("#1f2328")).scale(0.97)
                        )
                        .padding(Edges::only(15, 9, 15, 9))
                        .label("GitHub")
                )
        );

    let quick_start = View::new()
        .display(Display::Flex)
        .flex_direction(FlexDirection::Column)
        .gap(0, 16)
        .padding(Responsive::new(Edges::symmetric(24, 32)).md(Edges::symmetric(120, 40)))
        .child(section_title("Hızlı Başlangıç"))
        .child(
            code_block(
                "use xengui::*;\n\nfn main() {\n    let mut app = App::new(AppConfig::default());\n    app.render(|| {\n        View::new()\n            .padding(px!(24))\n            .child(Label::new().label(\"Merhaba, XenGui!\"))\n    });\n    app.run().unwrap();\n}"
            )
        );

    let concepts = View::new()
        .display(Display::Flex)
        .flex_direction(FlexDirection::Column)
        .gap(0, 20)
        .padding(Responsive::new(Edges::symmetric(24, 0)).md(Edges::symmetric(120, 0)))
        .margin(Edges::only(0, 0, 0, 40))
        .child(section_title("Temel Kavramlar"))
        .child(
            View::new()
                .display(Display::Flex)
                .flex_wrap(FlexWrap::Wrap)
                .gap(16, 16)
                .child(
                    concept_card(
                        "Widget'lar",
                        "View, Label, Button gibi kompozit edilebilir yapı taşları; her biri kendi stil ve layout API'sine sahiptir."
                    )
                )
                .child(
                    concept_card(
                        "Stil Sistemi",
                        "Tailwind'e benzer responsive değerler, tema tabanlı renkler ve CSS'e yakın kutu modeli."
                    )
                )
                .child(
                    concept_card(
                        "Hook'lar",
                        "use_state, use_effect ve use_resource ile React'e benzer bildirimsel durum yönetimi."
                    )
                )
                .child(
                    concept_card(
                        "Animasyon",
                        "Transition ve Easing tabanlı, otomatik ara değerleri hesaplayan bir animasyon motoru."
                    )
                )
                .child(
                    concept_card(
                        "Layout Motoru",
                        "Taffy tabanlı flexbox/grid layout; her frame'de yeniden hesaplanan responsive breakpoint'ler."
                    )
                )
                .child(
                    concept_card(
                        "Render Backend",
                        "wgpu tabanlı GPU render katmanı; RenderBackend trait'i sayesinde platform bağımsız."
                    )
                )
        );

    Box::new(
        View::new()
            .display(Display::Flex)
            .flex_direction(FlexDirection::Column)
            .child(hero)
            .child(quick_start)
            .child(concepts)
    )
}
