// SPDX-License-Identifier: Apache-2.0
//! Material 3 linear and circular determinate progress indicators.

use crate::{
    Background, BorderRadius, Color, Constraints, Interaction, LayoutBox, Length, MeasureContext,
    MeasureResult, PaintContext, RectCommand, StrokeCommand, Style, StyleBuilder, Widget,
    WidgetBase,
};

const DEFAULT_THICKNESS: f32 = 4.0;
const LINEAR_WAVE_AMPLITUDE: f32 = 3.0;
const LINEAR_WAVE_LENGTH: f32 = 40.0;
const CIRCULAR_WAVE_AMPLITUDE: f32 = 1.6;
const CIRCULAR_WAVE_LENGTH: f32 = 15.0;
const DEFAULT_CIRCULAR_SIZE: f32 = 40.0;
const TRACK_ACTIVE_GAP: f32 = 4.0;
const STOP_SIZE: f32 = 4.0;

/// Shape of a progress indicator's active segment.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ProgressIndicatorShape {
    /// A straight line or circular arc.
    #[default]
    Flat,
    /// The Material 3 Expressive wavy active segment.
    Wavy,
}

fn valid_fraction(value: f32) -> f32 {
    if value.is_finite() {
        value.clamp(0.0, 1.0)
    } else {
        0.0
    }
}

