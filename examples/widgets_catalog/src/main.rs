// hide console window on windows subsystem
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::cell::Cell;
use std::rc::Rc;
use web_time::Duration;
use xenframe::{ App, AppConfig };
#[cfg(not(target_arch = "wasm32"))]
use xenframe::WindowPosition;
use xengui::*;
use xengui_icons::{ IconAxes, codepoints };

#[path = "../components/mod.rs"]
mod components;
use components::*;

const WAND_ICON: &str =
    r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24"
viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"
stroke-linecap="round" stroke-linejoin="round">
<path d="m21.64 3.64-1.28-1.28a1.21 1.21 0 0 0-1.72 0L2.36 18.64a1.21 1.21 0 0 0 0 1.72l1.28 1.28a1.2 1.2 0 0 0 1.72 0L21.64 5.36a1.2 1.2 0 0 0 0-1.72"/>
<path d="m14 7 3 3"/>
<path d="M5 6v4"/>
<path d="M19 14v4"/>
<path d="M10 2v2"/>
<path d="M7 8H3"/>
<path d="M21 16h-4"/>
<path d="M11 3H9"/>
</svg>"#;

// Card wrapper shared by every demo section: title, subtitle, and an
// arbitrary widget body (View, Table, composite widgets, ...).
fn section(theme: &Theme, title: &str, subtitle: &str, body: impl Widget + 'static) -> View {
    View::new()
        .display(Display::Flex)
        .flex_direction(FlexDirection::Column)
        .gap(0.0, 10.0)
        .padding(Edges::all(18.0))
        .background(theme.surface)
        .border(Border::all(1.0, theme.outline_variant).radius(theme.radius_lg))
        .child(
            View::new()
                .display(Display::Flex)
                .flex_direction(FlexDirection::Column)
                .gap(0.0, 2.0)
                .child(
                    Label::new()
                        .label(title)
                        .font_size(18.0)
                        .font_weight(FontWeight::SemiBold)
                        .color(theme.on_surface)
                )
                .child(Label::new().label(subtitle).font_size(13.0).color(theme.on_surface_variant))
        )
        .child(body)
}

fn section_buttons(theme: &Theme, click_count: i32, set_click_count: SetState<i32>) -> View {
    let primary = Button::new()
        .label(format!("Primary ({click_count})"))
        .font_size(14.0)
        .padding(Edges::only(14.0, 8.0, 14.0, 8.0))
        .background(theme.primary)
        .color(theme.on_primary)
        .border(Border::all(1.0, theme.primary).radius(8.0))
        .transition_all(Transition::new(Duration::from_millis(150)))
        .hover_style(|s, theme: &Theme| s.background(theme.primary_fixed_dim))
        .pressed_style(|s, _theme: &Theme| s.scale(0.97))
        .on_click(move |_ctx| set_click_count.set(click_count + 1));

    let test = Button::new()
        .label("Test")
        .background(Color::NEUTRAL_700)
        .focus_style(|ctx: StylePatch, _theme: &Theme| ctx.background(Color::AMBER_500))
        .pressed_style(|ctx: StylePatch, _theme: &Theme| ctx.background(Color::RED_500));

    let icon_start = Button::new()
        .label("Icon start")
        .icon(WAND_ICON)
        .icon_position(IconPosition::Start)
        .icon_gap(8.0)
        .font_size(14.0)
        .padding(Edges::only(14.0, 8.0, 14.0, 8.0))
        .background(theme.secondary_container)
        .color(theme.on_secondary_container)
        .border(Border::all(1.0, theme.secondary_container).radius(8.0));

    let icon_end = Button::new()
        .label("Icon end")
        .icon(WAND_ICON)
        .icon_position(IconPosition::End)
        .font_size(14.0)
        .padding(Edges::only(14.0, 8.0, 14.0, 8.0))
        .background(theme.tertiary_container)
        .color(theme.on_tertiary_container)
        .border(Border::all(1.0, theme.tertiary_container).radius(8.0));

    let disabled = Button::new()
        .label("Disabled")
        .font_size(14.0)
        .padding(Edges::only(14.0, 8.0, 14.0, 8.0))
        .background(theme.surface_container)
        .color(theme.on_surface_variant)
        .border(Border::all(1.0, theme.outline_variant).radius(8.0))
        .enabled(false);

    section(
        theme,
        "Button",
        "Hover/press transitions (transition_all + scale), start/end icons, and a disabled state.",
        Row::new()
            .gap(10.0, 10.0)
            .flex_wrap(FlexWrap::Wrap)
            .child(primary)
            .child(icon_start)
            .child(icon_end)
            .child(disabled)
            .child(test)
    )
}

