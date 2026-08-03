// SPDX-License-Identifier: Apache-2.0
use crate::{
    AnimationManager,
    BackdropFilterCommand,
    BoxShadowCommand,
    DrawCommand,
    FilteredCommand,
    ImageCommand,
    LayoutContext,
    LayoutEngine,
    PaintContext,
    RectCommand,
    RenderBackend,
    RenderCache,
    SystemTheme,
    TriangleCommand,
    Widget,
};
use std::collections::HashSet;
use web_time::Instant;

/// Backend-agnostic frame orchestration: layout, paint-tree walk, command
/// batching and z-ordering. Every actual draw call is delegated to a
/// [`RenderBackend`] implementation.
pub struct FrameRenderer {
    render_cache: RenderCache,
    anim: AnimationManager,
    last_tick: Instant,
    force_layout: bool,
}

impl FrameRenderer {
    pub fn new() -> Self {
        Self {
            render_cache: RenderCache::new(),
            anim: AnimationManager::new(),
            last_tick: Instant::now(),
            force_layout: false,
        }
    }

    pub fn anim(&mut self) -> &mut AnimationManager {
        &mut self.anim
    }

    pub fn is_animating(&self) -> bool {
        self.anim.is_animating()
    }

    pub fn resize(&mut self) {
        self.force_layout = true;
    }

