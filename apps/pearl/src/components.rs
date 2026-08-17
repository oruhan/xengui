// SPDX-License-Identifier: Apache-2.0
//! Shared composite widgets reused across Pearl's pages/titlebar/sidebar.
#![allow(clippy::type_complexity)]
use std::rc::Rc;
use web_time::Duration;
use xengui::*;
use xengui_icons::codepoints;

pub struct IconButton {
    base: WidgetBase,
    layout_box: LayoutBox,
    inner: Vec<Box<dyn Widget>>,
    hooks_id: WidgetId,
    codepoint: char,
    color: Color,
    size: f32,
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
                .child(VariableIcon::new(self.codepoint).size(icon_size))
                .on_click(move |ctx| {
                    if let Some(f) = &on_click {
                        f(ctx);
                    }
                })
        )
    }
}

xengui::impl_composite_widget!(IconButton);

pub struct AlbumArt {
    base: WidgetBase,
    layout_box: LayoutBox,
    inner: Vec<Box<dyn Widget>>,
    hooks_id: WidgetId,
    color: Color,
    size: f32,
    icon_size: f32,
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
        Box::new(
            View::new()
                .width(px!(self.size))
                .height(px!(self.size))
                .align_items(Align::Center)
                .justify_content(JustifyContent::Center)
                .background(self.color)
                .color(Color::WHITE.with_alpha_f32(0.92))
                .border(Border::all(0.0, Color::TRANSPARENT).radius(8.0))
                .child(VariableIcon::new(codepoints::MUSIC_NOTE).size(self.icon_size))
        )
    }
}

xengui::impl_composite_widget!(AlbumArt);
