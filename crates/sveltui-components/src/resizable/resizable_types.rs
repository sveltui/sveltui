use std::{cell::RefCell, rc::Rc};
use sveltui::prelude::*;

use super::resizable_layout::Constraints;

#[derive(Props)]
pub struct PanelGroupProps {
    #[prop(optional)]
    pub id: Option<FocusId>,
    /// Panels sit side by side when horizontal, stacked when vertical.
    #[prop(default = Orientation::Horizontal)]
    pub direction: Orientation,
    /// Panel sizes in percent, in order. Pass it as `bind:layout={state}` to
    /// share it with the parent.
    #[prop(bindable)]
    pub layout: Bindable<Vec<f32>>,
    /// The sizes at first, when `layout` isn't passed. Panels fall back to
    /// their own `default_size`.
    #[prop(optional)]
    pub default_layout: Option<Vec<f32>>,
    #[prop(optional)]
    pub on_layout_change: Option<Rc<dyn Fn(Vec<f32>)>>,
    /// Percent moved by one arrow key press on a focused handle. Shift
    /// moves five times as far.
    #[prop(default = 5.0)]
    pub keyboard_step: f32,
    #[snippet(optional)]
    pub children: Option<Snippet>,
}

#[derive(Props)]
pub struct PanelProps {
    #[prop(optional)]
    pub id: Option<FocusId>,
    /// The panel's size in percent. Panels without one share what's left.
    #[prop(optional)]
    pub default_size: Option<f32>,
    #[prop(default = 0.0)]
    pub min_size: f32,
    #[prop(default = 100.0)]
    pub max_size: f32,
    /// Whether dragging the panel below half its minimum, or pressing Enter
    /// on its handle, collapses it to `collapsed_size`.
    #[prop(default = false)]
    pub collapsible: bool,
    #[prop(default = 0.0)]
    pub collapsed_size: f32,
    #[prop(optional)]
    pub on_collapse: Option<Rc<dyn Fn()>>,
    #[prop(optional)]
    pub on_expand: Option<Rc<dyn Fn()>>,
    #[snippet(optional)]
    pub children: Option<Snippet>,
}

#[derive(Props)]
pub struct HandleProps {
    #[prop(optional)]
    pub id: Option<FocusId>,
    #[prop(default = false)]
    pub disabled: bool,
    /// Draws a grip in the middle of the handle, like shadcn's `withHandle`.
    #[prop(default = false)]
    pub with_handle: bool,
    /// Names the handle for assistive tech.
    #[prop(optional)]
    pub label: Option<String>,
}

/// What a panel told the group about itself while rendering.
#[derive(Clone)]
pub(crate) struct PanelInfo {
    pub id: FocusId,
    pub default_size: Option<f32>,
    pub constraints: Constraints,
    pub on_collapse: Option<Rc<dyn Fn()>>,
    pub on_expand: Option<Rc<dyn Fn()>>,
}

#[derive(Clone)]
pub(crate) struct GroupContext {
    pub id: FocusId,
    pub direction: Orientation,
    pub step: f32,
    pub panels: Rc<RefCell<Vec<PanelInfo>>>,
    /// The sizes the last render settled on, always one per panel.
    pub current: Rc<RefCell<Vec<f32>>>,
    pub set_layout: Rc<dyn Fn(Vec<f32>)>,
    /// Sizes panels had before collapsing, to restore them with.
    pub restore: Rc<RefCell<Vec<Option<f32>>>>,
}

/// A drag in progress: where it started and the sizes it started from.
#[derive(Clone)]
pub(crate) struct Drag {
    pub origin: (u16, u16),
    pub sizes: Vec<f32>,
}
