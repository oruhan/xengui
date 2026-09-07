// SPDX-License-Identifier: Apache-2.0
//! Interactive documentation hub for the XenGui website.

use std::time::Duration;
use xen_router::RouteParams;
use xengui::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum DocsSection {
    #[default]
    Start,
    Concepts,
    Widgets,
    Styling,
    Rendering,
    Api,
}

const NAV_ITEMS: &[(DocsSection, &str, &str)] = &[
    (DocsSection::Start, "Başlarken", "Kurulum ve ilk uygulama"),
    (
        DocsSection::Concepts,
        "Temel kavramlar",
        "Tree, state ve yaşam döngüsü",
    ),
    (DocsSection::Widgets, "Widget'lar", "Temel bileşen kataloğu"),
    (
        DocsSection::Styling,
        "Stil ve layout",
        "Tema, flex, grid, responsive",
    ),
    (
        DocsSection::Rendering,
        "Renderer",
        "wgpu, WebGPU ve hata yönetimi",
    ),
    (DocsSection::Api, "API referansı", "Rustdoc paketleri"),
];

fn paragraph(text: &str, font_size: f32, line_height: f32) -> RichText {
    let available_width = (viewport_size().0 - 40.0).clamp(200.0, 760.0);
    RichText::new()
        .with_content(text)
        .width(pct!(100.0))
        .min_width(px!(0.0))
        .max_width(px!(available_width))
        .font_size(font_size)
        .line_height(px!(line_height))
        .color(|theme: &Theme| theme.on_surface_variant)
}

fn heading(kicker: &str, title: &str, description: &str) -> View {
    Column::new()
        .width(pct!(100.0))
        .min_width(px!(0.0))
        .gap(0.0, 12.0)
        .child(
            Label::new()
                .label(kicker.to_uppercase())
                .font_size(12.0)
                .font_weight(FontWeight::SemiBold)
                .letter_spacing(px!(1.2))
                .color(|theme: &Theme| theme.primary),
        )
        .child(
            RichText::new()
                .with_content(title)
                .width(pct!(100.0))
                .max_width(px!((viewport_size().0 - 40.0).clamp(220.0, 760.0)))
                .font_size(Responsive::new(px!(30.0)).md(px!(44.0)))
                .font_weight(FontWeight::SemiBold)
                .letter_spacing(px!(-1.5))
                .color(|theme: &Theme| theme.on_background),
        )
        .child(paragraph(description, 15.0, 24.0))
}

fn section_title(title: &str, description: &str) -> View {
    Column::new()
        .width(pct!(100.0))
        .min_width(px!(0.0))
        .gap(0.0, 6.0)
        .child(
            RichText::new()
                .with_content(title)
                .width(pct!(100.0))
                .max_width(px!((viewport_size().0 - 40.0).clamp(220.0, 760.0)))
                .font_size(22.0)
                .font_weight(FontWeight::SemiBold),
        )
        .child(paragraph(description, 14.0, 22.0))
}

fn code_block(label: &str, code: &str) -> CodeBlock {
    let language = match label {
        "Cargo.toml" => CodeLanguage::Toml,
        "Terminal" => CodeLanguage::Shell,
        _ => CodeLanguage::Rust,
    };
    CodeBlock::new(code)
        .label(label)
        .language(language)
        .copy_label("Kopyala")
}

fn note(title: &str, text: &str) -> View {
    Row::new()
        // HTML'deki block-level `display:flex; width:auto` gibi mevcut
        // satırı doldur. Yüzde genişlikli RichText'in intrinsic ölçüme
        // katılamadığı shrink-to-fit döngüsünü bu sınır kırar.
        .width(pct!(100.0))
        .min_width(px!(0.0))
        .gap(12.0, 0.0)
        .padding(Edges::all(18.0))
        .background(|theme: &Theme| theme.primary_container.with_alpha_f32(0.62))
        .border(|theme: &Theme| Border::all(1.0, theme.primary.with_alpha_f32(0.28)).radius(20.0))
        .child(VariableIcon::new(xengui_icons::codepoints::INFO).size(20.0))
        .child(
            Column::new()
                .flex_grow(1.0)
                .min_width(px!(0.0))
                .gap(0.0, 4.0)
                .child(Label::new().label(title).font_weight(FontWeight::SemiBold))
                .child(
                    paragraph(text, 13.0, 20.0)
                        .max_width(px!((viewport_size().0 - 104.0).clamp(180.0, 660.0))),
                ),
        )
}