    pub fn render_frame(
        &mut self,
        tree: &mut [Box<dyn Widget>],
        backend: &mut dyn RenderBackend,
        theme: SystemTheme,
        scale_factor: f32,
        width: u32,
        height: u32
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

        let needs_full_layout =
            std::mem::take(&mut self.force_layout) ||
            tree_is_dirty(tree) ||
            self.anim
                .active_keys()
                .any(|k| {
                    k.property.affects_layout() || k.property == crate::AnimProperty::ScrollOffset
                });

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
                height as f32
            );
        } else {
            LayoutEngine::cascade(tree, &mut layout_ctx);
        }

        let mut commands: Vec<(i32, DrawCommand)> = Vec::new();
        let mut focus_commands: Vec<RectCommand> = Vec::new();
        let mut top_commands: Vec<DrawCommand> = Vec::new();
        let mut live_keys: HashSet<String> = HashSet::new();

        for (i, node) in tree.iter().enumerate() {
            let segment = crate::path_segment(node.as_ref(), i);
            paint_recursive(
                node.as_ref(),
                &segment,
                &mut self.render_cache,
                &mut commands,
                &mut focus_commands,
                &mut top_commands,
                &mut live_keys,
                None,
                scale_factor,
                0
            );
        }
        self.render_cache.retain_keys(&live_keys);

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
            Filtered,
            BackdropFilter,
        }

        let mut current_kind: Option<RunKind> = None;
        let mut rect_buf: Vec<RectCommand> = Vec::new();
        let mut tri_buf: Vec<TriangleCommand> = Vec::new();
        let mut img_buf: Vec<ImageCommand> = Vec::new();
        let mut shadow_buf: Vec<BoxShadowCommand> = Vec::new();

        macro_rules! flush_run {
            () => {
                match current_kind {
                    Some(RunKind::Rect) => backend.draw_rects(&rect_buf),
                    Some(RunKind::Triangle) => backend.draw_triangles(&tri_buf),
                    Some(RunKind::Image) => backend.draw_images(&img_buf),
                    Some(RunKind::BoxShadow) => backend.draw_box_shadows(&shadow_buf),
                    Some(RunKind::Text) => {
                        backend.flush_text();
                        let decorations = backend.take_text_decorations();
                        if !decorations.is_empty() {
                            backend.draw_rects(&decorations);
                        }
                    }
                    Some(RunKind::Filtered) => {}
                    Some(RunKind::BackdropFilter) => {}
                    None => {}
                }
                rect_buf.clear();
                tri_buf.clear();
                img_buf.clear();
                shadow_buf.clear();
            };
        }

        // Draws each contiguous run of same-type commands in the order
        // z-index (then paint order) puts them in, instead of always
        // drawing every rect, then every triangle, then every image/text.
        for (_z, command) in commands {
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
                DrawCommand::Filtered(filtered) => {
                    if current_kind != Some(RunKind::Filtered) {
                        flush_run!();
                        current_kind = Some(RunKind::Filtered);
                    }
                    // Filtered subtrees don't batch with anything else -
                    // each is its own isolated offscreen pass, so it's
                    // dispatched immediately rather than buffered.
                    backend.flush_text();
                    backend.draw_filtered(&filtered.commands, &filtered.chain, filtered.bounds);
                }
                DrawCommand::BackdropFilter(cmd) => {
                    if current_kind != Some(RunKind::BackdropFilter) {
                        flush_run!();
                        current_kind = Some(RunKind::BackdropFilter);
                    }
                    backend.flush_text();
                    backend.draw_backdrop_filtered(&cmd.chain, cmd.bounds, cmd.clip_rect);
                }
            }
        }
        flush_run!();

        // Top layer: rendered strictly after the main pass, so a popup
        // here always sits above every other widget's content. Within the
        // top layer itself, commands still interleave by paint order
        // (rect/triangle/image/text) instead of being grouped by type.
        if !top_commands.is_empty() {
            let mut top_rect_buf: Vec<RectCommand> = Vec::new();
            let mut top_tri_buf: Vec<TriangleCommand> = Vec::new();
            let mut top_img_buf: Vec<ImageCommand> = Vec::new();
            let mut top_shadow_buf: Vec<BoxShadowCommand> = Vec::new();
            let mut top_kind: Option<RunKind> = None;

            macro_rules! flush_top_run {
                () => {
                    match top_kind {
                        Some(RunKind::Rect) => backend.draw_rects(&top_rect_buf),
                        Some(RunKind::Triangle) => backend.draw_triangles(&top_tri_buf),
                        Some(RunKind::Image) => backend.draw_images(&top_img_buf),
                        Some(RunKind::Text) => {
                            backend.flush_text();
                            let decorations = backend.take_text_decorations();
                            if !decorations.is_empty() {
                                backend.draw_rects(&decorations);
                            }
                        }
                        Some(RunKind::BoxShadow) => backend.draw_box_shadows(&top_shadow_buf),
                        Some(RunKind::Filtered) => {}
                        Some(RunKind::BackdropFilter) => {}
                        None => {}
                    }
                    top_rect_buf.clear();
                    top_tri_buf.clear();
                    top_img_buf.clear();
                };
            }

            for command in top_commands {
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
                        top_rect_buf.push(cmd);
                    }
                    DrawCommand::Triangle(cmd) => {
                        if top_kind != Some(RunKind::Triangle) {
                            flush_top_run!();
                            top_kind = Some(RunKind::Triangle);
                        }
                        top_tri_buf.push(cmd);
                    }
                    DrawCommand::Image(cmd) => {
                        if top_kind != Some(RunKind::Image) {
                            flush_top_run!();
                            top_kind = Some(RunKind::Image);
                        }
                        top_img_buf.push(*cmd);
                    }
                    DrawCommand::BoxShadow(cmd) => {
                        if current_kind != Some(RunKind::BoxShadow) {
                            flush_top_run!();
                            current_kind = Some(RunKind::BoxShadow);
                        }
                        top_shadow_buf.push(cmd);
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
            backend.draw_rects(&focus_commands);
        }

        backend.end_frame();
    }
}

