//! A tree written as markup: `Root` around `Branch` and `Item` tags. The
//! parts only describe the tree; `Root` hands what they describe to the
//! same `TreeView` that draws data-driven trees, so the two behave alike.

use crossterm::style::Color;
use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;
use sveltui::prelude::*;

use super::tree_view_component::TreeView;
use super::tree_view_types::*;
use crate::data_grid::GridRows;

/// One item the parts described.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PartNode {
    pub(crate) key: String,
    pub(crate) label: String,
    pub(crate) branch: bool,
    pub(crate) children: Vec<PartNode>,
}

/// Where the parts inside a `Root` or `Branch` add themselves.
#[derive(Clone)]
struct Collector(Rc<RefCell<Vec<PartNode>>>);

fn collect(part: &str, node: PartNode) {
    use_context::<Collector>()
        .unwrap_or_else(|| panic!("TreeView::{part} requires TreeView::Root"))
        .0
        .borrow_mut()
        .push(node);
}

#[derive(Props)]
pub struct RootProps {
    #[prop(default = Dimension::Length(12.0))]
    pub height: Dimension,
    #[prop(default = Dimension::Percent(1.0))]
    pub width: Dimension,
    #[prop(optional)]
    pub id: Option<FocusId>,
    #[prop(optional)]
    pub title: Option<String>,
    #[prop(optional)]
    pub label: Option<String>,
    #[prop(default = BorderStyle::Rounded)]
    pub border: BorderStyle,
    #[prop(default = Color::DarkGrey)]
    pub border_color: Color,
    #[prop(bindable)]
    pub expanded: Bindable<HashSet<String>>,
    #[prop(optional)]
    pub default_expanded: Option<HashSet<String>>,
    #[prop(optional)]
    pub on_expanded_change: Option<Rc<dyn Fn(HashSet<String>)>>,
    #[prop(bindable)]
    pub selected: Bindable<HashSet<String>>,
    #[prop(optional)]
    pub on_selected_change: Option<Rc<dyn Fn(HashSet<String>)>>,
    #[prop(default = SelectionMode::Single)]
    pub selection_mode: SelectionMode,
    #[prop(optional)]
    pub on_activate: Option<Rc<dyn Fn(String)>>,
    #[snippet(optional)]
    pub children: Option<Snippet>,
}

#[derive(Props)]
pub struct BranchProps {
    /// Names the branch in `expanded`, `selected` and `on_activate`.
    pub value: String,
    pub label: String,
    #[snippet(optional)]
    pub children: Option<Snippet>,
}

#[derive(Props)]
pub struct ItemProps {
    /// Names the item in `selected` and `on_activate`.
    pub value: String,
    pub label: String,
}

/// A tree described by the `Branch` and `Item` parts inside it.
#[component]
pub fn Root(props: RootProps) -> VNode {
    let collected = Rc::new(RefCell::new(Vec::new()));
    // The parts only register themselves; their markup isn't shown.
    let _ = provide_context(Collector(collected.clone()), || props.children.render(()));
    let nodes = collected.take();

    // The same tree keeps the same roots, so the tree's flattening cache
    // holds between frames.
    let roots = use_hook(|| Rc::new(RefCell::new(Rc::new(Vec::<PartNode>::new()))));
    if *roots.borrow().as_ref() != nodes {
        *roots.borrow_mut() = Rc::new(nodes);
    }
    let roots = roots.borrow().clone();

    let model = TreeModel::new(
        |n: &PartNode| n.key.clone(),
        |n: &PartNode| n.label.clone(),
        |n: &PartNode| n.children.clone(),
    )
    .is_branch(|n: &PartNode| n.branch);
    TreeView(TreeViewProps {
        roots: GridRows::Shared(roots),
        model,
        height: props.height,
        width: props.width,
        id: props.id,
        title: props.title,
        label: props.label,
        border: props.border,
        border_color: props.border_color,
        expanded: props.expanded,
        default_expanded: props.default_expanded,
        on_expanded_change: props.on_expanded_change,
        selected: props.selected,
        on_selected_change: props.on_selected_change,
        selection_mode: props.selection_mode,
        on_activate: props.on_activate,
    })
    .into_node()
}

/// An item with items of its own inside it.
#[component]
pub fn Branch(props: BranchProps) -> VNode {
    let children = Rc::new(RefCell::new(Vec::new()));
    let _ = provide_context(Collector(children.clone()), || props.children.render(()));
    collect(
        "Branch",
        PartNode {
            key: props.value,
            label: props.label,
            branch: true,
            children: children.take(),
        },
    );
    VNode::Empty
}

/// An item with nothing inside it.
#[component]
pub fn Item(props: ItemProps) -> VNode {
    collect(
        "Item",
        PartNode {
            key: props.value,
            label: props.label,
            branch: false,
            children: Vec::new(),
        },
    );
    VNode::Empty
}
