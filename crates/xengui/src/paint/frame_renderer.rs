// SPDX-License-Identifier: Apache-2.0
use crate::{
    AnimationManager, BackdropFilterCommand, BoxShadowCommand, DrawCommand, FilteredCommand,
    ImageCommand, LayoutContext, LayoutEngine, PaintContext, Position, RectCommand, RenderBackend,
    RenderCache, StrokeCommand, SystemTheme, TriangleCommand, VariableIconCommand, Widget,
    WidgetPath,
};
use web_time::Instant;

/// Backend-agnostic frame orchestration: layout, paint-tree walk, command
/// batching and z-ordering. Every actual draw call is delegated to a
/// [`RenderBackend`] implementation.
pub struct FrameRenderer {
    render_cache: RenderCache,
    anim: AnimationManager,
    last_tick: Instant,
    force_layout: bool,
    frame_arena: FrameArena,
}

/// Resettable storage for data whose lifetime is exactly one frame.
///
/// `clear` resets lengths without releasing capacity, giving the paint hot
/// path bump-arena behavior after the high-water mark has been reached.
#[derive(Default)]
struct FrameArena {
    commands: Vec<(i32, DrawCommand)>,
    focus_commands: Vec<RectCommand>,
    top_commands: Vec<DrawCommand>,
    rects: Vec<RectCommand>,
    triangles: Vec<TriangleCommand>,
    images: Vec<ImageCommand>,
    shadows: Vec<BoxShadowCommand>,
    strokes: Vec<StrokeCommand>,
    icons: Vec<VariableIconCommand>,
    decorations: Vec<RectCommand>,
    paint_scratch: Vec<DrawCommand>,
    path: WidgetPath,
}

impl FrameArena {
    fn reset(&mut self) {
        self.commands.clear();
        self.focus_commands.clear();
        self.top_commands.clear();
        self.rects.clear();
        self.triangles.clear();
        self.images.clear();
        self.shadows.clear();
        self.strokes.clear();
        self.icons.clear();
        self.decorations.clear();
        self.paint_scratch.clear();
        self.path.restore(0);
    }
}

impl FrameRenderer {
    /// Creates a value with its default configuration.
    pub fn new() -> Self {
        Self {
            render_cache: RenderCache::new(),
            anim: AnimationManager::new(),
            last_tick: Instant::now(),
            force_layout: false,
            frame_arena: FrameArena::default(),
        }
    }

    /// Returns or updates the `anim` value.
    pub fn anim(&mut self) -> &mut AnimationManager {
        &mut self.anim
    }

    /// Returns whether the `is_animating` condition is satisfied.
    pub fn is_animating(&self) -> bool {
        self.anim.is_animating()
    }

    /// Returns or updates the `resize` value.
    pub fn resize(&mut self) {
        self.force_layout = true;
    }

