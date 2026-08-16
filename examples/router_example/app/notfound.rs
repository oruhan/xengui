use xengui::{ Label, Widget };

pub fn not_found() -> Box<dyn Widget> {
    Box::new(Label::new().label("404 - Not Found"))
}