fn section_checkbox(
    theme: &Theme,
    checked: bool,
    indeterminate: bool,
    set_checked: SetState<bool>,
    set_indeterminate: SetState<bool>
) -> View {
    let row = |theme: &Theme, checkbox: Checkbox, text: &str| -> View {
        Row::new()
            .gap(8.0, 0.0)
            .align_items(Align::Center)
            .child(checkbox)
            .child(Label::new().label(text).color(theme.on_surface).font_size(13.0))
    };

    let live = row(
        theme,
        Checkbox::new()
            .checked(checked)
            .on_change(move |value, _ctx| set_checked.set(value)),
        "Interactive"
    );

    let indet = row(
        theme,
        Checkbox::new()
            .checked(false)
            .indeterminate(indeterminate)
            .on_change(move |_value, _ctx| set_indeterminate.set(!indeterminate)),
        "Indeterminate (click the box to toggle)"
    );

    let checked_disabled = row(
        theme,
        Checkbox::new().checked(true).enabled(false),
        "Disabled + checked"
    );
    let unchecked_disabled = row(
        theme,
        Checkbox::new().checked(false).enabled(false),
        "Disabled + unchecked"
    );

    section(
        theme,
        "Checkbox",
        "Controlled checked/indeterminate state, plus disabled variants.",
        Column::new()
            .gap(0.0, 10.0)
            .child(live)
            .child(indet)
            .child(checked_disabled)
            .child(unchecked_disabled)
    )
}

fn section_switch(theme: &Theme, switch_on: bool, set_switch_on: SetState<bool>) -> View {
    let row = |theme: &Theme, switch: Switch, text: &str| -> View {
        Row::new()
            .gap(10.0, 0.0)
            .align_items(Align::Center)
            .child(switch)
            .child(Label::new().label(text).color(theme.on_surface).font_size(13.0))
    };

    let live = row(
        theme,
        Switch::new()
            .checked(switch_on)
            .on_change({
                let set_switch_on = set_switch_on.clone();
                move |value, _ctx| set_switch_on.set(value)
            }),
        "Interactive"
    );
    let small = row(theme, Switch::new().checked(switch_on).size(0.7), "Scaled (size 0.7)");
    let on_disabled = row(theme, Switch::new().checked(true).enabled(false), "Disabled + on");
    let off_disabled = row(theme, Switch::new().checked(false).enabled(false), "Disabled + off");

    section(
        theme,
        "Switch",
        "Material-style toggle with an animated thumb, plus scaled and disabled variants.",
        Column::new().gap(0.0, 10.0).child(live).child(small).child(on_disabled).child(off_disabled)
    )
}

fn section_radio(theme: &Theme, selected: usize, set_selected: SetState<usize>) -> View {
    let option = |theme: &Theme, index: usize, label: &str, set_selected: SetState<usize>| -> View {
        Row::new()
            .gap(8.0, 0.0)
            .align_items(Align::Center)
            .child(
                RadioButton::new()
                    .selected(selected == index)
                    .on_select(move |_ctx| set_selected.set(index))
            )
            .child(Label::new().label(label).color(theme.on_surface).font_size(13.0))
    };

    let disabled_option = Row::new()
        .gap(8.0, 0.0)
        .align_items(Align::Center)
        .child(RadioButton::new().selected(false).enabled(false))
        .child(
            Label::new().label("Disabled option").color(theme.on_surface_variant).font_size(13.0)
        );

    section(
        theme,
        "RadioButton",
        "Single-selection group backed by shared state, plus a disabled option.",
        Row::new()
            .gap(20.0, 0.0)
            .flex_wrap(FlexWrap::Wrap)
            .child(option(theme, 0, "Option A", set_selected.clone()))
            .child(option(theme, 1, "Option B", set_selected.clone()))
            .child(option(theme, 2, "Option C", set_selected.clone()))
            .child(disabled_option)
    )
}

fn section_textbox(theme: &Theme, text_value: &str, set_text_value: SetState<String>) -> View {
    let styled = |tb: TextBox, theme: &Theme| -> TextBox {
        tb.padding(Edges::all(8.0))
            .background(theme.surface)
            .border(Border::all(1.0, theme.outline).radius(8.0))
            .focus_style(|s, theme: &Theme| {
                s.border(Border::all(1.5, theme.primary).radius(8.0)).outline(
                    Outline::new(1.5, theme.primary, Some(BorderRadius::all(8.0)), 0.0)
                )
            })
    };

    let live = styled(
        TextBox::new()
            .value(text_value.to_string())
            .placeholder("Type something…")
            .on_change(move |value, _ctx| set_text_value.set(value.to_string())),
        theme
    );

    let limited = styled(TextBox::new().placeholder("Max 8 characters").max_length(8), theme);
    let read_only = styled(TextBox::new().value("Read only value"), theme).read_only(true);
    let disabled = styled(TextBox::new().value("Disabled"), theme).enabled(false);

    section(
        theme,
        "TextBox",
        "Controlled value, max_length, read_only, and disabled states.",
        Column::new().gap(0.0, 10.0).child(live).child(limited).child(read_only).child(disabled)
    )
}