    /// Returns or updates the `render_frame` value.
    pub fn render_frame(
        &mut self,
        tree: &mut [Box<dyn Widget>],
        backend: &mut dyn RenderBackend,
        theme: SystemTheme,
        scale_factor: f32,
        width: u32,
        height: u32,
    ) {
        let now = Instant::now();
        let dt = now.duration_since(self.last_tick);
        self.last_tick = now;
        self.anim.tick(dt);

        // Keeps Theme::auto() resolving against the OS's real light/dark
        // state, which the render backend already tracks and passes in
        // here as `theme`.
        crate::style::theme::set_system_is_dark(matches!(theme, SystemTheme::Dark));
        let app_background = crate::current_theme().background;

        if !backend.begin_frame(app_background, width, height) {
            return;
        }

        let needs_full_layout = std::mem::take(&mut self.force_layout)
            || tree_needs_layout(tree)
            || self.anim.active_keys().any(|k| k.property.affects_layout());

        let mut layout_ctx = LayoutContext {
            text: backend.text_measurer(),
            anim: &mut self.anim,
            scale_factor,
        };

        if needs_full_layout {
            LayoutEngine::layout(
                tree,
                &mut layout_ctx,
                &mut self.render_cache,
                width as f32,
                height as f32,
            );
            LayoutEngine::sync_scroll_offsets(tree);
            reset_layout_dirty_recursive(tree);
        } else {
            LayoutEngine::cascade(tree, &mut layout_ctx);
            // Scrolling never changes box sizes, so reposition the
            // already-laid-out subtree directly instead of paying for a
            // full taffy re-layout every animated-scroll frame. A no-op
            // walk when nothing actually scrolled this frame.
            LayoutEngine::reflow_scroll(tree, scale_factor);
        }

        let mut frame_arena = std::mem::take(&mut self.frame_arena);
        frame_arena.reset();
        self.render_cache.begin_frame();
        let FrameArena {
            commands,
            focus_commands,
            top_commands,
            rects: rect_buf,
            triangles: tri_buf,
            images: img_buf,
            shadows: shadow_buf,
            strokes: stroke_buf,
            icons: icon_buf,
            decorations,
            paint_scratch,
            path,
        } = &mut frame_arena;

        for (i, node) in tree.iter().enumerate() {
            let checkpoint = path.checkpoint();
            path.push(node.as_ref(), i);
            paint_recursive(
                node.as_ref(),
                path,
                &mut self.render_cache,
                commands,
                focus_commands,
                top_commands,
                paint_scratch,
                None,
                scale_factor,
                0,
            );
            path.restore(checkpoint);
        }
        self.render_cache.finish_frame();

        for node in tree.iter_mut() {
            reset_dirty_recursive(node.as_mut());
        }

        // Stable sort keeps original paint order for widgets sharing the
        // same z-index; only different values get reordered.
        commands.sort_by_key(|(z, _)| *z);

        #[derive(PartialEq, Clone, Copy)]
        enum RunKind {
            Rect,
            Triangle,
            Image,
            Text,
            BoxShadow,
            Stroke,
            Filtered,
            BackdropFilter,
            VariableIcon,
        }

        let mut current_kind: Option<RunKind> = None;
        macro_rules! flush_run {
            () => {
                match current_kind {
                    Some(RunKind::Rect) => backend.draw_rects(&rect_buf),
                    Some(RunKind::Triangle) => backend.draw_triangles(&tri_buf),
                    Some(RunKind::Image) => backend.draw_images(&img_buf),
                    Some(RunKind::BoxShadow) => backend.draw_box_shadows(&shadow_buf),
                    Some(RunKind::Stroke) => backend.draw_strokes(&stroke_buf),
                    Some(RunKind::Text) => {
                        backend.flush_text();
                        decorations.clear();
                        backend.drain_text_decorations(decorations);
                        if !decorations.is_empty() {
                            backend.draw_rects(&decorations);
                        }
                    }
                    Some(RunKind::Filtered) => {}
                    Some(RunKind::BackdropFilter) => {}
                    Some(RunKind::VariableIcon) => backend.draw_variable_icons(&icon_buf),
                    None => {}
                }
                rect_buf.clear();
                tri_buf.clear();
                img_buf.clear();
                shadow_buf.clear();
                stroke_buf.clear();
                icon_buf.clear();
            };
        }

        // Draws each contiguous run of same-type commands in the order
        // z-index (then paint order) puts them in, instead of always
        // drawing every rect, then every triangle, then every image/text.
        for (_z, command) in commands.drain(..) {
            match command {
                DrawCommand::Text(cmd) => {
                    if current_kind != Some(RunKind::Text) {
                        flush_run!();
                        current_kind = Some(RunKind::Text);
                    }
                    backend.draw_text(theme, scale_factor, &cmd);
                }
                DrawCommand::Rect(cmd) => {
                    if current_kind != Some(RunKind::Rect) {
                        flush_run!();
                        current_kind = Some(RunKind::Rect);
                    }
                    rect_buf.push(cmd);
                }
                DrawCommand::Triangle(cmd) => {
                    if current_kind != Some(RunKind::Triangle) {
                        flush_run!();
                        current_kind = Some(RunKind::Triangle);
                    }
                    tri_buf.push(cmd);
                }
                DrawCommand::Image(cmd) => {
                    if current_kind != Some(RunKind::Image) {
                        flush_run!();
                        current_kind = Some(RunKind::Image);
                    }
                    img_buf.push(*cmd);
                }
                DrawCommand::BoxShadow(cmd) => {
                    if current_kind != Some(RunKind::BoxShadow) {
                        flush_run!();
                        current_kind = Some(RunKind::BoxShadow);
                    }
                    shadow_buf.push(cmd);
                }
                DrawCommand::Stroke(cmd) => {
                    if current_kind != Some(RunKind::Stroke) {
                        flush_run!();
                        current_kind = Some(RunKind::Stroke);
                    }
                    stroke_buf.push(cmd);
                }
                DrawCommand::VariableIcon(cmd) => {
                    if current_kind != Some(RunKind::VariableIcon) {
                        flush_run!();
                        current_kind = Some(RunKind::VariableIcon);
                    }
                    icon_buf.push(*cmd);
                }
                DrawCommand::Filtered(filtered) => {
                    if current_kind != Some(RunKind::Filtered) {
                        flush_run!();
                        current_kind = Some(RunKind::Filtered);
                    }
                    // Filtered subtrees don't batch with anything else -
                    // each is its own isolated offscreen pass, so it's
                    // dispatched immediately rather than buffered.
                    backend.flush_text();
                    backend.draw_filtered(
                        &filtered.commands,
                        &filtered.chain,
                        filtered.bounds,
                        filtered.clip_rect,
                    );
                }
                DrawCommand::BackdropFilter(cmd) => {
                    if current_kind != Some(RunKind::BackdropFilter) {
                        flush_run!();
                        current_kind = Some(RunKind::BackdropFilter);
                    }
                    backend.flush_text();
                    backend.draw_backdrop_filtered(
                        &cmd.chain,
                        cmd.bounds,
                        cmd.clip_rect,
                        cmd.radius,
                    );
                }
            }
        }
        flush_run!();

        // Top layer: rendered strictly after the main pass, so a popup
        // here always sits above every other widget's content. Within the
        // top layer itself, commands still interleave by paint order
        // (rect/triangle/image/text) instead of being grouped by type.
        if !top_commands.is_empty() {
            let mut top_kind: Option<RunKind> = None;

            macro_rules! flush_top_run {
                () => {
                    match top_kind {
                        Some(RunKind::Rect) => backend.draw_rects(rect_buf),
                        Some(RunKind::Triangle) => backend.draw_triangles(tri_buf),
                        Some(RunKind::Image) => backend.draw_images(img_buf),
                        Some(RunKind::Text) => {
                            backend.flush_text();
                            decorations.clear();
                            backend.drain_text_decorations(decorations);
                            if !decorations.is_empty() {
                                backend.draw_rects(&decorations);
                            }
                        }
                        Some(RunKind::BoxShadow) => backend.draw_box_shadows(shadow_buf),
                        Some(RunKind::Stroke) => backend.draw_strokes(stroke_buf),
                        Some(RunKind::Filtered) => {}
                        Some(RunKind::BackdropFilter) => {}
                        Some(RunKind::VariableIcon) => backend.draw_variable_icons(icon_buf),
                        None => {}
                    }
                    rect_buf.clear();
                    tri_buf.clear();
                    img_buf.clear();
                    shadow_buf.clear();
                    stroke_buf.clear();
                    icon_buf.clear();
                };
            }

            for command in top_commands.drain(..) {
                match command {
                    DrawCommand::Text(cmd) => {
                        if top_kind != Some(RunKind::Text) {
                            flush_top_run!();
                            top_kind = Some(RunKind::Text);
                        }
                        backend.draw_text(theme, scale_factor, &cmd);
                    }
                    DrawCommand::Rect(cmd) => {
                        if top_kind != Some(RunKind::Rect) {
                            flush_top_run!();
                            top_kind = Some(RunKind::Rect);
                        }
                        rect_buf.push(cmd);
                    }
                    DrawCommand::Triangle(cmd) => {
                        if top_kind != Some(RunKind::Triangle) {
                            flush_top_run!();
                            top_kind = Some(RunKind::Triangle);
                        }
                        tri_buf.push(cmd);
                    }
                    DrawCommand::Image(cmd) => {
                        if top_kind != Some(RunKind::Image) {
                            flush_top_run!();
                            top_kind = Some(RunKind::Image);
                        }
                        img_buf.push(*cmd);
                    }
                    DrawCommand::BoxShadow(cmd) => {
                        if top_kind != Some(RunKind::BoxShadow) {
                            flush_top_run!();
                            top_kind = Some(RunKind::BoxShadow);
                        }
                        shadow_buf.push(cmd);
                    }
                    DrawCommand::Stroke(cmd) => {
                        if top_kind != Some(RunKind::Stroke) {
                            flush_top_run!();
                            top_kind = Some(RunKind::Stroke);
                        }
                        stroke_buf.push(cmd);
                    }
                    DrawCommand::VariableIcon(cmd) => {
                        if top_kind != Some(RunKind::VariableIcon) {
                            flush_top_run!();
                            top_kind = Some(RunKind::VariableIcon);
                        }
                        icon_buf.push(*cmd);
                    }
                    DrawCommand::Filtered(_) => {}
                    // Overlay/top-layer content never produces a backdrop
                    // filter today - paint_recursive only emits it for the
                    // main tree walk.
                    DrawCommand::BackdropFilter(_) => {}
                }
            }
            flush_top_run!();
        }

        // Focus rings paint last, above everything else including the top
        // layer. All text (main pass and top layer) is already flushed to
        // the GPU by this point via the per-run flush_text() calls above.
        if !focus_commands.is_empty() {
            backend.draw_rects(focus_commands);
        }

        backend.end_frame();
        self.frame_arena = frame_arena;
    }
}

