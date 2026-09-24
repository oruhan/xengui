// SPDX-License-Identifier: Apache-2.0

mod export;
mod trace;

use std::cell::Cell;
use std::rc::Rc;
use std::sync::Arc;
use trace::TraceStore;
#[cfg(not(target_arch = "wasm32"))]
use xenframe::WindowPosition;
use xenframe::{App, AppConfig, AppDiagnostic, AppThemeMode};
use xengui::*;
use xengui_icons::{IconAxes, codepoints};

fn icon(codepoint: char, size: f32, filled: bool) -> VariableIcon {
    VariableIcon::new(codepoint).size(size).axes(
        IconAxes::default()
            .fill(if filled { 1.0 } else { 0.0 })
            .weight(500.0)
            .optical_size(size),
    )
}

fn violet_theme(dark: bool) -> Theme {
    if dark {
        Theme::dark()
            .background(Color::rgb(15, 12, 20))
            .surface(Color::rgb(15, 12, 20))
            .surface_container_low(Color::rgb(23, 18, 31))
            .surface_container(Color::rgb(33, 25, 42))
            .surface_container_high(Color::rgb(45, 35, 56))
            .surface_container_highest(Color::rgb(57, 45, 70))
            .on_surface(Color::rgb(250, 247, 252))
            .on_surface_variant(Color::rgb(216, 203, 224))
            .outline(Color::rgb(168, 149, 180))
            .outline_variant(Color::rgb(78, 62, 88))
            .primary(Color::rgb(214, 176, 255))
            .on_primary(Color::rgb(56, 30, 114))
            .primary_container(Color::rgb(103, 58, 183))
            .on_primary_container(Color::rgb(250, 239, 255))
            .secondary(Color::rgb(204, 194, 220))
            .on_secondary(Color::rgb(51, 45, 65))
            .secondary_container(Color::rgb(74, 68, 88))
            .on_secondary_container(Color::rgb(232, 222, 248))
            .tertiary(Color::rgb(239, 184, 200))
            .on_tertiary(Color::rgb(73, 37, 50))
            .tertiary_container(Color::rgb(99, 59, 72))
            .on_tertiary_container(Color::rgb(255, 216, 228))
            .selection(Color::rgb(208, 188, 255).with_alpha(80))
            .caret_color(Color::rgb(208, 188, 255))
    } else {
        Theme::light()
            .background(Color::rgb(255, 247, 255))
            .surface(Color::rgb(255, 247, 255))
            .surface_container_low(Color::rgb(249, 239, 252))
            .surface_container(Color::rgb(243, 232, 247))
            .surface_container_high(Color::rgb(235, 221, 241))
            .surface_container_highest(Color::rgb(225, 207, 233))
            .on_surface(Color::rgb(31, 25, 34))
            .on_surface_variant(Color::rgb(75, 65, 80))
            .outline(Color::rgb(117, 101, 125))
            .outline_variant(Color::rgb(212, 194, 220))
            .primary(Color::rgb(103, 80, 164))
            .on_primary(Color::WHITE)
            .primary_container(Color::rgb(234, 221, 255))
            .on_primary_container(Color::rgb(33, 0, 93))
            .secondary(Color::rgb(98, 91, 113))
            .on_secondary(Color::WHITE)
            .secondary_container(Color::rgb(232, 222, 248))
            .on_secondary_container(Color::rgb(29, 25, 43))
            .tertiary(Color::rgb(125, 82, 96))
            .on_tertiary(Color::WHITE)
            .tertiary_container(Color::rgb(255, 216, 228))
            .on_tertiary_container(Color::rgb(49, 17, 29))
            .selection(Color::rgb(103, 80, 164).with_alpha(72))
            .caret_color(Color::rgb(103, 80, 164))
    }
}

fn blue_theme(dark: bool) -> Theme {
    if dark {
        Theme::dark()
            .background(Color::rgb(8, 14, 24))
            .surface(Color::rgb(8, 14, 24))
            .surface_container_low(Color::rgb(14, 24, 39))
            .surface_container(Color::rgb(21, 34, 52))
            .surface_container_high(Color::rgb(30, 47, 69))
            .surface_container_highest(Color::rgb(40, 61, 86))
            .on_surface(Color::rgb(246, 249, 255))
            .on_surface_variant(Color::rgb(202, 216, 238))
            .outline(Color::rgb(142, 164, 196))
            .outline_variant(Color::rgb(54, 75, 103))
            .primary(Color::rgb(138, 180, 255))
            .on_primary(Color::rgb(0, 46, 105))
            .primary_container(Color::rgb(0, 87, 194))
            .on_primary_container(Color::rgb(234, 241, 255))
            .secondary(Color::rgb(190, 198, 220))
            .on_secondary(Color::rgb(40, 49, 65))
            .secondary_container(Color::rgb(62, 71, 89))
            .on_secondary_container(Color::rgb(218, 226, 249))
            .tertiary(Color::rgb(221, 188, 224))
            .on_tertiary(Color::rgb(62, 40, 65))
            .tertiary_container(Color::rgb(86, 62, 88))
            .on_tertiary_container(Color::rgb(250, 216, 252))
            .selection(Color::rgb(173, 198, 255).with_alpha(80))
            .caret_color(Color::rgb(173, 198, 255))
    } else {
        Theme::light()
            .background(Color::rgb(247, 249, 255))
            .surface(Color::rgb(247, 249, 255))
            .surface_container_low(Color::rgb(238, 244, 255))
            .surface_container(Color::rgb(229, 238, 252))
            .surface_container_high(Color::rgb(216, 229, 247))
            .surface_container_highest(Color::rgb(202, 218, 240))
            .on_surface(Color::rgb(18, 27, 40))
            .on_surface_variant(Color::rgb(58, 70, 88))
            .outline(Color::rgb(91, 108, 133))
            .outline_variant(Color::rgb(184, 201, 225))
            .primary(Color::rgb(0, 82, 204))
            .on_primary(Color::WHITE)
            .primary_container(Color::rgb(216, 226, 255))
            .on_primary_container(Color::rgb(0, 26, 65))
            .secondary(Color::rgb(86, 94, 113))
            .on_secondary(Color::WHITE)
            .secondary_container(Color::rgb(218, 226, 249))
            .on_secondary_container(Color::rgb(19, 27, 43))
            .tertiary(Color::rgb(112, 85, 117))
            .on_tertiary(Color::WHITE)
            .tertiary_container(Color::rgb(250, 216, 252))
            .on_tertiary_container(Color::rgb(40, 19, 45))
            .selection(Color::rgb(0, 90, 193).with_alpha(72))
            .caret_color(Color::rgb(0, 90, 193))
    }
}