fn section_link(theme: &Theme) -> View {
    let normal = Link::new()
        .label("xengui on GitHub")
        .href("https://github.com/randseas/xengui")
        .color(theme.primary)
        .font_size(14.0);

    let selectable = Link::new()
        .label("Selectable link text — try dragging to select it")
        .href("https://example.com")
        .selectable(true)
        .color(theme.primary)
        .font_size(13.0);

    let new_tab = Link::new()
        .label("target_blank(true) — opens in a new tab")
        .href("https://example.com")
        .target_blank(true)
        .color(theme.primary)
        .font_size(13.0);

    let disabled = Link::new()
        .label("Disabled link")
        .href("https://example.com")
        .enabled(false)
        .color(theme.on_surface_variant)
        .font_size(13.0);

    section(
        theme,
        "Link",
        "Selectable text, new-tab target, and a disabled state.",
        Column::new().gap(0.0, 8.0).child(normal).child(selectable).child(new_tab).child(disabled)
    )
}

fn section_image_svg(theme: &Theme) -> View {
    let fit_demo = |theme: &Theme, fit: ObjectFit, label: &str| -> View {
        Column::new()
            .gap(0.0, 4.0)
            .align_items(Align::Center)
            .child(
                View::new()
                    .width(90.0)
                    .height(60.0)
                    .border(Border::all(1.0, theme.outline_variant).radius(6.0))
                    .overflow_x(Overflow::Hidden)
                    .overflow_y(Overflow::Hidden)
                    .child(
                        Image::new()
                            .bytes(
                                include_bytes!(
                                    concat!(env!("CARGO_MANIFEST_DIR"), "/assets/ferris.png")
                                )
                            )
                            .object_fit(fit)
                            .width(90.0)
                            .height(60.0)
                    )
            )
            .child(Label::new().label(label).color(theme.on_surface_variant).font_size(11.0))
    };

    // Programmatically built SVG (as opposed to Svg::from_string parsing markup).
    let built_svg = Svg::new()
        .view_box(0.0, 0.0, 24.0, 24.0)
        .circle(12.0, 12.0, 10.0, |c| c.fill(theme.primary))
        .rect(7.0, 7.0, 10.0, 10.0, |r| r.radius(2.0).fill(Color::WHITE.with_alpha(200)))
        .width(40.0)
        .height(40.0);

    let wand_svg = Svg::from_string(WAND_ICON).width(28.0).height(28.0).color(theme.on_background);

    section(
        theme,
        "Image & Svg",
        "object_fit variants, plus a parsed SVG (currentColor) and a builder-constructed SVG.",
        Column::new()
            .gap(0.0, 14.0)
            .child(
                Row::new()
                    .gap(16.0, 0.0)
                    .child(fit_demo(theme, ObjectFit::Fill, "Fill"))
                    .child(fit_demo(theme, ObjectFit::Contain, "Contain"))
                    .child(fit_demo(theme, ObjectFit::Cover, "Cover"))
                    .child(fit_demo(theme, ObjectFit::None, "None"))
            )
            .child(
                Row::new()
                    .gap(16.0, 0.0)
                    .align_items(Align::Center)
                    .child(built_svg)
                    .child(wand_svg)
            )
    )
}

fn section_tooltip(theme: &Theme) -> View {
    let chip = |theme: &Theme, placement: TooltipPlacement, text: &str, tip: &str| -> Tooltip {
        Tooltip::new(tip)
            .placement(placement)
            .background(theme.inverse_surface)
            .text_color(theme.inverse_on_surface)
            .child(
                View::new()
                    .padding(Edges::symmetric(12.0, 8.0))
                    .background(theme.surface_container)
                    .border(Border::all(1.0, theme.outline_variant).radius(8.0))
                    .child(Label::new().label(text).color(theme.on_surface).font_size(12.0))
            )
    };

    section(
        theme,
        "Tooltip",
        "Hover each box; the popup placement differs per side.",
        Row::new()
            .gap(16.0, 0.0)
            .flex_wrap(FlexWrap::Wrap)
            .child(chip(theme, TooltipPlacement::Top, "Top", "I appear above"))
            .child(chip(theme, TooltipPlacement::Bottom, "Bottom", "I appear below"))
            .child(chip(theme, TooltipPlacement::Left, "Left", "I appear to the left"))
            .child(chip(theme, TooltipPlacement::Right, "Right", "I appear to the right"))
    )
}

