// SPDX-License-Identifier: Apache-2.0
/// Allows a widget to receive shorthand text content from [`view!`](crate::view!).
pub trait WidgetContent: Sized {
    /// Returns the widget with its primary text content replaced.
    fn with_content(self, content: impl Into<smol_str::SmolStr>) -> Self;
}

impl WidgetContent for crate::Label {
    fn with_content(self, content: impl Into<smol_str::SmolStr>) -> Self {
        self.label(content)
    }
}

/// ```ignore
/// view! {
///     View {
///         width: 400,
///         height: 300,
///         padding: 20,
///         background: Color::BLACK,
///         Label("Hello"),
///         View { flex_direction: FlexDirection::Row, Label("World") }
///     }
/// }
/// ```
#[macro_export]
macro_rules! view {
    (
        $widget:ident { $($rest:tt)* }
    ) => {
        $crate::view_props!( $widget::new() ; $($rest)* )
    };
    ($widget:ident($content:expr)) => {
        $crate::WidgetContent::with_content($widget::new(), $content)
    };
}

#[macro_export]
/// Expands builder-like properties and child declarations used by [`view!`](crate::view!).
macro_rules! view_props {
    ($acc:expr;) => { $acc };

    // prop: (a, b, ...) - for builder methods taking multiple positional
    // arguments (e.g. `gap: (4, 0)` -> `.gap(4, 0)`), distinct from a
    // single expr that happens to be a parenthesized tuple.
    ($acc:expr; $key:ident: ($($val:expr),+ $(,)?)) => {
        $acc.$key($($val),+)
    };
    (
        $acc:expr;
        $key:ident: ($($val:expr),+ $(,)?),
        $($rest:tt)*
    ) => {
        $crate::view_props!( $acc.$key($($val),+) ; $($rest)* )
    };

    // Child { ... }
    (
        $acc:expr;
        $widget:ident { $($inner:tt)* }
    ) => {
        {
        let mut __parent = $acc;
        let __child = $crate::view_props!( $widget::new() ; $($inner)* );
        __parent = __parent.child(__child);
        __parent
        }
    };
    // Child { ... }
    (
        $acc:expr;
        $widget:ident { $($inner:tt)* },
        $($rest:tt)*
    ) => {
        $crate::view_props!({
            let mut __parent = $acc;
            let __child = $crate::view_props!( $widget::new() ; $($inner)* );
            __parent = __parent.child(__child);
            __parent
        } ; $($rest)*)
    };

    // Child(expr)
    ($acc:expr; $widget:ident($content:expr)) => {
        {
        let mut __parent = $acc;
        let __child = $crate::WidgetContent::with_content($widget::new(), $content);
        __parent = __parent.child(__child);
        __parent
        }
    };
    // Child(expr)
    (
        $acc:expr;
        $widget:ident($content:expr),
        $($rest:tt)*
    ) => {
        $crate::view_props!({
            let mut __parent = $acc;
            let __child = $crate::WidgetContent::with_content($widget::new(), $content);
            __parent = __parent.child(__child);
            __parent
        } ; $($rest)*)
    };
}

#[macro_export]
/// Expands the `impl_themed_style_builders` convenience syntax.
macro_rules! impl_themed_style_builders {
    ($ty:ty; $($method:ident => $field:ident),+ $(,)?) => {
        impl $ty {
            $(
                #[doc = concat!("Builds the themed `", stringify!($method), "` style state.")]
                pub fn $method(
                    mut self,
                    build: impl FnOnce($crate::StylePatch, &$crate::Theme) -> $crate::StylePatch
                ) -> Self {
                    let theme = $crate::current_theme();
                    self.$field = Some(build($crate::StylePatch::new(), &theme).build());
                    self.mark_dirty();
                    self
                }
            )+
        }
    };
    (base $ty:ty; $($method:ident => $field:ident),+ $(,)?) => {
        impl $ty {
            $(
                #[doc = concat!("Builds the themed `", stringify!($method), "` style state.")]
                pub fn $method(
                    mut self,
                    build: impl FnOnce($crate::StylePatch, &$crate::Theme) -> $crate::StylePatch
                ) -> Self {
                    let theme = $crate::current_theme();
                    self.base.$field = Some(build($crate::StylePatch::new(), &theme).build());
                    self.mark_dirty();
                    self
                }
            )+
        }
    };
}

#[macro_export]
/// Expands the `impl_common_style_builders` convenience syntax.
macro_rules! impl_common_style_builders {
    (base $ty:ty) => {
        impl $ty {
            /// Stable identity among siblings, kept across rebuilds even when this
            /// widget moves position (reorder, insert, remove). Use for list items
            /// instead of relying on array index.
            pub fn key(mut self, key: impl Into<smol_str::SmolStr>) -> Self {
                self.base.key = Some(key.into());
                self
            }

            /// Global identifier for this widget, usable with `xengui::dom` to
            /// trigger it from anywhere (e.g. `dom::click("submitBtn")`), the same
            /// way HTML's `id` + JS's `getElementById(id).click()` work.
            pub fn id(mut self, id: impl Into<smol_str::SmolStr>) -> Self {
                self.base.id = Some(id.into());
                self
            }

            /// Selects the font family used by this widget's text.
            pub fn font(mut self, font: impl Into<smol_str::SmolStr>) -> Self {
                self.base.style.font = Some(font.into());
                self.mark_dirty();
                self
            }

            /// Sets the background used while the pointer hovers this widget.
            pub fn hover_background<M>(
                mut self,
                background: impl $crate::IntoThemed<$crate::Background, M>,
            ) -> Self {
                self.base
                    .hover_style
                    .get_or_insert_with($crate::Style::default)
                    .background = Some(background.resolve_themed());
                self.mark_dirty();
                self
            }

            /// Sets the background used while this widget is pressed.
            pub fn pressed_background<M>(
                mut self,
                background: impl $crate::IntoThemed<$crate::Background, M>,
            ) -> Self {
                self.base
                    .pressed_style
                    .get_or_insert_with($crate::Style::default)
                    .background = Some(background.resolve_themed());
                self.mark_dirty();
                self
            }

            /// Sets the background used while this widget is disabled.
            pub fn disabled_background<M>(
                mut self,
                background: impl $crate::IntoThemed<$crate::Background, M>,
            ) -> Self {
                self.base
                    .disabled_style
                    .get_or_insert_with($crate::Style::default)
                    .background = Some(background.resolve_themed());
                self.mark_dirty();
                self
            }

            /// Enables or disables input handling for this widget.
            pub fn enabled(mut self, enabled: bool) -> Self {
                self.base.interaction.set_enabled(enabled);
                self.mark_dirty();
                self
            }
        }
    };
}