fn green_theme(dark: bool) -> Theme {
    if dark {
        Theme::dark()
            .background(Color::rgb(8, 18, 8))
            .surface(Color::rgb(8, 18, 8))
            .surface_container_low(Color::rgb(14, 29, 13))
            .surface_container(Color::rgb(21, 41, 19))
            .surface_container_high(Color::rgb(31, 56, 28))
            .surface_container_highest(Color::rgb(42, 72, 37))
            .on_surface(Color::rgb(246, 255, 241))
            .on_surface_variant(Color::rgb(205, 225, 196))
            .outline(Color::rgb(143, 174, 131))
            .outline_variant(Color::rgb(57, 84, 50))
            .primary(Color::rgb(146, 227, 95))
            .on_primary(Color::rgb(12, 57, 0))
            .primary_container(Color::rgb(46, 125, 18))
            .on_primary_container(Color::rgb(237, 255, 227))
            .secondary(Color::rgb(188, 203, 177))
            .on_secondary(Color::rgb(39, 52, 34))
            .secondary_container(Color::rgb(61, 75, 55))
            .on_secondary_container(Color::rgb(216, 232, 204))
            .tertiary(Color::rgb(160, 207, 209))
            .on_tertiary(Color::rgb(0, 55, 57))
            .tertiary_container(Color::rgb(31, 78, 80))
            .on_tertiary_container(Color::rgb(188, 235, 237))
            .selection(Color::rgb(156, 214, 125).with_alpha(80))
            .caret_color(Color::rgb(156, 214, 125))
    } else {
        Theme::light()
            .background(Color::rgb(248, 255, 244))
            .surface(Color::rgb(248, 255, 244))
            .surface_container_low(Color::rgb(238, 249, 232))
            .surface_container(Color::rgb(228, 242, 221))
            .surface_container_high(Color::rgb(215, 232, 206))
            .surface_container_highest(Color::rgb(201, 221, 191))
            .on_surface(Color::rgb(21, 31, 17))
            .on_surface_variant(Color::rgb(61, 77, 54))
            .outline(Color::rgb(93, 118, 83))
            .outline_variant(Color::rgb(184, 205, 174))
            .primary(Color::rgb(56, 106, 32))
            .on_primary(Color::WHITE)
            .primary_container(Color::rgb(183, 243, 151))
            .on_primary_container(Color::rgb(4, 33, 0))
            .secondary(Color::rgb(85, 99, 77))
            .on_secondary(Color::WHITE)
            .secondary_container(Color::rgb(216, 232, 204))
            .on_secondary_container(Color::rgb(19, 31, 14))
            .tertiary(Color::rgb(56, 102, 104))
            .on_tertiary(Color::WHITE)
            .tertiary_container(Color::rgb(188, 235, 237))
            .on_tertiary_container(Color::rgb(0, 31, 32))
            .selection(Color::rgb(56, 106, 32).with_alpha(72))
            .caret_color(Color::rgb(56, 106, 32))
    }
}

fn themes() -> Vec<Theme> {
    vec![
        violet_theme(false),
        violet_theme(true),
        blue_theme(false),
        blue_theme(true),
        green_theme(false),
        green_theme(true),
    ]
}

fn bg() -> Color {
    current_theme().surface
}
fn surface() -> Color {
    current_theme().surface_container
}
fn surface_high() -> Color {
    current_theme().surface_container_high
}
fn outline() -> Color {
    current_theme().outline_variant
}
fn text() -> Color {
    current_theme().on_surface
}
fn muted() -> Color {
    current_theme().on_surface_variant
}
fn primary() -> Color {
    current_theme().primary
}
fn primary_container() -> Color {
    current_theme().primary_container
}
fn on_primary_container() -> Color {
    current_theme().on_primary_container
}
fn green() -> Color {
    current_theme().success
}
fn red() -> Color {
    current_theme().error
}
fn cyan() -> Color {
    current_theme().tertiary
}

fn state_layer(base: Color, content: Color, opacity: f32) -> Color {
    let opacity = opacity.clamp(0.0, 1.0);
    Color::rgba_f32(
        base.r() * (1.0 - opacity) + content.r() * opacity,
        base.g() * (1.0 - opacity) + content.g() * opacity,
        base.b() * (1.0 - opacity) + content.b() * opacity,
        1.0,
    )
}