fn section_richtext_kbd(theme: &Theme, text_value: &str) -> View {
    let richtext = RichText::new()
        .font_size(14.0)
        .span(TextSpan::new("Normal, ").color(theme.on_surface))
        .span(TextSpan::new("bold, ").color(theme.on_surface).weight(FontWeight::Bold))
        .span(TextSpan::new("italic, ").color(theme.on_surface).style(FontStyle::Italic))
        .span(
            TextSpan::new("underlined, ").color(theme.primary).decoration(TextDecoration::UNDERLINE)
        )
        .span(
            TextSpan::new("strikethrough ")
                .color(theme.on_surface_variant)
                .decoration(TextDecoration::STRIKETHROUGH)
        )
        .span(
            TextSpan::new(format!("and live: \"{text_value}\""))
                .color(theme.tertiary)
                .weight(FontWeight::SemiBold)
        );

    let kbd_row = Row::new()
        .gap(6.0, 0.0)
        .align_items(Align::Center)
        .child(Kbd::new().label("Ctrl"))
        .child(Label::new().label("+").color(theme.on_surface_variant))
        .child(Kbd::new().label("Shift"))
        .child(Label::new().label("+").color(theme.on_surface_variant))
        .child(Kbd::new().label("P"))
        .child(
            Label::new().label("  command palette").color(theme.on_surface_variant).font_size(12.0)
        );

    section(
        theme,
        "RichText & Kbd",
        "Mixed inline span styling (color/weight/style/decoration) and keyboard-shortcut badges.",
        Column::new().gap(0.0, 12.0).child(richtext).child(kbd_row)
    )
}

fn section_table(theme: &Theme) -> View {
    let table = Table::new()
        .width(Length::pct(100.0))
        .striped(true)
        .row_hover_background(theme.surface_container_high)
        .border_color(theme.outline_variant)
        .columns(
            vec![
                TableColumn::new("Widget", Length::pct(45.0)),
                TableColumn::new("Category", Length::pct(30.0)),
                TableColumn::new("Status", Length::pct(25.0))
            ]
        )
        .row(TableRow::new().text("Button").text("Input").text("Stable"))
        .row(TableRow::new().text("Checkbox").text("Input").text("Stable"))
        .row(TableRow::new().text("Switch").text("Input").text("Stable"))
        .row(TableRow::new().text("ContextMenu").text("Overlay").text("Stable"))
        .row(TableRow::new().text("SplitPane").text("Layout").text("Stable"))
        .row(TableRow::new().text("Portal").text("Layout").text("Stable"));

    section(
        theme,
        "Table",
        "Composite widget built from View/Label; striped rows and a custom hover background.",
        table
    )
}

fn section_variable_icons(theme: &Theme) -> View {
    let sample = |theme: &Theme, fill: f32, weight: f32, label: &str| -> View {
        Column::new()
            .gap(0.0, 4.0)
            .align_items(Align::Center)
            .child(
                VariableIcon::new(codepoints::CHECK)
                    .size(28.0)
                    .axes(IconAxes::default().fill(fill).weight(weight))
                    .color(theme.primary)
            )
            .child(Label::new().label(label).color(theme.on_surface_variant).font_size(11.0))
    };

    section(
        theme,
        "VariableIcon",
        "Continuous weight/fill blending straight from the variable font — no pre-baked SVG variants.",
        Row::new()
            .gap(20.0, 0.0)
            .align_items(Align::Center)
            .child(sample(theme, 0.0, 300.0, "wght 300"))
            .child(sample(theme, 0.0, 700.0, "wght 700"))
            .child(sample(theme, 1.0, 400.0, "fill 1, wght 400"))
            .child(sample(theme, 1.0, 700.0, "fill 1, wght 700"))
    )
}

fn cursor_chip(theme: &Theme, cursor: Cursor, label: &str) -> View {
    View::new()
        .padding(Edges::symmetric(10.0, 6.0))
        .background(theme.surface_container)
        .border(Border::all(1.0, theme.outline_variant).radius(6.0))
        .cursor(cursor)
        .child(Label::new().label(label).color(theme.on_surface).font_size(12.0))
}

fn section_cursors(theme: &Theme) -> View {
    section(
        theme,
        "Cursors",
        "Hover each chip to see the platform cursor icon change.",
        Row::new()
            .gap(8.0, 8.0)
            .flex_wrap(FlexWrap::Wrap)
            .child(cursor_chip(theme, Cursor::Pointer, "Pointer"))
            .child(cursor_chip(theme, Cursor::Grab, "Grab"))
            .child(cursor_chip(theme, Cursor::Crosshair, "Crosshair"))
            .child(cursor_chip(theme, Cursor::NotAllowed, "NotAllowed"))
            .child(cursor_chip(theme, Cursor::Text, "Text"))
            .child(cursor_chip(theme, Cursor::EwResize, "EwResize"))
    )
}

