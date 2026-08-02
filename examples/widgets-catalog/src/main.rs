// SPDX-License-Identifier: Apache-2.0
// hide console window on windows subsystem
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use web_time::Duration;
use xenframe::{ App, AppConfig };
#[cfg(not(target_arch = "wasm32"))]
use xenframe::WindowPosition;
use xengui::*;

#[path = "../components/mod.rs"]
mod components;
use components::*;

const REPO_BASE: &str = "https://github.com/randseas/xengui/blob/main/crates/xengui/src/widgets/";

const WAND_ICON: &str =
    r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="m21.64 3.64-1.28-1.28a1.21 1.21 0 0 0-1.72 0L2.36 18.64a1.21 1.21 0 0 0 0 1.72l1.28 1.28a1.2 1.2 0 0 0 1.72 0L21.64 5.36a1.2 1.2 0 0 0 0-1.72"/><path d="m14 7 3 3"/><path d="M5 6v4"/><path d="M19 14v4"/><path d="M10 2v2"/><path d="M7 8H3"/><path d="M21 16h-4"/><path d="M11 3H9"/></svg>"#;

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
        title: "XenGui - Widgets Catalog".into(),
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
        let (click_count, set_click_count) = use_state(0i32);
        let (checked, set_checked) = use_state(false);
        let (switch_on, set_switch_on) = use_state(false);
        let (radio_selected, set_radio_selected) = use_state(0usize);
        let (text_value, set_text_value) = use_state(String::from("XenGui"));
        let (last_menu_action, set_last_menu_action) = use_state(String::from("—"));

        let button_row = TableRow::new()
            .cell(|| Link::new().label("Button").href(format!("{REPO_BASE}button.rs")))
            .cell({
                let set_click_count = set_click_count.clone();
                move || {
                    let set_click_count = set_click_count.clone();
                    Button::new()
                        .label(format!("Click ({click_count})"))
                        .font_size(14)
                        .background(Color::BLUE_500)
                        .padding(Edges::only(10, 6, 10, 6))
                        .border(Border::new(1, Color::BLUE_500, Length::px(8.0)))
                        .transition_all(Transition::new(Duration::from_millis(150)))
                        .hover_style(|s, _theme: &Theme| s.background(Color::BLUE_600))
                        .pressed_style(|s, _theme: &Theme|
                            s.background(Color::BLUE_700).scale(0.97)
                        )
                        .on_click(move |_ctx| set_click_count.set(click_count + 1))
                }
            });

        let checkbox_row = TableRow::new()
            .cell(|| Link::new().label("Checkbox").href(format!("{REPO_BASE}checkbox.rs")))
            .cell({
                let set_checked = set_checked.clone();
                move || {
                    Checkbox::new()
                        .checked(checked)
                        .on_change({
                            let set_checked = set_checked.clone();
                            move |value, _ctx| set_checked.set(value)
                        })
                }
            });

        let switch_row = TableRow::new()
            .cell(|| Link::new().label("Switch").href(format!("{REPO_BASE}switch.rs")))
            .cell({
                let set_switch_on = set_switch_on.clone();
                move || {
                    Switch::new()
                        .checked(switch_on)
                        .on_change({
                            let set_switch_on = set_switch_on.clone();
                            move |value, _ctx| set_switch_on.set(value)
                        })
                }
            });

        let radio_row = TableRow::new()
            .cell(|| Link::new().label("RadioButton").href(format!("{REPO_BASE}radio.rs")))
            .cell({
                let set_radio_selected = set_radio_selected.clone();
                move || {
                    let set_a = set_radio_selected.clone();
                    let set_b = set_radio_selected.clone();
                    let set_c = set_radio_selected.clone();
                    Row::new()
                        .gap(14, 0)
                        .align_items(AlignItems::Center)
                        .child(
                            RadioButton::new()
                                .selected(radio_selected == 0)
                                .on_select(move |_ctx| set_a.set(0))
                        )
                        .child(
                            Label::new()
                                .label("A")
                                .color(|theme: &Theme| theme.foreground)
                        )
                        .child(
                            RadioButton::new()
                                .selected(radio_selected == 1)
                                .on_select(move |_ctx| set_b.set(1))
                        )
                        .child(
                            Label::new()
                                .label("B")
                                .color(|theme: &Theme| theme.foreground)
                        )
                        .child(
                            RadioButton::new()
                                .selected(radio_selected == 2)
                                .on_select(move |_ctx| set_c.set(2))
                        )
                        .child(
                            Label::new()
                                .label("C")
                                .color(|theme: &Theme| theme.foreground)
                        )
                }
            });

        let textbox_row = TableRow::new()
            .cell(|| Link::new().label("TextBox").href(format!("{REPO_BASE}textbox.rs")))
            .cell({
                let text_value = text_value.clone();
                let set_text_value = set_text_value.clone();
                move || {
                    TextBox::new()
                        .value(text_value.clone())
                        .enabled(checked)
                        .read_only(switch_on)
                        .placeholder("Mark checkbox to enable")
                        .min_width(Length::px(160.0))
                        .padding(Edges::all(8))
                        .background(|theme: &Theme| theme.surface)
                        .border(|theme: &Theme| Border::new(1, theme.border, Length::px(8.0)))
                        .on_change({
                            let set_text_value = set_text_value.clone();
                            move |value, _ctx| set_text_value.set(value.to_string())
                        })
                }
            });

        let link_row = TableRow::new()
            .cell(|| Link::new().label("Link").href(format!("{REPO_BASE}link.rs")))
            .cell(||
                Link::new()
                    .label("github.com/randseas/xengui")
                    .href("https://github.com/randseas/xengui")
                    .color(|theme: &Theme| theme.primary)
            );

        let image_row = TableRow::new()
            .cell(|| Link::new().label("Image").href(format!("{REPO_BASE}image.rs")))
            .cell(||
                Image::new()
                    .bytes(
                        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/ferris.png"))
                    )
                    .object_fit(ObjectFit::Contain)
                    .width(64)
                    .height(42)
            );

        let svg_row = TableRow::new()
            .cell(|| Link::new().label("Svg").href(format!("{REPO_BASE}svg.rs")))
            .cell(||
                View::new()
                    .color(|theme: &Theme| theme.foreground_muted)
                    .child(Svg::from_string(WAND_ICON).width(22).height(22))
            );

        let tooltip_row = TableRow::new()
            .cell(|| Link::new().label("Tooltip").href(format!("{REPO_BASE}tooltip.rs")))
            .cell(||
                Tooltip::new("This is an tooltip").child(
                    Label::new()
                        .label("Hover")
                        .color(|theme: &Theme| theme.foreground)
                )
            );

        let rich_text_row = TableRow::new()
            .cell(|| Link::new().label("RichText").href(format!("{REPO_BASE}rich_text.rs")))
            .cell({
                let text_value = text_value.clone();
                move || {
                    RichText::new()
                        .span(TextSpan::new("Text: ").color(Color::NEUTRAL_500))
                        .span(TextSpan::new(text_value.clone()).weight(FontWeight::SemiBold))
                }
            });

        let kbd_row = TableRow::new()
            .cell(|| Link::new().label("Kbd").href(format!("{REPO_BASE}kbd.rs")))
            .cell(||
                Row::new().gap(6, 0).child(Kbd::new().label("Ctrl")).child(Kbd::new().label("K"))
            );

        let context_menu_row = TableRow::new()
            .cell(|| Link::new().label("ContextMenu").href(format!("{REPO_BASE}context_menu.rs")))
            .cell({
                let last_menu_action = last_menu_action.clone();
                move || {
                    Label::new()
                        .label(format!("Right click. Last action: {last_menu_action}"))
                        .color(|theme: &Theme| theme.foreground_muted)
                }
            });

        let test_button_row = TableRow::new()
            .cell(||
                Link::new()
                    .label("TestButton (example)")
                    .href(
                        "https://github.com/randseas/xengui/blob/main/examples/widgets-catalog/components/testbutton.rs"
                    )
            )
            .cell(|| TestButton::new().label("Composite").color(Color::BLUE_500));

        let table = Table::new()
            .width(Length::pct(100.0))
            .striped(true)
            .columns(
                vec![
                    TableColumn::new("Source", Length::pct(35.0)),
                    TableColumn::new("Widget", Length::pct(65.0))
                ]
            )
            .row(button_row)
            .row(checkbox_row)
            .row(switch_row)
            .row(radio_row)
            .row(textbox_row)
            .row(link_row)
            .row(image_row)
            .row(svg_row)
            .row(tooltip_row)
            .row(rich_text_row)
            .row(kbd_row)
            .row(context_menu_row)
            .row(test_button_row);

        Box::new(
            ContextMenu::new()
                .font("Noto_Sans")
                .menu_background(|theme: &Theme| theme.surface)
                .border(|theme: &Theme| Border::new(1, theme.border, Length::px(8.0)))
                .item_hover_background(|theme: &Theme| theme.hover)
                .item(
                    ContextMenuItem::new("Reset counter").on_click({
                        let set_click_count = set_click_count.clone();
                        let set_last_menu_action = set_last_menu_action.clone();
                        move |_ctx| {
                            set_click_count.set(0);
                            set_last_menu_action.set("Counter reset".to_string());
                        }
                    })
                )
                .item(
                    ContextMenuItem::new("Clear text").on_click({
                        let set_text_value = set_text_value.clone();
                        let set_last_menu_action = set_last_menu_action.clone();
                        move |_ctx| {
                            set_text_value.set(String::new());
                            set_last_menu_action.set("Text cleared".to_string());
                        }
                    })
                )
                .divider()
                .item(
                    ContextMenuItem::new("About").on_click({
                        let set_last_menu_action = set_last_menu_action.clone();
                        move |_ctx| set_last_menu_action.set("XenGui Widgets Catalog".to_string())
                    })
                )
                .child(
                    View::new()
                        .font("Noto_Sans")
                        .display(Display::Flex)
                        .flex_direction(FlexDirection::Column)
                        .justify_content(JustifyContent::Center)
                        .align_items(AlignItems::Center)
                        .width(Length::pct(100.0))
                        .height(Length::pct(100.0))
                        .background(|theme: &Theme| theme.background)
                        .padding(Edges::all(15))
                        .gap(0, 10)
                        .child(
                            Label::new()
                                .label("Widgets Catalog")
                                .font_size(Length::px(18.0))
                                .color(|theme: &Theme| theme.foreground)
                        )
                        .child(
                            View::new()
                                .flex_direction(FlexDirection::Column)
                                .overflow_y(Overflow::Auto)
                                .scrollbar_track_color(|theme: &Theme| theme.border)
                                .scrollbar_thumb_color(|theme: &Theme| theme.foreground_muted)
                                .scrollbar_arrow_color(|theme: &Theme| theme.foreground_muted)
                                .child(table)
                        )
                )
        )
    });

    if let Err(e) = app.run() {
        eprintln!("Error running app: {:?}", e);
    }

    Ok(())
}
