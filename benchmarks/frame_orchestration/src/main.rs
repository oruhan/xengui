use std::time::Instant;
use xengui::*;

struct ApproximateText;

impl TextMeasurer for ApproximateText {
    fn measure(
        &mut self,
        text: &str,
        _font: Option<&str>,
        font_size: f32,
        _font_weight: FontWeight,
        _font_style: FontStyle,
        letter_spacing: f32,
        line_height: f32,
        max_width: Option<f32>,
        scale_factor: f32,
    ) -> MeasureResult {
        let natural =
            text.chars().count() as f32 * (font_size * 0.55 + letter_spacing) * scale_factor;
        MeasureResult::new(
            max_width.map_or(natural, |limit| natural.min(limit)),
            line_height * scale_factor,
        )
    }

    fn character_offsets(
        &mut self,
        text: &str,
        _font: Option<&str>,
        font_size: f32,
        _font_weight: FontWeight,
        _font_style: FontStyle,
        letter_spacing: f32,
        _line_height: f32,
        scale_factor: f32,
    ) -> Vec<f32> {
        let advance = (font_size * 0.55 + letter_spacing) * scale_factor;
        (0..=text.chars().count())
            .map(|index| index as f32 * advance)
            .collect()
    }

    fn ascent(
        &mut self,
        _font: Option<&str>,
        font_size: f32,
        _font_weight: FontWeight,
        _font_style: FontStyle,
        scale_factor: f32,
    ) -> f32 {
        font_size * 0.8 * scale_factor
    }

    fn descent(
        &mut self,
        _font: Option<&str>,
        font_size: f32,
        _font_weight: FontWeight,
        _font_style: FontStyle,
        scale_factor: f32,
    ) -> f32 {
        font_size * 0.2 * scale_factor
    }

    fn line_height(
        &mut self,
        _font: Option<&str>,
        font_size: f32,
        _font_weight: FontWeight,
        _font_style: FontStyle,
        scale_factor: f32,
    ) -> f32 {
        font_size * 1.5 * scale_factor
    }
}

struct CountingBackend {
    text: ApproximateText,
    commands: usize,
}

impl CountingBackend {
    fn new() -> Self {
        Self {
            text: ApproximateText,
            commands: 0,
        }
    }
}

impl RenderBackend for CountingBackend {
    fn capabilities(&self) -> BackendCapabilities {
        BackendCapabilities::default()
    }
    fn unsupported_feature_policy(&self) -> UnsupportedFeaturePolicy {
        UnsupportedFeaturePolicy::Fallback
    }
    fn report_diagnostic(&mut self, _diagnostic: BackendDiagnostic) {}
    fn text_measurer(&mut self) -> &mut dyn TextMeasurer {
        &mut self.text
    }
    fn begin_frame(&mut self, _background: Color, _width: u32, _height: u32) -> bool {
        true
    }
    fn draw_rects(&mut self, commands: &[RectCommand]) {
        self.commands += commands.len();
    }
    fn draw_ripples(&mut self, commands: &[RippleCommand]) -> Result<(), BackendError> {
        self.commands += commands.len();
        Ok(())
    }
    fn draw_triangles(&mut self, commands: &[TriangleCommand]) {
        self.commands += commands.len();
    }
    fn draw_images(&mut self, commands: &[ImageCommand]) {
        self.commands += commands.len();
    }
    fn draw_box_shadows(&mut self, commands: &[BoxShadowCommand]) {
        self.commands += commands.len();
    }
    fn draw_strokes(&mut self, commands: &[StrokeCommand]) {
        self.commands += commands.len();
    }
    fn draw_variable_icons(
        &mut self,
        commands: &[VariableIconCommand],
    ) -> Result<(), BackendError> {
        self.commands += commands.len();
        Ok(())
    }
    fn draw_text(&mut self, _theme: SystemTheme, _scale_factor: f32, _command: &TextCommand) {
        self.commands += 1;
    }
    fn draw_composited(&mut self, _command: &CompositedCommand) -> Result<(), BackendError> {
        self.commands += 1;
        Ok(())
    }
    fn draw_filtered(
        &mut self,
        commands: &[DrawCommand],
        _chain: &FilterChain,
        _bounds: (f32, f32, f32, f32),
        _clip_rect: Option<(f32, f32, f32, f32)>,
    ) -> Result<(), BackendError> {
        self.commands += commands.len();
        Ok(())
    }
    fn draw_backdrop_filtered(
        &mut self,
        _chain: &FilterChain,
        _bounds: (f32, f32, f32, f32),
        _clip_rect: Option<(f32, f32, f32, f32)>,
        _radius: [f32; 4],
    ) -> Result<(), BackendError> {
        self.commands += 1;
        Ok(())
    }
    fn take_text_decorations(&mut self) -> Vec<RectCommand> {
        Vec::new()
    }
    fn flush_text(&mut self) -> Result<(), BackendError> {
        Ok(())
    }
    fn end_frame(&mut self) {}
    fn resize(&mut self, _width: u32, _height: u32) {}
}

fn workload() -> Vec<Box<dyn Widget>> {
    let mut grid = View::new()
        .display(Display::Grid)
        .grid_template_columns(vec![
            GridTrack::Fr(1.0),
            GridTrack::Fr(1.0),
            GridTrack::Fr(1.0),
        ])
        .gap(12.0, 12.0)
        .padding(Edges::all(24.0));

    for index in 0..180 {
        grid = grid.child(
            Column::new()
                .key(format!("card-{index}"))
                .gap(0.0, 8.0)
                .padding(Edges::all(16.0))
                .background(|theme: &Theme| theme.surface_container_low)
                .border(|theme: &Theme| Border::all(1.0, theme.outline_variant).radius(16.0))
                .child(Label::new().label(format!("Component {index}")))
                .child(
                    Label::new()
                        .label("Stable layout and paint workload")
                        .font_size(12.0),
                )
                .child(ProgressBar::new().value((index % 10) as f32 / 10.0)),
        );
    }
    vec![Box::new(grid)]
}

fn main() {
    const WARMUP: usize = 20;
    const SAMPLES: usize = 15;
    const FRAMES_PER_SAMPLE: usize = 80;
    let mut tree = workload();
    let mut renderer = FrameRenderer::new();
    let mut backend = CountingBackend::new();

    for _ in 0..WARMUP {
        renderer.resize();
        renderer
            .render_frame(&mut tree, &mut backend, SystemTheme::Light, 1.0, 1280, 900)
            .unwrap();
    }

    let mut samples = Vec::with_capacity(SAMPLES);
    for _ in 0..SAMPLES {
        let started = Instant::now();
        for _ in 0..FRAMES_PER_SAMPLE {
            renderer.resize();
            renderer
                .render_frame(&mut tree, &mut backend, SystemTheme::Light, 1.0, 1280, 900)
                .unwrap();
        }
        samples.push(started.elapsed().as_nanos() / FRAMES_PER_SAMPLE as u128);
    }
    samples.sort_unstable();
    let median = samples[SAMPLES / 2];
    let p95 = samples[(SAMPLES * 95 / 100).min(SAMPLES - 1)];
    let reference = std::env::var("BENCH_REF").unwrap_or_else(|_| "working-tree".to_string());
    println!(
        "{{\"ref\":\"{reference}\",\"metric\":\"full_layout_paint_cpu\",\"median_ns_per_frame\":{median},\"p95_ns_per_frame\":{p95},\"samples\":{SAMPLES},\"frames_per_sample\":{FRAMES_PER_SAMPLE},\"commands_observed\":{}}}",
        backend.commands
    );
}