fn feature_card(title: &str, description: &str) -> View {
    Column::new()
        .flex_basis(Responsive::new(pct!(100.0)).md(pct!(48.0)))
        .gap(0.0, 10.0)
        .padding(Edges::all(20.0))
        .background(|theme: &Theme| theme.surface_container_low)
        .border(|theme: &Theme| Border::all(1.0, theme.outline_variant).radius(22.0))
        .child(
            Label::new()
                .label(title)
                .font_size(16.0)
                .font_weight(FontWeight::SemiBold),
        )
        .child(
            paragraph(description, 13.0, 20.0)
                .max_width(px!((viewport_size().0 - 76.0).clamp(180.0, 400.0))),
        )
}

fn nav_button(
    active: DocsSection,
    section: DocsSection,
    title: &'static str,
    subtitle: &'static str,
    set_active: SetState<DocsSection>,
    compact: bool,
) -> Button {
    let selected = active == section;
    Button::new()
        .width(if compact { px!(142.0) } else { pct!(100.0) })
        .flex_shrink(0.0)
        .label(if compact {
            title.to_owned()
        } else {
            let index = NAV_ITEMS
                .iter()
                .position(|(item, _, _)| *item == section)
                .unwrap_or_default()
                + 1;
            format!("{index:02}   {title}\n       {subtitle}")
        })
        .font_size(13.0)
        .line_height(px!(19.0))
        .color(move |theme: &Theme| {
            if selected {
                theme.on_primary_container
            } else {
                theme.on_surface_variant
            }
        })
        .background(move |theme: &Theme| {
            if selected {
                theme.primary_container
            } else {
                Color::TRANSPARENT
            }
        })
        .border(Border::all(0.0, Color::TRANSPARENT).radius(18.0))
        .padding(Edges::symmetric(15.0, 12.0))
        .hover_style(|style, theme| style.background(theme.surface_container_high))
        .transition_all(Transition::new(Duration::from_millis(140)).easing(Easing::EaseOut))
        .on_click(move |_ctx| set_active.set(section))
}

fn cards(items: &[(&str, &str)]) -> View {
    let mut grid = View::new()
        .display(Display::Flex)
        .flex_wrap(FlexWrap::Wrap)
        .gap(14.0, 14.0);
    for (title, description) in items {
        grid = grid.child(feature_card(title, description));
    }
    grid
}

fn meta_chip(label: &str) -> View {
    View::new()
        .padding(Edges::symmetric(11.0, 7.0))
        .background(|theme: &Theme| theme.surface_container_lowest)
        .border(|theme: &Theme| Border::all(1.0, theme.outline_variant).radius(8.0))
        .child(
            Label::new()
                .label(label)
                .font_size(11.0)
                .font_weight(FontWeight::Medium)
                .color(|theme: &Theme| theme.on_surface_variant),
        )
}