/// Generates the `Widget` trait accessor methods every `base: WidgetBase` +
/// `layout_box: LayoutBox` widget needs (identity, dirty flag, style
/// pointers, interaction, layout box). Widget-specific behavior
/// (`debug_name`, `children`, `measure`, `paint`, `event`, ...) is still
/// implemented by hand next to this macro call.
#[macro_export]
macro_rules! impl_widget_boilerplate {
    () => {
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }

        fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
            self
        }

        fn get_key(&self) -> Option<&smol_str::SmolStr> {
            self.base.key.as_ref()
        }

        fn is_dirty(&self) -> bool {
            self.base.dirty
        }

        fn set_dirty(&mut self, dirty: bool) {
            self.base.dirty = dirty;
        }

        fn is_layout_dirty(&self) -> bool {
            self.base.layout_dirty
        }

        fn set_layout_dirty(&mut self, value: bool) {
            self.base.layout_dirty = value;
        }

        fn style(&self) -> &$crate::Style {
            &self.base.style
        }

        fn style_mut(&mut self) -> &mut $crate::Style {
            &mut self.base.style
        }

        fn computed_style(&self) -> &$crate::Style {
            &self.base.computed_style
        }

        fn interaction(&self) -> Option<&$crate::Interaction> {
            Some(&self.base.interaction)
        }

        fn interaction_mut(&mut self) -> Option<&mut $crate::Interaction> {
            Some(&mut self.base.interaction)
        }

        fn layout(&mut self, rect: $crate::LayoutBox) {
            self.layout_box = rect;
        }

        fn layout_box(&self) -> &$crate::LayoutBox {
            &self.layout_box
        }
    };
}

/// Generates the `Widget` implementation for a user-defined composite
/// widget. The target type must have these fields:
/// `base: WidgetBase`, `layout_box: LayoutBox`,
/// `inner: Vec<Box<dyn Widget>>`, `hooks_id: WidgetId`,
/// and must implement `Render`.
#[macro_export]
macro_rules! impl_composite_widget {
    ($ty:ty) => {
        impl $crate::Widget for $ty {
            $crate::impl_widget_boilerplate!();

            fn debug_name(&self) -> &'static str {
                stringify!($ty)
            }

            fn children(&self) -> &[Box<dyn $crate::Widget>] {
                &self.inner
            }

            fn children_mut(&mut self) -> Option<&mut Vec<Box<dyn $crate::Widget>>> {
                Some(&mut self.inner)
            }

            fn measure(
                &self,
                _ctx: &mut $crate::MeasureContext,
                _constraints: $crate::Constraints,
            ) -> $crate::MeasureResult {
                $crate::MeasureResult::new(0.0, 0.0)
            }

            fn paint(&self, _ctx: &mut $crate::PaintContext) {}

            fn cascade_style(
                &mut self,
                parent: &$crate::Style,
                anim: &mut $crate::AnimationManager,
            ) {
                self.base.inherited_style = parent.clone();
                self.base.recompute_style();

                // First mount only - there was no predecessor for
                // `transfer_composite_children` to reconcile against, so
                // this is the sole place content ever gets built from an
                // empty `inner`.
                if self.inner.is_empty() {
                    let key = format!("{}#{}", stringify!($ty), self.hooks_id.get());
                    let built = $crate::component(key, || $crate::composite::Render::render(self));
                    self.inner = vec![built];
                }

                for child in self.inner.iter_mut() {
                    child.cascade_style(&self.base.computed_style, anim);
                }
            }

            fn transfer_interaction_state(&mut self, old: &dyn $crate::Widget) {
                if let (Some(new), Some(old_i)) = (self.interaction_mut(), old.interaction()) {
                    new.transfer_from(old_i);
                }
                if let Some(old) = old.as_any().downcast_ref::<$ty>() {
                    self.hooks_id = old.hooks_id;
                }
            }

            fn transfer_composite_children(&mut self, old: &mut dyn $crate::Widget) {
                // Re-renders against current props, then reconciles the
                // result against the predecessor's already-committed
                // content, so descendant interaction/hook state survives
                // a parent prop update instead of rebuilding from scratch.
                let key = format!("{}#{}", stringify!($ty), self.hooks_id.get());
                let rendered = $crate::component(key, || $crate::composite::Render::render(self));

                if let Some(old) = old.as_any_mut().downcast_mut::<$ty>() {
                    let mut old_inner = std::mem::take(&mut old.inner);
                    self.inner = $crate::reconciler::reconcile_now(vec![rendered], &mut old_inner);
                } else {
                    self.inner = vec![rendered];
                }
            }
        }
    };
}