fn draw_capsule(ctx: &mut PaintContext, position: (f32, f32), size: (f32, f32), color: Color) {
    ctx.draw_rect(RectCommand {
        position,
        size,
        background: Some(Background::Color(color)),
        border_radius: Some(BorderRadius::all(Length::px(size.1 * 0.5))),
        border_width: None,
        border_color: None,
        clip_rect: None,
    });
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn draw_wave(
    ctx: &mut PaintContext,
    start_x: f32,
    end_x: f32,
    center_y: f32,
    amplitude: f32,
    wavelength: f32,
    phase: f32,
    thickness: f32,
    color: Color,
) {
    if end_x <= start_x {
        return;
    }
    // Twenty-four chords per wavelength keep the curve visually continuous
    // with the analytic stroke AA while avoiding hundreds of tiny draw
    // commands on high-density mobile displays.
    let step = (wavelength / 24.0).max(0.75);
    let mut x0 = start_x;
    let mut y0 = center_y + amplitude * phase.sin();
    while x0 < end_x {
        let x1 = (x0 + step).min(end_x);
        let sample_phase = (x1 - start_x) / wavelength * std::f32::consts::TAU + phase;
        let y1 = center_y + amplitude * sample_phase.sin();
        ctx.draw_stroke(StrokeCommand {
            p0: (x0, y0),
            p1: (x1, y1),
            thickness,
            color,
            clip_rect: None,
        });
        x0 = x1;
        y0 = y1;
    }
}

/// A horizontal Material 3 determinate progress indicator.
///
/// Values are clamped to the inclusive `0.0..=1.0` range. The indicator
/// defaults to the M3 4dp thickness and supports both flat and Expressive
/// wavy active segments.
pub struct LinearProgressIndicator {
    base: WidgetBase,
    layout_box: LayoutBox,
    value: f32,
    track_color: Option<Color>,
    active_color: Option<Color>,
    thickness: f32,
    shape: ProgressIndicatorShape,
    show_stop: bool,
}

impl LinearProgressIndicator {
    /// Creates a flat determinate linear indicator.
    pub fn new() -> Self {
        Self {
            base: WidgetBase::new(Interaction::new()),
            layout_box: LayoutBox::default(),
            value: 0.0,
            track_color: None,
            active_color: None,
            thickness: DEFAULT_THICKNESS,
            shape: ProgressIndicatorShape::Flat,
            show_stop: true,
        }
    }

    /// Sets progress as a fraction in `0.0..=1.0`.
    pub fn value(mut self, value: f32) -> Self {
        self.value = valid_fraction(value);
        self
    }

    /// Overrides the unfilled track color.
    pub fn track_color(mut self, color: Color) -> Self {
        self.track_color = Some(color);
        self
    }

    /// Overrides the active indicator color.
    pub fn active_color(mut self, color: Color) -> Self {
        self.active_color = Some(color);
        self
    }

    /// Backwards-compatible alias for [`Self::active_color`].
    pub fn fill_color(self, color: Color) -> Self {
        self.active_color(color)
    }

    /// Sets the active and track thickness in logical pixels.
    pub fn thickness(mut self, thickness: f32) -> Self {
        self.thickness = thickness.max(1.0);
        self
    }

    /// Backwards-compatible alias for [`Self::thickness`].
    pub fn bar_height(mut self, height: impl Into<Length>) -> Self {
        self.thickness = height.into().value().max(1.0);
        self
    }

    /// Selects the flat or wavy active shape.
    pub fn indicator_shape(mut self, shape: ProgressIndicatorShape) -> Self {
        self.shape = shape;
        self
    }

    /// Shows or hides the 4dp end-stop accessibility marker.
    pub fn show_stop(mut self, show: bool) -> Self {
        self.show_stop = show;
        self
    }
}

impl Default for LinearProgressIndicator {
    fn default() -> Self {
        Self::new()
    }
}

impl StyleBuilder for LinearProgressIndicator {
    fn style_mut(&mut self) -> &mut Style {
        &mut self.base.style
    }

    fn mark_dirty(&mut self) {
        self.base.dirty = true;
    }
}

crate::impl_common_style_builders!(base LinearProgressIndicator);

impl Widget for LinearProgressIndicator {
    crate::impl_widget_boilerplate!();

    fn debug_name(&self) -> &'static str {
        "Widget#LinearProgressIndicator"
    }

    fn measure(&self, ctx: &mut MeasureContext, constraints: Constraints) -> MeasureResult {
        let logical_height = match self.shape {
            ProgressIndicatorShape::Flat => self.thickness,
            ProgressIndicatorShape::Wavy => self.thickness + LINEAR_WAVE_AMPLITUDE * 2.0,
        };
        let width = constraints.max_width.unwrap_or(120.0 * ctx.scale_factor);
        let (width, height) = constraints.constrain_size(width, logical_height * ctx.scale_factor);
        MeasureResult::new(width, height)
    }

    fn paint(&self, ctx: &mut PaintContext) {
        let theme = crate::current_theme();
        let b = self.layout_box;
        let sf = ctx.scale_factor;
        let thickness = self.thickness * sf;
        let gap = TRACK_ACTIVE_GAP * sf;
        let center_y = b.y + b.height * 0.5;
        let active_end = b.x + b.width * self.value;
        let active_color = self.active_color.unwrap_or(theme.primary);
        let track_color = self.track_color.unwrap_or(theme.secondary_container);

        let track_start = (active_end + gap).min(b.x + b.width);
        if track_start < b.x + b.width {
            draw_capsule(
                ctx,
                (track_start, center_y - thickness * 0.5),
                (b.x + b.width - track_start, thickness),
                track_color,
            );
        }

        let active_width = (active_end - b.x - gap * 0.5).max(0.0);
        if active_width > 0.0 {
            match self.shape {
                ProgressIndicatorShape::Flat => draw_capsule(
                    ctx,
                    (b.x, center_y - thickness * 0.5),
                    (active_width, thickness),
                    active_color,
                ),
                ProgressIndicatorShape::Wavy => draw_wave(
                    ctx,
                    b.x,
                    b.x + active_width,
                    center_y,
                    LINEAR_WAVE_AMPLITUDE * sf,
                    LINEAR_WAVE_LENGTH * sf,
                    0.0,
                    thickness,
                    active_color,
                ),
            }
        }

        if self.show_stop {
            let stop = STOP_SIZE * sf;
            draw_capsule(
                ctx,
                (b.x + b.width - stop, center_y - stop * 0.5),
                (stop, stop),
                active_color,
            );
        }
    }

    fn content_eq(&self, other: &dyn Widget) -> bool {
        let Some(other) = other.as_any().downcast_ref::<Self>() else {
            return false;
        };
        self.value == other.value
            && self.track_color == other.track_color
            && self.active_color == other.active_color
            && self.thickness == other.thickness
            && self.shape == other.shape
            && self.show_stop == other.show_stop
            && self.base.authored_styles_eq(&other.base)
    }
}

