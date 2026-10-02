use std::time::Duration;
use crate::site_tokens::{type_scale, radius};
use xen_router::RouteParams;
use xengui::*;

fn example_card(kind: &str, title: &str, desc: &str, preview: impl Widget + 'static) -> View {
    Column::new()
        .flex_basis(Responsive::new(pct!(100.0)).md(pct!(48.0)).lg(pct!(31.0)))
        .min_width(px!(0.0))
        .background(|theme: &Theme| theme.surface)
        .border(|theme: &Theme| Border::all(1.0, theme.outline_variant).radius(radius::LG))
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
                        .font_size(type_scale::LABEL_SM)
                        .font_weight(FontWeight::SemiBold)
                        .letter_spacing(px!(1.0))
                        .color(|theme: &Theme| theme.primary),
                )
                .child(
                    Label::new()
                        .label(title)
                        .font_weight(FontWeight::SemiBold)
                        .font_size(type_scale::TITLE_MD)
                        .color(|theme: &Theme| theme.on_background),
                )
                .child(
                    RichText::new()
                        .with_content(desc)
                        .width(pct!(100.0))
                        .max_width(px!(420.0))
                        .font_size(type_scale::BODY_MD)
                        .line_height(px!(type_scale::BODY_MD_LINE))
                        .color(|theme: &Theme| theme.on_surface_variant),
                ),
        )
}

pub fn page(_params: &RouteParams) -> Box<dyn Widget> {
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
                .label("COMPONENT GALLERY")
                .font_size(type_scale::LABEL_SM)
                .font_weight(FontWeight::SemiBold)
                .letter_spacing(px!(1.1))
                .color(|theme: &Theme| theme.primary),
        )
        .child(
            RichText::new()
                .with_content("Core building blocks, with real behavior.")
                .width(pct!(100.0))
                .max_width(px!(760.0))
                .font_size(Responsive::new(px!(type_scale::DISPLAY_SM)).md(px!(type_scale::DISPLAY_MD)))
                .line_height(Responsive::new(px!(type_scale::DISPLAY_SM_LINE)).md(px!(type_scale::DISPLAY_MD_LINE)).resolve())
                .font_weight(FontWeight::SemiBold)
                .letter_spacing(px!(-1.8))
                .color(|theme: &Theme| theme.on_background),
        )
        .child(
            RichText::new()
                .with_content("Inspect XenGui widgets together with their theme, input, and layout behavior. These controls are interactive, not static mockups.")
                .width(pct!(100.0))
                .max_width(px!(680.0))
                .font_size(type_scale::BODY_LG)
                .line_height(px!(type_scale::BODY_LG_LINE))
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
            "Style hover, pressed, and focus states in one builder chain.",
            Button::new()
                .label("Save changes")
                .background(|theme: &Theme| theme.primary)
                .color(|theme: &Theme| theme.on_primary)
                .padding(Edges::symmetric(16.0, 10.0))
                .border(Border::all(0.0, Color::TRANSPARENT).radius(9.0)),
        ))
        .child(example_card(
            "BOOLEAN",
            "Switch",
            "A controlled on/off value with a change callback.",
            Row::new()
                .align_items(Align::Center)
                .gap(12.0, 0.0)
                .child(Switch::new().checked(true))
                .child(Label::new().label("Notifications").font_size(type_scale::BODY_MD)),
        ))
        .child(example_card(
            "SELECTION",
            "RadioButton",
            "Keyboard and pointer input for single-choice groups.",
            Row::new()
                .align_items(Align::Center)
                .gap(12.0, 0.0)
                .child(RadioButton::new().selected(true))
                .child(Label::new().label("Stable channel").font_size(type_scale::BODY_MD)),
        ))
        .child(example_card(
            "INPUT",
            "TextBox",
            "Selection, placeholder, IME, and submit behavior in one control.",
            TextBox::new()
                .placeholder("project-name")
                .width(Responsive::new(pct!(100.0)).md(px!(220.0))),
        ))
        .child(example_card(
            "FEEDBACK",
            "ProgressBar",
            "Determinate progress driven by application state.",
            Column::new()
                .width(Responsive::new(pct!(100.0)).md(px!(220.0)))
                .gap(0.0, 10.0)
                .child(ProgressBar::new().value(0.68))
                .child(
                    Label::new()
                        .label("Building · 68%")
                        .font_size(type_scale::BODY_SM)
                        .color(|theme: &Theme| theme.on_surface_variant),
                ),
        ))
        .child(example_card(
            "STATUS",
            "Badge + Kbd",
            "Compact status and keyboard hints for dense interfaces.",
            Row::new()
                .align_items(Align::Center)
                .gap(12.0, 0.0)
                .child(Badge::new().label("Stable"))
                .child(Kbd::new().label("Ctrl K")),
        ));

    Box::new(Column::new().width(pct!(100.0)).child(hero).child(grid))
}
