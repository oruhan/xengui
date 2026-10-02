use crate::site_tokens::{radius, type_scale};
use xengui::*;

pub fn not_found() -> Box<dyn Widget> {
    Box::new(
        View::new()
            .font("Inter")
            .display(Display::Flex)
            .flex_direction(FlexDirection::Column)
            .align_items(Align::Center)
            .justify_content(JustifyContent::Center)
            .gap(0, Responsive::new(px!(14.0)).md(px!(18.0)))
            .min_height(px!(620.0))
            .padding(Responsive::new(Edges::symmetric(20, 56)).md(Edges::symmetric(64, 80)))
            .background(|theme: &Theme| theme.background)
            .child(
                Label::new()
                    .label("ROUTE / 404")
                    .font_weight(FontWeight::SemiBold)
                    .font_size(type_scale::LABEL_SM)
                    .letter_spacing(px!(1.1))
                    .color(|theme: &Theme| theme.primary),
            )
            .child(
                RichText::new()
                    .with_content("This page is not here.")
                    .width(pct!(100.0))
                    .text_align(TextAlign::Center)
                    .font_size(Responsive::new(px!(type_scale::DISPLAY_SM)).md(px!(type_scale::DISPLAY_MD)))
                    .font_weight(FontWeight::SemiBold)
                    .letter_spacing(px!(-1.5))
                    .color(|theme: &Theme| theme.on_background),
            )
            .child(
                RichText::new()
                    .with_content(
                        "The address may have changed, or this route may never have existed.",
                    )
                    .width(pct!(100.0))
                    .font_size(px!(type_scale::BODY_LG))
                    .line_height(px!(type_scale::BODY_LG_LINE))
                    .text_align(TextAlign::Center)
                    .max_width(px!(420.0))
                    .color(|theme: &Theme| theme.on_surface_variant),
            )
            .child(
                Button::new()
                    .label("Back to home  →")
                    .transform_origin(TransformOrigin::CENTER)
                    .margin(Edges::only(0, 18, 0, 0))
                    .background(|theme: &Theme| theme.on_background)
                    .color(|theme: &Theme| theme.background)
                    .padding(Edges::symmetric(18, 11))
                    .border(Border::all(0, Color::TRANSPARENT).radius(radius::MD))
                    .transition_all(
                        Transition::new(std::time::Duration::from_millis(150))
                            .easing(Easing::EaseInOut),
                    )
                    .pressed_style(|ctx: StylePatch, _theme: &Theme| ctx.scale(0.97))
                    .on_click(|_ctx| xen_router::push("/")),
            ),
    )
}
