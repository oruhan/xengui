// SPDX-License-Identifier: Apache-2.0
//! Shared composite widgets reused across Pearl's pages/titlebar/sidebar.
#![allow(clippy::type_complexity)]
use std::rc::Rc;
use web_time::Duration;
use xengui::*;
use xengui_icons::{ IconAxes, codepoints };

/* ------ Icon Button ------ */

pub struct IconButton {
    base: WidgetBase,
    layout_box: LayoutBox,
    inner: Vec<Box<dyn Widget>>,
    hooks_id: WidgetId,
    codepoint: char,
    color: Color,
    size: f32,
    axes: IconAxes,
    on_click: Option<Rc<dyn Fn(&mut EventCtx)>>,
}

impl IconButton {
    pub fn new(codepoint: char) -> Self {
        Self {
            base: WidgetBase::new(Interaction::new()),
            layout_box: LayoutBox::default(),
            inner: Vec::new(),
            hooks_id: WidgetId::new_unique(),
            codepoint,
            color: Color::WHITE,
            size: 32.0,
            axes: IconAxes::default(),
            on_click: None,
        }
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    pub fn axes(mut self, axes: IconAxes) -> Self {
        self.axes = axes;
        self
    }

    pub fn on_click(mut self, f: impl Fn(&mut EventCtx) + 'static) -> Self {
        self.on_click = Some(Rc::new(f));
        self
    }
}

impl StyleBuilder for IconButton {
    fn style_mut(&mut self) -> &mut Style {
        &mut self.base.style
    }
    fn mark_dirty(&mut self) {
        self.base.dirty = true;
    }
}

impl Render for IconButton {
    fn render(&self) -> Box<dyn Widget> {
        let on_click = self.on_click.clone();
        let icon_size = self.size * 0.5;

        Box::new(
            View::new()
                .width(px!(self.size))
                .height(px!(self.size))
                .align_items(Align::Center)
                .justify_content(JustifyContent::Center)
                .background(Color::TRANSPARENT)
                .color(self.color)
                .border(Border::all(0.0, Color::TRANSPARENT).radius(self.size * 0.5))
                .transition_all(Transition::new(Duration::from_millis(140)).easing(Easing::EaseOut))
                .hover_style(|s, theme: &Theme| s.background(theme.surface_container_high))
                .pressed_style(|s, theme: &Theme|
                    s.background(theme.surface_container_highest).scale(0.88).content_scale(1.0)
                )
                .child(VariableIcon::new(self.codepoint).size(icon_size).axes(self.axes))
                .on_click(move |ctx| {
                    if let Some(f) = &on_click {
                        f(ctx);
                    }
                })
        )
    }
}

xengui::impl_composite_widget!(IconButton);

/* ------ AlbumArt ------ */

pub struct AlbumArt {
    base: WidgetBase,
    layout_box: LayoutBox,
    inner: Vec<Box<dyn Widget>>,
    hooks_id: WidgetId,
    color: Color,
    size: f32,
    icon_size: f32,
    image: Option<ImageSource>,
}

impl AlbumArt {
    pub fn new(color: Color) -> Self {
        Self {
            base: WidgetBase::new(Interaction::new()),
            layout_box: LayoutBox::default(),
            inner: Vec::new(),
            hooks_id: WidgetId::new_unique(),
            color,
            size: 52.0,
            icon_size: 20.0,
            image: None,
        }
    }

    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    pub fn icon_size(mut self, icon_size: f32) -> Self {
        self.icon_size = icon_size;
        self
    }

    pub fn image(mut self, image: Option<ImageSource>) -> Self {
        self.image = image;
        self
    }
}

impl StyleBuilder for AlbumArt {
    fn style_mut(&mut self) -> &mut Style {
        &mut self.base.style
    }
    fn mark_dirty(&mut self) {
        self.base.dirty = true;
    }
}

impl Render for AlbumArt {
    fn render(&self) -> Box<dyn Widget> {
        let mut view = View::new()
            .width(px!(self.size))
            .height(px!(self.size))
            .align_items(Align::Center)
            .justify_content(JustifyContent::Center)
            .background(self.color)
            .color(Color::WHITE.with_alpha_f32(0.92))
            .border(Border::all(0.0, Color::TRANSPARENT).radius(12.0));

        view = match &self.image {
            Some(source) =>
                view.child(
                    Image::new()
                        .source(source.clone())
                        .object_fit(ObjectFit::Cover)
                        .width(px!(self.size))
                        .height(px!(self.size))
                        .border(Border::all(0.0, Color::TRANSPARENT).radius(12.0))
                ),
            None => view.child(VariableIcon::new(codepoints::MUSIC_NOTE).size(self.icon_size)),
        };

        Box::new(view)
    }
}

xengui::impl_composite_widget!(AlbumArt);

/* ------ PlaybackTicker ------ */

pub struct PlaybackTicker {
    base: WidgetBase,
    layout_box: LayoutBox,
    active: bool,
    on_tick: Rc<dyn Fn()>,
}

impl PlaybackTicker {
    pub fn new(active: bool, on_tick: impl Fn() + 'static) -> Self {
        Self {
            base: WidgetBase::new(Interaction::new()),
            layout_box: LayoutBox::default(),
            active,
            on_tick: Rc::new(on_tick),
        }
    }
}

impl StyleBuilder for PlaybackTicker {
    fn style_mut(&mut self) -> &mut Style {
        &mut self.base.style
    }
}

impl Widget for PlaybackTicker {
    xengui::impl_widget_boilerplate!();

    fn debug_name(&self) -> &'static str {
        "Widget#PlaybackTicker"
    }

    fn measure(&self, _ctx: &mut MeasureContext, _constraints: Constraints) -> MeasureResult {
        MeasureResult::new(0.0, 0.0)
    }

    fn paint(&self, _ctx: &mut PaintContext) {}

    fn wants_animation_frame(&self) -> bool {
        self.active
    }

    fn event(&mut self, event: &InputEvent, ctx: &mut EventCtx) -> EventStatus {
        if matches!(event, InputEvent::AnimationTick { .. }) {
            (self.on_tick)();
            ctx.request_redraw();
            return EventStatus::Handled;
        }
        EventStatus::Ignored
    }

    fn content_eq(&self, other: &dyn Widget) -> bool {
        other
            .as_any()
            .downcast_ref::<PlaybackTicker>()
            .is_some_and(|o| self.active == o.active)
    }
}
