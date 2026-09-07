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
                    .font_size(11.0)
                    .letter_spacing(px!(1.1))
                    .color(|theme: &Theme| theme.primary),
            )
            .child(
                RichText::new()
                    .with_content("Bu sayfa burada değil.")
                    .width(pct!(100.0))
                    .text_align(TextAlign::Center)
                    .font_size(Responsive::new(px!(34.0)).md(px!(52.0)))
                    .font_weight(FontWeight::SemiBold)
                    .letter_spacing(px!(-1.5))
                    .color(|theme: &Theme| theme.on_background),
            )
            .child(
                RichText::new()
                    .with_content(
                        "Adres değişmiş olabilir ya da bu rota hiç oluşturulmamış olabilir.",
                    )
                    .width(pct!(100.0))
                    .font_size(px!(15.0))
                    .line_height(px!(23.0))
                    .text_align(TextAlign::Center)
                    .max_width(px!(420.0))
                    .color(|theme: &Theme| theme.on_surface_variant),
            )
            .child(
                Button::new()
                    .label("Ana sayfaya dön  →")
                    .margin(Edges::only(0, 18, 0, 0))
                    .background(|theme: &Theme| theme.on_background)
                    .color(|theme: &Theme| theme.background)
                    .padding(Edges::symmetric(18, 11))
                    .border(Border::all(0, Color::TRANSPARENT).radius(10))
                    .transition_all(
                        Transition::new(std::time::Duration::from_millis(150))
                            .easing(Easing::EaseInOut),
                    )
                    .pressed_style(|ctx: StylePatch, _theme: &Theme| ctx.scale(0.97))
                    .on_click(|_ctx| xen_router::push("/")),
            ),
    )
}
