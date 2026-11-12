use crossterm::style::Color;
use std::collections::HashSet;
use std::rc::Rc;
use sveltui::prelude::*;

use crate::data_grid::GridRows;

/// Where a tree's top-level items come from: items in state, so the tree
/// follows changes to them, or items that don't change.
pub type TreeRoots<T> = GridRows<T>;

/// What a custom row renderer gets to know about its item.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TreeItemState {
    pub key: String,
    /// How deep the item sits; top-level items are 0.
    pub depth: usize,
    pub branch: bool,
    pub expanded: bool,
    /// The item has the tree's cursor (and the tree has focus).
    pub active: bool,
    pub selected: bool,
}

/// Reads something out of an item.
type Read<T, R> = Rc<dyn Fn(&T) -> R>;
/// Draws an item's content.
type Render<T> = Rc<dyn Fn(&T, TreeItemState) -> VNode>;

/// How to read a tree out of your items: a key that names each one, the
/// label it shows, and its children. Children are only asked for when a
/// branch is open, so a tree can load lazily (a directory read on expand).
pub struct TreeModel<T> {
    pub(crate) key: Read<T, String>,
    pub(crate) label: Read<T, String>,
    pub(crate) children: Read<T, Vec<T>>,
    pub(crate) is_branch: Option<Read<T, bool>>,
    pub(crate) render: Option<Render<T>>,
}

impl<T> Clone for TreeModel<T> {
    fn clone(&self) -> Self {
        Self {
            key: self.key.clone(),
            label: self.label.clone(),
            children: self.children.clone(),
            is_branch: self.is_branch.clone(),
            render: self.render.clone(),
        }
    }
}

impl<T: 'static> TreeModel<T> {
    /// `key` must be unique across the whole tree. `label` is what's shown,
    /// what assistive tech reads, and what typing jumps to.
    pub fn new(
        key: impl Fn(&T) -> String + 'static,
        label: impl Fn(&T) -> String + 'static,
        children: impl Fn(&T) -> Vec<T> + 'static,
    ) -> Self {
        Self {
            key: Rc::new(key),
            label: Rc::new(label),
            children: Rc::new(children),
            is_branch: None,
            render: None,
        }
    }

    /// Says which items are branches without asking for their children.
    /// Without it, a closed item is a branch when it has children, so
    /// `children` runs for every item on screen; pass this for lazy trees.
    pub fn is_branch(mut self, is_branch: impl Fn(&T) -> bool + 'static) -> Self {
        self.is_branch = Some(Rc::new(is_branch));
        self
    }

    /// Draws an item's content instead of its label. The label is still
    /// what assistive tech reads and what typing jumps to.
    pub fn render(mut self, render: impl Fn(&T, TreeItemState) -> VNode + 'static) -> Self {
        self.render = Some(Rc::new(render));
        self
    }
}

#[derive(Props)]
pub struct TreeViewProps<T: Clone + 'static> {
    pub roots: TreeRoots<T>,
    pub model: TreeModel<T>,
    /// The tree's height, border included. Only the rows that fit are built.
    #[prop(default = Dimension::Length(12.0))]
    pub height: Dimension,
    /// Fills its parent unless set.
    #[prop(default = Dimension::Percent(1.0))]
    pub width: Dimension,
    #[prop(optional)]
    pub id: Option<FocusId>,
    /// Shown in the top border.
    #[prop(optional)]
    pub title: Option<String>,
    /// What the tree holds, for assistive tech. Defaults to the title.
    #[prop(optional)]
    pub label: Option<String>,
    #[prop(default = BorderStyle::Rounded)]
    pub border: BorderStyle,
    #[prop(default = Color::DarkGrey)]
    pub border_color: Color,
    /// The keys of the open branches. Use `bind:expanded` to share it.
    #[prop(bindable)]
    pub expanded: Bindable<HashSet<String>>,
    #[prop(optional)]
    pub default_expanded: Option<HashSet<String>>,
    #[prop(optional)]
    pub on_expanded_change: Option<Rc<dyn Fn(HashSet<String>)>>,
    /// The keys of the selected items. Use `bind:selected` to share it.
    #[prop(bindable)]
    pub selected: Bindable<HashSet<String>>,
    #[prop(optional)]
    pub on_selected_change: Option<Rc<dyn Fn(HashSet<String>)>>,
    /// One item selected at a time, or any number.
    #[prop(default = SelectionMode::Single)]
    pub selection_mode: SelectionMode,
    /// Called with an item's key when Enter is pressed on it.
    #[prop(optional)]
    pub on_activate: Option<Rc<dyn Fn(String)>>,
}
