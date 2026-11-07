use crossterm::style::Color;
use std::{cell::RefCell, collections::HashMap, rc::Rc};
use sveltui::prelude::*;

use super::tabs_types::*;

/// Owns the selection and provides it to `List`, `Trigger` and `Content`.
/// Place `List` before the `Content` parts so a default selection resolves
/// before panels render.
#[component]
pub fn Root(props: RootProps) -> VNode {
    let local = state(props.default_value.clone());
    let ids = use_hook(|| Rc::new(RefCell::new(HashMap::new())));
    let resolved = use_hook(|| Rc::new(RefCell::new(None)));
    *resolved.borrow_mut() = props.value.clone().or_else(|| local.get());

    let controlled = props.value.is_some();
    let on_value_change = props.on_value_change.clone();
    let current = resolved.clone();
    let select: Rc<dyn Fn(String)> = Rc::new(move |next: String| {
        if current.borrow().as_deref() == Some(next.as_str()) {
            return;
        }
        if !controlled {
            local.set(Some(next.clone()));
        }
        if let Some(callback) = &on_value_change {
            callback(next);
        }
    });

    let context = TabsContext {
        resolved,
        ids,
        select,
        orientation: props.orientation,
        activation: props.activation_mode,
        loop_focus: props.loop_focus,
    };
    let children = provide_context(context, || props.children.render(()));
    match props.id {
        Some(id) => PressableElement::new()
            .focus_id(id)
            .child(children)
            .into_node(),
        None => children,
    }
}

/// The row (or column) of tabs. It is a single tab stop; arrow keys move
/// between its triggers.
#[component]
pub fn List(props: ListProps) -> VNode {
    let tabs = use_context::<TabsContext>().expect("Tabs::List requires Tabs::Root");
    let group = FocusGroup {
        orientation: tabs.orientation,
        wrap: tabs.loop_focus,
        typeahead: false,
        disabled_focusable: false,
    };
    let direction = match tabs.orientation {
        Orientation::Horizontal => FlexDirection::Row,
        Orientation::Vertical => FlexDirection::Column,
    };
    let mut list = PressableElement::new()
        .focus_group(group)
        .semantics(Semantics::new(Role::TabList, ""));
    if let Some(id) = props.id {
        list = list.focus_id(id);
    }
    list.child(
        ViewElement::new()
            .flex_direction(direction)
            .gap(1)
            .child(props.children.render(())),
    )
    .into_node()
}

#[component]
pub fn Trigger(props: TriggerProps) -> VNode {
    let tabs = use_context::<TabsContext>().expect("Tabs::Trigger requires Tabs::Root");
    let (default_id, panel_id) = tabs.ids_for(&props.value);
    let id = props.id.unwrap_or(default_id);
    if props.id.is_some() {
        tabs.ids
            .borrow_mut()
            .insert(props.value.clone(), (id, panel_id));
    }

    // With no explicit selection, the first enabled tab is selected.
    if !props.disabled {
        let mut resolved = tabs.resolved.borrow_mut();
        if resolved.is_none() {
            *resolved = Some(props.value.clone());
        }
    }
    let selected = tabs.is_selected(&props.value);
    let focused = !props.disabled && use_focus(id).is_focused();

    let select = tabs.select.clone();
    let value = props.value.clone();
    let on_click: Option<Rc<dyn Fn()>> =
        (!props.disabled).then(|| -> Rc<dyn Fn()> { Rc::new(move || select(value.clone())) });
    let on_focus = on_click
        .clone()
        .filter(|_| tabs.activation == TabsActivation::Automatic);

    let (fg, bg) = if props.disabled {
        (Color::DarkGrey, None)
    } else if focused {
        (Color::Black, Some(Color::White))
    } else if selected {
        (Color::White, None)
    } else {
        (Color::DarkGrey, None)
    };
    let label = props.children.render(());
    let semantics = Semantics::new(Role::Tab, "")
        .selected(selected)
        .controls(panel_id);

    view! {
        <Pressable
            focus_id=id
            tab_stop=true
            disabled=props.disabled
            activation=Activation::Press
            semantics=semantics
            on_click=on_click
            on_focus=on_focus
        >
            <View flex_direction=FlexDirection::Row background_color=bg>
                <TextSpan>" "</TextSpan>
                <View foreground_color=fg>
                    {styled(label, fg, selected)}
                </View>
                <TextSpan>" "</TextSpan>
            </View>
        </Pressable>
    }
}

/// The panel for one tab. Only the selected panel is rendered.
#[component]
pub fn Content(props: ContentProps) -> VNode {
    let tabs = use_context::<TabsContext>().expect("Tabs::Content requires Tabs::Root");
    if !tabs.is_selected(&props.value) {
        return VNode::Empty;
    }
    // Panel ids are owned by Tabs so the tab's `controls` always matches.
    let (tab_id, id) = tabs.ids_for(&props.value);
    let semantics = Semantics::new(Role::TabPanel, "").labelled_by(tab_id);
    let children = props.children.render(());
    // APG: the panel is itself a tab stop only when it holds nothing
    // focusable, so Tab from the tab reaches the content in one step.
    let tab_stop = !has_tab_stop(&children);
    view! {
        <Pressable focus_id=id tab_stop=tab_stop semantics=semantics>
            <View>{children}</View>
        </Pressable>
    }
}

fn has_tab_stop(node: &VNode) -> bool {
    match node {
        VNode::Pressable(p) => p.tab_stop || p.children.iter().any(has_tab_stop),
        VNode::View(v) => v.children.iter().any(has_tab_stop),
        VNode::Fragment(items) => items.iter().any(has_tab_stop),
        VNode::Overlay(o) => o.trigger.as_deref().is_some_and(has_tab_stop),
        _ => false,
    }
}

/// Applies the tab's text color, and bold + underline when selected, to
/// plain text children that don't set their own color.
fn styled(mut node: VNode, fg: Color, selected: bool) -> VNode {
    fn walk(node: &mut VNode, fg: Color, selected: bool) {
        match node {
            VNode::Text(text) => {
                if text.fg.is_none() {
                    text.fg = Some(fg);
                }
                text.bold |= selected;
                text.underline |= selected;
            }
            VNode::View(view) => view.children.iter_mut().for_each(|c| walk(c, fg, selected)),
            VNode::Fragment(items) => items.iter_mut().for_each(|c| walk(c, fg, selected)),
            _ => {}
        }
    }
    walk(&mut node, fg, selected);
    node
}