impl Default for FrameRenderer {
    fn default() -> Self {
        Self::new()
    }
}

// Positioned widgets (relative/sticky/absolute/fixed) paint above static
// in-flow siblings sharing the same explicit z-index, matching CSS's
// default z-index:auto stacking order.
fn effective_z_index(widget: &dyn Widget, parent_z_index: i32) -> i32 {
    if let Some(z) = widget.computed_style().z_index {
        return z;
    }
    let positioned = !matches!(
        widget.computed_style().position.unwrap_or_default(),
        Position::Static
    );
    if positioned {
        parent_z_index + 1
    } else {
        parent_z_index
    }
}

#[allow(clippy::too_many_arguments)]
fn paint_recursive(
    widget: &dyn Widget,
    path: &mut WidgetPath,
    cache: &mut RenderCache,
    commands: &mut Vec<(i32, DrawCommand)>,
    focus_commands: &mut Vec<RectCommand>,
    top_commands: &mut Vec<DrawCommand>,
    paint_scratch: &mut Vec<DrawCommand>,
    clip_rect: Option<(f32, f32, f32, f32)>,
    scale_factor: f32,
    parent_z_index: i32,
) {
    let layout_box = *widget.layout_box();

    if let Some((cx, cy, cw, ch)) = clip_rect {
        let visible = layout_box.x < cx + cw
            && layout_box.x + layout_box.width > cx
            && layout_box.y < cy + ch
            && layout_box.y + layout_box.height > cy;
        if !visible {
            return;
        }
    }

    cache.mark_live(path.as_str());

    let z_index = effective_z_index(widget, parent_z_index);

    // A filtered widget's own subtree (paint + descendants, but not its
    // overlay/top/focus layers - those stay outside the filter so a
    // scrollbar or focus ring is never blurred/discolored along with the
    // content it belongs to) is recorded in isolation and wrapped in a
    // single `DrawCommand::Filtered`, instead of being interleaved into
    // the normal z-sorted command stream.
    if let Some(chain) = widget.filter().filter(|c| !c.is_empty()) {
        let mut subtree: Vec<(i32, DrawCommand)> = Vec::new();
        paint_subtree_for_filter(
            widget,
            path,
            cache,
            &mut subtree,
            paint_scratch,
            scale_factor,
            z_index,
        );
        subtree.sort_by_key(|(z, _)| *z);

        // Outset shadows are pulled out of the offscreen-filtered bitmap
        // entirely and pushed onto the main command stream instead, so
        // they paint straight onto the scene as a crisp background layer
        // before the (possibly blurred) content composites on top of them.
        let mut shadow_layer: Vec<(i32, DrawCommand)> = Vec::new();
        subtree.retain(|(z, cmd)| {
            if let DrawCommand::BoxShadow(sc) = cmd
                && !sc.inset
            {
                shadow_layer.push((*z, cmd.clone()));
                return false;
            }
            true
        });

        for (shadow_z, mut shadow_cmd) in shadow_layer {
            apply_clip(&mut shadow_cmd, clip_rect);
            commands.push((shadow_z, shadow_cmd));
        }

        let b = layout_box;
        let filtered_cmd = FilteredCommand {
            commands: subtree.into_iter().map(|(_, c)| c).collect(),
            chain: chain.clone(),
            bounds: (b.x, b.y, b.width, b.height),
            clip_rect,
        };
        commands.push((z_index, DrawCommand::Filtered(Box::new(filtered_cmd))));
        paint_chrome_layers_inline(
            widget,
            clip_rect,
            scale_factor,
            top_commands,
            focus_commands,
            paint_scratch,
        );
        return;
    }

    paint_scratch.clear();
    match cache.try_reuse(path.as_str(), layout_box, widget.is_dirty()) {
        Some(cached) => paint_scratch.extend_from_slice(cached),
        None => {
            {
                let mut paint_ctx = PaintContext::new(paint_scratch, scale_factor);
                widget.paint(&mut paint_ctx);
            }
            cache.store(path.as_str(), layout_box, paint_scratch.clone());
        }
    }

    if let Some(backdrop_chain) = widget.backdrop_filter().filter(|c| !c.is_empty()) {
        let b = layout_box;
        let own_bounds = (b.x, b.y, b.width, b.height);
        let radius = widget
            .computed_style()
            .border
            .as_ref()
            .and_then(|border| border.radius)
            .map(|r| r.to_physical_array(scale_factor, b.width, b.height))
            .unwrap_or([0.0; 4]);
        let backdrop_clip = Some(clip_intersect(clip_rect, own_bounds));
        let mut backdrop_cmd = Some(DrawCommand::BackdropFilter(Box::new(
            BackdropFilterCommand {
                chain: backdrop_chain.clone(),
                bounds: own_bounds,
                clip_rect: backdrop_clip,
                radius,
            },
        )));

        // paint_box emits an outset box-shadow before its background rect;
        // capturing right before that rect keeps the shadow's own halo
        // outside the box unblurred, while the background and everything
        // after it composites on top of the blurred result instead of the
        // shadow's near-opaque fill hiding it.
        let insert_at = paint_scratch
            .iter()
            .take_while(|c| matches!(c, DrawCommand::BoxShadow(_)))
            .count();

        for (i, mut command) in paint_scratch.drain(..).enumerate() {
            if i == insert_at
                && let Some(cmd) = backdrop_cmd.take()
            {
                commands.push((z_index, cmd));
            }
            apply_clip(&mut command, clip_rect);
            commands.push((z_index, command));
        }
        if let Some(cmd) = backdrop_cmd.take() {
            commands.push((z_index, cmd));
        }
    } else {
        for mut command in paint_scratch.drain(..) {
            apply_clip(&mut command, clip_rect);
            commands.push((z_index, command));
        }
    }

    let child_clip = match widget.clip_children() {
        Some(rect) => Some(clip_intersect(clip_rect, rect)),
        None => clip_rect,
    };

    for (i, child) in widget.children().iter().enumerate() {
        let checkpoint = path.checkpoint();
        path.push(child.as_ref(), i);

        if child.is_portal() {
            paint_portal_subtree(
                child.as_ref(),
                path,
                cache,
                top_commands,
                focus_commands,
                paint_scratch,
                scale_factor,
            );
            path.restore(checkpoint);
            continue;
        }

        paint_recursive(
            child.as_ref(),
            path,
            cache,
            commands,
            focus_commands,
            top_commands,
            paint_scratch,
            child_clip,
            scale_factor,
            z_index,
        );
        path.restore(checkpoint);
    }

    paint_chrome_layers_inline(
        widget,
        clip_rect,
        scale_factor,
        top_commands,
        focus_commands,
        paint_scratch,
    );
}

