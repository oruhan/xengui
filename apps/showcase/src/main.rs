// SPDX-License-Identifier: Apache-2.0
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::time::Duration;

#[cfg(not(target_arch = "wasm32"))]
use xenframe::WindowPosition;
use xenframe::{App, AppConfig, AppThemeMode};
use xengui::*;
use xengui_icons::{IconAxes, codepoints};

const CANVAS: Color = Color::rgb(13, 15, 23);
const SURFACE_LOW: Color = Color::rgb(20, 23, 34);
const SURFACE: Color = Color::rgb(27, 30, 43);
const SURFACE_HIGH: Color = Color::rgb(36, 39, 54);
const OUTLINE: Color = Color::rgb(58, 62, 79);
const TEXT: Color = Color::rgb(242, 241, 249);
const MUTED: Color = Color::rgb(180, 181, 197);
const PRIMARY: Color = Color::rgb(180, 157, 255);
const ON_PRIMARY: Color = Color::rgb(39, 18, 91);
const PRIMARY_CONTAINER: Color = Color::rgb(53, 39, 91);
const ON_PRIMARY_CONTAINER: Color = Color::rgb(234, 221, 255);
const CYAN: Color = Color::rgb(103, 218, 226);
const GREEN: Color = Color::rgb(115, 222, 157);
const AMBER: Color = Color::rgb(255, 202, 104);

fn icon(codepoint: char, size: f32, fill: bool) -> VariableIcon {
    VariableIcon::new(codepoint).size(size).axes(
        IconAxes::default()
            .weight(500.0)
            .grade(0.0)
            .optical_size(size)
            .fill(if fill { 1.0 } else { 0.0 }),
    )
}

fn card() -> View {
    View::new()
        .display(Display::Flex)
        .flex_direction(FlexDirection::Column)
        .background(SURFACE)
        .border(Border::all(1.0, OUTLINE).radius(12.0))
}

fn section_title(title: &'static str, subtitle: &'static str) -> View {
    View::new()
        .display(Display::Flex)
        .flex_direction(FlexDirection::Column)
        .gap(0.0, 3.0)
        .child(
            Label::new()
                .label(title)
                .font_size(px!(17.0))
                .line_height(px!(24.0))
                .font_weight(FontWeight::SemiBold)
                .color(TEXT),
        )
        .child(
            Label::new()
                .label(subtitle)
                .font_size(px!(12.0))
                .line_height(px!(17.0))
                .color(MUTED),
        )
}

fn nav_item(label: &'static str, codepoint: char, selected: bool) -> View {
    let hover_background = if selected {
        Color::rgb(68, 52, 107)
    } else {
        SURFACE_HIGH
    };
    let hover_color = if selected { ON_PRIMARY_CONTAINER } else { TEXT };

    View::new()
        .display(Display::Flex)
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .gap(12.0, 0.0)
        .height(px!(48.0))
        .padding(Edges::symmetric(14.0, 0.0))
        .background(if selected {
            PRIMARY_CONTAINER
        } else {
            Color::TRANSPARENT
        })
        .border(Border::all(0.0, Color::TRANSPARENT).radius(24.0))
        .color(if selected {
            ON_PRIMARY_CONTAINER
        } else {
            MUTED
        })
        .transition_colors(Transition::new(Duration::from_millis(160)).easing(Easing::EaseOut))
        .hover_style(move |style, _| style.background(hover_background).color(hover_color))
        .child(icon(codepoint, 21.0, selected))
        .child(
            Label::new()
                .label(label)
                .font_size(px!(13.0))
                .font_weight(if selected {
                    FontWeight::SemiBold
                } else {
                    FontWeight::Medium
                }),
        )
}

fn platform_chip(label: &'static str, codepoint: char) -> View {
    View::new()
        .display(Display::Flex)
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .gap(7.0, 0.0)
        .height(px!(32.0))
        .padding(Edges::symmetric(11.0, 0.0))
        .background(SURFACE_HIGH)
        .border(Border::all(1.0, OUTLINE).radius(16.0))
        .color(MUTED)
        .child(icon(codepoint, 16.0, false))
        .child(Label::new().label(label).font_size(px!(11.0)))
}

