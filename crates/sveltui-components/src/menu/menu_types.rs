use std::{
    cell::{Cell, RefCell},
    collections::HashSet,
    rc::Rc,
};
use sveltui::prelude::*;

#[derive(Clone)]
pub(super) struct MenuContext {
    pub(super) id: FocusId,
    pub(super) trigger: Rc<Cell<FocusId>>,
    pub(super) triggered: Rc<Cell<bool>>,
    pub(super) open: bool,
    pub(super) change: Rc<dyn Fn(bool)>,
    pub(super) kind: MenuTriggerKind,
}

#[derive(Clone)]
pub(super) struct BarContext {
    pub(super) values: Rc<RefCell<HashSet<String>>>,
    pub(super) value: Option<String>,
    pub(super) change: Rc<dyn Fn(Option<String>)>,
}

#[derive(Props)]
pub struct RootProps {
    #[prop(optional)]
    pub id: Option<FocusId>,
    #[prop(optional)]
    pub open: Option<bool>,
    #[prop(default = false)]
    pub default_open: bool,
    #[prop(optional)]
    pub on_open_change: Option<Rc<dyn Fn(bool)>>,
    #[snippet(optional)]
    pub children: Option<Snippet>,
}

pub type SubProps = RootProps;

#[derive(Props)]
pub struct BarRootProps {
    #[prop(optional)]
    pub id: Option<FocusId>,
    #[prop(optional)]
    pub value: Option<Option<String>>,
    #[prop(optional)]
    pub default_value: Option<String>,
    #[prop(optional)]
    pub on_value_change: Option<Rc<dyn Fn(Option<String>)>>,
    #[snippet(optional)]
    pub children: Option<Snippet>,
}

#[derive(Props)]
pub struct BarMenuProps {
    pub value: String,
    #[prop(optional)]
    pub id: Option<FocusId>,
    #[snippet(optional)]
    pub children: Option<Snippet>,
}

#[derive(Props)]
pub struct TriggerProps {
    #[prop(optional)]
    pub id: Option<FocusId>,
    #[prop(default = false)]
    pub disabled: bool,
    #[prop(optional)]
    pub focus_bg: Option<Color>,
    #[prop(optional)]
    pub focus_fg: Option<Color>,
    #[snippet(optional)]
    pub children: Option<Snippet>,
}

pub type SubTriggerProps = TriggerProps;

#[derive(Props)]
pub struct ContentProps {
    #[prop(optional)]
    pub anchor: Option<FocusId>,
    #[prop(optional)]
    pub placement: Option<Placement>,
    #[prop(default = Dimension::Length(24.0))]
    pub width: Dimension,
    #[prop(default = BorderStyle::Single)]
    pub border: BorderStyle,
    #[prop(optional)]
    pub border_color: Option<Color>,
    #[prop(default = Color::Black)]
    pub bg: Color,
    #[prop(default = Padding::zero())]
    pub padding: Padding,
    #[prop(default = true)]
    pub loop_focus: bool,
    #[prop(optional)]
    pub on_key: Option<KeyHandler>,
    #[snippet(optional)]
    pub children: Option<Snippet>,
}

pub type SubContentProps = ContentProps;

#[derive(Props)]
pub struct ItemProps {
    #[prop(optional)]
    pub id: Option<FocusId>,
    #[prop(default = false)]
    pub disabled: bool,
    #[prop(optional)]
    pub on_select: Option<Rc<dyn Fn()>>,
    #[prop(optional)]
    pub focus_bg: Option<Color>,
    #[prop(optional)]
    pub focus_fg: Option<Color>,
    #[snippet(optional)]
    pub children: Option<Snippet>,
}

#[derive(Props)]
pub struct GroupProps {
    #[snippet(optional)]
    pub children: Option<Snippet>,
}

pub type GroupHeadingProps = GroupProps;

#[derive(Props)]
pub struct SeparatorProps {}