fn section_style_playground(theme: &Theme) -> View {
    let gradient_card = View::new()
        .width(150.0)
        .height(90.0)
        .background(
            Background::LinearGradient(
                LinearGradient::new(
                    135.0,
                    vec![
                        GradientStop::new(Color::VIOLET_400, 0.0),
                        GradientStop::new(Color::PINK_400, 1.0)
                    ]
                )
            )
        )
        .border(Border::all(0.0, Color::TRANSPARENT).radius(12.0))
        .align_items(Align::Center)
        .justify_content(JustifyContent::Center)
        .child(Label::new().label("LinearGradient").color(Color::WHITE).font_size(12.0));

    let radial_card = View::new()
        .width(150.0)
        .height(90.0)
        .background(
            Background::RadialGradient(
                RadialGradient::new(
                    vec![
                        GradientStop::new(Color::CYAN_300, 0.0),
                        GradientStop::new(Color::BLUE_700, 1.0)
                    ]
                )
            )
        )
        .border(Border::all(0.0, Color::TRANSPARENT).radius(12.0))
        .align_items(Align::Center)
        .justify_content(JustifyContent::Center)
        .child(Label::new().label("RadialGradient").color(Color::WHITE).font_size(12.0));

    let shadow_card = View::new()
        .width(150.0)
        .height(90.0)
        .margin(Edges::all(10.0))
        .background(theme.surface)
        .border(Border::all(1.0, theme.outline_variant).radius(12.0))
        .box_shadow(
            vec![
                BoxShadow::new(0.0, 8.0, 20.0, Color::BLACK.with_alpha(70)),
                BoxShadow::new(0.0, 1.0, 3.0, Color::BLACK.with_alpha(120))
            ]
        )
        .align_items(Align::Center)
        .justify_content(JustifyContent::Center)
        .child(Label::new().label("box_shadow x2").color(theme.on_surface).font_size(12.0));

    let filter_card = View::new()
        .width(150.0)
        .height(90.0)
        .background(
            Background::LinearGradient(
                LinearGradient::new(
                    90.0,
                    vec![
                        GradientStop::new(Color::LIME_400, 0.0),
                        GradientStop::new(Color::TEAL_500, 1.0)
                    ]
                )
            )
        )
        .border(Border::all(0.0, Color::TRANSPARENT).radius(12.0))
        .filter(FilterChain::new().push(Filter::Grayscale(0.6)).push(Filter::Brightness(1.1)))
        .align_items(Align::Center)
        .justify_content(JustifyContent::Center)
        .child(Label::new().label("Filter chain").color(Color::WHITE).font_size(12.0));

    let drop_shadow_card = View::new()
        .width(150.0)
        .height(90.0)
        .margin(Edges::all(10.0))
        .background(theme.surface)
        .filter(Filter::DropShadow(DropShadow::new(3.0, 6.0, 8.0, Color::BLACK.with_alpha(140))))
        .align_items(Align::Center)
        .justify_content(JustifyContent::Center)
        .child(Label::new().label("Filter::DropShadow").color(theme.on_surface).font_size(12.0));

    // Two stacked layers: a gradient behind, and a translucent absolute
    // overlay with backdrop_filter blurring whatever is already painted
    // underneath it (CSS backdrop-filter equivalent).
    let backdrop_card = View::new()
        .position(Position::Relative)
        .width(150.0)
        .height(90.0)
        .child(
            View::new()
                .width(Length::pct(100.0))
                .height(Length::pct(100.0))
                .background(
                    Background::LinearGradient(
                        LinearGradient::new(
                            45.0,
                            vec![
                                GradientStop::new(Color::ORANGE_400, 0.0),
                                GradientStop::new(Color::FUCHSIA_500, 1.0)
                            ]
                        )
                    )
                )
        )
        .child(
            View::new()
                .position(Position::Absolute)
                .top(0.0)
                .left(0.0)
                .width(Length::pct(100.0))
                .height(Length::pct(100.0))
                .background(Color::WHITE.with_alpha(50))
                .backdrop_filter(Filter::Blur(Length::px(6.0)))
                .align_items(Align::Center)
                .justify_content(JustifyContent::Center)
                .child(Label::new().label("backdrop_filter").color(Color::WHITE).font_size(12.0))
        );

    let radius_card = View::new()
        .width(150.0)
        .height(90.0)
        .background(theme.primary)
        .border(Border::all(2.0, theme.outline).radius(BorderRadius::only(28.0, 4.0, 28.0, 4.0)))
        .align_items(Align::Center)
        .justify_content(JustifyContent::Center)
        .child(Label::new().label("Per-corner radius").color(theme.on_primary).font_size(12.0));

    let outline_card = View::new()
        .width(150.0)
        .height(90.0)
        .background(theme.surface)
        .border(Border::all(1.0, theme.outline_variant).radius(8.0))
        .outline(Outline::new(2.0, theme.primary, Some(BorderRadius::all(10.0)), 3.0))
        .align_items(Align::Center)
        .justify_content(JustifyContent::Center)
        .child(Label::new().label("Static outline").color(theme.on_surface).font_size(12.0));

    // cursor(...) also activates Interaction::is_active(), which is what
    // lets a plain View receive hover state and apply hover_style at all.
    let hover_card = View::new()
        .width(150.0)
        .height(90.0)
        .background(theme.surface_container)
        .border(Border::all(1.0, theme.outline_variant).radius(12.0))
        .cursor(Cursor::Pointer)
        .transition_colors(Transition::new(Duration::from_millis(200)).easing(Easing::EaseOut))
        .hover_style(|s, theme: &Theme| s.background(theme.primary_container))
        .align_items(Align::Center)
        .justify_content(JustifyContent::Center)
        .child(
            Label::new().label("Hover: transition_colors").color(theme.on_surface).font_size(12.0)
        );

    section(
        theme,
        "Style playground",
        "Gradients, layered box_shadow, GPU filter chains, drop-shadow, backdrop-filter, per-corner radius, a static outline, and an animated hover transition.",
        Row::new()
            .gap(14.0, 14.0)
            .flex_wrap(FlexWrap::Wrap)
            .child(gradient_card)
            .child(radial_card)
            .child(shadow_card)
            .child(filter_card)
            .child(drop_shadow_card)
            .child(backdrop_card)
            .child(radius_card)
            .child(outline_card)
            .child(hover_card)
    )
}

