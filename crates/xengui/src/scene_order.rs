// SPDX-License-Identifier: Apache-2.0
//! Shared paint and hit-test ordering for a laid-out widget tree.

use crate::{LayoutBox, Position, Widget, WidgetPath, WidgetPathSegment};
use std::collections::HashMap;

type Rect = (f32, f32, f32, f32);

#[derive(Clone, Copy, Debug)]
struct SceneTransform {
    scale: f32,
    tx: f32,
    ty: f32,
}

impl SceneTransform {
    const IDENTITY: Self = Self {
        scale: 1.0,
        tx: 0.0,
        ty: 0.0,
    };

    fn scale_about(pivot: (f32, f32), scale: f32) -> Self {
        Self {
            scale,
            tx: pivot.0 * (1.0 - scale),
            ty: pivot.1 * (1.0 - scale),
        }
    }

    /// Applies `local`, then `self`.
    fn compose(self, local: Self) -> Self {
        Self {
            scale: self.scale * local.scale,
            tx: self.scale * local.tx + self.tx,
            ty: self.scale * local.ty + self.ty,
        }
    }

    fn point(self, point: (f32, f32)) -> (f32, f32) {
        (
            point.0 * self.scale + self.tx,
            point.1 * self.scale + self.ty,
        )
    }

    fn inverse_point(self, point: (f32, f32)) -> Option<(f32, f32)> {
        if self.scale.abs() <= f32::EPSILON {
            return None;
        }
        Some((
            (point.0 - self.tx) / self.scale,
            (point.1 - self.ty) / self.scale,
        ))
    }

    fn rect(self, rect: Rect) -> Rect {
        let p0 = self.point((rect.0, rect.1));
        let p1 = self.point((rect.0 + rect.2, rect.1 + rect.3));
        (
            p0.0.min(p1.0),
            p0.1.min(p1.1),
            (p1.0 - p0.0).abs(),
            (p1.1 - p0.1).abs(),
        )
    }
}

#[derive(Clone, Debug)]
pub(crate) struct SceneNode {
    pub(crate) effective_z: i32,
    pub(crate) clip_rect: Option<Rect>,
    screen_clip: Option<Rect>,
    transform: SceneTransform,
    #[allow(dead_code)]
    stacking_context: usize,
    #[allow(dead_code)]
    top_layer: bool,
    hit_test_visible: bool,
}

/// Immutable ordering and geometry snapshot shared by painting and hit-testing.
///
/// A snapshot is rebuilt after layout. It records effective z-order, stacking
/// context membership, clip chains (collapsed to their intersection), the
/// accumulated transform, top-layer membership, and hit-test visibility.
#[derive(Clone, Debug, Default)]
pub struct SceneOrder {
    nodes: HashMap<WidgetPath, SceneNode>,
    paint_order: Vec<WidgetPath>,
}

impl SceneOrder {
    /// Builds a snapshot from an already cascaded and laid-out tree.
    pub fn build(tree: &[Box<dyn Widget>], scale_factor: f32) -> Self {
        let mut builder = SceneBuilder::new(scale_factor);
        let root = WidgetPath::new();
        for (index, widget) in tree.iter().enumerate() {
            let mut path = root.clone();
            path.push(widget.as_ref(), index);
            builder.visit(
                widget.as_ref(),
                path,
                0,
                None,
                None,
                SceneTransform::IDENTITY,
                false,
                0,
            );
        }
        builder.finish(tree)
    }

    pub(crate) fn node(&self, path: &WidgetPath) -> Option<&SceneNode> {
        self.nodes.get(path)
    }

    /// Returns whether the snapshot contains no painted widgets.
    pub fn is_empty(&self) -> bool {
        self.paint_order.is_empty()
    }

