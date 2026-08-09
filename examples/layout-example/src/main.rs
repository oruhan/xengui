// SPDX-License-Identifier: Apache-2.0
// hide console window on windows subsystem
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use std::{ cell::Cell, rc::Rc };

use xenframe::{ App, AppConfig };
#[cfg(not(target_arch = "wasm32"))]
use xenframe::WindowPosition;
use xengui::*;

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
        title: "Layout Example".into(),
        #[cfg(not(target_arch = "wasm32"))]
        width: 800,
        #[cfg(not(target_arch = "wasm32"))]
        height: 600,
        #[cfg(not(target_arch = "wasm32"))]
        position: WindowPosition::Center,
        theme_mode: xenframe::AppThemeMode::System,

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
        let (sidebar_width, _) = use_state(Rc::new(Cell::new(240.0)));
        let (outline_width, _) = use_state(Rc::new(Cell::new(240.0)));
        let (bottom_height, _) = use_state(Rc::new(Cell::new(200.0)));

        Box::new(
            ContextMenu::new()
                .font("Noto_Sans")
                .menu_background(|theme: &Theme| theme.surface)
                .border(|theme: &Theme| Border::all(1, theme.outline).radius(8))
                .item_hover_background(|theme: &Theme| theme.surface)
                .item(ContextMenuItem::new("Reset counter"))
                .item(ContextMenuItem::new("Clear text"))
                .divider()
                .item(ContextMenuItem::new("About"))
                .child(
                    View::new()
                        .font("Noto_Sans")
                        .display(Display::Flex)
                        .flex_direction(FlexDirection::Column)
                        .justify_content(JustifyContent::Center)
                        .align_items(Align::Center)
                        .width(Length::pct(100.0))
                        .height(Length::pct(100.0))
                        .background(|theme: &Theme| theme.background)
                        .padding(Edges::all(15))
                        .gap(0, 10)
                        .child(
                            split_pane_with_bottom(
                                Some(
                                    SplitPanel::new(
                                        View::new()
                                            .display(Display::Flex)
                                            .align_items(Align::Center)
                                            .justify_content(JustifyContent::Center)
                                            .background(|theme: &Theme| theme.tertiary)
                                            .color(|theme: &Theme| theme.on_tertiary)
                                            .width(pct!(100))
                                            .child(Label::new().label("Sidebar box")),
                                        sidebar_width.clone()
                                    )
                                        .min_size(180.0)
                                        .max_size(480.0)
                                        .key("sidebar")
                                ),
                                View::new()
                                    .display(Display::Flex)
                                    .align_items(Align::Center)
                                    .justify_content(JustifyContent::Center)
                                    .background(|theme: &Theme| theme.secondary)
                                    .color(|theme: &Theme| theme.on_secondary)
                                    .width(pct!(100))
                                    .child(Label::new().label("Content box")),
                                Some(
                                    SplitPanel::new(
                                        View::new()
                                            .display(Display::Flex)
                                            .align_items(Align::Center)
                                            .justify_content(JustifyContent::Center)
                                            .background(|theme: &Theme| theme.tertiary)
                                            .color(|theme: &Theme| theme.on_tertiary)
                                            .width(pct!(100))
                                            .child(Label::new().label("Outline box")),
                                        outline_width.clone()
                                    )
                                        .min_size(160.0)
                                        .max_size(400.0)
                                        .key("outline")
                                ),
                                Some(
                                    SplitPanel::new(
                                        View::new()
                                            .display(Display::Flex)
                                            .align_items(Align::Center)
                                            .justify_content(JustifyContent::Center)
                                            .background(|theme: &Theme| theme.primary)
                                            .color(|theme: &Theme| theme.on_primary)
                                            .width(pct!(100))
                                            .child(Label::new().label("Bottom box")),
                                        bottom_height.clone()
                                    )
                                        .min_size(160.0)
                                        .max_size(400.0)
                                        .key("bottom")
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
