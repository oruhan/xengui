// SPDX-License-Identifier: Apache-2.0
use xengui::{
    BorderRadius, Color, Constraints, Edges, Length, MeasureResult, Overflow, Size, Style,
};

#[test]
fn color_hex_parses_short_and_long_forms() {
    let short = Color::hex("#F0A");
    let long = Color::hex("#FF00AA");
    assert_eq!(short.to_f32_array(), long.to_f32_array());
}

#[test]
fn color_hex_invalid_falls_back_to_black() {
    let c = Color::hex("nothex");
    assert_eq!(c.to_f32_array(), Color::BLACK.to_f32_array());
}

#[test]
fn color_rgba_roundtrip() {
    let c = Color::rgba(255, 0, 128, 64);
    let arr = c.to_f32_array();
    assert!((arr[0] - 1.0).abs() < f32::EPSILON);
    assert!((arr[1] - 0.0).abs() < f32::EPSILON);
}

#[test]
fn length_px_to_physical_scales_with_factor() {
    let l = Length::px(10.0);
    assert_eq!(l.to_physical(2.0), 20.0);
}

#[test]
fn length_percent_ignores_scale_factor() {
    let l = Length::Percent(50.0);
    assert_eq!(l.to_physical(2.0), 50.0);
}

#[test]
fn length_add_sub_px() {
    let l = Length::px(10.0);
    assert_eq!(l.add_px(5.0).value(), 15.0);
    assert_eq!(l.sub_px(5.0).value(), 5.0);
    // sub_px never goes below zero.
    assert_eq!(l.sub_px(50.0).value(), 0.0);
}

#[test]
fn border_radius_all_sets_every_corner() {
    let r = BorderRadius::all(8.0);
    assert!(r.is_uniform());
    assert_eq!(r.max_value(), 8.0);
}

#[test]
fn border_radius_only_sets_per_corner() {
    let r = BorderRadius::only(1.0, 2.0, 3.0, 4.0);
    assert_eq!(r.top_left, Length::px(1.0));
    assert_eq!(r.bottom_left, Length::px(4.0));
    assert!(!r.is_uniform());
}

#[test]
fn border_radius_overlap_correction_scales_down() {
    // Two adjacent 100px radii on a 50px-wide box must be scaled down
    // so they no longer overlap, matching CSS border-radius correction.
    let r = BorderRadius::all(100.0);
    let physical = r.to_physical_array(1.0, 50.0, 50.0);
    for v in physical {
        assert!(v <= 25.0 + 0.001);
    }
}

#[test]
fn constraints_constrain_width_prefers_known() {
    let c = Constraints::new()
        .with_known_width(50.0)
        .with_max_width(100.0);
    assert_eq!(c.constrain_width(10.0), 50.0);
}

#[test]
fn constraints_constrain_width_clamps_to_max() {
    let c = Constraints::new().with_max_width(20.0);
    assert_eq!(c.constrain_width(50.0), 20.0);
}

#[test]
fn constraints_unbounded_passes_through() {
    let c = Constraints::UNBOUNDED;
    assert_eq!(c.constrain_size(30.0, 40.0), (30.0, 40.0));
}

#[test]
fn measure_result_baseline_builder() {
    let m = MeasureResult::new(10.0, 20.0).baseline(5.0);
    assert_eq!(m.baseline, Some(5.0));
    assert_eq!(m.into_tuple(), (10.0, 20.0));
}

#[test]
fn edges_all_symmetric_only() {
    let a = Edges::all(4.0);
    assert_eq!(a.left(), Length::px(4.0));
    assert_eq!(a.top(), Length::px(4.0));

    let s = Edges::symmetric(2.0, 6.0);
    assert_eq!(s.left(), Length::px(2.0));
    assert_eq!(s.top(), Length::px(6.0));

    let o = Edges::only(1.0, 2.0, 3.0, 4.0);
    assert_eq!(o.left(), Length::px(1.0));
    assert_eq!(o.bottom(), Length::px(4.0));
}

#[test]
fn style_inherit_fills_unset_inheritable_fields() {
    let parent = Style {
        color: Some(Color::RED_500),
        ..Default::default()
    };
    let child = Style::default();
    let merged = parent.inherit_style(&child);
    assert_eq!(merged.color, Some(Color::RED_500));
}

#[test]
fn style_inherit_child_value_wins_over_parent() {
    let parent = Style {
        color: Some(Color::RED_500),
        ..Default::default()
    };
    let child = Style {
        color: Some(Color::BLUE_500),
        ..Default::default()
    };
    let merged = parent.inherit_style(&child);
    assert_eq!(merged.color, Some(Color::BLUE_500));
}

#[test]
fn style_overlay_non_inherited_field_falls_back_to_base() {
    let base = Style {
        size: Some(Size::new(Length::px(10.0), Length::px(20.0))),
        ..Default::default()
    };
    let patch = Style::default();
    let merged = base.overlay(&patch);
    assert_eq!(merged.size.unwrap().width, Some(Length::px(10.0)));
}

#[test]
fn style_overlay_patch_overrides_base() {
    let base = Style {
        overflow_x: Some(Overflow::Hidden),
        ..Default::default()
    };
    let patch = Style {
        overflow_x: Some(Overflow::Scroll),
        ..Default::default()
    };
    let merged = base.overlay(&patch);
    assert_eq!(merged.overflow_x, Some(Overflow::Scroll));
}