fn scroll_demo_box(theme: &Theme, overscroll: Overscroll, label: &str) -> View {
    let mut inner = View::new()
        .display(Display::Flex)
        .flex_direction(FlexDirection::Column)
        .gap(0.0, 6.0)
        .padding(Edges::all(8.0));

    for i in 1..=20 {
        inner = inner.child(
            Label::new().label(format!("{label} item {i}")).color(theme.on_surface).font_size(12.0)
        );
    }

    View::new()
        .width(200.0)
        .height(140.0)
        .overflow_y(Overflow::Auto)
        .overscroll(overscroll)
        .scrollbar_thickness(6.0)
        .scrollbar_thumb_color(theme.primary)
        .scrollbar_track_color(theme.surface_container)
        .scrollbar_gutter(ScrollbarGutter::Stable)
        .border(Border::all(1.0, theme.outline_variant).radius(8.0))
        .background(theme.surface)
        .child(inner)
}

fn section_scrolling(theme: &Theme) -> View {
    section(
        theme,
        "Scrolling & Overscroll",
        "Custom scrollbar colors/thickness and three Overscroll behaviors — scroll each box past its bounds.",
        Row::new()
            .gap(16.0, 0.0)
            .flex_wrap(FlexWrap::Wrap)
            .child(scroll_demo_box(theme, Overscroll::Disabled, "Disabled"))
            .child(scroll_demo_box(theme, Overscroll::Bounce, "Bounce"))
            .child(scroll_demo_box(theme, Overscroll::Glow, "Glow"))
    )
}

fn grid_cell(theme: &Theme, label: &str) -> View {
    View::new()
        .display(Display::Flex)
        .align_items(Align::Center)
        .justify_content(JustifyContent::Center)
        .height(48.0)
        .background(theme.surface_container)
        .border(Border::all(1.0, theme.outline_variant).radius(6.0))
        .child(Label::new().label(label).color(theme.on_surface).font_size(13.0))
}

fn section_grid(theme: &Theme) -> View {
    let mut wide = grid_cell(theme, "2 (columns 2–4)");
    wide = wide.grid_column(GridPlacement::span(2, 4));

    let grid = View::new()
        .display(Display::Grid)
        .width(Length::pct(100.0))
        .grid_template_columns(vec![GridTrack::Fr(1.0), GridTrack::Fr(1.0), GridTrack::Fr(1.0)])
        .gap(10.0, 10.0)
        .child(grid_cell(theme, "1"))
        .child(wide)
        .child(grid_cell(theme, "3"))
        .child(grid_cell(theme, "4"));

    section(
        theme,
        "CSS Grid layout",
        "Display::Grid with grid_template_columns and explicit grid_column placement.",
        grid
    )
}