/// Backwards-compatible name for [`LinearProgressIndicator`].
pub type ProgressBar = LinearProgressIndicator;

/// A Material 3 circular determinate progress indicator.
pub struct CircularProgressIndicator {
    base: WidgetBase,
    layout_box: LayoutBox,
    value: f32,
    track_color: Option<Color>,
    active_color: Option<Color>,
    thickness: f32,
    size: f32,
    shape: ProgressIndicatorShape,
}

impl CircularProgressIndicator {
    /// Creates a flat 40dp circular progress indicator.
    pub fn new() -> Self {
        Self {
            base: WidgetBase::new(Interaction::new()),
            layout_box: LayoutBox::default(),
            value: 0.0,
            track_color: None,
            active_color: None,
            thickness: DEFAULT_THICKNESS,
            size: DEFAULT_CIRCULAR_SIZE,
            shape: ProgressIndicatorShape::Flat,
        }
    }

    /// Sets progress as a fraction in `0.0..=1.0`.
    pub fn value(mut self, value: f32) -> Self {
        self.value = valid_fraction(value);
        self
    }

    /// Overrides the track color.
    pub fn track_color(mut self, color: Color) -> Self {
        self.track_color = Some(color);
        self
    }

    /// Overrides the active indicator color.
    pub fn active_color(mut self, color: Color) -> Self {
        self.active_color = Some(color);
        self
    }

    /// Sets the active and track thickness in logical pixels.
    pub fn thickness(mut self, thickness: f32) -> Self {
        self.thickness = thickness.max(1.0);
        self
    }

    /// Sets the outer logical size. M3 supports 24dp through 240dp.
    pub fn indicator_size(mut self, size: f32) -> Self {
        self.size = size.clamp(24.0, 240.0);
        self
    }

    /// Selects the flat or wavy active shape.
    pub fn indicator_shape(mut self, shape: ProgressIndicatorShape) -> Self {
        self.shape = shape;
        self
    }
}

impl Default for CircularProgressIndicator {
    fn default() -> Self {
        Self::new()
    }
}

impl StyleBuilder for CircularProgressIndicator {
    fn style_mut(&mut self) -> &mut Style {
        &mut self.base.style
    }

    fn mark_dirty(&mut self) {
        self.base.dirty = true;
    }
}

crate::impl_common_style_builders!(base CircularProgressIndicator);

impl Widget for CircularProgressIndicator {
    crate::impl_widget_boilerplate!();

