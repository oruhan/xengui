use xen_router::RouteParams;
use xengui::{ FlexDirection, Label, StyleBuilder, View, Widget };

pub fn layout(_params: &RouteParams, child: Box<dyn Widget>) -> Box<dyn Widget> {
    Box::new(
        View::new()
            .flex_direction(FlexDirection::Column)
            .children_vec(
                vec![Box::new(Label::new().label("My App Header")) as Box<dyn Widget>, child]
            )
    )
}