fn card() -> View {
    View::new()
        .display(Display::Flex)
        .flex_direction(FlexDirection::Column)
        .width(pct!(100.0))
        .min_width(px!(0.0))
        .padding(Edges::all(16.0))
        .gap(0.0, 12.0)
        .background(surface())
        .color(text())
        .border(Border::all(1.0, outline()).radius(16.0))
}

fn heading(title: impl Into<smol_str::SmolStr>, detail: impl Into<smol_str::SmolStr>) -> View {
    View::new()
        .display(Display::Flex)
        .flex_direction(FlexDirection::Column)
        .gap(0.0, 3.0)
        .child(
            Label::new()
                .label(title)
                .font_size(px!(18.0))
                .font_weight(FontWeight::SemiBold)
                .color(text()),
        )
        .child(
            Label::new()
                .label(detail)
                .font_size(px!(13.0))
                .color(muted()),
        )
}

fn action_button(label: &'static str, on_click: impl FnMut(&mut EventCtx) + 'static) -> Button {
    Button::new()
        .label(label)
        .height(px!(44.0))
        .padding(Edges::symmetric(16.0, 0.0))
        .background(primary())
        .color(current_theme().on_primary)
        .border(Border::none().radius(22.0))
        .font_weight(FontWeight::SemiBold)
        .on_click(on_click)
}

fn issue_button(store: TraceStore, area: &'static str) -> Button {
    Button::new()
        .label("Görsel sorun işaretle")
        .height(px!(40.0))
        .padding(Edges::symmetric(14.0, 0.0))
        .background(current_theme().error_container)
        .color(current_theme().on_error_container)
        .border(Border::none().radius(20.0))
        .on_click(move |_| {
            store.manual_issue(
                area,
                "Kullanıcı bu laboratuvar bölümünde görsel/etkileşim sorunu işaretledi",
            );
        })
}

fn metric(label: &'static str, value: String, color: Color) -> View {
    card()
        .flex_grow(1.0)
        .min_width(px!(140.0))
        .child(
            Label::new()
                .label(value)
                .font_size(px!(25.0))
                .font_weight(FontWeight::Bold)
                .color(color),
        )
        .child(
            Label::new()
                .label(label)
                .font_size(px!(11.0))
                .color(muted()),
        )
}

fn overview(store: &TraceStore) -> View {
    let (passed, failed, manual) = store.counts();
    let store_for_issue = store.clone();
    let inventory = trace::WIDGET_INVENTORY.join(" · ");

    let metric_columns = if current_breakpoint() == Breakpoint::Compact {
        vec![GridTrack::Fr(1.0), GridTrack::Fr(1.0)]
    } else {
        vec![
            GridTrack::Fr(1.0),
            GridTrack::Fr(1.0),
            GridTrack::Fr(1.0),
            GridTrack::Fr(1.0),
        ]
    };

    View::new()
        .display(Display::Flex)
        .flex_direction(FlexDirection::Column)
        .gap(0.0, 16.0)
        .child(
            View::new()
                .display(Display::Grid)
                .grid_template_columns(metric_columns)
                .gap(12.0, 12.0)
                .child(metric("otomatik geçti", passed.to_string(), green()))
                .child(metric("başarısız", failed.to_string(), red()))
                .child(metric("manuel inceleme", manual.to_string(), cyan()))
                .child(metric(
                    "widget ailesi",
                    trace::WIDGET_INVENTORY.len().to_string(),
                    primary(),
                )),
        )
        .child(
            card()
                .background(primary_container())
                .border(Border::all(1.0, primary().with_alpha(80)).radius(28.0))
                .padding(Edges::all(24.0))
                .child(
                    Label::new()
                        .label("Tek uygulama. Üç hedef. Bir teşhis dosyası.")
                        .font_size(px!(28.0))
                        .line_height(px!(34.0))
                        .font_weight(FontWeight::Bold)
                        .color(on_primary_container()),
                )
                .child(
                    Label::new()
                        .label("Her etkileşim trace'e eklenir. Otomatik smoke kontrolleri, runtime GPU tanıları, kullanıcı işaretleri ve widget render günlüğü tek JSON içinde AI'ye aktarılır.")
                        .max_width(px!(760.0))
                        .line_height(px!(20.0))
                        .color(on_primary_container().with_alpha(200)),
                )
                .child(issue_button(store_for_issue, "overview")),
        )
        .child(
            card()
                .child(heading("Kapsam envanteri", "Trace her public widget ailesini isimle taşır"))
                .child(
                    Label::new()
                        .label(inventory)
                        .width(pct!(100.0))
                        .line_height(px!(21.0))
                        .color(muted()),
                ),
        )
}

