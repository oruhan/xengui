// SPDX-License-Identifier: Apache-2.0

use crate::{LayoutBox, Widget, WidgetPath};

/// Platform-independent accessibility role exposed by a widget.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SemanticRole {
    /// A momentary action control.
    Button,
    /// A binary or mixed-state checkbox.
    Checkbox,
    /// An on/off switch.
    Switch,
    /// A mutually exclusive option.
    Radio,
    /// A continuous or discrete range control.
    Slider,
    /// An editable text field.
    TextField,
    /// A navigational link.
    Link,
    /// Static text.
    Label,
    /// An image or illustration.
    Image,
    /// A menu container.
    Menu,
    /// An item inside a menu.
    MenuItem,
    /// A structural group with no more specific role.
    Group,
}

/// Action an accessibility adapter may request from a semantic node.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SemanticAction {
    /// Move keyboard/accessibility focus to the node.
    Focus,
    /// Activate the node's primary action.
    Activate,
    /// Replace the value of an editable or range control.
    SetValue,
    /// Increase a range control's value.
    Increment,
    /// Decrease a range control's value.
    Decrement,
}

/// Checked state exposed by toggle controls.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SemanticCheckedState {
    /// The control is off.
    Off,
    /// The control is on.
    On,
    /// The control represents a partially selected collection.
    Mixed,
}

/// Accessibility information contributed by one widget.
#[derive(Clone, Debug, PartialEq)]
pub struct Semantics {
    /// The widget's semantic role.
    pub role: SemanticRole,
    /// Human-readable accessible name.
    pub label: Option<String>,
    /// Additional help or context.
    pub description: Option<String>,
    /// Textual representation of the current value.
    pub value: Option<String>,
    /// Checked state for checkbox and switch controls.
    pub checked: Option<SemanticCheckedState>,
    /// Selected state for options such as radio buttons.
    pub selected: Option<bool>,
    /// Whether interaction is disabled.
    pub disabled: bool,
    /// Actions accepted by the node.
    pub actions: Vec<SemanticAction>,
}

impl Semantics {
    /// Creates semantic data for `role` with no optional metadata.
    pub fn new(role: SemanticRole) -> Self {
        Self {
            role,
            label: None,
            description: None,
            value: None,
            checked: None,
            selected: None,
            disabled: false,
            actions: Vec::new(),
        }
    }

    /// Assigns the accessible name.
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Assigns the exposed value.
    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.value = Some(value.into());
        self
    }

    /// Adds an action when it is not already present.
    pub fn action(mut self, action: SemanticAction) -> Self {
        if !self.actions.contains(&action) {
            self.actions.push(action);
        }
        self
    }
}

/// One node in the accessibility tree produced for a frame.
#[derive(Clone, Debug, PartialEq)]
pub struct SemanticsNode {
    /// Runtime identity shared with input, layout, and paint traversal.
    pub path: WidgetPath,
    /// Screen-space bounds of the semantic node.
    pub bounds: LayoutBox,
    /// Widget-provided semantic data.
    pub semantics: Semantics,
    /// Semantic descendants, independent of purely visual wrapper widgets.
    pub children: Vec<SemanticsNode>,
}

/// Builds accessibility roots from the current widget tree.
///
/// Widgets without semantics are transparent: their semantic descendants are
/// attached to the nearest semantic ancestor instead of creating empty nodes.
pub fn build_semantics_tree(tree: &[Box<dyn Widget>]) -> Vec<SemanticsNode> {
    let mut roots = Vec::new();
    let root = WidgetPath::new();
    collect_semantics(tree, &root, &mut roots);
    roots
}

fn collect_semantics(
    widgets: &[Box<dyn Widget>],
    parent_path: &WidgetPath,
    output: &mut Vec<SemanticsNode>,
) {
    for (index, widget) in widgets.iter().enumerate() {
        let mut path = parent_path.clone();
        path.push(widget.as_ref(), index);
        let mut descendants = Vec::new();
        collect_semantics(widget.children(), &path, &mut descendants);

        if let Some(semantics) = widget.semantics() {
            output.push(SemanticsNode {
                path,
                bounds: *widget.layout_box(),
                semantics,
                children: descendants,
            });
        } else {
            output.extend(descendants);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{SemanticAction, SemanticCheckedState, SemanticRole, build_semantics_tree};
    use crate::{Button, Checkbox, Slider, View, Widget};

    #[test]
    fn visual_wrappers_are_transparent_to_the_semantics_tree() {
        let tree: Vec<Box<dyn Widget>> =
            vec![Box::new(View::new().child(Button::new().label("Save")))];

        let semantics = build_semantics_tree(&tree);

        assert_eq!(semantics.len(), 1);
        assert_eq!(semantics[0].semantics.role, SemanticRole::Button);
        assert_eq!(semantics[0].semantics.label.as_deref(), Some("Save"));
        assert!(
            semantics[0]
                .semantics
                .actions
                .contains(&SemanticAction::Activate)
        );
    }

    #[test]
    fn control_state_is_exposed_without_inferring_it_from_paint() {
        let tree: Vec<Box<dyn Widget>> = vec![
            Box::new(
                Checkbox::new()
                    .checked(true)
                    .accessible_label("Notifications"),
            ),
            Box::new(Slider::new().value(0.25)),
        ];

        let semantics = build_semantics_tree(&tree);

        assert_eq!(
            semantics[0].semantics.checked,
            Some(SemanticCheckedState::On)
        );
        assert_eq!(
            semantics[0].semantics.label.as_deref(),
            Some("Notifications")
        );
        assert_eq!(semantics[1].semantics.value.as_deref(), Some("0.25"));
    }
}