    /// Returns the front-most hit widget using this snapshot's paint order.
    pub fn hit_test(&self, tree: &[Box<dyn Widget>], point: (f32, f32)) -> Option<WidgetPath> {
        for path in self.paint_order.iter().rev() {
            let node = self.nodes.get(path)?;
            if !node.hit_test_visible
                || node
                    .screen_clip
                    .is_some_and(|clip| !rect_contains(clip, point))
            {
                continue;
            }
            let Some(local_point) = node.transform.inverse_point(point) else {
                continue;
            };
            let Some(widget) = find_widget(tree, path) else {
                continue;
            };
            if !widget.hit_test(local_point) || descendant_is_blocked(self, tree, path, point) {
                continue;
            }
            return Some(path.clone());
        }
        None
    }
}

struct SceneBuilder {
    scale_factor: f32,
    nodes: HashMap<WidgetPath, SceneNode>,
    top: Vec<WidgetPath>,
    next_context: usize,
}

impl SceneBuilder {
    fn new(scale_factor: f32) -> Self {
        Self {
            scale_factor,
            nodes: HashMap::new(),
            top: Vec::new(),
            next_context: 1,
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn visit(
        &mut self,
        widget: &dyn Widget,
        path: WidgetPath,
        parent_z: i32,
        clip_rect: Option<Rect>,
        screen_clip: Option<Rect>,
        ancestor_transform: SceneTransform,
        in_top_layer: bool,
        parent_context: usize,
    ) {
        let ancestor_transform = if widget.is_portal() {
            SceneTransform::IDENTITY
        } else {
            ancestor_transform
        };
        let z = effective_z_index(widget, parent_z);
        let style = widget.computed_style();
        let root_transform = ancestor_transform.compose(widget_transform(
            widget.layout_box(),
            style.scale.unwrap_or(1.0),
            style.transform_origin.unwrap_or_default(),
            self.scale_factor,
        ));
        let creates_context = style.z_index.is_some()
            || style.scale.is_some()
            || widget.filter().is_some()
            || widget.is_portal();
        let context = if creates_context {
            let id = self.next_context;
            self.next_context += 1;
            id
        } else {
            parent_context
        };
        let top_layer = in_top_layer || widget.is_portal();
        let node_screen_clip = if top_layer { None } else { screen_clip };
        let node = SceneNode {
            effective_z: z,
            clip_rect: if top_layer { None } else { clip_rect },
            screen_clip: node_screen_clip,
            transform: root_transform,
            stacking_context: context,
            top_layer,
            hit_test_visible: root_transform.scale.abs() > f32::EPSILON,
        };
        self.nodes.insert(path.clone(), node);
        if top_layer {
            self.top.push(path.clone());
        }

        let content_scale = style.content_scale.unwrap_or(style.scale.unwrap_or(1.0));
        let child_transform = ancestor_transform.compose(widget_transform(
            widget.layout_box(),
            content_scale,
            style.transform_origin.unwrap_or_default(),
            self.scale_factor,
        ));
        let (child_clip, child_screen_clip) = if top_layer {
            (None, None)
        } else if let Some(own_clip) = widget.clip_children() {
            (
                Some(clip_intersect(clip_rect, own_clip)),
                Some(clip_intersect(screen_clip, child_transform.rect(own_clip))),
            )
        } else {
            (clip_rect, screen_clip)
        };

        for (index, child) in widget.children().iter().enumerate() {
            let mut child_path = path.clone();
            child_path.push(child.as_ref(), index);
            self.visit(
                child.as_ref(),
                child_path,
                z,
                child_clip,
                child_screen_clip,
                child_transform,
                top_layer,
                context,
            );
        }
    }

    fn finish(self, tree: &[Box<dyn Widget>]) -> SceneOrder {
        let mut atoms = Vec::new();
        for (index, widget) in tree.iter().enumerate() {
            let mut path = WidgetPath::new();
            path.push(widget.as_ref(), index);
            atoms.extend(self.paint_atoms(widget.as_ref(), &path));
        }
        atoms.sort_by_key(|atom| atom.z);
        let mut paint_order = atoms
            .into_iter()
            .flat_map(|atom| atom.paths)
            .collect::<Vec<_>>();
        paint_order.extend(self.top);
        SceneOrder {
            nodes: self.nodes,
            paint_order,
        }
    }

    fn paint_atoms(&self, widget: &dyn Widget, path: &WidgetPath) -> Vec<PaintAtom> {
        let node = self.nodes.get(path).expect("visited scene node");
        if widget.filter().is_some_and(|filter| !filter.is_empty()) {
            let mut paths = Vec::new();
            self.filtered_paths(widget, path, &mut paths);
            paths.sort_by_key(|path| self.nodes[path].effective_z);
            return vec![PaintAtom {
                z: node.effective_z,
                paths,
            }];
        }

        let mut atoms = vec![PaintAtom {
            z: node.effective_z,
            paths: vec![path.clone()],
        }];
        let groups_children = (widget
            .computed_style()
            .content_scale
            .unwrap_or(widget.computed_style().scale.unwrap_or(1.0))
            - 1.0)
            .abs()
            >= f32::EPSILON;
        for (index, child) in widget.children().iter().enumerate() {
            if child.is_portal() {
                continue;
            }
            let mut child_path = path.clone();
            child_path.push(child.as_ref(), index);
            let mut child_atoms = self.paint_atoms(child.as_ref(), &child_path);
            if groups_children {
                child_atoms.sort_by_key(|atom| atom.z);
                atoms.push(PaintAtom {
                    z: node.effective_z,
                    paths: child_atoms
                        .into_iter()
                        .flat_map(|atom| atom.paths)
                        .collect(),
                });
            } else {
                atoms.extend(child_atoms);
            }
        }
        atoms
    }

    fn filtered_paths(&self, widget: &dyn Widget, path: &WidgetPath, out: &mut Vec<WidgetPath>) {
        out.push(path.clone());
        for (index, child) in widget.children().iter().enumerate() {
            if child.is_portal() {
                continue;
            }
            let mut child_path = path.clone();
            child_path.push(child.as_ref(), index);
            self.filtered_paths(child.as_ref(), &child_path, out);
        }
    }
}

struct PaintAtom {
    z: i32,
    paths: Vec<WidgetPath>,
}

pub(crate) fn effective_z_index(widget: &dyn Widget, parent_z: i32) -> i32 {
    if let Some(z) = widget.computed_style().z_index {
        z
    } else if !matches!(
        widget.computed_style().position.unwrap_or_default(),
        Position::Static
    ) {
        parent_z + 1
    } else {
        parent_z
    }
}

fn widget_transform(
    layout: &LayoutBox,
    scale: f32,
    origin: crate::TransformOrigin,
    scale_factor: f32,
) -> SceneTransform {
    let (ox, oy) = origin.resolve(layout.width, layout.height, scale_factor);
    SceneTransform::scale_about((layout.x + ox, layout.y + oy), scale)
}

fn find_widget<'a>(tree: &'a [Box<dyn Widget>], path: &WidgetPath) -> Option<&'a dyn Widget> {
    let mut segments = path.segments.iter();
    let mut current = resolve_segment(tree, segments.next()?)?;
    for segment in segments {
        current = resolve_segment(current.children(), segment)?;
    }
    Some(current)
}

