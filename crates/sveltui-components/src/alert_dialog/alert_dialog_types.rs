use std::cell::RefCell;
use std::rc::Rc;
use sveltui::prelude::*;

use crate::controllable::Controllable;

/// What a press outside an open alert dialog does. Because the dialog needs
/// an answer, by default it does nothing.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum OutsidePress {
    #[default]
    Ignore,
    /// Rings the terminal bell, and the dialog stays open.
    Bell,
    /// Cancels, as the Cancel button would.
    Close,
}

#[derive(Props)]
pub struct RootProps {
    /// Whether the dialog is open. Use `bind:open` to share it, or pass a
    /// plain value and follow `on_open_change`.
    #[prop(bindable)]
    pub open: Bindable<bool>,
    #[prop(default = false)]
    pub default_open: bool,
    #[prop(optional)]
    pub on_open_change: Option<Rc<dyn Fn(bool)>>,
    /// Whether Escape cancels the dialog.
    #[prop(default = true)]
    pub close_on_escape: bool,
    #[prop(default = OutsidePress::Ignore)]
    pub outside_press: OutsidePress,
    #[snippet(optional)]
    pub children: Option<Snippet>,
}

#[derive(Props)]
pub struct TriggerProps {
    #[prop(optional)]
    pub label: Option<String>,
    #[snippet(optional)]
    pub children: Option<Snippet>,
}

#[derive(Props)]
pub struct ContentProps {
    #[prop(default = Dimension::Length(50.0))]
    pub width: Dimension,
    #[prop(default = Placement::Center)]
    pub placement: Placement,
    #[prop(default = true)]
    pub backdrop: bool,
    #[snippet(optional)]
    pub children: Option<Snippet>,
}

#[derive(Props)]
pub struct TitleProps {
    #[snippet(optional)]
    pub children: Option<Snippet>,
}

#[derive(Props)]
pub struct DescriptionProps {
    #[snippet(optional)]
    pub children: Option<Snippet>,
}

#[derive(Props)]
pub struct CancelProps {
    /// Runs when the dialog is cancelled: this button, Escape, or a press
    /// outside with `OutsidePress::Close`.
    #[prop(optional)]
    pub on_click: Option<Rc<dyn Fn()>>,
    #[prop(optional)]
    pub label: Option<String>,
    #[snippet(optional)]
    pub children: Option<Snippet>,
}

#[derive(Props)]
pub struct ActionProps {
    /// Runs when the button is pressed, before the dialog closes.
    #[prop(optional)]
    pub on_click: Option<Rc<dyn Fn()>>,
    #[prop(optional)]
    pub label: Option<String>,
    #[snippet(optional)]
    pub children: Option<Snippet>,
}

/// The stable ids the parts point at each other with.
#[derive(Clone, Copy)]
pub(crate) struct Ids {
    pub(crate) trigger: FocusId,
    pub(crate) layer: FocusId,
    pub(crate) title: FocusId,
    pub(crate) description: FocusId,
    pub(crate) cancel: FocusId,
}

/// Cancel's handler for this render, if a Cancel rendered.
pub(crate) type CancelSlot = Rc<RefCell<Option<Rc<dyn Fn()>>>>;

/// What the root shares with its parts.
#[derive(Clone)]
pub(crate) struct AlertDialogContext {
    pub(crate) open: Controllable<bool>,
    pub(crate) ids: Ids,
    /// What cancelling does this render: Cancel's handler when there is a
    /// Cancel, otherwise just closing.
    pub(crate) cancel: CancelSlot,
    pub(crate) close_on_escape: bool,
    pub(crate) outside_press: OutsidePress,
}

impl AlertDialogContext {
    pub(crate) fn cancel(&self) {
        let handler = self.cancel.borrow().clone();
        match handler {
            Some(cancel) => cancel(),
            None => self.open.set(false),
        }
    }
}