fn section_portal(theme: &Theme, open: bool, set_open: SetState<bool>) -> View {
    let toggle_label = if open { "Hide portal content" } else { "Show portal content" };

    let mut clipper = View::new()
        .width(260.0)
        .height(90.0)
        .overflow_y(Overflow::Hidden)
        .position(Position::Relative)
        .border(Border::all(1.0, theme.outline_variant).radius(8.0))
        .background(theme.surface)
        .padding(Edges::all(10.0))
        .child(
            Label::new()
                .label("This box clips overflow (overflow_y: Hidden).")
                .color(theme.on_surface_variant)
                .font_size(12.0)
        );

    if open {
        clipper = clipper.child(
            Portal::new().child(
                View::new()
                    .position(Position::Absolute)
                    .top(50.0)
                    .left(40.0)
                    .width(180.0)
                    .height(70.0)
                    .background(theme.primary)
                    .border(Border::all(1.0, theme.primary).radius(8.0))
                    .box_shadow(BoxShadow::new(0.0, 6.0, 16.0, Color::BLACK.with_alpha(90)))
                    .align_items(Align::Center)
                    .justify_content(JustifyContent::Center)
                    .child(
                        Label::new()
                            .label("Escaped the clip via Portal")
                            .color(theme.on_primary)
                            .font_size(12.0)
                    )
            )
        );
    }

    section(
        theme,
        "Portal",
        "Renders unclipped in the top paint layer, escaping an ancestor's overflow clip.",
        Column::new()
            .gap(0.0, 10.0)
            .child(clipper)
            .child(
                Button::new()
                    .label(toggle_label)
                    .font_size(13.0)
                    .padding(Edges::symmetric(10.0, 6.0))
                    .background(theme.surface_container)
                    .color(theme.on_surface)
                    .border(Border::all(1.0, theme.outline_variant).radius(6.0))
                    .on_click(move |_ctx| set_open.set(!open))
            )
    )
}

fn section_splitpane(theme: &Theme, left_size: Rc<Cell<f32>>, right_size: Rc<Cell<f32>>) -> View {
    let panel = |theme: &Theme, label: &str| -> View {
        View::new()
            .width(Length::pct(100.0))
            .height(Length::pct(100.0))
            .align_items(Align::Center)
            .justify_content(JustifyContent::Center)
            .background(theme.surface_container)
            .child(Label::new().label(label).color(theme.on_surface).font_size(12.0))
    };

    let pane = split_pane(
        Some(
            SplitPanel::new(panel(theme, "Left panel"), left_size).min_size(100.0).max_size(260.0)
        ),
        panel(theme, "Center content"),
        Some(
            SplitPanel::new(panel(theme, "Right panel"), right_size).min_size(100.0).max_size(260.0)
        )
    );

    section(
        theme,
        "SplitPane",
        "Drag the thin dividers to resize the side panels. Size state is kept in use_state as Rc<Cell<f32>>.",
        View::new()
            .width(Length::pct(100.0))
            .height(220.0)
            .border(Border::all(1.0, theme.outline_variant).radius(8.0))
            .overflow_x(Overflow::Hidden)
            .child(pane)
    )
}

fn section_composite(theme: &Theme) -> View {
    section(
        theme,
        "Composite widgets (Render trait)",
        "TestButton is composed from View/Label via the Render trait and its own use_state, reconciled like any built-in widget.",
        Row::new()
            .gap(10.0, 0.0)
            .child(TestButton::new().label("Composite A").color(Color::BLUE_500))
            .child(TestButton::new().label("Composite B").color(Color::VIOLET_500))
            .child(TestButton::new().label("Composite C").color(Color::EMERALD_500))
    )
}