fn docs_hero(set_active: SetState<DocsSection>) -> View {
    let set_api = set_active.clone();
    Column::new()
        .width(pct!(100.0))
        .gap(0.0, 18.0)
        .padding(Responsive::new(Edges::all(24.0)).md(Edges::all(40.0)))
        .background(|theme: &Theme| theme.surface_container_low)
        .border(|theme: &Theme| Border::all(1.0, theme.outline_variant).radius(14.0))
        .child(
            Row::new()
                .align_items(Align::Center)
                .gap(8.0, 0.0)
                .child(
                    StyleBuilder::color(
                        VariableIcon::new(xengui_icons::codepoints::AUTO_STORIES).size(18.0),
                        |theme: &Theme| theme.primary,
                    ),
                )
                .child(
                    Label::new()
                        .label("XENGUI DOCUMENTATION")
                        .font_size(12.0)
                        .font_weight(FontWeight::SemiBold)
                        .letter_spacing(px!(1.1))
                        .color(|theme: &Theme| theme.on_surface_variant),
                ),
        )
        .child(
            RichText::new()
                .with_content("Rust UI, baştan sona anlaşılır.")
                .width(pct!(100.0))
                .max_width(px!((viewport_size().0 - 88.0).clamp(220.0, 820.0)))
                .font_size(Responsive::new(px!(38.0)).md(px!(58.0)))
                .line_height(Responsive::new(px!(44.0)).md(px!(64.0)).resolve())
                .font_weight(FontWeight::SemiBold)
                .letter_spacing(px!(-2.0))
                .color(|theme: &Theme| theme.on_background),
        )
        .child(
            paragraph(
                "Kurulumdan renderer mimarisine kadar, üretimde ihtiyaç duyacağın kavramları çalışan örneklerle öğren.",
                16.0,
                25.0,
            )
            .max_width(px!((viewport_size().0 - 88.0).clamp(220.0, 680.0))),
        )
        .child(
            View::new()
                .display(Display::Flex)
                .flex_wrap(FlexWrap::Wrap)
                .gap(10.0, 10.0)
                .child(
                    Button::new()
                        .label("Başlangıç rehberi  →")
                        .font_weight(FontWeight::SemiBold)
                        .background(|theme: &Theme| theme.on_background)
                        .color(|theme: &Theme| theme.background)
                        .border(Border::all(0.0, Color::TRANSPARENT).radius(10.0))
                        .padding(Edges::symmetric(16.0, 10.0))
                        .on_click(move |_ctx| set_active.set(DocsSection::Start)),
                )
                .child(
                    Button::new()
                        .label("API referansı")
                        .font_weight(FontWeight::SemiBold)
                        .background(Color::TRANSPARENT)
                        .color(|theme: &Theme| theme.on_surface)
                        .border(|theme: &Theme| {
                            Border::all(1.0, theme.outline_variant).radius(10.0)
                        })
                        .padding(Edges::symmetric(16.0, 10.0))
                        .on_click(move |_ctx| set_api.set(DocsSection::Api)),
                ),
        )
        .child(
            View::new()
                .display(Display::Flex)
                .flex_wrap(FlexWrap::Wrap)
                .gap(8.0, 8.0)
                .child(meta_chip("xengui 0.2.8"))
                .child(meta_chip("Native + Web"))
                .child(meta_chip("wgpu"))
                .child(meta_chip("Rust 1.92+")),
        )
}

fn start_page() -> View {
    Column::new()
        .width(pct!(100.0))
        .min_width(px!(0.0))
        .gap(0.0, 28.0)
        .child(heading(
            "XenGui Docs",
            "Rust ile ilk arayüzünü oluştur",
            "Aynı bildirimsel widget ağacını masaüstünde wgpu, tarayıcıda WebGPU/WebGL üzerinde çalıştır.",
        ))
        .child(note(
            "Hibrit dokümantasyon",
            "Bu rehber öğrenme akışını ve çalışan örnekleri açıklar. İmzalar ve public API için rustdoc tek doğruluk kaynağıdır.",
        ))
        .child(section_title(
            "1. Bağımlılıkları ekle",
            "Uygulama kabuğu için xenframe, widget API'si için xengui kullanılır.",
        ))
        .child(code_block(
            "Cargo.toml",
            "[dependencies]\nxengui = \"0.2.8\"\nxenframe = \"0.1.2\"\nxengui-wgpu = \"0.1.2\"",
        ))
        .child(section_title(
            "2. İlk pencere",
            "Render closure her state değişiminde yeni widget ağacını üretir.",
        ))
        .child(code_block(
            "src/main.rs",
            "use xenframe::{App, AppConfig};\nuse xengui::*;\n\nfn main() -> Result<(), Box<dyn std::error::Error>> {\n    let mut app = App::new(AppConfig::default());\n    app.render(|| Box::new(\n        Column::new()\n            .padding(Edges::all(24.0))\n            .gap(0.0, 12.0)\n            .child(Label::new().label(\"Merhaba, XenGui!\"))\n            .child(Button::new().label(\"Devam et\"))\n    ));\n    app.run()?;\n    Ok(())\n}",
        ))
        .child(section_title(
            "3. Çalıştır",
            "Native uygulama Cargo, web uygulaması Trunk ile çalışır.",
        ))
        .child(code_block(
            "Terminal",
            "# Native\ncargo run\n\n# Web\nrustup target add wasm32-unknown-unknown\ntrunk serve --open",
        ))
}

