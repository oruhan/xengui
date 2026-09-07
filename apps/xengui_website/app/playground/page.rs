use xen_router::RouteParams;
use xengui::*;

fn panel_header(title: &str, meta: &str, dark: bool) -> View {
    Row::new()
        .align_items(Align::Center)
        .justify_content(JustifyContent::SpaceBetween)
        .padding(Edges::symmetric(16.0, 12.0))
        .border(if dark {
            Border::bottom(1.0, Color::NEUTRAL_800)
        } else {
            Border::bottom(1.0, Color::NEUTRAL_300)
        })
        .child(
            Label::new()
                .label(title)
                .font_size(12.0)
                .font_weight(FontWeight::SemiBold)
                .color(if dark {
                    Color::NEUTRAL_200
                } else {
                    Color::NEUTRAL_800
                }),
        )
        .child(Label::new().label(meta).font_size(10.0).color(if dark {
            Color::NEUTRAL_500
        } else {
            Color::NEUTRAL_500
        }))
}

pub fn page(_params: &RouteParams) -> Box<dyn Widget> {
    let stacked = !responsive_bool(Breakpoint::Lg, true);
    let content_width = (viewport_size().0 - 40.0).clamp(280.0, 620.0);

    let intro = Column::new()
        .gap(0.0, 15.0)
        .padding(
            Responsive::new(Edges::only(20.0, 48.0, 20.0, 36.0))
                .md(Edges::only(64.0, 72.0, 64.0, 48.0))
                .lg(Edges::only(120.0, 80.0, 120.0, 56.0)),
        )
        .child(
            Label::new()
                .label("PLAYGROUND")
                .font_size(11.0)
                .font_weight(FontWeight::SemiBold)
                .letter_spacing(px!(1.1))
                .color(|theme: &Theme| theme.primary),
        )
        .child(
            RichText::new()
                .with_content("Kod ve çıktı, yan yana.")
                .width(pct!(100.0))
                .max_width(px!(content_width))
                .font_size(Responsive::new(px!(36.0)).md(px!(52.0)))
                .line_height(Responsive::new(px!(41.0)).md(px!(58.0)).resolve())
                .font_weight(FontWeight::SemiBold)
                .letter_spacing(px!(-1.8))
                .color(|theme: &Theme| theme.on_background),
        )
        .child(
            RichText::new()
                .with_content("Builder zincirini değiştirirken bileşenin görünümünü ve davranışını aynı bağlamda değerlendirin.")
                .width(pct!(100.0))
                .font_size(15.0)
                .line_height(px!(24.0))
                .max_width(px!(content_width))
                .color(|theme: &Theme| theme.on_surface_variant),
        );

    let code_panel = Column::new()
        .flex_grow(1.0)
        .min_width(px!(0.0))
        .min_height(Responsive::new(px!(300.0)).md(px!(420.0)))
        .background(Color::NEUTRAL_950)
        .border(Border::all(1.0, Color::NEUTRAL_800).radius(14.0))
        .overflow_x(Overflow::Hidden)
        .child(panel_header("src/components/submit.rs", "RUST", true))
        .child(
            Label::new()
                .selectable(true)
                .label(
                    "Button::new()\n    .label(\"Gönder\")\n    .font_weight(FontWeight::SemiBold)\n    .background(|theme: &Theme| theme.primary)\n    .color(|theme: &Theme| theme.on_primary)\n    .padding(Edges::symmetric(18.0, 11.0))\n    .border(Border::all(0.0, Color::TRANSPARENT)\n        .radius(10.0))\n    .hover_style(|style, theme|\n        style.background(theme.inverse_primary)\n    )\n    .on_click(|_ctx| submit())",
                )
                .padding(Responsive::new(Edges::all(18.0)).md(Edges::all(24.0)))
                .font_size(13.0)
                .line_height(px!(21.0))
                .color(Color::NEUTRAL_200),
        );

    let preview_panel = Column::new()
        .flex_grow(1.0)
        .min_width(px!(0.0))
        .min_height(Responsive::new(px!(300.0)).md(px!(420.0)))
        .background(|theme: &Theme| theme.surface_container_low)
        .border(|theme: &Theme| Border::all(1.0, theme.outline_variant).radius(14.0))
        .overflow_x(Overflow::Hidden)
        .child(panel_header("Önizleme", "100%", false))
        .child(
            Column::new()
                .flex_grow(1.0)
                .align_items(Align::Center)
                .justify_content(JustifyContent::Center)
                .gap(0.0, 18.0)
                .padding(Edges::all(24.0))
                .child(
                    Button::new()
                        .label("Gönder")
                        .font_weight(FontWeight::SemiBold)
                        .background(|theme: &Theme| theme.primary)
                        .color(|theme: &Theme| theme.on_primary)
                        .padding(Edges::symmetric(18.0, 11.0))
                        .border(Border::all(0.0, Color::TRANSPARENT).radius(10.0))
                        .hover_style(|style: StylePatch, theme: &Theme| {
                            style.background(theme.inverse_primary)
                        }),
                )
                .child(
                    Label::new()
                        .label("Etkileşimli widget")
                        .font_size(12.0)
                        .color(|theme: &Theme| theme.on_surface_variant),
                ),
        );

    let workspace = View::new()
        .display(Display::Flex)
        .flex_direction(if stacked {
            FlexDirection::Column
        } else {
            FlexDirection::Row
        })
        .align_items(Align::Stretch)
        .gap(14.0, 14.0)
        .width(pct!(100.0))
        .padding(
            Responsive::new(Edges::only(20.0, 0.0, 20.0, 72.0))
                .md(Edges::only(64.0, 0.0, 64.0, 88.0))
                .lg(Edges::only(120.0, 0.0, 120.0, 104.0)),
        )
        .child(code_panel)
        .child(preview_panel);

    Box::new(
        Column::new()
            .width(pct!(100.0))
            .child(intro)
            .child(workspace),
    )
}