fn section_devtools_hint(theme: &Theme) -> View {
    section(
        theme,
        "DevTools",
        "The built-in render/repaint inspector overlay: rerenders, repaints, layout passes, warnings, and errors, color-coded with an HH:MM:SS timeline.",
        Row::new()
            .gap(8.0, 0.0)
            .align_items(Align::Center)
            .child(Kbd::new().label("F12"))
            .child(
                Label::new()
                    .label("toggle the DevTools panel")
                    .color(theme.on_surface_variant)
                    .font_size(13.0)
            )
    )
}

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
        title: "XenGui Widgets".into(),
        #[cfg(not(target_arch = "wasm32"))]
        width: 1000,
        #[cfg(not(target_arch = "wasm32"))]
        height: 760,
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
        let theme = current_theme();

        let (click_count, set_click_count) = use_state(0i32);
        let (checked, set_checked) = use_state(false);
        let (indeterminate, set_indeterminate) = use_state(false);
        let (switch_on, set_switch_on) = use_state(false);
        let (radio_selected, set_radio_selected) = use_state(0usize);
        let (text_value, set_text_value) = use_state(String::from("XenGui"));
        let (portal_open, set_portal_open) = use_state(false);
        let (last_menu_action, set_last_menu_action) = use_state(String::from("—"));
        let (left_panel_size, _) = use_state(Rc::new(Cell::new(160.0f32)));
        let (right_panel_size, _) = use_state(Rc::new(Cell::new(160.0f32)));

        use_effect(
            move || {
                log::info!("switch_on changed to {switch_on}");
            },
            [switch_on]
        );

        let header = View::new()
            .display(Display::Flex)
            .flex_direction(FlexDirection::Column)
            .gap(0.0, 4.0)
            .padding(Edges::only(24.0, 24.0, 24.0, 8.0))
            .child(
                Label::new()
                    .label("XenGui Widgets")
                    .font_size(24.0)
                    .font_weight(FontWeight::Bold)
                    .color(theme.on_background)
            )
            .child(
                Label::new()
                    .label(
                        format!(
                            "Every widget, every state · theme: {} · right-click for a menu · F12 for DevTools",
                            theme.name()
                        )
                    )
                    .font_size(13.0)
                    .color(theme.on_surface_variant)
            )
            .child(
                Label::new()
                    .label(format!("Last context-menu action: {last_menu_action}"))
                    .font_size(12.0)
                    .color(theme.tertiary)
            );

        let body = View::new()
            .display(Display::Flex)
            .flex_direction(FlexDirection::Column)
            .width(pct!(100.0))
            .height(pct!(100.0))
            .overflow_y(Overflow::Auto)
            .padding(Edges::only(24.0, 8.0, 24.0, 24.0))
            .gap(0.0, 16.0)
            .child(section_buttons(&theme, click_count, set_click_count.clone()))
            .child(
                section_checkbox(
                    &theme,
                    checked,
                    indeterminate,
                    set_checked.clone(),
                    set_indeterminate.clone()
                )
            )
            .child(
                Row::new()
                    .child(Checkbox::new().checked(checked).id("eula_accept"))
                    .child(Label::new().label("I Accept the EULA").for_control("eula_accept"))
            )
            .child(section_switch(&theme, switch_on, set_switch_on.clone()))
            .child(section_radio(&theme, radio_selected, set_radio_selected.clone()))
            .child(section_textbox(&theme, &text_value, set_text_value.clone()))
            .child(section_link(&theme))
            .child(section_image_svg(&theme))
            .child(section_tooltip(&theme))
            .child(section_richtext_kbd(&theme, &text_value))
            .child(section_table(&theme))
            .child(section_variable_icons(&theme))
            .child(section_cursors(&theme))
            .child(section_style_playground(&theme))
            .child(section_scrolling(&theme))
            .child(section_grid(&theme))
            .child(section_portal(&theme, portal_open, set_portal_open.clone()))
            .child(section_splitpane(&theme, left_panel_size, right_panel_size))
            .child(section_composite(&theme))
            .child(section_devtools_hint(&theme));

        let content = View::new()
            .font("Noto_Sans")
            .display(Display::Flex)
            .flex_direction(FlexDirection::Column)
            .width(Length::pct(100.0))
            .height(Length::pct(100.0))
            .background(theme.background)
            .child(header)
            .child(body);

        Box::new(
            ContextMenu::new()
                .font("Noto_Sans")
                .menu_background(theme.surface)
                .border(Border::all(1.0, theme.outline).radius(8.0))
                .item_hover_background(theme.surface_container_high)
                .item(
                    ContextMenuItem::new("Reset click counter").on_click({
                        let set_click_count = set_click_count.clone();
                        let set_last_menu_action = set_last_menu_action.clone();
                        move |_ctx| {
                            set_click_count.set(0);
                            set_last_menu_action.set("Counter reset".to_string());
                        }
                    })
                )
                .item(
                    ContextMenuItem::new("Clear text field").on_click({
                        let set_text_value = set_text_value.clone();
                        let set_last_menu_action = set_last_menu_action.clone();
                        move |_ctx| {
                            set_text_value.set(String::new());
                            set_last_menu_action.set("Text cleared".to_string());
                        }
                    })
                )
                .item(ContextMenuItem::new("Disabled item (no-op)").enabled(false))
                .divider()
                .item(
                    ContextMenuItem::new("More")
                        .submenu_item(
                            ContextMenuItem::new("Toggle switch")
                                .shortcut("Ctrl+S")
                                .on_click({
                                    let set_switch_on = set_switch_on.clone();
                                    move |_ctx|
                                        set_switch_on.update(|v| {
                                            *v = !*v;
                                        })
                                })
                        )
                        .submenu_divider()
                        .submenu_item(
                            ContextMenuItem::new("About XenGui").on_click({
                                let set_last_menu_action = set_last_menu_action.clone();
                                move |_ctx|
                                    set_last_menu_action.set("XenGui Widgets Catalog".to_string())
                            })
                        )
                )
                .child(content)
        )
    });

    if let Err(e) = app.run() {
        eprintln!("Error running app: {:?}", e);
    }

    Ok(())
}