fn controls_lab(store: &TraceStore) -> View {
    let (checked, set_checked) = use_state(true);
    let (switched, set_switched) = use_state(false);
    let (radio, set_radio) = use_state(0usize);
    let (slider, set_slider) = use_state(0.42_f32);
    let (text, set_text) = use_state(String::new());
    let (clicks, set_clicks) = use_state(0usize);

    let click_store = store.clone();
    let checkbox_store = store.clone();
    let switch_store = store.clone();
    let radio_store = store.clone();
    let slider_store = store.clone();
    let text_store = store.clone();

    let mut radios = View::new()
        .display(Display::Flex)
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .gap(12.0, 0.0);
    for index in 0..3 {
        let setter = set_radio.clone();
        let event_store = radio_store.clone();
        radios = radios.child(
            View::new()
                .display(Display::Flex)
                .flex_direction(FlexDirection::Row)
                .align_items(Align::Center)
                .gap(8.0, 0.0)
                .child(
                    RadioButton::new()
                        .selected(radio == index)
                        .accessible_label(format!("Seçenek {}", index + 1))
                        .on_select(move |_| {
                            setter.set(index);
                            event_store.event("change", "RadioButton", format!("selected={index}"));
                            event_store.check(
                                "behavior.radio",
                                "controls",
                                true,
                                "radio selection callback fired",
                            );
                        }),
                )
                .child(Label::new().label(format!("R{}", index + 1)).color(muted())),
        );
    }

    card()
        .child(heading(
            "Kontroller",
            "Pointer, klavye, focus, controlled state ve IME",
        ))
        .child(
            View::new()
                .display(Display::Flex)
                .flex_direction(FlexDirection::Row)
                .align_items(Align::Center)
                .gap(12.0, 0.0)
                .flex_wrap(FlexWrap::Wrap)
                .child(action_button("Tıkla", move |_| {
                    set_clicks.set(clicks + 1);
                    click_store.event("activate", "Button", format!("count={}", clicks + 1));
                    click_store.check("behavior.button", "controls", true, "button callback fired");
                }))
                .child(Badge::new().label(format!("{clicks} tıklama")))
                .child(Tooltip::new("Klavye ipucu ortalanmalı").child(Kbd::new().label("Ctrl K")))
                .child(
                    Link::new()
                        .label("xengui.dev")
                        .href("https://xengui.dev")
                        .target_blank(true),
                ),
        )
        .child(
            View::new()
                .display(Display::Flex)
                .flex_direction(FlexDirection::Row)
                .align_items(Align::Center)
                .gap(18.0, 0.0)
                .flex_wrap(FlexWrap::Wrap)
                .child(
                    Checkbox::new()
                        .checked(checked)
                        .accessible_label("Checkbox probe")
                        .on_change(move |value, _| {
                            set_checked.set(value);
                            checkbox_store.event("change", "Checkbox", format!("checked={value}"));
                            checkbox_store.check(
                                "behavior.checkbox",
                                "controls",
                                true,
                                "checkbox toggled",
                            );
                        }),
                )
                .child(
                    Switch::new()
                        .checked(switched)
                        .accessible_label("Switch probe")
                        .on_change(move |value, _| {
                            set_switched.set(value);
                            switch_store.event("change", "Switch", format!("checked={value}"));
                            switch_store.check(
                                "behavior.switch",
                                "controls",
                                true,
                                "switch toggled",
                            );
                        }),
                )
                .child(radios),
        )
        .child(
            TextBox::new()
                .value(text)
                .placeholder("IME, seçim, kopyala/yapıştır ve submit dene…")
                .accessible_label("TextBox behavior probe")
                .max_length(160)
                .width(pct!(100.0))
                .height(px!(48.0))
                .padding(Edges::symmetric(14.0, 0.0))
                .background(surface_high())
                .border(Border::all(1.0, outline()).radius(12.0))
                .on_change(move |value, _| {
                    set_text.set(value.to_string());
                    text_store.event(
                        "input",
                        "TextBox",
                        format!("chars={}", value.chars().count()),
                    );
                    text_store.check("behavior.textbox", "controls", true, "text input changed");
                }),
        )
        .child(
            View::new()
                .display(Display::Flex)
                .flex_direction(FlexDirection::Column)
                .gap(0.0, 8.0)
                .child(
                    Label::new()
                        .label(format!("Slider: {:.2}", slider))
                        .color(muted()),
                )
                .child(
                    Slider::new()
                        .value(slider)
                        .accessible_label("Slider behavior probe")
                        .on_change(move |value, _| {
                            set_slider.set(value);
                            slider_store.event("change", "Slider", format!("value={value:.3}"));
                            slider_store.check(
                                "behavior.slider",
                                "controls",
                                true,
                                "slider changed",
                            );
                        }),
                )
                .child(ProgressBar::new().value(slider).bar_height(px!(6.0))),
        )
        .child(issue_button(store.clone(), "controls"))
}

