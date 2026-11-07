use std::cmp::Ordering;
use std::collections::HashSet;
use std::rc::Rc;
use sveltui::prelude::*;

use crate::table::{CellAlign, TableRules};

/// Which way a column is sorted.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SortDirection {
    Ascending,
    Descending,
}

/// The column the grid is sorted by, and which way.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SortState {
    pub column: usize,
    pub direction: SortDirection,
}

impl IntoPropOption<SortState> for SortState {
    fn into_prop_option(self) -> Option<SortState> {
        Some(self)
    }
}

/// A cell, by the row's index in the data (not its place on screen, which
/// changes with sorting) and its column.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CellRef {
    pub row: usize,
    pub column: usize,
}

/// What a custom cell renderer gets to know about its cell.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CellState {
    /// The row's index in the data.
    pub row: usize,
    pub column: usize,
    /// The cell has the grid's cursor (and the grid has focus).
    pub active: bool,
    /// The row is selected.
    pub selected: bool,
}

/// Orders two rows for sorting.
type Compare<T> = Rc<dyn Fn(&T, &T) -> Ordering>;
/// Draws a cell's content.
type Render<T> = Rc<dyn Fn(&T, CellState) -> VNode>;

/// One column of a `DataGrid`: its header, how to show a row's value, and
/// optionally how to sort by it or draw it.
pub struct GridColumn<T> {
    pub(crate) header: String,
    pub(crate) width: Track,
    pub(crate) align: CellAlign,
    pub(crate) text: Rc<dyn Fn(&T) -> String>,
    pub(crate) compare: Option<Compare<T>>,
    pub(crate) render: Option<Render<T>>,
}

impl<T> Clone for GridColumn<T> {
    fn clone(&self) -> Self {
        Self {
            header: self.header.clone(),
            width: self.width.clone(),
            align: self.align,
            text: self.text.clone(),
            compare: self.compare.clone(),
            render: self.render.clone(),
        }
    }
}

impl<T: 'static> GridColumn<T> {
    /// A column titled `header`, showing `text(row)` in each cell. It's as
    /// wide as its header until `width` says otherwise.
    pub fn new(header: impl Into<String>, text: impl Fn(&T) -> String + 'static) -> Self {
        Self {
            header: header.into(),
            width: Track::Auto,
            align: CellAlign::Start,
            text: Rc::new(text),
            compare: None,
            render: None,
        }
    }

    /// The column's width, e.g. `length(10)` or `fr(1.0)`. Content-sized
    /// widths (`auto()`, `min_content()`, `max_content()`) size to the header,
    /// so columns never shift as different rows scroll past.
    pub fn width(mut self, width: Track) -> Self {
        self.width = width;
        self
    }

    pub fn align(mut self, align: CellAlign) -> Self {
        self.align = align;
        self
    }

    /// Lets the header sort the grid by this column's text.
    pub fn sortable(mut self) -> Self {
        let text = self.text.clone();
        self.compare = Some(Rc::new(move |a, b| text(a).cmp(&text(b))));
        self
    }

    /// Lets the header sort the grid by this comparison, for values whose
    /// text doesn't sort the right way (numbers, dates).
    pub fn sort_by(mut self, compare: impl Fn(&T, &T) -> Ordering + 'static) -> Self {
        self.compare = Some(Rc::new(compare));
        self
    }

    /// Draws the cell's content instead of its text. The text is still what
    /// assistive tech reads and what `sortable` sorts by.
    pub fn render(mut self, render: impl Fn(&T, CellState) -> VNode + 'static) -> Self {
        self.render = Some(Rc::new(render));
        self
    }

    pub(crate) fn is_sortable(&self) -> bool {
        self.compare.is_some()
    }
}

/// Where a `DataGrid`'s rows come from. The grid only ever borrows them, to
/// sort and to copy out the rows on screen.
pub enum GridRows<T: 'static> {
    /// Rows in state, so the grid follows changes to them.
    State(State<Vec<T>>),
    /// Rows that don't change for as long as this value lives.
    Shared(Rc<Vec<T>>),
}

impl<T: 'static> Clone for GridRows<T> {
    fn clone(&self) -> Self {
        match self {
            Self::State(state) => Self::State(*state),
            Self::Shared(rows) => Self::Shared(rows.clone()),
        }
    }
}

impl<T: Clone + 'static> GridRows<T> {
    pub(crate) fn with<R>(&self, f: impl FnOnce(&[T]) -> R) -> R {
        match self {
            Self::State(state) => state.with(|rows| f(rows)),
            Self::Shared(rows) => f(rows),
        }
    }

    pub(crate) fn len(&self) -> usize {
        self.with(<[T]>::len)
    }

    /// Changes whenever the rows might have. Shared rows never change, so
    /// their identity is enough.
    pub(crate) fn version(&self) -> u64 {
        match self {
            Self::State(state) => state.version(),
            Self::Shared(rows) => Rc::as_ptr(rows) as u64,
        }
    }
}

impl<T: 'static> From<State<Vec<T>>> for GridRows<T> {
    fn from(state: State<Vec<T>>) -> Self {
        Self::State(state)
    }
}

impl<T: 'static> From<Rc<Vec<T>>> for GridRows<T> {
    fn from(rows: Rc<Vec<T>>) -> Self {
        Self::Shared(rows)
    }
}

impl<T: 'static> From<Vec<T>> for GridRows<T> {
    fn from(rows: Vec<T>) -> Self {
        Self::Shared(Rc::new(rows))
    }
}

#[derive(Props)]
pub struct DataGridProps<T: Clone + 'static> {
    pub rows: GridRows<T>,
    pub columns: Vec<GridColumn<T>>,
    /// The grid's height, border included. Only the rows that fit are built.
    #[prop(default = Dimension::Length(12.0))]
    pub height: Dimension,
    #[prop(optional)]
    pub id: Option<FocusId>,
    /// Shown in the top border.
    #[prop(optional)]
    pub title: Option<String>,
    /// What the grid holds, for assistive tech. Defaults to the title.
    #[prop(optional)]
    pub label: Option<String>,
    #[prop(default = BorderStyle::Rounded)]
    pub border: BorderStyle,
    #[prop(default = Color::DarkGrey)]
    pub border_color: Color,
    #[prop(default = TableRules::Columns)]
    pub rules: TableRules,
    #[prop(default = BorderStyle::Single)]
    pub rule_style: BorderStyle,
    /// One row selected at a time, or any number.
    #[prop(default = SelectionMode::Multiple)]
    pub selection_mode: SelectionMode,
    /// The sort. Pass it to control sorting yourself; clicking a header then
    /// only calls `on_sort_change`. Use `bind:sort` to share it.
    #[prop(bindable)]
    pub sort: Bindable<Option<SortState>>,
    #[prop(optional)]
    pub default_sort: Option<SortState>,
    #[prop(optional)]
    pub on_sort_change: Option<Rc<dyn Fn(Option<SortState>)>>,
    /// The selected rows, by their index in the data, so a selection stays
    /// with its rows when the grid is sorted. Use `bind:selected` to share it.
    #[prop(bindable)]
    pub selected: Bindable<HashSet<usize>>,
    #[prop(optional)]
    pub on_selected_change: Option<Rc<dyn Fn(HashSet<usize>)>>,
    /// Called with the active cell when Enter is pressed on a row.
    #[prop(optional)]
    pub on_activate: Option<Rc<dyn Fn(CellRef)>>,
}
