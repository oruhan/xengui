use xen_router::RouteParams;
use xengui::{ Label, Widget };

pub fn page(_params: &RouteParams) -> Box<dyn Widget> {
    Box::new(Label::new().label("Home"))
}