fn content_lab(store: &TraceStore) -> View {
    let image = image_source_from_rgba8(
        vec![
            184, 160, 255, 255, 104, 219, 230, 255, 110, 224, 157, 255, 255, 190, 90, 255,
        ],
        2,
        2,
    )
    .ok();
    let svg = Svg::from_string(
        r##"<svg viewBox="0 0 64 64"><circle cx="32" cy="32" r="28" fill="#B8A0FF"/><path d="M18 33 L28 43 L47 21" fill="none" stroke="#241448" stroke-width="6"/></svg>"##,
    )
    .ok();

    let table = Table::new()
        .column(TableColumn::new("Widget", pct!(45.0)))
        .column(TableColumn::new("Durum", pct!(30.0)))
        .column(TableColumn::new("Hedef", pct!(25.0)))
        .row(
            TableRow::new()
                .text("TextBox")
                .text("interactive")
                .text("all"),
        )
        .row(TableRow::new().text("Tooltip").text("visual").text("all"))
        .row(TableRow::new().text("View").text("stress").text("all"))
        .striped(true)
        .border_color(outline());

    let context_store = store.clone();
    let context = ContextMenu::new()
        .child(
            View::new()
                .height(px!(76.0))
                .padding(Edges::all(14.0))
                .background(surface_high())
                .border(Border::all(1.0, outline()).radius(12.0))
                .child(
                    Label::new()
                        .label("Sağ tık / uzun bas: ContextMenu")
                        .color(muted()),
                ),
        )
        .item(ContextMenuItem::new("Trace'e kaydet").on_click(move |_| {
            context_store.event("activate", "ContextMenu", "menu item selected");
            context_store.check(
                "behavior.context-menu",
                "content",
                true,
                "context menu item selected",
            );
        }))
        .divider()
        .item(ContextMenuItem::new("Pasif öğe").enabled(false));

    let mut media = View::new()
        .display(Display::Flex)
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .gap(14.0, 0.0)
        .flex_wrap(FlexWrap::Wrap);
    if let Some(source) = image {
        media = media.child(
            Image::new()
                .source(source)
                .object_fit(ObjectFit::Cover)
                .size(px!(84.0), px!(84.0))
                .border(Border::none().radius(16.0))
                .accessible_label("RGBA image probe"),
        );
    }
    if let Some(svg) = svg {
        media = media.child(
            View::new()
                .size(px!(84.0), px!(84.0))
                .accessible_label("SVG probe")
                .child(svg.size(pct!(100.0), pct!(100.0))),
        );
    }

    View::new()
        .display(Display::Flex)
        .flex_direction(FlexDirection::Column)
        .width(pct!(100.0))
        .min_width(px!(0.0))
        .gap(0.0, 16.0)
        .child(
            card()
                .child(heading(
                    "Metin ve medya",
                    "Seçim sınırları, wrapping, asset ve syntax paint",
                ))
                .child(
                    Label::new()
                        .label("Label seçimi yalnız gerçek glyph alanında bitmeli.")
                        .selectable(true),
                )
                .child(
                    RichText::new()
                        .selectable(true)
                        .span(
                            TextSpan::new("RichText ")
                                .weight(FontWeight::Bold)
                                .color(primary()),
                        )
                        .span("çok satırlı seçim ve boşluk hit-test davranışını ")
                        .span(TextSpan::new("doğrular.").color(cyan())),
                )
                .child(Separator::new())
                .child(media)
                .child(
                    CodeBlock::new("fn main() {\n    println!(\"XenGui benchmark\");\n}")
                        .label("Rust")
                        .language(CodeLanguage::Rust)
                        .copy_label("Kopyala")
                        .copied_label("Kopyalandı")
                        .border(Border::none()),
                )
                .child(issue_button(store.clone(), "text-media")),
        )
        .child(
            card()
                .child(heading("Table", "Kolon ölçümü, satır boyama ve hover"))
                .child(table),
        )
        .child(
            card()
                .child(heading("ContextMenu", "Sağ tık ve dokunmatik uzun basma"))
                .child(context),
        )
        .child(
            card()
                .child(heading("Portal", "Normal akış dışı overlay paint/hit-test"))
                .child(
                    Portal::new().child(
                        View::new()
                            .padding(Edges::symmetric(12.0, 8.0))
                            .background(primary_container())
                            .border(Border::none().radius(12.0))
                            .child(
                                Label::new()
                                    .label("Portal içeriği")
                                    .color(on_primary_container()),
                            ),
                    ),
                ),
        )
}

fn navigation_lab(store: &TraceStore) -> View {
    let (active, set_active) = use_state(0usize);
    let nav_store = store.clone();
    let nav = NavigationBar::new()
        .item(NavItem::new(codepoints::HOME, "Home"))
        .item(NavItem::new(codepoints::WIDGETS, "Widgets"))
        .item(NavItem::new(codepoints::SPEED, "Perf"))
        .active_index(active)
        .on_select(move |index| {
            set_active.set(index);
            nav_store.event("navigate", "NavigationBar", format!("active={index}"));
            nav_store.check(
                "behavior.navigation",
                "navigation",
                true,
                "destination changed",
            );
        });

    let compact = current_breakpoint() == Breakpoint::Compact;
    let panel_size = if compact { 76.0 } else { 190.0 };
    let panel_min = if compact { 56.0 } else { 120.0 };
    let (left_size, _) = use_state(Rc::new(Cell::new(panel_size)));
    let (right_size, _) = use_state(Rc::new(Cell::new(panel_size)));
    let split = split_pane(
        Some(
            SplitPanel::new(
                View::new()
                    .background(surface_high())
                    .color(text())
                    .padding(Edges::all(12.0))
                    .child(Label::new().label("Sol panel")),
                left_size,
            )
            .min_size(panel_min),
        ),
        View::new()
            .min_width(px!(0.0))
            .background(primary_container())
            .color(on_primary_container())
            .padding(Edges::all(12.0))
            .child(Label::new().label("Esnek merkez")),
        Some(
            SplitPanel::new(
                View::new()
                    .background(surface_high())
                    .color(text())
                    .padding(Edges::all(12.0))
                    .child(Label::new().label("Sağ panel")),
                right_size,
            )
            .min_size(panel_min),
        ),
    )
    .height(px!(180.0));

    View::new()
        .display(Display::Flex)
        .flex_direction(FlexDirection::Column)
        .gap(0.0, 16.0)
        .child(
            card()
                .child(heading("NavigationBar", "Seçim, blur ve adaptif genişlik"))
                .child(nav),
        )
        .child(
            card()
                .child(heading(
                    "SplitPane",
                    "Handle sürükleme ve min/max sınırları",
                ))
                .child(split),
        )
        .child(issue_button(store.clone(), "navigation-layout"))
}

