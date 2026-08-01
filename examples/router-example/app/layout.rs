use xen_router::RouteParams;
use xengui::{ FlexDirection, Label, View, Widget };

pub fn layout(_params: &RouteParams, child: Box<dyn Widget>) -> Box<dyn Widget> {
    Box::new(
        View::new()
            .flex_direction(FlexDirection::Column)
            .child(Label::new().label("My App Header"))
            .child(child)
    )
}
