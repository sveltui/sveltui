use std::cell::Cell as Counter;
use std::rc::Rc;
use sveltui::prelude::*;

/// Which lines a table draws between its cells.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TableRules {
    /// No lines; columns are kept apart by `column_gap`.
    None,
    /// One line under the header.
    #[default]
    Header,
    /// A line between every row.
    Rows,
    /// A line between every column, and one under the header.
    Columns,
    /// Lines between every row and every column.
    All,
}

impl TableRules {
    pub(crate) fn columns(self) -> bool {
        matches!(self, TableRules::Columns | TableRules::All)
    }

    /// Whether the header needs a line of its own, because row lines
    /// don't already draw one.
    pub(crate) fn header_line(self) -> bool {
        matches!(self, TableRules::Header | TableRules::Columns)
    }
}

/// Where a cell's content sits across its column.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CellAlign {
    #[default]
    Start,
    Center,
    End,
}

#[derive(Props)]
pub struct RootProps {
    /// The columns, left to right, e.g. `[auto(), fr(1.0), length(8)]`.
    /// Every row lines up to these, whatever size its content is.
    pub columns: Vec<Track>,
    /// The border around the whole table.
    #[prop(default = BorderStyle::None)]
    pub border: BorderStyle,
    #[prop(default = Color::DarkGrey)]
    pub border_color: Color,
    /// Shown in the top border, when there is one.
    #[prop(optional)]
    pub title: Option<String>,
    /// What the table holds, for assistive tech. Defaults to the title.
    #[prop(optional)]
    pub label: Option<String>,
    #[prop(default = TableRules::Header)]
    pub rules: TableRules,
    /// The style of the lines between cells.
    #[prop(default = BorderStyle::Single)]
    pub rule_style: BorderStyle,
    /// Space between columns when no line separates them.
    #[prop(default = 2)]
    pub column_gap: u16,
    #[snippet]
    pub children: Snippet,
}

/// Groups the header rows. Their cells are column headers.
#[derive(Props)]
pub struct HeaderProps {
    #[snippet]
    pub children: Snippet,
}

/// Groups the body rows.
#[derive(Props)]
pub struct BodyProps {
    #[snippet]
    pub children: Snippet,
}

#[derive(Props)]
pub struct RowProps {
    #[snippet]
    pub children: Snippet,
}

#[derive(Props)]
pub struct CellProps {
    #[prop(default = CellAlign::Start)]
    pub align: CellAlign,
    /// How many columns the cell covers.
    #[prop(default = 1)]
    pub colspan: u16,
    #[snippet(optional)]
    pub children: Option<Snippet>,
}

/// What a table's parts share while it renders: its shape, and where the
/// next cell goes.
#[derive(Clone)]
pub(crate) struct TableContext(pub(crate) Rc<TableState>);

pub(crate) struct TableState {
    pub columns: usize,
    pub rules: TableRules,
    /// Space between a cell's content and the lines beside it.
    pub cell_padding: u16,
    pub in_header: Counter<bool>,
    pub row: Counter<usize>,
    pub column: Counter<usize>,
}
