use xengui::{ Label, View, Widget };

pub fn not_found() -> Box<dyn Widget> {
    Box::new(View::new().font("Inter").child(Label::new().label("404 - Not Found")))
}