/// Records a widget's own `paint()` output plus every descendant's,
/// z-sorted the same way the main tree would be, but into a standalone
/// buffer instead of the shared `commands` stream - the input a
/// `RenderBackend::draw_filtered` call is built from.
///
/// Portal children are skipped: a portal already escapes to the top
/// layer regardless of an ancestor's filter, and running it through the
/// filter here would double-count it once more when the top layer paints.
#[allow(clippy::too_many_arguments)]
fn paint_subtree_for_filter(
    widget: &dyn Widget,
    path: &mut WidgetPath,
    cache: &mut RenderCache,
    out: &mut Vec<(i32, DrawCommand)>,
    paint_scratch: &mut Vec<DrawCommand>,
    scale_factor: f32,
    z_index: i32,
) {
    cache.mark_live(path.as_str());

    paint_scratch.clear();
    match cache.try_reuse(path.as_str(), *widget.layout_box(), widget.is_dirty()) {
        Some(cached) => paint_scratch.extend_from_slice(cached),
        None => {
            {
                let mut paint_ctx = PaintContext::new(paint_scratch, scale_factor);
                widget.paint(&mut paint_ctx);
            }
            cache.store(path.as_str(), *widget.layout_box(), paint_scratch.clone());
        }
    }
    for command in paint_scratch.drain(..) {
        out.push((z_index, command));
    }

    for (i, child) in widget.children().iter().enumerate() {
        if child.is_portal() {
            continue;
        }
        let checkpoint = path.checkpoint();
        path.push(child.as_ref(), i);
        let child_z = effective_z_index(child.as_ref(), z_index);
        paint_subtree_for_filter(
            child.as_ref(),
            path,
            cache,
            out,
            paint_scratch,
            scale_factor,
            child_z,
        );
        path.restore(checkpoint);
    }
}