fn concepts_page() -> View {
    Column::new()
        .gap(0.0, 28.0)
        .child(heading(
            "Temel kavramlar",
            "Basit veri akışı, öngörülebilir render",
            "State yeni bir ağaç üretir; reconciler kimlikleri eşleştirir; layout ve paint gerekli düğümlerde yenilenir.",
        ))
        .child(cards(&[
            ("Widget ağacı", "Her widget ölçüm, layout, paint ve input davranışını aynı Widget sözleşmesiyle sunar."),
            ("Kontrollü state", "use_state değeri ve setter'ı döndürür; form kontrolleri değişimi callback ile bildirir."),
            ("Reconciliation", "Key verilen kardeşler taşınsa bile state ve etkileşim kimliklerini korur."),
            ("Efekt yaşam döngüsü", "Cleanup yeniden çalışmadan önce ve component unmount olduğunda çağrılır."),
        ]))
        .child(section_title("State örneği", "Hook çağrı sırası koşulsuz ve kararlı olmalıdır."))
        .child(code_block(
            "Component state",
            "let (count, set_count) = use_state(0);\n\nButton::new()\n    .label(format!(\"Sayaç: {count}\"))\n    .on_click(move |_ctx| set_count.set(count + 1))",
        ))
        .child(section_title("Kalıcı liste kimliği", "Dinamik listelerde index yerine domain kimliği kullanın."))
        .child(code_block(
            "Keyed list",
            "for item in items {\n    list = list.child(\n        Row::new()\n            .key(item.id.to_string())\n            .child(Label::new().label(item.title))\n    );\n}",
        ))
}

fn demo(title: &str, description: &str, child: impl Widget + 'static) -> View {
    Column::new()
        .flex_basis(Responsive::new(pct!(100.0)).lg(pct!(48.0)))
        .gap(0.0, 14.0)
        .padding(Edges::all(18.0))
        .background(|theme: &Theme| theme.surface)
        .border(|theme: &Theme| Border::all(1.0, theme.outline_variant).radius(12.0))
        .child(section_title(title, description))
        .child(child)
}

fn widgets_page(
    enabled: bool,
    set_enabled: SetState<bool>,
    checked: bool,
    set_checked: SetState<bool>,
    progress: f32,
    set_progress: SetState<f32>,
) -> View {
    View::new()
        .display(Display::Flex)
        .flex_direction(FlexDirection::Column)
        .gap(0.0, 28.0)
        .child(heading(
            "Widget kataloğu",
            "Temel yapı taşları",
            "Tema varsayılanlarıyla çalışan widget'ları builder API'siyle yerel olarak özelleştirebilirsin.",
        ))
        .child(
            View::new()
                .display(Display::Flex)
                .flex_wrap(FlexWrap::Wrap)
                .gap(14.0, 14.0)
                .child(demo(
                    "Button ve Badge",
                    "Aksiyon ve kısa durum bilgisi.",
                    Row::new()
                        .align_items(Align::Center)
                        .gap(10.0, 0.0)
                        .child(Button::new().label("Kaydet"))
                        .child(Badge::new().label("Yeni")),
                ))
                .child(demo(
                    "Switch ve Checkbox",
                    "Kontrollü boolean girdiler.",
                    Row::new()
                        .align_items(Align::Center)
                        .gap(16.0, 0.0)
                        .child(
                            Switch::new().checked(enabled).on_change(move |value, _ctx| {
                                set_enabled.set(value);
                            }),
                        )
                        .child(
                            Checkbox::new()
                                .checked(checked)
                                .on_change(move |value, _ctx| set_checked.set(value)),
                        ),
                ))
                .child(demo(
                    "ProgressBar",
                    "0 ile 1 arasında kontrollü ilerleme.",
                    Column::new()
                        .gap(0.0, 10.0)
                        .child(ProgressBar::new().value(progress))
                        .child(
                            Slider::new()
                                .value(progress)
                                .on_change(move |value, _ctx| set_progress.set(value)),
                        ),
                ))
                .child(demo(
                    "Separator",
                    "İçerik gruplarını görsel olarak ayırır.",
                    Column::new()
                        .gap(0.0, 10.0)
                        .child(Label::new().label("Hesap"))
                        .child(Separator::new())
                        .child(Label::new().label("Gizlilik")),
                ))
                .child(demo(
                    "TextBox",
                    "Placeholder, seçim, IME ve mobil input köprüsü.",
                    TextBox::new().placeholder("E-posta adresi"),
                ))
                .child(demo(
                    "Kbd ve Tooltip",
                    "Kısayol ve bağlamsal yardım.",
                    Tooltip::new("Komut paletini aç").child(Kbd::new().label("Ctrl K")),
                )),
        )
        .child(note(
            "Sıradaki input widget'ları",
            "Select/ComboBox ve çok satırlı TextArea; klavye, IME ve erişilebilirlik sözleşmesi birlikte tamamlandıktan sonra eklenmeli.",
        ))
}