fn capability_card(
    title: &'static str,
    value: &'static str,
    codepoint: char,
    accent: Color,
) -> View {
    card()
        .flex_grow(1.0)
        .min_width(px!(190.0))
        .padding(Edges::all(17.0))
        .gap(0.0, 13.0)
        .child(
            View::new()
                .display(Display::Flex)
                .flex_direction(FlexDirection::Row)
                .align_items(Align::Center)
                .justify_content(JustifyContent::SpaceBetween)
                .child(
                    View::new()
                        .display(Display::Flex)
                        .align_items(Align::Center)
                        .justify_content(JustifyContent::Center)
                        .width(px!(40.0))
                        .height(px!(40.0))
                        .background(accent.with_alpha(28))
                        .border(Border::all(0.0, Color::TRANSPARENT).radius(12.0))
                        .color(accent)
                        .child(icon(codepoint, 21.0, false)),
                )
                .child(
                    View::new()
                        .display(Display::Flex)
                        .flex_direction(FlexDirection::Row)
                        .align_items(Align::Center)
                        .gap(5.0, 0.0)
                        .color(GREEN)
                        .child(
                            View::new()
                                .width(px!(6.0))
                                .height(px!(6.0))
                                .background(GREEN)
                                .border(Border::all(0.0, Color::TRANSPARENT).radius(3.0)),
                        )
                        .child(Label::new().label("ready").font_size(px!(10.0))),
                ),
        )
        .child(
            Label::new()
                .label(value)
                .font_size(px!(19.0))
                .line_height(px!(25.0))
                .font_weight(FontWeight::SemiBold)
                .color(TEXT),
        )
        .child(Label::new().label(title).font_size(px!(11.0)).color(MUTED))
}

fn pipeline_node(
    title: &'static str,
    subtitle: &'static str,
    codepoint: char,
    accent: Color,
) -> View {
    View::new()
        .display(Display::Flex)
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .gap(10.0, 0.0)
        .width(pct!(100.0))
        .padding(Edges::symmetric(12.0, 11.0))
        .background(Color::rgba(255, 255, 255, 14))
        .border(Border::all(1.0, Color::rgba(255, 255, 255, 25)).radius(12.0))
        .child(
            View::new()
                .display(Display::Flex)
                .align_items(Align::Center)
                .justify_content(JustifyContent::Center)
                .width(px!(36.0))
                .height(px!(36.0))
                .background(accent.with_alpha(35))
                .border(Border::all(0.0, Color::TRANSPARENT).radius(10.0))
                .color(accent)
                .child(icon(codepoint, 19.0, false)),
        )
        .child(
            View::new()
                .display(Display::Flex)
                .flex_direction(FlexDirection::Column)
                .gap(0.0, 2.0)
                .child(
                    Label::new()
                        .label(title)
                        .font_size(px!(12.0))
                        .font_weight(FontWeight::SemiBold)
                        .color(TEXT),
                )
                .child(
                    Label::new()
                        .label(subtitle)
                        .font_size(px!(10.0))
                        .color(MUTED),
                ),
        )
}

fn setting_row(
    title: &'static str,
    subtitle: &'static str,
    control: impl Widget + 'static,
) -> View {
    View::new()
        .display(Display::Flex)
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .justify_content(JustifyContent::SpaceBetween)
        .gap(16.0, 0.0)
        .min_height(px!(52.0))
        .child(
            View::new()
                .display(Display::Flex)
                .flex_direction(FlexDirection::Column)
                .gap(0.0, 2.0)
                .flex_grow(1.0)
                .child(
                    Label::new()
                        .label(title)
                        .font_size(px!(12.0))
                        .font_weight(FontWeight::Medium)
                        .color(TEXT),
                )
                .child(
                    Label::new()
                        .label(subtitle)
                        .font_size(px!(10.0))
                        .color(MUTED),
                ),
        )
        .child(control)
}