/// Paints a widget's overlay/top/focus chrome - the parts of
/// `paint_recursive`'s normal flow that must run on the real widget even
/// when its main content went through the filtered path, since chrome
/// (scrollbars, popups, focus rings) is explicitly meant to stay crisp
/// and unfiltered.
fn paint_chrome_layers_inline(
    widget: &dyn Widget,
    clip_rect: Option<(f32, f32, f32, f32)>,
    scale_factor: f32,
    top_commands: &mut Vec<DrawCommand>,
    focus_commands: &mut Vec<RectCommand>,
    paint_scratch: &mut Vec<DrawCommand>,
) {
    paint_scratch.clear();
    {
        let mut paint_ctx = PaintContext::new(paint_scratch, scale_factor);
        widget.paint_overlay(&mut paint_ctx);
    }
    for mut command in paint_scratch.drain(..) {
        apply_clip(&mut command, clip_rect);
        top_commands.push(command);
    }

    paint_scratch.clear();
    {
        let mut paint_ctx = PaintContext::new(paint_scratch, scale_factor);
        widget.paint_top(&mut paint_ctx);
    }
    for mut command in paint_scratch.drain(..) {
        apply_clip(&mut command, clip_rect);
        top_commands.push(command);
    }

    paint_scratch.clear();
    {
        let mut paint_ctx = PaintContext::new(paint_scratch, scale_factor);
        widget.paint_focus(&mut paint_ctx);
    }
    for mut command in paint_scratch.drain(..) {
        apply_clip(&mut command, clip_rect);
        if let DrawCommand::Rect(rect_cmd) = command {
            focus_commands.push(rect_cmd);
        }
    }
}