fn stress_lab(store: &TraceStore) -> View {
    let (nodes, set_nodes) = use_state(120usize);
    let (generation, set_generation) = use_state(0usize);
    let store_for_effect = store.clone();
    use_effect(
        move || {
            store_for_effect.event(
                "commit",
                "stress-grid",
                format!("nodes={nodes}; generation={generation}"),
            );
            store_for_effect.check(
                "stress.commit",
                "performance",
                true,
                format!("committed {nodes} nodes"),
            );
        },
        [nodes, generation],
    );

    let mut grid = View::new()
        .display(Display::Grid)
        .grid_template_columns(vec![
            GridTrack::Fr(1.0),
            GridTrack::Fr(1.0),
            GridTrack::Fr(1.0),
        ])
        .gap(8.0, 8.0);
    for index in 0..nodes {
        let accented = (index + generation) % 2 != 0;
        grid = grid.child(
            View::new()
                .key(format!("stress-{index}"))
                .height(px!(34.0))
                .align_items(Align::Center)
                .padding(Edges::symmetric(8.0, 0.0))
                .background(if accented {
                    primary_container()
                } else {
                    surface_high()
                })
                .color(if accented {
                    on_primary_container()
                } else {
                    text()
                })
                .border(Border::none().radius(8.0))
                .child(
                    Label::new()
                        .label(format!("node {index}"))
                        .font_size(px!(10.0)),
                ),
        );
    }

    let store_500 = store.clone();
    let store_pulse = store.clone();
    let set_nodes_small = set_nodes.clone();
    card()
        .child(heading(
            "Layout + paint stresi",
            "Keyed reconciliation, grid, scroll ve repaint churn",
        ))
        .child(
            View::new()
                .display(Display::Flex)
                .flex_direction(FlexDirection::Row)
                .gap(10.0, 0.0)
                .flex_wrap(FlexWrap::Wrap)
                .child(action_button("120 node", move |_| set_nodes_small.set(120)))
                .child(action_button("600 node", move |_| {
                    store_500.event("stress", "grid", "requested 600 nodes");
                    set_nodes.set(600);
                }))
                .child(action_button("Repaint pulse", move |_| {
                    store_pulse.event("stress", "grid", format!("pulse={}", generation + 1));
                    set_generation.set(generation + 1);
                })),
        )
        .child(
            View::new()
                .height(px!(420.0))
                .overflow_y(Overflow::Auto)
                .padding(Edges::all(8.0))
                .background(bg())
                .border(Border::all(1.0, outline()).radius(12.0))
                .child(grid),
        )
        .child(issue_button(store.clone(), "stress"))
}

fn trace_lab(
    store: &TraceStore,
    export_message: String,
    set_export_message: SetState<String>,
) -> View {
    let export_store = store.clone();
    let copy_store = store.clone();
    let copy_setter = set_export_message.clone();
    card()
        .child(heading("AI trace", "Tek JSON: ortam + kontroller + olaylar + runtime tanıları + render log"))
        .child(
            RichText::new()
                .span(TextSpan::new("Akış: ").weight(FontWeight::Bold).color(primary()))
                .span("sorunu yeniden üret → görsel sorun işaretle → trace'i dışa aktar → dosyayı AI'ye ekle."),
        )
        .child(
            View::new()
                .display(Display::Flex)
                .flex_direction(FlexDirection::Row)
                .gap(10.0, 0.0)
                .flex_wrap(FlexWrap::Wrap)
                .child(action_button("Trace dışa aktar", move |ctx| {
                    match export_store.json() {
                        Ok(json) => {
                            let _ = ctx.platform_services().clipboard().write_text(json.clone());
                            let message = export::export(&json).unwrap_or_else(|error| format!("Dışa aktarma hatası: {error}"));
                            export_store.event("export", "trace", &message);
                            set_export_message.set(message);
                        }
                        Err(error) => set_export_message.set(format!("JSON hatası: {error}")),
                    }
                }))
                .child(
                    Button::new()
                        .label("JSON'u panoya kopyala")
                        .height(px!(44.0))
                        .padding(Edges::symmetric(16.0, 0.0))
                        .background(surface_high())
                        .color(text())
                        .border(Border::all(1.0, outline()).radius(22.0))
                        .on_click(move |ctx| match copy_store.json() {
                            Ok(json) => {
                                let result = ctx.platform_services().clipboard().write_text(json);
                                copy_setter.set(if result.is_ok() { "Trace panoya kopyalandı".into() } else { "Pano kullanılamıyor".into() });
                            }
                            Err(error) => copy_setter.set(format!("JSON hatası: {error}")),
                        }),
                ),
        )
        .child(
            Label::new()
                .label(if export_message.is_empty() { "Henüz dump alınmadı".to_string() } else { export_message })
                .font_size(px!(12.0))
                .color(cyan()),
        )
        .child(
            CodeBlock::new(
                r#"{
  "schema": "xengui.ai-trace/v1",
  "summary": { "passed": 0, "failed": 0 },
  "checks": [], "events": [], "render_log": []
}"#,
            )
            .label("Trace şeması")
            .language(CodeLanguage::Json)
            .border(Border::none()),
        )
}

