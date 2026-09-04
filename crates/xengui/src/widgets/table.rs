// SPDX-License-Identifier: Apache-2.0
use crate::{
    Align, Background, Border, Color, Display, Edges, FlexDirection, Interaction, Label, LayoutBox,
    Length, Render, Style, StyleBuilder, View, Widget, WidgetBase, WidgetId,
};
use smol_str::SmolStr;

/// Data and behavior represented by `TableColumn`.
pub struct TableColumn {
    /// The `header` value carried by this type.
    pub header: SmolStr,
    /// The `width` value carried by this type.
    pub width: Length,
    /// The `align` value carried by this type.
    pub align: Align,
}

impl TableColumn {
    /// Creates a value with its default configuration.
    pub fn new(header: impl Into<SmolStr>, width: impl Into<Length>) -> Self {
        Self {
            header: header.into(),
            width: width.into(),
            align: Align::Start,
        }
    }

    /// Returns or updates the `align` value.
    pub fn align(mut self, align: Align) -> Self {
        self.align = align;
        self
    }
}

// Cells are stored as factories instead of built widgets, since a
// composite widget's `render` only gets `&self` and can't move a
// non-Clone `Box<dyn Widget>` out of it.
/// Data and behavior represented by `TableRow`.
pub struct TableRow {
    cells: Vec<Box<dyn Fn() -> Box<dyn Widget>>>,
}

impl TableRow {
    /// Creates a value with its default configuration.
    pub fn new() -> Self {
        Self { cells: Vec::new() }
    }

    /// Returns or updates the `cell` value.
    pub fn cell<W: Widget + 'static>(mut self, build: impl (Fn() -> W) + 'static) -> Self {
        self.cells
            .push(Box::new(move || Box::new(build()) as Box<dyn Widget>));
        self
    }

    /// Returns or updates the `text` value.
    pub fn text(self, content: impl Into<SmolStr>) -> Self {
        let content = content.into();
        self.cell(move || Label::new().label(content.clone()))
    }
}

impl Default for TableRow {
    fn default() -> Self {
        Self::new()
    }
}

/// A fully customizable data table, built from `View`/`Label` primitives.
/// Every visual aspect - header/row backgrounds, borders, padding, hover
/// highlight - can be overridden; unset fields fall back to theme defaults.
pub struct Table {
    base: WidgetBase,
    layout_box: LayoutBox,
    inner: Vec<Box<dyn Widget>>,
    hooks_id: WidgetId,

    columns: Vec<TableColumn>,
    rows: Vec<TableRow>,

    show_header: bool,
    header_background: Option<Background>,
    header_text_color: Option<Color>,
    header_padding: Option<Edges>,

    row_background: Option<Background>,
    row_alt_background: Option<Background>,
    row_hover_background: Option<Background>,
    // Alternates rows between a light and dark gray when no explicit
    // `row_background`/`row_alt_background` is set. Off by default.
    striped: bool,
    cell_padding: Option<Edges>,

    border_color: Option<Color>,
    row_height: Option<Length>,
}

impl Table {
    /// Creates a value with its default configuration.
    pub fn new() -> Self {
        Self {
            base: WidgetBase::new(Interaction::new()),
            layout_box: LayoutBox::default(),
            inner: Vec::new(),
            hooks_id: WidgetId::new_unique(),

            columns: Vec::new(),
            rows: Vec::new(),

            show_header: true,
            header_background: None,
            header_text_color: None,
            header_padding: None,

            row_background: None,
            row_alt_background: None,
            row_hover_background: None,
            striped: false,
            cell_padding: None,

            border_color: None,
            row_height: None,
        }
    }

    /// Returns or updates the `column` value.
    pub fn column(mut self, column: TableColumn) -> Self {
        self.columns.push(column);
        self
    }

    /// Returns or updates the `columns` value.
    pub fn columns(mut self, columns: impl Into<Vec<TableColumn>>) -> Self {
        self.columns = columns.into();
        self
    }

    /// Returns or updates the `row` value.
    pub fn row(mut self, row: TableRow) -> Self {
        self.rows.push(row);
        self
    }

    /// Returns or updates the `rows` value.
    pub fn rows(mut self, rows: impl Into<Vec<TableRow>>) -> Self {
        self.rows = rows.into();
        self
    }

    /// Returns or updates the `show_header` value.
    pub fn show_header(mut self, value: bool) -> Self {
        self.show_header = value;
        self
    }

    /// Returns or updates the `header_background` value.
    pub fn header_background(mut self, background: impl Into<Background>) -> Self {
        self.header_background = Some(background.into());
        self
    }

    /// Returns or updates the `header_text_color` value.
    pub fn header_text_color(mut self, color: Color) -> Self {
        self.header_text_color = Some(color);
        self
    }

    /// Returns or updates the `header_padding` value.
    pub fn header_padding(mut self, padding: impl Into<Edges>) -> Self {
        self.header_padding = Some(padding.into());
        self
    }

    /// Returns or updates the `row_background` value.
    pub fn row_background(mut self, background: impl Into<Background>) -> Self {
        self.row_background = Some(background.into());
        self
    }