fn clip_intersect(
    existing: Option<(f32, f32, f32, f32)>,
    ancestor: (f32, f32, f32, f32),
) -> (f32, f32, f32, f32) {
    let Some((ex, ey, ew, eh)) = existing else {
        return ancestor;
    };
    let (ax, ay, aw, ah) = ancestor;
    let x0 = ex.max(ax);
    let y0 = ey.max(ay);
    let x1 = (ex + ew).min(ax + aw);
    let y1 = (ey + eh).min(ay + ah);
    (x0, y0, (x1 - x0).max(0.0), (y1 - y0).max(0.0))
}

fn apply_clip(command: &mut DrawCommand, clip_rect: Option<(f32, f32, f32, f32)>) {
    let Some(ancestor_clip) = clip_rect else {
        return;
    };
    let target = match command {
        DrawCommand::Rect(cmd) => &mut cmd.clip_rect,
        DrawCommand::Image(cmd) => &mut cmd.clip_rect,
        DrawCommand::Text(cmd) => &mut cmd.clip_rect,
        DrawCommand::Triangle(cmd) => &mut cmd.clip_rect,
        DrawCommand::BoxShadow(cmd) => &mut cmd.clip_rect,
        DrawCommand::Stroke(cmd) => &mut cmd.clip_rect,
        DrawCommand::Filtered(cmd) => &mut cmd.clip_rect,
        DrawCommand::VariableIcon(cmd) => &mut cmd.clip_rect,
        DrawCommand::BackdropFilter(cmd) => &mut cmd.clip_rect,
    };
    *target = Some(clip_intersect(*target, ancestor_clip));
}

