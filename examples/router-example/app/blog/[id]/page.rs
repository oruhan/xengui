use xen_router::RouteParams;
use xengui::{ Label, Widget };

pub fn page(params: &RouteParams) -> Box<dyn Widget> {
    let id = params.get("id").unwrap_or("?");
    Box::new(Label::new().label(format!("Blog post {id}")))
}
