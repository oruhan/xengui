// SPDX-License-Identifier: Apache-2.0
use xengui::*;
use xenframe::{ App, AppConfig };

fn filter_card(title: &str, chain: FilterChain) -> View {
    View::new()
        .flex_direction(FlexDirection::Column)
        .gap(0, 6)
        .child(
            View::new()
                .width(120)
                .height(90)
                .background(Color::ROSE_500)
                .border(Border::all(0, Color::TRANSPARENT).radius(12))
                .filter(chain)
                .child(Label::new().label("XenGUI").color(Color::WHITE))
        )
        .child(
            Label::new()
                .label(title)
                .font_size(12)
                .color(|t: &Theme| t.on_background)
        )
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = App::new(AppConfig::default());

    app.render(|| {
        Box::new(
            View::new()
                .flex_direction(FlexDirection::Row)
                .flex_wrap(FlexWrap::Wrap)
                .gap(16, 16)
                .padding(Edges::all(24))
                .child(filter_card("Blur", Filter::Blur((6.0).into()).into()))
                .child(filter_card("Brightness 1.5", Filter::Brightness(1.5).into()))
                .child(filter_card("Contrast 1.8", Filter::Contrast(1.8).into()))
                .child(filter_card("Saturate 2.5", Filter::Saturate(2.5).into()))
                .child(filter_card("Grayscale", Filter::Grayscale(1.0).into()))
                .child(filter_card("Hue Rotate 90", Filter::HueRotate(90.0).into()))
                .child(filter_card("Invert", Filter::Invert(1.0).into()))
                .child(filter_card("Opacity 0.4", Filter::Opacity(0.4).into()))
                .child(filter_card("Gamma 2.2", Filter::Gamma(2.2).into()))
                .child(
                    filter_card(
                        "Drop Shadow",
                        Filter::DropShadow(
                            DropShadow::new(4, 4, 8, Color::BLACK.with_alpha(160))
                        ).into()
                    )
                )
                .child(
                    filter_card(
                        "Combined chain",
                        FilterChain::new()
                            .push(Filter::Grayscale(0.6))
                            .push(Filter::Contrast(1.2))
                            .push(Filter::Blur((3.0).into()))
                    )
                )
        )
    });

    app.run()?;
    Ok(())
}