    fn debug_name(&self) -> &'static str {
        "Widget#CircularProgressIndicator"
    }

    fn measure(&self, ctx: &mut MeasureContext, constraints: Constraints) -> MeasureResult {
        let wave_extra = if self.shape == ProgressIndicatorShape::Wavy {
            CIRCULAR_WAVE_AMPLITUDE * 2.0
        } else {
            0.0
        };
        let size = (self.size + wave_extra) * ctx.scale_factor;
        let (width, height) = constraints.constrain_size(size, size);
        MeasureResult::new(width, height)
    }

    fn paint(&self, ctx: &mut PaintContext) {
        let theme = crate::current_theme();
        let b = self.layout_box;
        let sf = ctx.scale_factor;
        let thickness = self.thickness * sf;
        let amplitude = if self.shape == ProgressIndicatorShape::Wavy {
            CIRCULAR_WAVE_AMPLITUDE * sf
        } else {
            0.0
        };
        let center = (b.x + b.width * 0.5, b.y + b.height * 0.5);
        let radius = (b.width.min(b.height) - thickness - amplitude * 2.0).max(1.0) * 0.5;
        let circumference = std::f32::consts::TAU * radius;
        let wavelength = CIRCULAR_WAVE_LENGTH * sf;
        // Keep tessellation density in logical pixels. Scaling it from the
        // physical circumference made a 3x-density phone emit roughly three
        // times as many strokes as desktop for the same 48dp indicator.
        // Ninety-six chords already stay sub-2dp at the default sizes and
        // the analytic stroke shader smooths their joins.
        let logical_circumference = circumference / sf.max(f32::EPSILON);
        let segments = ((logical_circumference / 1.5).ceil() as usize).max(96);
        let active_sweep = std::f32::consts::TAU * self.value;
        let gap_angle = (TRACK_ACTIVE_GAP * sf / radius).min(std::f32::consts::PI * 0.25);
        let active_color = self.active_color.unwrap_or(theme.primary);
        let track_color = self.track_color.unwrap_or(theme.secondary_container);
        let start_angle = -std::f32::consts::FRAC_PI_2;

        let draw_arc = |ctx: &mut PaintContext, from: f32, to: f32, color: Color, wavy: bool| {
            if to <= from {
                return;
            }
            let count = (((to - from) / std::f32::consts::TAU) * segments as f32)
                .ceil()
                .max(1.0) as usize;
            for index in 0..count {
                let t0 = index as f32 / count as f32;
                let t1 = (index + 1) as f32 / count as f32;
                let a0 = from + (to - from) * t0;
                let a1 = from + (to - from) * t1;
                let wave_radius = |angle: f32| {
                    if wavy {
                        radius
                            + amplitude
                                * (((angle - start_angle) * radius / wavelength)
                                    * std::f32::consts::TAU)
                                    .sin()
                    } else {
                        radius
                    }
                };
                let r0 = wave_radius(a0);
                let r1 = wave_radius(a1);
                ctx.draw_stroke(StrokeCommand {
                    p0: (center.0 + r0 * a0.cos(), center.1 + r0 * a0.sin()),
                    p1: (center.0 + r1 * a1.cos(), center.1 + r1 * a1.sin()),
                    thickness,
                    color,
                    clip_rect: None,
                });
            }
        };

        let active_end = start_angle + active_sweep;
        if self.value < 1.0 {
            draw_arc(
                ctx,
                active_end + gap_angle,
                start_angle + std::f32::consts::TAU - gap_angle,
                track_color,
                false,
            );
        }
        draw_arc(
            ctx,
            start_angle,
            active_end,
            active_color,
            self.shape == ProgressIndicatorShape::Wavy,
        );
    }

    fn content_eq(&self, other: &dyn Widget) -> bool {
        let Some(other) = other.as_any().downcast_ref::<Self>() else {
            return false;
        };
        self.value == other.value
            && self.track_color == other.track_color
            && self.active_color == other.active_color
            && self.thickness == other.thickness
            && self.size == other.size
            && self.shape == other.shape
            && self.base.authored_styles_eq(&other.base)
    }
}

#[cfg(test)]
mod tests {
    use super::{CircularProgressIndicator, LinearProgressIndicator};

    #[test]
    fn clamps_values_to_valid_fraction() {
        assert_eq!(LinearProgressIndicator::new().value(-1.0).value, 0.0);
        assert_eq!(LinearProgressIndicator::new().value(2.0).value, 1.0);
        assert_eq!(CircularProgressIndicator::new().value(f32::NAN).value, 0.0);
    }

    #[test]
    fn circular_size_stays_inside_material_range() {
        assert_eq!(
            CircularProgressIndicator::new().indicator_size(8.0).size,
            24.0
        );
        assert_eq!(
            CircularProgressIndicator::new().indicator_size(500.0).size,
            240.0
        );
    }
}
