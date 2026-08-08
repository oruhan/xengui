use xen_router::RouteParams;
use xengui::*;

pub fn page(_params: &RouteParams) -> Box<dyn Widget> {
    Box::new(
        View::new()
            .display(Display::Flex)
            .flex_direction(FlexDirection::Column)
            .align_items(Align::Start)
            .justify_content(JustifyContent::Center)
            .min_height(pct!(100.0))
            .padding(Edges::only(120, 160, 120, 0))
            .child(
                View::new()
                    .display(Display::Flex)
                    .flex_direction(FlexDirection::Column)
                    .align_items(Align::Start)
                    .justify_content(JustifyContent::Start)
                    .child(
                        View::new()
                            .display(Display::Flex)
                            .flex_direction(FlexDirection::Column)
                            .font_size(60)
                            .font_weight(FontWeight::Medium)
                            .line_height(pct!(64.0))
                            .letter_spacing(px!(-2.25))
                            .color(|theme: &Theme|  theme.on_background)
                            .child(Label::new().label("Playground"))
                            .child(Label::new().label("page"))
                    )
            )
            .child(
                View::new()
                    .background(|theme: &Theme| theme.surface)
                    .border(|theme: &Theme| Border::all(1, theme.outline).radius(12))
                    .width(pct!(100))
                    .height(px!(640))
                    .child(Label::new().label("Playground code/view etc..."))
            )
    )
}
