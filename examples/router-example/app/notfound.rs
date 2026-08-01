use xen_router::RouteParams;
use xengui::{ Label, Widget };

pub fn notfound(_params: &RouteParams) -> Box<dyn Widget> {
    Box::new(Label::new().label("404 - Not Found"))
}
