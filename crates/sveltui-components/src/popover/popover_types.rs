use std::rc::Rc;
use sveltui::prelude::*;

#[derive(Props)]
pub struct PopoverProps {
    #[prop(optional)]
    pub id: Option<FocusId>,
    /// Whether the popover is open. Pass it to control that yourself.
    /// Without `open` or `bind_open`, the popover manages itself.
    /// Whether the popover is open, shared with the parent and updated when
    /// it opens or closes. Pass it as `bind:open={state}`.
    #[prop(bindable)]
    pub open: Bindable<bool>,
    #[prop(default = Placement::Bottom)]
    pub placement: Placement,
    #[prop(default = false)]
    pub backdrop: bool,
    #[prop(optional)]
    pub on_close: Option<Rc<dyn Fn()>>,
    #[prop(optional)]
    pub on_open: Option<Rc<dyn Fn()>>,
    #[prop(optional)]
    pub on_key: Option<KeyHandler>,
    #[snippet(optional)]
    pub trigger: Option<Snippet>,
    #[snippet(optional)]
    pub children: Option<Snippet>,
}