impl Default for FrameRenderer {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(clippy::too_many_arguments)]
fn paint_recursive(
    widget: &dyn Widget,
    path: &str,
    cache: &mut RenderCache,
    commands: &mut Vec<(i32, DrawCommand)>,
    focus_commands: &mut Vec<RectCommand>,
    top_commands: &mut Vec<DrawCommand>,
    live_keys: &mut HashSet<String>,
    clip_rect: Option<(f32, f32, f32, f32)>,
    scale_factor: f32,
    parent_z_index: i32
) {
    let layout_box = *widget.layout_box();

    if let Some((cx, cy, cw, ch)) = clip_rect {
        let visible =
            layout_box.x < cx + cw &&
            layout_box.x + layout_box.width > cx &&
            layout_box.y < cy + ch &&
            layout_box.y + layout_box.height > cy;
        if !visible {
            return;
        }
    }

    live_keys.insert(path.to_string());

    let z_index = widget.computed_style().z_index.unwrap_or(parent_z_index);

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
            live_keys,
            scale_factor,
            z_index
        );
        subtree.sort_by_key(|(z, _)| *z);

        let b = layout_box;
        let filtered_cmd = FilteredCommand {
            commands: subtree
                .into_iter()
                .map(|(_, c)| c)
                .collect(),
            chain: chain.clone(),
            bounds: (b.x, b.y, b.width, b.height),
            clip_rect,
        };
        commands.push((z_index, DrawCommand::Filtered(Box::new(filtered_cmd))));
        paint_chrome_layers_inline(widget, clip_rect, scale_factor, top_commands, focus_commands);
        return;
    }

    let own_commands: Vec<DrawCommand> = match cache.try_reuse(path, layout_box, widget.is_dirty()) {
        Some(cached) => cached.to_vec(),
        None => {
            let mut local = Vec::new();
            {
                let mut paint_ctx = PaintContext::new(&mut local, scale_factor);
                widget.paint(&mut paint_ctx);
            }
            cache.store(path, layout_box, local.clone());
            local
        }
    };

    if let Some(backdrop_chain) = widget.backdrop_filter().filter(|c| !c.is_empty()) {
        let b = layout_box;
        let own_bounds = (b.x, b.y, b.width, b.height);
        // Backdrop-filtered output must stay confined to the widget's own
        // box - without intersecting with its own bounds here, the blur
        // padding added during compositing bleeds into whatever sits
        // above/below the widget instead of stopping at its edges.
        let backdrop_clip = Some(clip_intersect(clip_rect, own_bounds));
        let mut backdrop_cmd = Some(
            DrawCommand::BackdropFilter(
                Box::new(BackdropFilterCommand {
                    chain: backdrop_chain.clone(),
                    bounds: own_bounds,
                    clip_rect: backdrop_clip,
                })
            )
        );

        // paint_box emits an outset box-shadow before its background rect;
        // capturing right before that rect keeps the shadow's own halo
        // outside the box unblurred, while the background and everything
        // after it composites on top of the blurred result instead of the
        // shadow's near-opaque fill hiding it.
        let insert_at = own_commands
            .iter()
            .position(|c| matches!(c, DrawCommand::Rect(_)))
            .unwrap_or(0);

        for (i, mut command) in own_commands.into_iter().enumerate() {
            if i == insert_at && let Some(cmd) = backdrop_cmd.take() {
                commands.push((z_index, cmd));
            }
            apply_clip(&mut command, clip_rect);
            commands.push((z_index, command));
        }
        if let Some(cmd) = backdrop_cmd.take() {
            commands.push((z_index, cmd));
        }
    } else {
        for mut command in own_commands {
            apply_clip(&mut command, clip_rect);
            commands.push((z_index, command));
        }
    }

    let child_clip = match widget.clip_children() {
        Some(rect) => Some(clip_intersect(clip_rect, rect)),
        None => clip_rect,
    };

    for (i, child) in widget.children().iter().enumerate() {
        let segment = crate::path_segment(child.as_ref(), i);
        let child_path = format!("{path}.{segment}");

        if child.is_portal() {
            paint_portal_subtree(
                child.as_ref(),
                &child_path,
                cache,
                top_commands,
                focus_commands,
                live_keys,
                scale_factor
            );
            continue;
        }

        paint_recursive(
            child.as_ref(),
            &child_path,
            cache,
            commands,
            focus_commands,
            top_commands,
            live_keys,
            child_clip,
            scale_factor,
            z_index
        );
    }

    paint_chrome_layers_inline(widget, clip_rect, scale_factor, top_commands, focus_commands);
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
    path: &str,
    cache: &mut RenderCache,
    out: &mut Vec<(i32, DrawCommand)>,
    live_keys: &mut HashSet<String>,
    scale_factor: f32,
    z_index: i32
) {
    live_keys.insert(path.to_string());

    let own_commands: Vec<DrawCommand> = match
        cache.try_reuse(path, *widget.layout_box(), widget.is_dirty())
    {
        Some(cached) => cached.to_vec(),
        None => {
            let mut local = Vec::new();
            {
                let mut paint_ctx = PaintContext::new(&mut local, scale_factor);
                widget.paint(&mut paint_ctx);
            }
            cache.store(path, *widget.layout_box(), local.clone());
            local
        }
    };
    for command in own_commands {
        out.push((z_index, command));
    }

    for (i, child) in widget.children().iter().enumerate() {
        if child.is_portal() {
            continue;
        }
        let segment = crate::path_segment(child.as_ref(), i);
        let child_path = format!("{path}.{segment}");
        let child_z = child.computed_style().z_index.unwrap_or(z_index);
        paint_subtree_for_filter(
            child.as_ref(),
            &child_path,
            cache,
            out,
            live_keys,
            scale_factor,
            child_z
        );
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
    focus_commands: &mut Vec<RectCommand>
) {
    let mut overlay = Vec::new();
    {
        let mut paint_ctx = PaintContext::new(&mut overlay, scale_factor);
        widget.paint_overlay(&mut paint_ctx);
    }
    for mut command in overlay {
        apply_clip(&mut command, clip_rect);
        top_commands.push(command);
    }

    let mut top_local = Vec::new();
    {
        let mut paint_ctx = PaintContext::new(&mut top_local, scale_factor);
        widget.paint_top(&mut paint_ctx);
    }
    for mut command in top_local {
        apply_clip(&mut command, clip_rect);
        top_commands.push(command);
    }

    let mut focus_local = Vec::new();
    {
        let mut paint_ctx = PaintContext::new(&mut focus_local, scale_factor);
        widget.paint_focus(&mut paint_ctx);
    }
    for mut command in focus_local {
        apply_clip(&mut command, clip_rect);
        if let DrawCommand::Rect(rect_cmd) = command {
            focus_commands.push(rect_cmd);
        }
    }
}

fn clip_intersect(
    existing: Option<(f32, f32, f32, f32)>,
    ancestor: (f32, f32, f32, f32)
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
        DrawCommand::Filtered(cmd) => &mut cmd.clip_rect,
        DrawCommand::BackdropFilter(cmd) => &mut cmd.clip_rect,
    };
    *target = Some(clip_intersect(*target, ancestor_clip));
}