fn styling_page() -> View {
    Column::new()
        .gap(0.0, 28.0)
        .child(heading(
            "Stil ve layout",
            "CSS'e yakın, Rust'a güvenli",
            "Length, Edges, flex, grid, tema tokenları ve responsive değerler aynı builder zincirinde birleşir.",
        ))
        .child(section_title("Tema tabanlı stil", "Closure aktif temayı render sırasında çözer."))
        .child(code_block(
            "Theme tokens",
            "View::new()\n    .padding(Edges::all(20.0))\n    .background(|theme: &Theme| theme.surface)\n    .color(|theme: &Theme| theme.on_surface)\n    .border(|theme: &Theme|\n        Border::all(1.0, theme.outline_variant).radius(theme.radius_lg)\n    )",
        ))
        .child(section_title("Responsive değerler", "Mobile-first değer breakpoint geldiğinde değişir."))
        .child(code_block(
            "Responsive layout",
            "View::new()\n    .padding(Responsive::new(Edges::all(16.0)).md(Edges::all(32.0)))\n    .width(Responsive::new(pct!(100.0)).lg(px!(960.0)))",
        ))
        .child(section_title("Grid", "Taffy tabanlı grid ve flex aynı layout ağacında birlikte kullanılabilir."))
        .child(code_block(
            "Grid layout",
            "View::new()\n    .display(Display::Grid)\n    .grid_template_columns(vec![GridTrack::Fr(1.0), GridTrack::Fr(1.0)])\n    .gap(16.0, 16.0)",
        ))
}

fn rendering_page() -> View {
    Column::new()
        .gap(0.0, 28.0)
        .child(heading(
            "Renderer",
            "Native wgpu, tarayıcıda WebGPU/WebGL",
            "xengui paint komutlarını platformdan bağımsız tutar; xengui-wgpu bunları GPU pipeline'larına dönüştürür.",
        ))
        .child(cards(&[
            ("Konservatif limitler", "Native downlevel ve WebGL2 limitleri adapter çözünürlüğüyle birleştirilir."),
            ("Dayanıklı surface", "Lost ve Outdated yeniden configure edilir; Timeout ve Occluded kontrollü sonuç döndürür."),
            ("Device recovery", "Recoverable GPU kaybında xenframe fontları koruyarak renderer'ı yeniden kurar."),
            ("Gerçek MSAA", "SVG üçgenleri adapter desteğine otomatik düşen özel MSAA target kullanır."),
        ]))
        .child(section_title("Renderer seçenekleri", "Varsayılan politika uyumluluk, vsync ve 4× triangle MSAA seçer."))
        .child(code_block(
            "Renderer configuration",
            "use xengui_wgpu::{PresentModePreference, RendererOptions, SampleCount};\n\nlet config = AppConfig {\n    renderer: RendererOptions {\n        present_mode: PresentModePreference::Vsync,\n        sample_count: SampleCount::X4,\n        ..Default::default()\n    },\n    ..Default::default()\n};",
        ))
}

fn api_card(name: &str, description: &str, href: &str) -> View {
    feature_card(name, description).child(
        Link::new()
            .label("Rustdoc'u aç →")
            .href(href)
            .target_blank(true)
            .font_size(13.0),
    )
}

