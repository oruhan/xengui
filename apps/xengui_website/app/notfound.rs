use xengui::*;

pub fn not_found() -> Box<dyn Widget> {
    Box::new(
        View::new()
            .font("Inter")
            .display(Display::Flex)
            .flex_direction(FlexDirection::Column)
            .align_items(Align::Center)
            .justify_content(JustifyContent::Center)
            .gap(0, Responsive::new(px!(16.0)).md(px!(24.0)))
            .min_height(pct!(100.0))
            .padding(Responsive::new(Edges::symmetric(24, 48)).md(Edges::symmetric(120, 48)))
            .background(|theme: &Theme| theme.background)
            .child(
                Label::new()
                    .label("404")
                    .font_weight(FontWeight::Medium)
                    .font_size(Responsive::new(px!(72.0)).md(px!(120.0)))
                    .letter_spacing(px!(-2.0))
                    .color(|theme: &Theme| theme.primary)
            )
            .child(
                Label::new()
                    .label("This page could not be found.")
                    .font_size(Responsive::new(px!(18.0)).md(px!(22.0)))
                    .font_weight(FontWeight::Medium)
                    .color(|theme: &Theme| theme.on_background)
            )
            .child(
                Label::new()
                    .label(
                        "The page you’re looking for may have been moved or may never have existed."
                    )
                    .font_size(px!(15.0))
                    .text_align(TextAlign::Center)
                    .max_width(px!(420.0))
                    .color(|theme: &Theme| theme.on_surface_variant)
            )
            .child(
                Button::new()
                    .label("Go back home")
                    .margin(Edges::only(0, 24, 0, 0))
                    .background(|theme: &Theme| theme.primary)
                    .color(|theme: &Theme| theme.on_primary)
                    .padding(Edges::only(20, 10, 20, 10))
                    .border(|theme: &Theme| Border::all(1, theme.primary).radius(theme.radius_4xl))
                    .transition_all(
                        Transition::new(std::time::Duration::from_millis(150)).easing(
                            Easing::EaseInOut
                        )
                    )
                    .hover_style(|ctx: StylePatch, theme: &Theme|
                        ctx.background(theme.inverse_primary)
                    )
                    .pressed_style(|ctx: StylePatch, _theme: &Theme| ctx.scale(0.97))
                    .on_click(|_ctx| xen_router::push("/"))
            )
    )
}
