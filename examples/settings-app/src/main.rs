// SPDX-License-Identifier: Apache-2.0
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use web_time::Duration;
use xenframe::{ App, AppConfig };

#[cfg(not(target_arch = "wasm32"))]
use xenframe::WindowPosition;
use xengui::{ properties::StyleValue, * };

fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(target_arch = "wasm32")]
    {
        console_error_panic_hook::set_once();
        let _ = console_log::init_with_level(log::Level::Info);
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = env_logger::Builder
            ::new()
            .filter_module("xengui", log::LevelFilter::Info)
            .filter_level(log::LevelFilter::Warn)
            .format_timestamp(None)
            .try_init();
    }

    let config = AppConfig {
        #[cfg(not(target_arch = "wasm32"))]
        title: "Settings".into(),

        #[cfg(not(target_arch = "wasm32"))]
        width: 900,
        #[cfg(not(target_arch = "wasm32"))]
        height: 700,

        #[cfg(not(target_arch = "wasm32"))]
        position: WindowPosition::Center,
        #[cfg(not(target_arch = "wasm32"))]
        decorations: false,

        ..Default::default()
    };

    let mut app = App::new(config);

    app.with_font(
        "Noto_Sans",
        include_bytes!(
            concat!(env!("CARGO_MANIFEST_DIR"), "/fonts/NotoSans-VariableFont.ttf")
        ).to_vec()
    );

    app.render(|| {
        let (text, set_text) = use_state(String::from(""));

        Box::new(
            View::new()
                .font("Noto_Sans")
                .display(Display::Flex)
                .flex_direction(FlexDirection::Column)
                .width(pct!(100.0))
                .height(pct!(100.0))
                .background(|theme: &Theme| theme.background)
                .child(
                    View::new()
                        .display(Display::Flex)
                        .flex_direction(FlexDirection::Row)
                        .align_items(Align::Center)
                        .width(pct!(100.0))
                        .height(px!(32.0))
                        .min_height(px!(32.0))
                        .background(|theme: &Theme| theme.surface)
                        .border(|theme: &Theme| { Border::bottom(1, theme.outline) })
                        .padding(Edges::only(12, 0, 0, 0))
                        .child(
                            Label::new()
                                .label("Settings")
                                .font_size(px!(13.0))
                                .color(|theme: &Theme| theme.on_background)
                        )
                        .child(
                            View::new()
                                .flex_grow(1.0)
                                .height(pct!(100.0))
                                // Mark this view as a window drag region
                                .window_drag_region(true)
                        )
                        .child(
                            View::new()
                                .display(Display::Flex)
                                .flex_direction(FlexDirection::Row)
                                .height(pct!(100.0))
                                .child(
                                    Button::new()
                                        .align_items(Align::Center)
                                        .justify_content(JustifyContent::Center)
                                        .height(pct!(100.0))
                                        .width(px!(44.0))
                                        .label("a")
                                        .font_size(14)
                                        .background(Color::TRANSPARENT)
                                        .color(|theme: &Theme| theme.on_background)
                                        .transition_all(
                                            Transition::new(Duration::from_millis(200)).easing(
                                                Easing::EaseInOut
                                            )
                                        )
                                        .hover_style(|s, theme: &Theme| {
                                            s.background(theme.on_surface)
                                        })
                                        .pressed_style(|s, theme: &Theme| {
                                            s.background(theme.on_surface_variant)
                                        })
                                        .on_click(move |_| xenframe::minimize_window())
                                )
                                .child(
                                    Button::new()
                                        .align_items(Align::Center)
                                        .justify_content(JustifyContent::Center)
                                        .height(pct!(100.0))
                                        .width(px!(44.0))
                                        .label("-")
                                        .font_size(14)
                                        .background(Color::TRANSPARENT)
                                        .color(|theme: &Theme| theme.on_background)
                                        .transition_all(
                                            Transition::new(Duration::from_millis(200)).easing(
                                                Easing::EaseInOut
                                            )
                                        )
                                        .hover_style(|s, theme: &Theme| {
                                            s.background(theme.on_surface)
                                        })
                                        .pressed_style(|s, theme: &Theme| {
                                            s.background(theme.on_surface_variant)
                                        })
                                        .on_click(move |_| xenframe::toggle_maximize_window())
                                )
                                .child(
                                    Button::new()
                                        .align_items(Align::Center)
                                        .justify_content(JustifyContent::Center)
                                        .height(pct!(100.0))
                                        .width(px!(44.0))
                                        .label("A")
                                        .font_size(14)
                                        .background(Color::TRANSPARENT)
                                        .color(|theme: &Theme| theme.on_background)
                                        .transition_all(
                                            Transition::new(Duration::from_millis(200)).easing(
                                                Easing::EaseInOut
                                            )
                                        )
                                        .hover_style(|s, _| {
                                            s.background(Color::RED_600).color(Color::WHITE)
                                        })
                                        .pressed_style(|s, _| {
                                            s.background(Color::RED_800).color(Color::WHITE)
                                        })
                                        .on_click(move |_| xenframe::close_window())
                                )
                        )
                )
                .child(
                    View::new()
                        .display(Display::Flex)
                        .flex_direction(FlexDirection::Column)
                        .flex_grow(1.0)
                        .padding(Edges::all(15))
                        .child(
                            Label::new()
                                .label("Settings")
                                .font_size(px!(18.0))
                                .color(|theme: &Theme| theme.on_background)
                                .margin(Edges::only(0, 0, 0, 6))
                        )
                        .child(
                            View::new()
                                .flex_direction(FlexDirection::Column)
                                .overflow_x(Overflow::Auto)
                                .overflow_y(Overflow::Auto)
                                .gap(0, 4)
                                .child(
                                    Label::new()
                                        .label("label1")
                                        .color(|theme: &Theme| theme.on_background)
                                )
                                .child(
                                    Button::new()
                                        .align_items(Align::Center)
                                        .justify_content(JustifyContent::Center)
                                        .height(px!(50))
                                        .width(px!(44.0))
                                        .label("A")
                                        .font_size(14)
                                        .background(Color::TRANSPARENT)
                                        .color(|theme: &Theme| theme.on_background)
                                        .transition_all(
                                            Transition::new(Duration::from_millis(200)).easing(
                                                Easing::EaseInOut
                                            )
                                        )
                                        .hover_style(|s, _| {
                                            s.background(Color::RED_600).color(Color::WHITE)
                                        })
                                        .pressed_style(|s, _| {
                                            s.background(Color::RED_800).color(Color::WHITE)
                                        })
                                )
                                .child(
                                    Button::new()
                                        .align_items(Align::Center)
                                        .justify_content(JustifyContent::Center)
                                        .height(px!(50.0))
                                        .width(px!(44.0))
                                        .label("Test")
                                        .font_size(14)
                                        .background(Color::TRANSPARENT)
                                        .color(|theme: &Theme| theme.on_background)
                                        .transition_all(
                                            Transition::new(Duration::from_millis(200)).easing(
                                                Easing::EaseInOut
                                            )
                                        )
                                        .hover_style(|s, _| {
                                            s.background(Color::RED_600).color(Color::WHITE)
                                        })
                                        .pressed_style(|s, _| {
                                            s.background(Color::RED_800).color(Color::WHITE)
                                        })
                                )
                                .child(
                                    TextBox::new()
                                        .value(text.clone())
                                        .color(|theme: &Theme| theme.on_background)
                                        .placeholder("Search in settings...")
                                        .font_size(14)
                                        .outline(StyleValue::None)
                                        .min_width(px!(180.0))
                                        .transition_all(
                                            Transition::new(Duration::from_millis(200)).easing(
                                                Easing::EaseInOut
                                            )
                                        )
                                        .padding(Edges::all(8))
                                        .background(|theme: &Theme| theme.surface)
                                        .border(|theme: &Theme|
                                            Border::all(1, theme.outline).radius(8)
                                        )
                                        .hover_style(|s, theme: &Theme|
                                            s.border(
                                                Border::all(1, theme.outline_variant).radius(8)
                                            )
                                        )
                                        .focus_style(|s, theme: &Theme|
                                            s.border(Border::all(2, theme.primary).radius(8))
                                        )
                                        .focused_hover_style(|s, theme: &Theme|
                                            s.border(Border::all(2, theme.primary).radius(8))
                                        )
                                        .on_change(move |value, _ctx|
                                            set_text.set(value.to_string())
                                        )
                                )
                                .child(Checkbox::new())
                                .child(
                                    Button::new()
                                        .label("button1")
                                        .font_size(14)
                                        .color(Color::NEUTRAL_500)
                                        .background(Color::NEUTRAL_100)
                                        .border(Border::all(1, Color::NEUTRAL_200).radius(8))
                                        .padding(Edges::only(9, 5, 9, 6))
                                        .transition_all(
                                            Transition::new(Duration::from_millis(200)).easing(
                                                Easing::EaseInOut
                                            )
                                        )
                                        .hover_style(|s, _theme: &Theme|
                                            s
                                                .background(Color::NEUTRAL_200)
                                                .border(
                                                    Border::all(1, Color::NEUTRAL_300).radius(8)
                                                )
                                                .color(Color::NEUTRAL_600)
                                        )
                                        .pressed_style(|s, _theme: &Theme|
                                            s
                                                .background(Color::NEUTRAL_200)
                                                .border(
                                                    Border::all(1, Color::NEUTRAL_400).radius(8)
                                                )
                                                .color(Color::NEUTRAL_700)
                                        )
                                        .disabled_style(|s, _theme: &Theme|
                                            s
                                                .background(Color::NEUTRAL_100)
                                                .color(Color::NEUTRAL_400)
                                        )
                                )
                                .child(
                                    Button::new()
                                        .label("button1")
                                        .font_size(14)
                                        .background(Color::BLUE_500)
                                        .border(Border::all(1, Color::BLUE_500).radius(8))
                                        .padding(Edges::only(9, 5, 9, 6))
                                        .transition_all(
                                            Transition::new(Duration::from_millis(200)).easing(
                                                Easing::EaseInOut
                                            )
                                        )
                                        .hover_style(|s, _theme: &Theme|
                                            s
                                                .background(Color::BLUE_600)
                                                .border(Border::all(1, Color::BLUE_600).radius(8))
                                        )
                                        .pressed_style(|s, _theme: &Theme|
                                            s
                                                .background(Color::BLUE_700)
                                                .scale(0.98)
                                                .content_scale(1.0)
                                                .border(Border::all(1, Color::BLUE_700).radius(8))
                                        )
                                )
                                .child(
                                    Button::new()
                                        .label("Sign in with GitHub")
                                        .font_size(15)
                                        .background(Color::NEUTRAL_800)
                                        .padding(Edges::only(12, 8, 12, 8))
                                        .border(Border::all(1, Color::NEUTRAL_700).radius(10))
                                        .icon(
                                            r#"<svg xmlns="http://www.w3.org/2000/svg" width="64" height="64" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-wand-sparkles-icon lucide-wand-sparkles"><path d="m21.64 3.64-1.28-1.28a1.21 1.21 0 0 0-1.72 0L2.36 18.64a1.21 1.21 0 0 0 0 1.72l1.28 1.28a1.2 1.2 0 0 0 1.72 0L21.64 5.36a1.2 1.2 0 0 0 0-1.72"/><path d="m14 7 3 3"/><path d="M5 6v4"/><path d="M19 14v4"/><path d="M10 2v2"/><path d="M7 8H3"/><path d="M21 16h-4"/><path d="M11 3H9"/></svg>"#
                                        )
                                        .icon_color(Color::WHITE)
                                        .transition_all(
                                            Transition::new(Duration::from_millis(200)).easing(
                                                Easing::EaseInOut
                                            )
                                        )
                                        .transition_transform(
                                            Transition::new(Duration::from_millis(200)).easing(
                                                Easing::EaseInOut
                                            )
                                        )
                                        .hover_style(|s, _theme: &Theme|
                                            s
                                                .background(Color::NEUTRAL_900)
                                                .border(
                                                    Border::all(1, Color::NEUTRAL_800).radius(10)
                                                )
                                        )
                                        .pressed_style(|s, _theme: &Theme|
                                            s
                                                .background(Color::NEUTRAL_900)
                                                .scale(0.98)
                                                .content_scale(1.0)
                                                .border(
                                                    Border::all(1, Color::NEUTRAL_800).radius(10)
                                                )
                                        )
                                )
                                .child(
                                    Button::new()
                                        .label("Animated Button")
                                        .font_size(15)
                                        .background(Color::BLUE_500)
                                        .padding(Edges::only(12, 8, 12, 8))
                                        .border(Border::all(1, Color::BLUE_500).radius(8))
                                        .transition_all(Transition::new(Duration::from_millis(200)))
                                        .hover_style(|s, _theme: &Theme|
                                            s.border(Border::all(1, Color::BLUE_500).radius(20))
                                        )
                                )
                                .child(
                                    Button::new()
                                        .label("")
                                        .font_size(15)
                                        .width(px!(64))
                                        .height(px!(48))
                                        .background(Color::BLUE_500)
                                        .padding(Edges::only(12, 8, 12, 8))
                                        .border(Border::all(1, Color::BLUE_500).radius(8))
                                        .transition_all(Transition::new(Duration::from_millis(200)))
                                        .hover_style(|s, _theme: &Theme|
                                            s.border(Border::all(1, Color::BLUE_500).radius(20))
                                        )
                                )
                                .child(
                                    Button::new()
                                        .label("disabled_button1")
                                        .enabled(false)
                                        .font_size(13)
                                        .color(Color::NEUTRAL_500)
                                        .background(Color::NEUTRAL_100)
                                        .border(Border::all(1, Color::NEUTRAL_200).radius(8))
                                        .padding(Edges::only(9, 5, 9, 6))
                                        .hover_style(|s, _theme: &Theme|
                                            s
                                                .background(Color::NEUTRAL_200)
                                                .border(
                                                    Border::all(1, Color::NEUTRAL_300).radius(8)
                                                )
                                                .color(Color::NEUTRAL_600)
                                        )
                                        .pressed_style(|s, _theme: &Theme|
                                            s
                                                .background(Color::NEUTRAL_200)
                                                .border(
                                                    Border::all(1, Color::NEUTRAL_400).radius(8)
                                                )
                                                .color(Color::NEUTRAL_700)
                                        )
                                        .disabled_style(|s, _theme: &Theme|
                                            s
                                                .background(Color::NEUTRAL_100)
                                                .color(Color::NEUTRAL_400)
                                        )
                                )
                                .child(
                                    View::new()
                                        .color(Color::NEUTRAL_400)
                                        .child(
                                            Svg::from_string(
                                                r##"<svg xmlns="http://www.w3.org/2000/svg" height="24px" viewBox="0 -960 960 960" width="24px" fill="#e3e3e3"><path d="M346-160H240q-33 0-56.5-23.5T160-240v-106l-77-78q-11-12-17-26.5T60-480q0-15 6-29.5T83-536l77-78v-106q0-33 23.5-56.5T240-800h106l78-77q12-11 26.5-17t29.5-6q15 0 29.5 6t26.5 17l78 77h106q33 0 56.5 23.5T800-720v106l77 78q11 12 17 26.5t6 29.5q0 15-6 29.5T877-424l-77 78v106q0 33-23.5 56.5T720-160H614l-78 77q-12 11-26.5 17T480-60q-15 0-29.5-6T424-83l-78-77Zm275.5-178.5Q680-397 680-480t-58.5-141.5Q563-680 480-680t-141.5 58.5Q280-563 280-480t58.5 141.5Q397-280 480-280t141.5-58.5ZM395-395q-35-35-35-85t35-85q35-35 85-35t85 35q35 35 35 85t-35 85q-35 35-85 35t-85-35Zm-15 155 100 100 100-100h140v-140l100-100-100-100v-140H580L480-820 380-720H240v140L140-480l100 100v140h140Zm100-240Z"/></svg>"##
                                            )
                                                .width(24)
                                                .height(24)
                                        )
                                )
                                .child(
                                    View::new()
                                        .color(Color::NEUTRAL_400)
                                        .child(
                                            Svg::from_string(
                                                r#" <svg xmlns="http://www.w3.org/2000/svg" width="64" height="64" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-qr-code-icon lucide-qr-code"><rect width="5" height="5" x="3" y="3" rx="1"/><rect width="5" height="5" x="16" y="3" rx="1"/><rect width="5" height="5" x="3" y="16" rx="1"/><path d="M21 16h-3a2 2 0 0 0-2 2v3"/><path d="M21 21v.01"/><path d="M12 7v3a2 2 0 0 1-2 2H7"/><path d="M3 12h.01"/><path d="M12 3h.01"/><path d="M12 16v.01"/><path d="M16 12h1"/><path d="M21 12v.01"/><path d="M12 21v-1"/></svg>"#
                                            )
                                                .width(64)
                                                .height(64)
                                        )
                                )
                        )
                )
        )
    });

    if let Err(e) = app.run() {
        eprintln!("Error running app: {:?}", e);
    }

    Ok(())
}