fn resolve_segment<'a>(
    siblings: &'a [Box<dyn Widget>],
    segment: &WidgetPathSegment,
) -> Option<&'a dyn Widget> {
    match segment {
        WidgetPathSegment::Key(key) => siblings
            .iter()
            .find(|widget| widget.get_key() == Some(key))
            .map(|widget| widget.as_ref()),
        WidgetPathSegment::Index(index) => siblings.get(*index).map(|widget| widget.as_ref()),
    }
}

fn descendant_is_blocked(
    scene: &SceneOrder,
    tree: &[Box<dyn Widget>],
    path: &WidgetPath,
    screen_point: (f32, f32),
) -> bool {
    let ancestors = path.ancestors();
    for ancestor in ancestors.iter().take(ancestors.len().saturating_sub(1)) {
        let Some(node) = scene.nodes.get(ancestor) else {
            continue;
        };
        let Some(point) = node.transform.inverse_point(screen_point) else {
            continue;
        };
        if let Some(widget) = find_widget(tree, ancestor)
            && widget.hit_test(point)
            && widget.blocks_children_hit_test(point)
        {
            return true;
        }
    }
    false
}

fn rect_contains(rect: Rect, point: (f32, f32)) -> bool {
    point.0 >= rect.0
        && point.0 <= rect.0 + rect.2
        && point.1 >= rect.1
        && point.1 <= rect.1 + rect.3
}