fn root(store: TraceStore) -> Box<dyn Widget> {
    let (section, set_section) = use_state(0usize);
    let (export_message, set_export_message) = use_state(String::new());
    let (color_theme, set_color_theme) = use_state(0usize);
    let (dark_mode, set_dark_mode) = use_state(true);
    let breakpoint = current_breakpoint();
    store.environment(viewport_size(), format!("{breakpoint:?}"));

    let sections = [
        ("Özet", codepoints::SPACE_DASHBOARD),
        ("Kontroller", codepoints::TOUCH_APP),
        ("İçerik", codepoints::WIDGETS),
        ("Yerleşim", codepoints::ACCOUNT_TREE),
        ("Stres", codepoints::SPEED),
        ("Trace", codepoints::DATA_OBJECT),
    ];

    let compact = breakpoint == Breakpoint::Compact;
    let sidebar_width = match breakpoint {
        Breakpoint::Compact => 0.0,
        Breakpoint::Medium => 196.0,
        Breakpoint::Expanded => 232.0,
        Breakpoint::Large => 264.0,
        Breakpoint::ExtraLarge => 288.0,
    };
    let mut nav = View::new()
        .width(pct!(100.0))
        .min_width(px!(0.0))
        .display(Display::Flex)
        .flex_direction(if compact {
            FlexDirection::Row
        } else {
            FlexDirection::Column
        })
        .gap(
            if compact { 8.0 } else { 0.0 },
            if compact { 0.0 } else { 8.0 },
        )
        .overflow_x(if compact {
            Overflow::Auto
        } else {
            Overflow::Visible
        });
    for (index, (label, codepoint)) in sections.into_iter().enumerate() {
        let setter = set_section.clone();
        let selected = section == index;
        nav = nav.child(
            View::new()
                .width(if compact { px!(128.0) } else { pct!(100.0) })
                .min_width(if compact { px!(128.0) } else { px!(0.0) })
                .height(px!(56.0))
                .display(Display::Flex)
                .flex_direction(FlexDirection::Row)
                .align_items(Align::Center)
                .gap(10.0, 0.0)
                .padding(Edges::symmetric(16.0, 0.0))
                .background(if selected {
                    current_theme().secondary_container
                } else {
                    Color::TRANSPARENT
                })
                .color(if selected {
                    current_theme().on_secondary_container
                } else {
                    muted()
                })
                .border(Border::none().radius(28.0))
                .hover_style(move |style: StylePatch, theme: &Theme| {
                    style.background(if selected {
                        state_layer(
                            theme.secondary_container,
                            theme.on_secondary_container,
                            0.08,
                        )
                    } else {
                        state_layer(theme.surface_container, theme.on_surface, 0.08)
                    })
                })
                .pressed_style(|style: StylePatch, _theme: &Theme| {
                    style.scale(0.96).content_scale(1.0)
                })
                .transition_colors(
                    Transition::new(std::time::Duration::from_millis(150))
                        .easing(Easing::cubic_bezier(0.31, 0.94, 0.34, 1.0)),
                )
                .transition_transform(
                    Transition::new(std::time::Duration::from_millis(350))
                        .easing(Easing::cubic_bezier(0.42, 1.67, 0.21, 0.90)),
                )
                .accessible_label(label)
                .on_click(move |_| setter.set(index))
                .child(icon(codepoint, 20.0, selected))
                .child(
                    Label::new()
                        .label(label)
                        .font_size(px!(14.0))
                        .font_weight(FontWeight::Medium),
                ),
        );
    }

    let content: Box<dyn Widget> = match section {
        1 => {
            let page_store = store.clone();
            Box::new(component("benchmark-controls", || {
                controls_lab(&page_store)
            }))
        }
        2 => {
            let page_store = store.clone();
            Box::new(component("benchmark-content", || content_lab(&page_store)))
        }
        3 => {
            let page_store = store.clone();
            Box::new(component("benchmark-navigation", || {
                navigation_lab(&page_store)
            }))
        }
        4 => {
            let page_store = store.clone();
            Box::new(component("benchmark-stress", || stress_lab(&page_store)))
        }
        5 => {
            let page_store = store.clone();
            Box::new(component("benchmark-trace", || {
                trace_lab(&page_store, export_message, set_export_message)
            }))
        }
        _ => Box::new(overview(&store)),
    };

    let color_setter = set_color_theme.clone();
    let color_store = store.clone();
    let mode_store = store.clone();
    let theme_controls = View::new()
        .display(Display::Flex)
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .gap(12.0, 0.0)
        .child(
            View::new().width(px!(184.0)).child(
                ComboBox::new(["Violet", "Blue", "Green"])
                    .label("Renk teması")
                    .selected_index(color_theme)
                    .on_change(move |index| {
                        color_setter.set(index);
                        set_active_theme(index * 2 + usize::from(dark_mode));
                        color_store.event("change", "ComboBox", format!("color_theme={index}"));
                        color_store.check(
                            "behavior.combo-box",
                            "theme",
                            true,
                            "color theme selection changed",
                        );
                    }),
            ),
        )
        .child(
            View::new()
                .display(Display::Flex)
                .flex_direction(FlexDirection::Row)
                .align_items(Align::Center)
                .gap(8.0, 0.0)
                .child(
                    Label::new()
                        .label("Koyu")
                        .font_size(px!(12.0))
                        .color(muted()),
                )
                .child(
                    Switch::new()
                        .checked(dark_mode)
                        .accessible_label("Koyu tema")
                        .on_change(move |checked, _| {
                            set_dark_mode.set(checked);
                            set_active_theme(color_theme * 2 + usize::from(checked));
                            mode_store.event("change", "Switch", format!("dark_mode={checked}"));
                            mode_store.check(
                                "behavior.theme-mode",
                                "theme",
                                true,
                                "light/dark mode changed",
                            );
                        }),
                ),
        );

    let header = View::new()
        .display(Display::Flex)
        .flex_direction(FlexDirection::Row)
        .flex_wrap(FlexWrap::Wrap)
        .align_items(Align::Center)
        .justify_content(JustifyContent::SpaceBetween)
        .gap(16.0, 12.0)
        .child(
            View::new()
                .display(Display::Flex)
                .flex_direction(FlexDirection::Column)
                .min_width(px!(0.0))
                .child(
                    Label::new()
                        .label("XenGui Benchmark Lab")
                        .font_size(px!(24.0))
                        .font_weight(FontWeight::Bold)
                        .color(text()),
                )
                .child(
                    Label::new()
                        .label(format!(
                            "{:?} · {}×{}",
                            breakpoint,
                            viewport_size().0 as u32,
                            viewport_size().1 as u32
                        ))
                        .font_size(px!(11.0))
                        .color(muted()),
                ),
        )
        .child(
            View::new()
                .display(Display::Flex)
                .flex_direction(FlexDirection::Row)
                .align_items(Align::Center)
                .gap(12.0, 0.0)
                .child(theme_controls)
                .child(Badge::new().label(if cfg!(target_os = "android") {
                    "Android"
                } else if cfg!(target_arch = "wasm32") {
                    "WASM"
                } else {
                    "Desktop"
                })),
        );

    let content_pane = View::new()
        .display(Display::Flex)
        .flex_direction(FlexDirection::Column)
        .flex_grow(1.0)
        .width(px!(0.0))
        .min_width(px!(0.0))
        .height(pct!(100.0))
        .overflow_x(Overflow::Hidden)
        .overflow_y(Overflow::Auto)
        // Every lab page shares one fixed content width. Without a stable
        // gutter, switching between a short page and a scrollable page adds
        // or removes the vertical track's layout padding and shifts the
        // entire pane horizontally.
        .scrollbar_gutter(ScrollbarGutter::Stable)
        .padding(if compact {
            Edges::all(16.0)
        } else {
            Edges::all(24.0)
        })
        .gap(0.0, 18.0)
        .child(header)
        .child_boxed(content);

    let root = if compact {
        View::new()
            .display(Display::Flex)
            .flex_direction(FlexDirection::Column)
            .width(pct!(100.0))
            .height(pct!(100.0))
            .min_width(px!(0.0))
            .overflow(Overflow::Hidden, Overflow::Hidden)
            .child(
                View::new()
                    .padding(Edges::only(12.0, 8.0, 12.0, 8.0))
                    .background(current_theme().surface_container)
                    .child(nav),
            )
            .child(content_pane)
    } else {
        View::new()
            .display(Display::Flex)
            .flex_direction(FlexDirection::Row)
            .width(pct!(100.0))
            .height(pct!(100.0))
            .min_width(px!(0.0))
            .overflow(Overflow::Hidden, Overflow::Hidden)
            .child(
                View::new()
                    .width(px!(sidebar_width))
                    .min_width(px!(sidebar_width))
                    .height(pct!(100.0))
                    .padding(Edges::all(16.0))
                    .background(current_theme().surface_container)
                    .border(Border::right(1.0, outline()))
                    .child(nav),
            )
            .child(content_pane)
    };

    Box::new(
        root.font("Inter")
            .font_size(px!(14.0))
            .color(text())
            .width(pct!(100.0))
            .height(pct!(100.0))
            .background(bg()),
    )
}