#[allow(clippy::too_many_arguments)]
fn paint_portal_subtree(
    widget: &dyn Widget,
    path: &mut WidgetPath,
    cache: &mut RenderCache,
    top_commands: &mut Vec<DrawCommand>,
    focus_commands: &mut Vec<RectCommand>,
    paint_scratch: &mut Vec<DrawCommand>,
    scale_factor: f32,
) {
    let layout_box = *widget.layout_box();
    cache.mark_live(path.as_str());

    paint_scratch.clear();
    match cache.try_reuse(path.as_str(), layout_box, widget.is_dirty()) {
        Some(cached) => paint_scratch.extend_from_slice(cached),
        None => {
            {
                let mut paint_ctx = PaintContext::new(paint_scratch, scale_factor);
                widget.paint(&mut paint_ctx);
            }
            cache.store(path.as_str(), layout_box, paint_scratch.clone());
        }
    }
    top_commands.append(paint_scratch);

    for (i, child) in widget.children().iter().enumerate() {
        let checkpoint = path.checkpoint();
        path.push(child.as_ref(), i);
        paint_portal_subtree(
            child.as_ref(),
            path,
            cache,
            top_commands,
            focus_commands,
            paint_scratch,
            scale_factor,
        );
        path.restore(checkpoint);
    }

    paint_scratch.clear();
    {
        let mut paint_ctx = PaintContext::new(paint_scratch, scale_factor);
        widget.paint_overlay(&mut paint_ctx);
    }
    top_commands.append(paint_scratch);

    paint_scratch.clear();
    {
        let mut paint_ctx = PaintContext::new(paint_scratch, scale_factor);
        widget.paint_top(&mut paint_ctx);
    }
    top_commands.append(paint_scratch);

    paint_scratch.clear();
    {
        let mut paint_ctx = PaintContext::new(paint_scratch, scale_factor);
        widget.paint_focus(&mut paint_ctx);
    }
    for command in paint_scratch.drain(..) {
        if let DrawCommand::Rect(rect_cmd) = command {
            focus_commands.push(rect_cmd);
        }
    }
}

fn reset_dirty_recursive(widget: &mut dyn Widget) {
    widget.set_dirty(false);
    if let Some(children) = widget.children_mut() {
        for child in children.iter_mut() {
            reset_dirty_recursive(child.as_mut());
        }
    }
}

fn tree_needs_layout(tree: &[Box<dyn Widget>]) -> bool {
    tree.iter()
        .any(|w| widget_needs_layout_recursive(w.as_ref()))
}

fn widget_needs_layout_recursive(widget: &dyn Widget) -> bool {
    widget.is_layout_dirty()
        || widget
            .children()
            .iter()
            .any(|c| widget_needs_layout_recursive(c.as_ref()))
}

fn reset_layout_dirty_recursive(tree: &mut [Box<dyn Widget>]) {
    for widget in tree.iter_mut() {
        widget.set_layout_dirty(false);
        if let Some(children) = widget.children_mut() {
            reset_layout_dirty_recursive(children);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::FrameArena;

    #[test]
    fn frame_arena_reset_retains_high_water_capacities() {
        let mut arena = FrameArena::default();
        arena.commands.reserve(64);
        arena.rects.reserve(128);
        arena.paint_scratch.reserve(32);

        let capacities = (
            arena.commands.capacity(),
            arena.rects.capacity(),
            arena.paint_scratch.capacity(),
        );
        arena.reset();

        assert_eq!(arena.commands.capacity(), capacities.0);
        assert_eq!(arena.rects.capacity(), capacities.1);
        assert_eq!(arena.paint_scratch.capacity(), capacities.2);
        assert!(arena.path.as_str().is_empty());
    }
}