#[allow(clippy::too_many_arguments)]
fn paint_portal_subtree(
    widget: &dyn Widget,
    path: &str,
    cache: &mut RenderCache,
    top_commands: &mut Vec<DrawCommand>,
    focus_commands: &mut Vec<RectCommand>,
    live_keys: &mut HashSet<String>,
    scale_factor: f32
) {
    let layout_box = *widget.layout_box();
    live_keys.insert(path.to_string());

    let own_commands: Vec<DrawCommand> = match cache.try_reuse(path, layout_box, widget.is_dirty()) {
        Some(cached) => cached.to_vec(),
        None => {
            let mut local = Vec::new();
            {
                let mut paint_ctx = PaintContext::new(&mut local, scale_factor);
                widget.paint(&mut paint_ctx);
            }
            cache.store(path, layout_box, local.clone());
            local
        }
    };
    top_commands.extend(own_commands);

    for (i, child) in widget.children().iter().enumerate() {
        let segment = crate::path_segment(child.as_ref(), i);
        paint_portal_subtree(
            child.as_ref(),
            &format!("{path}.{segment}"),
            cache,
            top_commands,
            focus_commands,
            live_keys,
            scale_factor
        );
    }

    let mut overlay = Vec::new();
    {
        let mut paint_ctx = PaintContext::new(&mut overlay, scale_factor);
        widget.paint_overlay(&mut paint_ctx);
    }
    top_commands.extend(overlay);

    let mut top_local = Vec::new();
    {
        let mut paint_ctx = PaintContext::new(&mut top_local, scale_factor);
        widget.paint_top(&mut paint_ctx);
    }
    top_commands.extend(top_local);

    let mut focus_local = Vec::new();
    {
        let mut paint_ctx = PaintContext::new(&mut focus_local, scale_factor);
        widget.paint_focus(&mut paint_ctx);
    }
    for command in focus_local {
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

fn tree_is_dirty(tree: &[Box<dyn Widget>]) -> bool {
    tree.iter().any(|w| widget_dirty_recursive(w.as_ref()))
}

fn widget_dirty_recursive(widget: &dyn Widget) -> bool {
    widget.is_dirty() ||
        widget
            .children()
            .iter()
            .any(|c| widget_dirty_recursive(c.as_ref()))
}