fn clip_intersect(existing: Option<Rect>, next: Rect) -> Rect {
    let Some(existing) = existing else {
        return next;
    };
    let x0 = existing.0.max(next.0);
    let y0 = existing.1.max(next.1);
    let x1 = (existing.0 + existing.2).min(next.0 + next.2);
    let y1 = (existing.1 + existing.3).min(next.1 + next.3);
    (x0, y0, (x1 - x0).max(0.0), (y1 - y0).max(0.0))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AnimationManager, Overflow, Portal, Style, StyleBuilder, View};

    fn box_at(widget: &mut dyn Widget, x: f32, y: f32, width: f32, height: f32) {
        widget.layout(LayoutBox {
            x,
            y,
            width,
            height,
        });
    }

    fn cascade(tree: &mut [Box<dyn Widget>]) {
        let mut animation = AnimationManager::new();
        for widget in tree {
            widget.cascade_style(&Style::default(), &mut animation);
        }
    }

    fn hit_key(scene: &SceneOrder, tree: &[Box<dyn Widget>], point: (f32, f32)) -> String {
        let path = scene.hit_test(tree, point).expect("expected a hit");
        find_widget(tree, &path)
            .and_then(Widget::get_key)
            .expect("hit widget should be keyed")
            .to_string()
    }

    #[test]
    fn positioned_auto_paints_and_hits_above_static_sibling() {
        let positioned = View::new()
            .key("positioned")
            .position(crate::Position::Relative);
        let static_widget = View::new().key("static");
        let mut tree: Vec<Box<dyn Widget>> = vec![Box::new(positioned), Box::new(static_widget)];
        for widget in &mut tree {
            box_at(widget.as_mut(), 0.0, 0.0, 100.0, 100.0);
        }
        cascade(&mut tree);

        let scene = SceneOrder::build(&tree, 1.0);
        assert_eq!(hit_key(&scene, &tree, (50.0, 50.0)), "positioned");
    }

    #[test]
    fn last_painted_is_first_hit_for_overlapping_z_order_property() {
        for first_z in -2..=2 {
            for second_z in -2..=2 {
                let mut tree: Vec<Box<dyn Widget>> = vec![
                    Box::new(View::new().key("first").z_index(first_z)),
                    Box::new(View::new().key("second").z_index(second_z)),
                ];
                for widget in &mut tree {
                    box_at(widget.as_mut(), 0.0, 0.0, 100.0, 100.0);
                }
                cascade(&mut tree);
                let scene = SceneOrder::build(&tree, 1.0);
                let painted_last = find_widget(
                    &tree,
                    scene.paint_order.last().expect("non-empty paint order"),
                )
                .and_then(Widget::get_key)
                .unwrap();
                assert_eq!(
                    hit_key(&scene, &tree, (50.0, 50.0)),
                    painted_last.as_str(),
                    "z combination ({first_z}, {second_z})"
                );
            }
        }
    }

    #[test]
    fn overflow_visible_child_is_hittable_outside_parent() {
        let child = View::new().key("overflow-child");
        let root = View::new().key("parent").child(child);
        let mut tree: Vec<Box<dyn Widget>> = vec![Box::new(root)];
        box_at(tree[0].as_mut(), 0.0, 0.0, 40.0, 40.0);
        box_at(
            tree[0].children_mut().unwrap()[0].as_mut(),
            60.0,
            0.0,
            30.0,
            30.0,
        );
        cascade(&mut tree);

        let scene = SceneOrder::build(&tree, 1.0);
        assert_eq!(hit_key(&scene, &tree, (70.0, 10.0)), "overflow-child");
    }

    #[test]
    fn nested_z_index_participates_in_the_shared_scene_order() {
        let nested = View::new().key("nested").z_index(5);
        let container = View::new().key("container").child(nested);
        let sibling = View::new().key("sibling").z_index(4);
        let mut tree: Vec<Box<dyn Widget>> = vec![Box::new(container), Box::new(sibling)];
        for widget in &mut tree {
            box_at(widget.as_mut(), 0.0, 0.0, 100.0, 100.0);
        }
        box_at(
            tree[0].children_mut().unwrap()[0].as_mut(),
            0.0,
            0.0,
            100.0,
            100.0,
        );
        cascade(&mut tree);

        let scene = SceneOrder::build(&tree, 1.0);
        assert_eq!(hit_key(&scene, &tree, (50.0, 50.0)), "nested");
    }

    #[test]
    fn overflow_clip_rejects_child_outside_parent() {
        let child = View::new().key("clipped-child");
        let root = View::new()
            .key("clipping-parent")
            .overflow(Overflow::Hidden, Overflow::Hidden)
            .child(child);
        let mut tree: Vec<Box<dyn Widget>> = vec![Box::new(root)];
        box_at(tree[0].as_mut(), 0.0, 0.0, 40.0, 40.0);
        box_at(
            tree[0].children_mut().unwrap()[0].as_mut(),
            60.0,
            0.0,
            30.0,
            30.0,
        );
        cascade(&mut tree);

        let scene = SceneOrder::build(&tree, 1.0);
        assert!(scene.hit_test(&tree, (70.0, 10.0)).is_none());
    }

    #[test]
    fn clip_excludes_child_but_portal_escapes_clip() {
        let clipped_child = View::new().key("clipped");
        let portal = Portal::new().child(View::new().key("portal-child"));
        let root = View::new()
            .key("parent")
            .overflow(Overflow::Hidden, Overflow::Hidden)
            .child(clipped_child)
            .child(portal);
        let mut tree: Vec<Box<dyn Widget>> = vec![Box::new(root)];
        box_at(tree[0].as_mut(), 0.0, 0.0, 40.0, 40.0);
        let children = tree[0].children_mut().unwrap();
        box_at(children[0].as_mut(), 60.0, 0.0, 30.0, 30.0);
        box_at(children[1].as_mut(), 0.0, 0.0, 0.0, 0.0);
        box_at(
            children[1].children_mut().unwrap()[0].as_mut(),
            60.0,
            0.0,
            30.0,
            30.0,
        );
        cascade(&mut tree);

        let scene = SceneOrder::build(&tree, 1.0);
        assert_eq!(hit_key(&scene, &tree, (70.0, 10.0)), "portal-child");
    }

    #[test]
    fn accumulated_scale_transform_controls_hit_geometry() {
        let child = View::new().key("scaled-child");
        let root = View::new().key("scaled-root").scale(0.5).child(child);
        let mut tree: Vec<Box<dyn Widget>> = vec![Box::new(root)];
        box_at(tree[0].as_mut(), 0.0, 0.0, 100.0, 100.0);
        box_at(
            tree[0].children_mut().unwrap()[0].as_mut(),
            0.0,
            0.0,
            100.0,
            100.0,
        );
        cascade(&mut tree);

        let scene = SceneOrder::build(&tree, 1.0);
        assert!(scene.hit_test(&tree, (10.0, 10.0)).is_none());
        assert_eq!(hit_key(&scene, &tree, (50.0, 50.0)), "scaled-child");
    }
}