    /// Returns or updates the `row_alt_background` value.
    pub fn row_alt_background(mut self, background: impl Into<Background>) -> Self {
        self.row_alt_background = Some(background.into());
        self
    }

    /// Returns or updates the `row_hover_background` value.
    pub fn row_hover_background(mut self, background: impl Into<Background>) -> Self {
        self.row_hover_background = Some(background.into());
        self
    }

    /// Alternates each row's background between a light and dark gray,
    /// like a classic "zebra striped" table. Disabled by default; an
    /// explicit `row_background`/`row_alt_background` always takes
    /// priority over this default striping.
    pub fn striped(mut self, value: bool) -> Self {
        self.striped = value;
        self
    }

    /// Returns or updates the `cell_padding` value.
    pub fn cell_padding(mut self, padding: impl Into<Edges>) -> Self {
        self.cell_padding = Some(padding.into());
        self
    }

    /// Returns or updates the `border_color` value.
    pub fn border_color(mut self, color: Color) -> Self {
        self.border_color = Some(color);
        self
    }

    /// Returns or updates the `row_height` value.
    pub fn row_height(mut self, height: impl Into<Length>) -> Self {
        self.row_height = Some(height.into());
        self
    }
}

impl Default for Table {
    fn default() -> Self {
        Self::new()
    }
}

impl StyleBuilder for Table {
    fn style_mut(&mut self) -> &mut Style {
        &mut self.base.style
    }

    fn mark_dirty(&mut self) {
        self.base.dirty = true;
    }
}

impl Render for Table {
    fn render(&self) -> Box<dyn Widget> {
        let theme = crate::current_theme();
        let border_color = self.border_color.unwrap_or(theme.outline_variant);
        let cell_padding = self
            .cell_padding
            .unwrap_or_else(|| Edges::symmetric(10.0, 8.0));

        let mut root = View::new()
            .display(Display::Flex)
            .flex_direction(FlexDirection::Column)
            .border(Border::all(1.0, border_color).radius(theme.radius_sm));

        if self.show_header && !self.columns.is_empty() {
            let mut header_row = View::new()
                .display(Display::Flex)
                .flex_direction(FlexDirection::Row)
                .background(
                    self.header_background
                        .clone()
                        .unwrap_or(Background::Color(theme.surface)),
                )
                // Rounds the header's top corners to match the table's own
                // outer radius instead of squaring it off.
                .border(Border::bottom(1.0, border_color).radius(theme.radius_sm));

            for column in &self.columns {
                let label = Label::new()
                    .label(column.header.clone())
                    .color(self.header_text_color.unwrap_or(theme.on_surface));

                let cell = View::new()
                    .width(column.width)
                    .padding(self.header_padding.unwrap_or(cell_padding))
                    .align_items(column.align)
                    .child(label);

                header_row = header_row.child(cell);
            }

            root = root.child(header_row);
        }

        let row_count = self.rows.len();
        let has_header = self.show_header && !self.columns.is_empty();

        for (i, row) in self.rows.iter().enumerate() {
            let is_alt = i % 2 == 1;

            // Explicit row_background/row_alt_background always win; the
            // striped default only fills the gap when nothing else was set.
            let background = if is_alt {
                self.row_alt_background
                    .clone()
                    .or_else(|| self.row_background.clone())
                    .or_else(|| {
                        self.striped
                            .then_some(Background::Color(theme.surface_container_high))
                    })
            } else {
                self.row_background
                    .clone()
                    .or_else(|| self.striped.then_some(Background::Color(theme.surface)))
            };

            let mut row_border = Border::bottom(1.0, border_color);
            // Rounds whichever edge touches the table's own rounded corners
            // (top of the first row when there's no header, bottom of the
            // last row always), so row backgrounds don't square them off.
            let is_first_visible = i == 0 && !has_header;
            let is_last = i + 1 == row_count;
            if is_first_visible || is_last {
                row_border = row_border.radius(theme.radius_sm);
            }

            let mut row_view = View::new()
                .display(Display::Flex)
                .flex_direction(FlexDirection::Row)
                .border(row_border);

            if let Some(bg) = background {
                row_view = row_view.background(bg);
            }
            if let Some(hover_bg) = self.row_hover_background.clone() {
                row_view = row_view.hover_background(hover_bg);
            }
            if let Some(height) = self.row_height {
                row_view = row_view.height(height);
            }

            for (col_index, cell_build) in row.cells.iter().enumerate() {
                let width = self
                    .columns
                    .get(col_index)
                    .map(|c| c.width)
                    .unwrap_or(Length::pct(100.0));
                let align = self
                    .columns
                    .get(col_index)
                    .map(|c| c.align)
                    .unwrap_or(Align::Start);

                let cell_widget = cell_build();
                let cell = View::new()
                    .width(width)
                    .padding(cell_padding)
                    .align_items(align)
                    .children_vec(vec![cell_widget]);

                row_view = row_view.child(cell);
            }

            root = root.child(row_view);
        }

        Box::new(root)
    }
}

crate::impl_composite_widget!(Table);
