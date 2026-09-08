// SPDX-License-Identifier: Apache-2.0
use super::{
    BoxShadowCommand, CompositedCommand, DrawCommand, ImageCommand, RectCommand, StrokeCommand,
    TextCommand, TriangleCommand, VariableIconCommand,
};

/// Data and behavior represented by `PaintContext`.
pub struct PaintContext<'a> {
    commands: &'a mut Vec<DrawCommand>,
    /// The `scale_factor` value carried by this type.
    pub scale_factor: f32,
}

impl<'a> PaintContext<'a> {
    pub(crate) fn new(commands: &'a mut Vec<DrawCommand>, scale_factor: f32) -> Self {
        Self {
            commands,
            scale_factor,
        }
    }

    /// Returns or updates the `draw_text` value.
    pub fn draw_text(&mut self, command: TextCommand) {
        self.commands.push(DrawCommand::Text(Box::new(command)));
    }

    /// Returns or updates the `draw_rect` value.
    pub fn draw_rect(&mut self, command: RectCommand) {
        self.commands.push(DrawCommand::Rect(command));
    }

    /// Returns or updates the `draw_triangle` value.
    pub fn draw_triangle(&mut self, command: TriangleCommand) {
        self.commands.push(DrawCommand::Triangle(command));
    }

    /// Returns or updates the `draw_image` value.
    pub fn draw_image(&mut self, command: ImageCommand) {
        self.commands.push(DrawCommand::Image(Box::new(command)));
    }

    /// Returns or updates the `draw_box_shadow` value.
    pub fn draw_box_shadow(&mut self, command: BoxShadowCommand) {
        self.commands.push(DrawCommand::BoxShadow(command));
    }

    /// Returns or updates the `draw_stroke` value.
    pub fn draw_stroke(&mut self, command: StrokeCommand) {
        self.commands.push(DrawCommand::Stroke(command));
    }

    /// Returns or updates the `draw_variable_icon` value.
    pub fn draw_variable_icon(&mut self, command: VariableIconCommand) {
        self.commands
            .push(DrawCommand::VariableIcon(Box::new(command)));
    }

    /// Draws text on the widget's content channel. The frame compositor uses
    /// this marker to apply `content_scale` without reshaping or rerasterizing
    /// the text at every animation frame.
    pub fn draw_content_text(&mut self, command: TextCommand) {
        self.commands
            .push(DrawCommand::Content(Box::new(DrawCommand::Text(Box::new(
                command,
            )))));
    }

    /// Draws a triangle on the widget's independently-scalable content layer.
    pub fn draw_content_triangle(&mut self, command: TriangleCommand) {
        self.commands
            .push(DrawCommand::Content(Box::new(DrawCommand::Triangle(
                command,
            ))));
    }

    /// Draws an image on the widget's independently-scalable content layer.
    pub fn draw_content_image(&mut self, command: ImageCommand) {
        self.commands
            .push(DrawCommand::Content(Box::new(DrawCommand::Image(
                Box::new(command),
            ))));
    }

    /// Appends a pre-recorded compositor layer.
    pub fn draw_composited(&mut self, command: CompositedCommand) {
        self.commands
            .push(DrawCommand::Composited(Box::new(command)));
    }
}
