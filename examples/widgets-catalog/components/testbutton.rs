use smol_str::SmolStr;
use xengui::composite::Render;
use xengui::*;

pub struct TestButton {
    base: WidgetBase,
    layout_box: LayoutBox,
    inner: Vec<Box<dyn Widget>>,
    hooks_id: WidgetId,

    label: SmolStr,
    color: Option<Color>,
}

impl TestButton {
    pub fn new() -> Self {
        Self {
            base: WidgetBase::new(Interaction::new()),
            layout_box: LayoutBox::default(),
            inner: Vec::new(),
            hooks_id: WidgetId::new_unique(),
            label: SmolStr::new(""),
            color: None,
        }
    }

    pub fn label(mut self, label: impl Into<SmolStr>) -> Self {
        self.label = label.into();
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }
}

impl Default for TestButton {
    fn default() -> Self {
        Self::new()
    }
}

impl Render for TestButton {
    fn render(&self) -> Box<dyn Widget> {
        let (clicks, set_clicks) = use_state(0i32);

        Box::new(
            View::new()
                .padding(Edges::symmetric(14.0, 8.0))
                .align_items(Align::Center)
                .justify_content(JustifyContent::Center)
                .background(self.color.unwrap_or(Color::BLUE_500))
                .border(Border::new(0.0, Color::TRANSPARENT, 8.0))
                .cursor(Cursor::Pointer)
                .on_click(move |_ctx| set_clicks.set(clicks + 1))
                .child(Label::new().label(format!("{} ({clicks})", self.label)).color(Color::WHITE))
        )
    }
}

impl_composite_widget!(TestButton);