fn api_page() -> View {
    Column::new()
        .gap(0.0, 28.0)
        .child(heading(
            "API referansı",
            "İmzalar doğrudan rustdoc'tan",
            "Paket yayınlandığında docs.rs kaynak koddan rustdoc'u yeniden üretir; website yalnızca öğrenme katmanını taşır.",
        ))
        .child(note(
            "Neden hibrit?",
            "Rustdoc tipler, metotlar ve trait sözleşmeleri için otoritedir. Website öğrenme sırası, mimari açıklama ve canlı örnekler için elle düzenlenir.",
        ))
        .child(
            View::new()
                .display(Display::Flex)
                .flex_wrap(FlexWrap::Wrap)
                .gap(14.0, 14.0)
                .child(api_card("xengui", "Widget, layout, style, input, hook ve paint çekirdeği.", "https://docs.rs/xengui/latest/xengui/"))
                .child(api_card("xenframe", "winit yaşam döngüsü ve platform entegrasyonu.", "https://docs.rs/xenframe/latest/xenframe/"))
                .child(api_card("xengui-wgpu", "GPU pipeline'ları, seçenekler ve frame sonuçları.", "https://docs.rs/xengui-wgpu/latest/xengui_wgpu/"))
                .child(api_card("xen-router", "Dosya tabanlı route ve navigation API'si.", "https://docs.rs/xen-router/latest/xen_router/"))
                .child(api_card("xen-svg", "SVG parse, transform ve tessellation API'si.", "https://docs.rs/xen-svg/latest/xen_svg/"))
                .child(api_card("xen-animation", "Transition, easing ve zamanlama API'si.", "https://docs.rs/xen-animation/latest/xen_animation/")),
        )
}

pub fn page(_params: &RouteParams) -> Box<dyn Widget> {
    let (active, set_active) = use_state(DocsSection::Start);
    // Demo hooks remain unconditional so switching documentation tabs never
    // changes the component's hook order.
    let (enabled, set_enabled) = use_state(true);
    let (checked, set_checked) = use_state(false);
    let (progress, set_progress) = use_state(0.64_f32);
    let compact = !responsive_bool(Breakpoint::Md, true);

    let mut navigation = View::new()
        .display(Display::Flex)
        .flex_direction(if compact {
            FlexDirection::Row
        } else {
            FlexDirection::Column
        })
        .width(if compact { pct!(100.0) } else { px!(220.0) })
        .gap(
            if compact { 8.0 } else { 0.0 },
            if compact { 0.0 } else { 6.0 },
        )
        .overflow_x(if compact {
            Overflow::Auto
        } else {
            Overflow::Visible
        });

    for (section, title, subtitle) in NAV_ITEMS {
        navigation = navigation.child(nav_button(
            active,
            *section,
            title,
            subtitle,
            set_active.clone(),
            compact,
        ));
    }

    let content = match active {
        DocsSection::Start => start_page(),
        DocsSection::Concepts => concepts_page(),
        DocsSection::Widgets => widgets_page(
            enabled,
            set_enabled,
            checked,
            set_checked,
            progress,
            set_progress,
        ),
        DocsSection::Styling => styling_page(),
        DocsSection::Rendering => rendering_page(),
        DocsSection::Api => api_page(),
    };

    let body = View::new()
        .display(Display::Flex)
        .flex_direction(if compact {
            FlexDirection::Column
        } else {
            FlexDirection::Row
        })
        .gap(
            if compact { 0.0 } else { 42.0 },
            if compact { 24.0 } else { 0.0 },
        )
        .width(pct!(100.0))
        .min_width(px!(0.0))
        .child(navigation)
        .child(
            View::new()
                .width(pct!(100.0))
                .flex_grow(1.0)
                .min_width(px!(0.0))
                .child(content),
        );

    Box::new(
        Column::new()
            .width(pct!(100.0))
            .min_width(px!(0.0))
            .gap(0.0, Responsive::new(px!(32.0)).md(px!(52.0)))
            .overflow_x(Overflow::Hidden)
            .padding(
                Responsive::new(Edges::only(20.0, 32.0, 20.0, 72.0))
                    .md(Edges::only(64.0, 48.0, 64.0, 88.0))
                    .lg(Edges::only(80.0, 56.0, 80.0, 104.0)),
            )
            .child(docs_hero(set_active))
            .child(body),
    )
}