fn create_app() -> App {
    let store = TraceStore::new();
    let diagnostic_store = store.clone();
    let diagnostics = Arc::new(move |diagnostic: AppDiagnostic| {
        diagnostic_store.diagnostic(format!("{diagnostic:?}"));
    });
    xengui::devtools::set_enabled(true);

    let mut app = App::new(AppConfig {
        title: "XenGui Benchmark Lab".to_string(),
        #[cfg(not(target_arch = "wasm32"))]
        width: 1280,
        #[cfg(not(target_arch = "wasm32"))]
        height: 820,
        #[cfg(not(target_arch = "wasm32"))]
        position: WindowPosition::Center,
        themes: themes(),
        active_theme: 1,
        dark_theme: 1,
        light_theme: 0,
        theme_mode: AppThemeMode::Fixed,
        diagnostics_sink: Some(diagnostics),
        reload_shortcut: true,
        ..Default::default()
    });
    app.with_font(
        "Inter",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../xengui_website/fonts/Inter-VariableFont.ttf"
        ))
        .to_vec(),
    );
    app.render(move || root(store.clone()));
    app
}

#[cfg(not(target_os = "android"))]
pub fn run_desktop() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(target_arch = "wasm32")]
    {
        console_error_panic_hook::set_once();
        let _ = console_log::init_with_level(log::Level::Info);
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = env_logger::Builder::new()
            .filter_module("xengui", log::LevelFilter::Info)
            .filter_module("xenframe", log::LevelFilter::Info)
            .format_timestamp(None)
            .try_init();
    }
    Ok(create_app().run()?)
}

#[cfg(target_os = "android")]
#[allow(unsafe_code)]
#[unsafe(no_mangle)]
fn android_main(android_app: winit::platform::android::activity::AndroidApp) {
    if let Some(path) = android_app.internal_data_path() {
        export::set_android_data_dir(path);
    }
    if let Err(error) = create_app().run_android(android_app) {
        eprintln!("xengui benchmark failed: {error}");
    }
}
