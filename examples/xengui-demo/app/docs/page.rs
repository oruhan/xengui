use std::time::Duration;
use xen_router::RouteParams;
use xengui::*;

pub fn page(_params: &RouteParams) -> Box<dyn Widget> {
    Box::new(
        View::new()
            .display(Display::Flex)
            .flex_direction(FlexDirection::Column)
            .align_items(AlignItems::Start)
            .justify_content(JustifyContent::Center)
            .min_height(pct!(100.0))
            .child(
                View::new()
                    .display(Display::Flex)
                    .flex_direction(FlexDirection::Column)
                    .align_items(AlignItems::Start)
                    .justify_content(JustifyContent::Start)
                    .padding(Edges::only(120, 100, 120, 0))
                    .child(
                        View::new()
                            .display(Display::Flex)
                            .flex_direction(FlexDirection::Column)
                            .font_size(60)
                            .font_weight(FontWeight::Medium)
                            .line_height(pct!(64.0))
                            .letter_spacing(px!(-2.25))
                            .color(|theme: &Theme| theme.foreground)
                            .child(Label::new().label("Docs"))
                            .child(Label::new().label("page"))
                            .child(
                                Label::new()
                                    .color(|theme: &Theme| theme.foreground_muted)
                                    .font_size(16)
                                    .font_weight(FontWeight::Regular)
                                    .line_height(px!(10.0))
                                    .letter_spacing(px!(-0.1))
                                    .margin(Edges::only(0, 16, 0, 8))
                                    .label(
                                        "Build native desktop, web, mobile, and embedded applications from a single codebase."
                                    )
                            )
                    )
                    .child(
                        View::new()
                            .display(Display::Flex)
                            .flex_direction(FlexDirection::Row)
                            .margin(Edges::only(0, 16, 0, 16))
                            .gap(4, 0)
                            .child(
                                Button::new()
                                    .background(Color::BLUE_500)
                                    .font_weight(FontWeight::Medium)
                                    .transition_all(
                                        Transition::new(Duration::from_millis(200)).easing(
                                            Easing::EaseInOut
                                        )
                                    )
                                    .border(Border::all(1, Color::BLUE_400).radius(10))
                                    .hover_style(|ctx: StylePatch, _theme: &Theme|
                                        ctx
                                            .background(Color::BLUE_600)
                                            .border(Border::all(1, Color::BLUE_500).radius(10))
                                    )
                                    .pressed_style(|ctx: StylePatch, _theme: &Theme|
                                        ctx
                                            .background(Color::BLUE_700)
                                            .border(Border::all(1, Color::BLUE_600).radius(10))
                                            .scale(0.97)
                                    )
                                    .padding(Edges::only(15, 9, 15, 9))
                                    .label("Get started")
                            )
                            .child(
                                Button::new()
                                    .background(Color::hex("#24292f"))
                                    .font_weight(FontWeight::Medium)
                                    .transition_all(
                                        Transition::new(Duration::from_millis(200)).easing(
                                            Easing::EaseInOut
                                        )
                                    )
                                    .border(Border::all(1, Color::hex("#3d444d")).radius(10))
                                    .hover_style(|ctx: StylePatch, _theme: &Theme|
                                        ctx
                                            .background(Color::hex("#30363d"))
                                            .border(
                                                Border::all(1, Color::hex("#4b5561")).radius(10)
                                            )
                                    )
                                    .pressed_style(|ctx: StylePatch, _theme: &Theme|
                                        ctx.background(Color::hex("#1f2328")).scale(0.97)
                                    )
                                    .padding(Edges::only(15, 9, 15, 9))
                                    .label("GitHub")
                            )
                    )
            )
    )
}
