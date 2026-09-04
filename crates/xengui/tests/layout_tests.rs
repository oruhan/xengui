// SPDX-License-Identifier: Apache-2.0
use xengui::LayoutBox;

fn make_box() -> LayoutBox {
    LayoutBox {
        x: 0.0,
        y: 0.0,
        width: 100.0,
        height: 50.0,
    }
}

#[test]
fn contains_rounded_center_point_is_inside() {
    let b = make_box();
    assert!(b.contains_rounded((50.0, 25.0), 10.0));
}

#[test]
fn contains_rounded_outside_point_is_outside() {
    let b = make_box();
    assert!(!b.contains_rounded((200.0, 200.0), 10.0));
}

#[test]
fn contains_rounded_zero_radius_behaves_like_plain_rect() {
    let b = make_box();
    assert!(b.contains_rounded((0.0, 0.0), 0.0));
    assert!(b.contains_rounded((100.0, 50.0), 0.0));
}

#[test]
fn contains_rounded_corner_cutoff_excludes_far_corner_point() {
    // A point right at the box's own corner should fail against a
    // large enough corner radius, since the circular cutoff excludes it.
    let b = make_box();
    assert!(!b.contains_rounded((0.0, 0.0), 40.0));
}