fn showcase_theme() -> Theme {
    Theme::dark()
        .background(CANVAS)
        .on_background(TEXT)
        .primary(PRIMARY)
        .on_primary(ON_PRIMARY)
        .primary_container(PRIMARY_CONTAINER)
        .on_primary_container(ON_PRIMARY_CONTAINER)
        .secondary(CYAN)
        .on_secondary(CANVAS)
        .secondary_container(Color::rgb(29, 67, 72))
        .on_secondary_container(Color::rgb(199, 246, 249))
        .tertiary(AMBER)
        .surface_dim(CANVAS)
        .surface(CANVAS)
        .surface_bright(SURFACE_HIGH)
        .surface_container_lowest(CANVAS)
        .surface_container_low(SURFACE_LOW)
        .surface_container(SURFACE)
        .surface_container_high(SURFACE_HIGH)
        .surface_container_highest(Color::rgb(47, 50, 66))
        .on_surface(TEXT)
        .on_surface_variant(MUTED)
        .outline(OUTLINE)
        .outline_variant(OUTLINE)
        .success(GREEN)
        .warning(AMBER)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(target_arch = "wasm32")]
    {
        console_error_panic_hook::set_once();
        let _ = console_log::init_with_level(log::Level::Info);
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = env_logger::Builder::new()
            .filter_module("xengui", log::LevelFilter::Info)
            .filter_level(log::LevelFilter::Warn)
            .format_timestamp(None)
            .try_init();
    }

    let config = AppConfig {
        title: "XenGui Showcase".into(),
        #[cfg(not(target_arch = "wasm32"))]
        width: 1600,
        #[cfg(not(target_arch = "wasm32"))]
        height: 1000,
        #[cfg(not(target_arch = "wasm32"))]
        position: WindowPosition::Center,
        #[cfg(not(target_arch = "wasm32"))]
        decorations: false,
        themes: vec![showcase_theme()],
        active_theme: 0,
        dark_theme: 0,
        light_theme: 0,
        theme_mode: AppThemeMode::Fixed,
        window_radius: 16.0,
        window_border: Some((1.0, OUTLINE)),
        ..Default::default()
    };

    let mut app = App::new(config);
    app.with_font(
        "Inter",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../xengui_website/fonts/Inter-VariableFont.ttf"
        ))
        .to_vec(),
    );

    app.render(|| {
        let (active_nav, set_active_nav) = use_state(0usize);
        let (gpu_debug, set_gpu_debug) = use_state(true);
        let (auto_layout, set_auto_layout) = use_state(true);
        let (accessibility, set_accessibility) = use_state(true);
        let (density, set_density) = use_state(0.68_f32);
        let (query, set_query) = use_state(String::new());
        let (preview_count, set_preview_count) = use_state(0usize);

        let nav_entries = [
            ("Overview", codepoints::SPACE_DASHBOARD),
            ("Components", codepoints::WIDGETS),
            ("Inspector", codepoints::SEARCH_INSIGHTS),
            ("Performance", codepoints::SPEED),
        ];

        let mut navigation = View::new()
            .display(Display::Flex)
            .flex_direction(FlexDirection::Column)
            .gap(0.0, 6.0);
        for (index, (label, codepoint)) in nav_entries.into_iter().enumerate() {
            let set_active = set_active_nav.clone();
            navigation = navigation.child(
                nav_item(label, codepoint, active_nav == index)
                    .accessible_label(label)
                    .on_click(move |_| set_active.set(index)),
            );
        }

        let close_button = View::new()
            .display(Display::Flex)
            .align_items(Align::Center)
            .justify_content(JustifyContent::Center)
            .width(px!(48.0))
            .height(px!(44.0))
            .color(MUTED)
            .accessible_label("Close window")
            .hover_style(|style, _| style.background(Color::RED_600).color(Color::WHITE))
            .on_click(|_| xenframe::close_window())
            .child(icon(codepoints::CLOSE, 18.0, false));

        let title_bar = View::new()
            .display(Display::Flex)
            .flex_direction(FlexDirection::Row)
            .align_items(Align::Center)
            .height(px!(44.0))
            .min_height(px!(44.0))
            .padding(Edges::only(18.0, 0.0, 0.0, 0.0))
            .background(SURFACE_LOW)
            .border(Border::bottom(1.0, OUTLINE))
            .child(
                View::new()
                    .display(Display::Flex)
                    .flex_direction(FlexDirection::Row)
                    .align_items(Align::Center)
                    .gap(9.0, 0.0)
                    .color(PRIMARY)
                    .child(icon(codepoints::DATA_OBJECT, 19.0, true))
                    .child(
                        Label::new()
                            .label("XENGUI")
                            .font_size(px!(12.0))
                            .font_weight(FontWeight::Bold)
                            .letter_spacing(px!(1.7)),
                    ),
            )
            .child(
                View::new()
                    .margin(Edges::only(14.0, 0.0, 0.0, 0.0))
                    .padding(Edges::symmetric(9.0, 4.0))
                    .background(PRIMARY_CONTAINER)
                    .border(Border::all(0.0, Color::TRANSPARENT).radius(12.0))
                    .child(
                        Label::new()
                            .label("rendered with XenGui")
                            .font_size(px!(10.0))
                            .color(ON_PRIMARY_CONTAINER),
                    ),
            )
            .child(View::new().flex_grow(1.0).height(pct!(100.0)).window_drag_region(true))
            .child(platform_chip("Native", codepoints::COMPUTER))
            .child(
                View::new()
                    .margin(Edges::only(8.0, 0.0, 0.0, 0.0))
                    .child(platform_chip("Web", codepoints::LANGUAGE)),
            )
            .child(
                View::new()
                    .margin(Edges::only(8.0, 0.0, 0.0, 0.0))
                    .child(platform_chip("Android", codepoints::ANDROID)),
            )
            .child(View::new().width(px!(14.0)))
            .child(close_button);

        let sidebar = View::new()
            .display(Display::Flex)
            .flex_direction(FlexDirection::Column)
            .width(px!(232.0))
            .min_width(px!(232.0))
            .height(pct!(100.0))
            .padding(Edges::only(16.0, 18.0, 16.0, 16.0))
            .background(SURFACE_LOW)
            .border(Border::right(1.0, OUTLINE))
            .child(
                Label::new()
                    .label("WORKSPACE")
                    .font_size(px!(10.0))
                    .font_weight(FontWeight::SemiBold)
                    .letter_spacing(px!(1.3))
                    .color(MUTED)
                    .margin(Edges::only(14.0, 0.0, 0.0, 10.0)),
            )
            .child(navigation)
            .child(View::new().flex_grow(1.0))
            .child(
                card()
                    .padding(Edges::all(15.0))
                    .gap(0.0, 10.0)
                    .background(PRIMARY_CONTAINER)
                    .border(Border::all(1.0, PRIMARY.with_alpha(42)).radius(12.0))
                    .child(
                        View::new()
                            .display(Display::Flex)
                            .align_items(Align::Center)
                            .justify_content(JustifyContent::SpaceBetween)
                            .color(PRIMARY)
                            .child(icon(codepoints::ROCKET_LAUNCH, 22.0, true))
                            .child(Badge::new().label("v0.2.8")),
                    )
                    .child(
                        Label::new()
                            .label("Build one tree. Ship everywhere.")
                            .font_size(px!(13.0))
                            .line_height(px!(18.0))
                            .font_weight(FontWeight::SemiBold)
                            .color(ON_PRIMARY_CONTAINER),
                    )
                    .child(
                        Label::new()
                            .label("Native, web and Android from a shared Rust UI.")
                            .font_size(px!(10.0))
                            .line_height(px!(15.0))
                            .color(MUTED),
                    ),
            )
            .child(
                View::new()
                    .display(Display::Flex)
                    .flex_direction(FlexDirection::Row)
                    .align_items(Align::Center)
                    .gap(8.0, 0.0)
                    .margin(Edges::only(8.0, 0.0, 0.0, 0.0))
                    .color(GREEN)
                    .child(
                        View::new()
                            .width(px!(7.0))
                            .height(px!(7.0))
                            .background(GREEN)
                            .border(Border::all(0.0, Color::TRANSPARENT).radius(4.0)),
                    )
                    .child(
                        Label::new()
                            .label("wgpu renderer online")
                            .font_size(px!(10.0)),
                    ),
            );

        let query_for_button = query.clone();
        let search = TextBox::new()
            .value(query)
            .placeholder("Search components…")
            .accessible_label("Search components")
            .width(px!(250.0))
            .height(px!(40.0))
            .padding(Edges::symmetric(15.0, 0.0))
            .font_size(px!(12.0))
            .line_height(px!(20.0))
            .background(SURFACE_LOW)
            .color(TEXT)
            .border(Border::all(1.0, OUTLINE).radius(20.0))
            .on_change(move |value, _| set_query.set(value.to_owned()));

        let set_preview = set_preview_count.clone();
        let preview_button = Button::new()
            .label(if preview_count == 0 {
                "Run preview"
            } else {
                "Preview ready"
            })
            .accessible_label("Run live preview")
            .height(px!(40.0))
            .padding(Edges::symmetric(18.0, 0.0))
            .background(PRIMARY)
            .color(ON_PRIMARY)
            .font_size(px!(12.0))
            .font_weight(FontWeight::SemiBold)
            .border(Border::all(0.0, Color::TRANSPARENT).radius(20.0))
            .hover_style(|style, _| style.background(Color::rgb(202, 184, 255)))
            .on_click(move |_| set_preview.set(preview_count + 1));

        let page_header = View::new()
            .display(Display::Flex)
            .flex_direction(FlexDirection::Row)
            .align_items(Align::Center)
            .justify_content(JustifyContent::SpaceBetween)
            .gap(20.0, 0.0)
            .child(
                View::new()
                    .display(Display::Flex)
                    .flex_direction(FlexDirection::Column)
                    .gap(0.0, 5.0)
                    .child(
                        Label::new()
                            .label(match active_nav {
                                1 => "Component system",
                                2 => "Runtime inspector",
                                3 => "Performance lab",
                                _ => "Engineering overview",
                            })
                            .font_size(px!(27.0))
                            .line_height(px!(34.0))
                            .font_weight(FontWeight::SemiBold)
                            .color(TEXT),
                    )
                    .child(
                        Label::new()
                            .label("A real XenGui application — every pixel below is rendered by the library.")
                            .font_size(px!(12.0))
                            .color(MUTED),
                    ),
            )
            .child(
                View::new()
                    .display(Display::Flex)
                    .flex_direction(FlexDirection::Row)
                    .align_items(Align::Center)
                    .gap(10.0, 0.0)
                    .child(search)
                    .child(preview_button),
            );

        let hero = View::new()
            .display(Display::Flex)
            .flex_direction(FlexDirection::Row)
            .min_height(px!(236.0))
            .padding(Edges::all(24.0))
            .gap(24.0, 0.0)
            .background(PRIMARY_CONTAINER)
            .border(Border::all(1.0, PRIMARY.with_alpha(45)).radius(28.0))
            .child(
                View::new()
                    .display(Display::Flex)
                    .flex_direction(FlexDirection::Column)
                    .justify_content(JustifyContent::Center)
                    .gap(0.0, 15.0)
                    .flex_grow(1.0)
                    .min_width(px!(360.0))
                    .child(
                        View::new()
                            .display(Display::Flex)
                            .flex_direction(FlexDirection::Row)
                            .align_items(Align::Center)
                            .gap(7.0, 0.0)
                            .color(CYAN)
                            .child(icon(codepoints::AUTO_AWESOME, 17.0, true))
                            .child(
                                Label::new()
                                    .label("RUST-NATIVE UI RUNTIME")
                                    .font_size(px!(10.0))
                                    .font_weight(FontWeight::Bold)
                                    .letter_spacing(px!(1.4)),
                            ),
                    )
                    .child(
                        Label::new()
                            .label("One declarative tree.\nEvery surface.")
                            .font_size(px!(34.0))
                            .line_height(px!(40.0))
                            .font_weight(FontWeight::SemiBold)
                            .color(ON_PRIMARY_CONTAINER),
                    )
                    .child(
                        Label::new()
                            .label("Retained rendering, responsive layout, typed state and GPU composition — designed as one coherent system.")
                            .max_width(px!(530.0))
                            .font_size(px!(12.0))
                            .line_height(px!(18.0))
                            .color(MUTED),
                    )
                    .child(
                        View::new()
                            .display(Display::Flex)
                            .flex_direction(FlexDirection::Row)
                            .gap(8.0, 0.0)
                            .child(platform_chip("Retained", codepoints::ACCOUNT_TREE))
                            .child(platform_chip("Accessible", codepoints::ACCESSIBILITY_NEW))
                            .child(platform_chip("GPU", codepoints::MEMORY)),
                    ),
            )
            .child(
                View::new()
                    .display(Display::Flex)
                    .flex_direction(FlexDirection::Column)
                    .width(px!(390.0))
                    .min_width(px!(390.0))
                    .padding(Edges::all(14.0))
                    .gap(0.0, 8.0)
                    .background(Color::rgba(9, 10, 17, 120))
                    .border(Border::all(1.0, Color::rgba(255, 255, 255, 25)).radius(20.0))
                    .child(pipeline_node(
                        "State + events",
                        "typed hooks and input routing",
                        codepoints::TOUCH_APP,
                        AMBER,
                    ))
                    .child(pipeline_node(
                        "Constraint layout",
                        "Taffy-powered responsive geometry",
                        codepoints::GRID_VIEW,
                        PRIMARY,
                    ))
                    .child(pipeline_node(
                        "GPU composition",
                        "batched text, shapes and images",
                        codepoints::VIEW_IN_AR,
                        CYAN,
                    )),
            );

        let capabilities = View::new()
            .display(Display::Flex)
            .flex_direction(FlexDirection::Row)
            .gap(14.0, 0.0)
            .child(capability_card(
                "Rendering architecture",
                "Retained tree",
                codepoints::ACCOUNT_TREE,
                PRIMARY,
            ))
            .child(capability_card(
                "Layout engine",
                "Responsive constraints",
                codepoints::GRID_VIEW,
                CYAN,
            ))
            .child(capability_card(
                "Interaction model",
                "Focus + semantics",
                codepoints::ACCESSIBILITY_NEW,
                GREEN,
            ))
            .child(capability_card(
                "Rendering backend",
                "wgpu accelerated",
                codepoints::MEMORY,
                AMBER,
            ));

        let frame_heights = [
            31.0, 47.0, 38.0, 56.0, 44.0, 62.0, 53.0, 72.0, 50.0, 67.0, 46.0, 58.0, 76.0,
            63.0, 70.0, 54.0, 81.0, 66.0, 73.0, 59.0, 78.0, 69.0, 84.0, 74.0,
        ];
        let mut bars = View::new()
            .display(Display::Flex)
            .flex_direction(FlexDirection::Row)
            .align_items(Align::End)
            .gap(5.0, 0.0)
            .height(px!(98.0));
        for (index, height) in frame_heights.into_iter().enumerate() {
            bars = bars.child(
                View::new()
                    .flex_grow(1.0)
                    .height(px!(height))
                    .background(if index > 20 { CYAN } else { PRIMARY })
                    .border(Border::all(0.0, Color::TRANSPARENT).radius(4.0)),
            );
        }

        let renderer_panel = card()
            .flex_grow(1.0)
            .min_width(px!(520.0))
            .padding(Edges::all(19.0))
            .gap(0.0, 17.0)
            .child(
                View::new()
                    .display(Display::Flex)
                    .flex_direction(FlexDirection::Row)
                    .align_items(Align::Center)
                    .justify_content(JustifyContent::SpaceBetween)
                    .child(section_title(
                        "Frame pacing preview",
                        "Illustrative telemetry rendered with ordinary XenGui views",
                    ))
                    .child(
                        View::new()
                            .display(Display::Flex)
                            .flex_direction(FlexDirection::Row)
                            .align_items(Align::Center)
                            .gap(6.0, 0.0)
                            .padding(Edges::symmetric(10.0, 5.0))
                            .background(GREEN.with_alpha(24))
                            .border(Border::all(1.0, GREEN.with_alpha(50)).radius(15.0))
                            .color(GREEN)
                            .child(
                                View::new()
                                    .width(px!(6.0))
                                    .height(px!(6.0))
                                    .background(GREEN)
                                    .border(Border::all(0.0, Color::TRANSPARENT).radius(3.0)),
                            )
                            .child(Label::new().label("LIVE UI").font_size(px!(9.0))),
                    ),
            )
            .child(bars)
            .child(
                View::new()
                    .display(Display::Flex)
                    .flex_direction(FlexDirection::Row)
                    .justify_content(JustifyContent::SpaceBetween)
                    .child(
                        Label::new()
                            .label("retained diff")
                            .font_size(px!(10.0))
                            .color(MUTED),
                    )
                    .child(
                        Label::new()
                            .label("layout")
                            .font_size(px!(10.0))
                            .color(MUTED),
                    )
                    .child(
                        Label::new()
                            .label("paint + present")
                            .font_size(px!(10.0))
                            .color(MUTED),
                    ),
            )
            .child(ProgressBar::new().value(0.76).bar_height(px!(4.0)));

        let set_gpu = set_gpu_debug.clone();
        let set_layout = set_auto_layout.clone();
        let set_a11y = set_accessibility.clone();
        let set_density_value = set_density.clone();
        let component_lab = card()
            .width(px!(390.0))
            .min_width(px!(390.0))
            .padding(Edges::all(19.0))
            .gap(0.0, 10.0)
            .child(section_title(
                "Live component lab",
                "Real controlled widgets — try them",
            ))
            .child(setting_row(
                "GPU diagnostics",
                "Show render instrumentation",
                Switch::new()
                    .checked(gpu_debug)
                    .accessible_label("GPU diagnostics")
                    .on_change(move |value, _| set_gpu.set(value)),
            ))
            .child(setting_row(
                "Adaptive layout",
                "React to the active breakpoint",
                Switch::new()
                    .checked(auto_layout)
                    .accessible_label("Adaptive layout")
                    .on_change(move |value, _| set_layout.set(value)),
            ))
            .child(setting_row(
                "Semantic tree",
                "Expose accessible widget roles",
                Checkbox::new()
                    .checked(accessibility)
                    .size(20.0)
                    .accessible_label("Semantic tree")
                    .on_change(move |value, _| set_a11y.set(value)),
            ))
            .child(
                View::new()
                    .display(Display::Flex)
                    .flex_direction(FlexDirection::Column)
                    .gap(0.0, 9.0)
                    .margin(Edges::only(0.0, 4.0, 0.0, 0.0))
                    .child(
                        View::new()
                            .display(Display::Flex)
                            .flex_direction(FlexDirection::Row)
                            .justify_content(JustifyContent::SpaceBetween)
                            .child(
                                Label::new()
                                    .label("Interface density")
                                    .font_size(px!(12.0))
                                    .color(TEXT),
                            )
                            .child(
                                Label::new()
                                    .label(format!("{:.0}%", density * 100.0))
                                    .font_size(px!(11.0))
                                    .color(PRIMARY),
                            ),
                    )
                    .child(
                        Slider::new()
                            .value(density)
                            .accessible_label("Interface density")
                            .on_change(move |value, _| set_density_value.set(value)),
                    ),
            );

        let lower = View::new()
            .display(Display::Flex)
            .flex_direction(FlexDirection::Row)
            .gap(14.0, 0.0)
            .child(renderer_panel)
            .child(component_lab);

        let footer = View::new()
            .display(Display::Flex)
            .flex_direction(FlexDirection::Row)
            .align_items(Align::Center)
            .justify_content(JustifyContent::SpaceBetween)
            .padding(Edges::symmetric(2.0, 0.0))
            .child(
                View::new()
                    .display(Display::Flex)
                    .flex_direction(FlexDirection::Row)
                    .align_items(Align::Center)
                    .gap(7.0, 0.0)
                    .color(MUTED)
                    .child(icon(codepoints::CHECK_CIRCLE, 15.0, true))
                    .child(
                        Label::new()
                            .label("All systems operational")
                            .font_size(px!(10.0)),
                    ),
            )
            .child(
                Label::new()
                    .label(if query_for_button.is_empty() {
                        "XenGui showcase · native wgpu surface"
                    } else {
                        "Component search is active"
                    })
                    .font_size(px!(10.0))
                    .color(MUTED),
            );

        Box::new(
            View::new()
                .font("Inter")
                .display(Display::Flex)
                .flex_direction(FlexDirection::Column)
                .width(pct!(100.0))
                .height(pct!(100.0))
                .background(CANVAS)
                .child(title_bar)
                .child(
                    View::new()
                        .display(Display::Flex)
                        .flex_direction(FlexDirection::Row)
                        .flex_grow(1.0)
                        .min_height(px!(0.0))
                        .child(sidebar)
                        .child(
                            View::new()
                                .display(Display::Flex)
                                .flex_direction(FlexDirection::Column)
                                .flex_grow(1.0)
                                .min_width(px!(0.0))
                                .height(pct!(100.0))
                                .overflow_y(Overflow::Auto)
                                .padding(Edges::all(24.0))
                                .gap(0.0, 18.0)
                                .child(page_header)
                                .child(hero)
                                .child(capabilities)
                                .child(lower)
                                .child(footer),
                        ),
                ),
        )
    });

    Ok(app.run()?)
}
