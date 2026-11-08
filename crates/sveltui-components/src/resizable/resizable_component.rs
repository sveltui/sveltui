use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::{cell::RefCell, collections::HashMap, rc::Rc};
use sveltui::element::{PointerEvent, PointerPhase};
use sveltui::prelude::*;

use super::resizable_layout::{Constraints, initial_layout, normalize, resize, toggle_collapse};
use super::resizable_types::*;
use crate::controllable::use_controllable;

fn identity(explicit: Option<FocusId>) -> FocusId {
    let stable = state_with(FocusId::new);
    explicit.unwrap_or_else(|| stable.get())
}

/// Gives each panel the weight its size works out to. Panels learn their
/// size only once the whole group has rendered, since a panel without a
/// stated size depends on all the others.
fn weigh(node: &mut VNode, weights: &HashMap<FocusId, f32>) {
    match node {
        VNode::View(v) => {
            if let Some(weight) = v
                .pressables
                .iter()
                .find_map(|p| p.focus_id.and_then(|id| weights.get(&id)))
            {
                let weighed = std::mem::take(&mut **v).flex_grow(*weight);
                **v = weighed;
            }
            v.children.iter_mut().for_each(|c| weigh(c, weights));
        }
        VNode::Fragment(items) => items.iter_mut().for_each(|c| weigh(c, weights)),
        _ => {}
    }
}

fn constraints_of(group: &GroupContext) -> Vec<Constraints> {
    group
        .panels
        .borrow()
        .iter()
        .map(|p| p.constraints)
        .collect()
}

/// Stores `next` as the layout, telling panels that collapsed or expanded
/// by it and remembering the size a collapsing panel had.
fn commit(group: &GroupContext, next: Vec<f32>) {
    let before = group.current.borrow().clone();
    if before == next {
        return;
    }
    let panels = group.panels.borrow();
    let mut restore = group.restore.borrow_mut();
    restore.resize(panels.len(), None);
    let mut callbacks = Vec::new();
    for (i, panel) in panels.iter().enumerate() {
        let (was, is) = (before.get(i).copied(), next.get(i).copied());
        let (Some(was), Some(is)) = (was, is) else {
            continue;
        };
        let c = panel.constraints;
        if !c.is_collapsed(was) && c.is_collapsed(is) {
            restore[i] = Some(was);
            callbacks.extend(panel.on_collapse.clone());
        } else if c.is_collapsed(was) && !c.is_collapsed(is) {
            callbacks.extend(panel.on_expand.clone());
        }
    }
    drop(panels);
    drop(restore);
    (group.set_layout)(next);
    for callback in callbacks {
        callback();
    }
}

/// Lays panels out in a row or column, with `Handle`s between them for
/// dragging or keyboard resizing. Sizes are percentages and always total
/// 100.
#[component]
pub fn PanelGroup(props: PanelGroupProps) -> VNode {
    let layout = use_controllable(
        props.layout.bound(),
        props.layout.value(),
        || props.default_layout.clone().unwrap_or_default(),
        props.on_layout_change.clone(),
    );
    let id = identity(props.id);
    let panels = Rc::new(RefCell::new(Vec::new()));
    let current = use_hook(|| Rc::new(RefCell::new(Vec::new())));
    let restore = use_hook(|| Rc::new(RefCell::new(Vec::new())));
    let setter = layout.clone();
    let context = GroupContext {
        id,
        direction: props.direction,
        step: props.keyboard_step,
        panels: panels.clone(),
        current: current.clone(),
        set_layout: Rc::new(move |next| setter.set(next)),
        restore,
    };
    let mut children = provide_context(context, || props.children.render(()));

    let infos = panels.borrow().clone();
    let constraints: Vec<_> = infos.iter().map(|p| p.constraints).collect();
    let stored = layout.get();
    let sizes = if stored.len() == infos.len() && !stored.is_empty() {
        normalize(&stored, &constraints)
    } else {
        let defaults: Vec<_> = infos.iter().map(|p| p.default_size).collect();
        initial_layout(&defaults, &constraints)
    };
    let weights = infos
        .iter()
        .map(|p| p.id)
        .zip(sizes.iter().copied())
        .collect();
    weigh(&mut children, &weights);
    *current.borrow_mut() = sizes;

    let direction = match props.direction {
        Orientation::Horizontal => FlexDirection::Row,
        Orientation::Vertical => FlexDirection::Column,
    };
    PressableElement::new()
        .focus_id(id)
        .child(
            ViewElement::new()
                .flex_direction(direction)
                .flex_grow(1.0)
                .width(Dimension::Percent(1.0))
                .height(Dimension::Percent(1.0))
                .align_items(AlignItems::Stretch)
                .child(children),
        )
        .into_node()
}

/// One resizable region of a `PanelGroup`.
#[component]
pub fn Panel(props: PanelProps) -> VNode {
    let group = use_context::<GroupContext>().expect("Resizable::Panel requires a PanelGroup");
    let id = identity(props.id);
    let collapsed = props.collapsed_size.clamp(0.0, props.min_size);
    group.panels.borrow_mut().push(PanelInfo {
        id,
        default_size: props.default_size,
        constraints: Constraints {
            min: props.min_size,
            max: props.max_size.max(props.min_size),
            collapsible: props.collapsible,
            collapsed,
        },
        on_collapse: props.on_collapse.clone(),
        on_expand: props.on_expand.clone(),
    });
    // The group sets the real weight once every panel has registered.
    PressableElement::new()
        .focus_id(id)
        .child(
            ViewElement::new()
                .flex_grow(props.default_size.unwrap_or(1.0))
                .flex_basis(Dimension::Length(0.0))
                .flex_shrink(1.0)
                .overflow(Overflow::Hidden)
                .child(props.children.render(())),
        )
        .into_node()
}

/// The draggable, focusable divider between two panels. It follows the
/// WAI-ARIA window splitter pattern: arrows resize, Shift takes bigger
/// steps, Home and End go to the smallest and largest, and Enter collapses
/// or restores a collapsible panel.
#[component]
pub fn Handle(props: HandleProps) -> VNode {
    let group = use_context::<GroupContext>().expect("Resizable::Handle requires a PanelGroup");
    let id = identity(props.id);
    let dragging = state(false);
    let drag = use_hook(|| Rc::new(RefCell::new(None::<Drag>)));
    // The panel before this handle is the one it resizes.
    let primary = group.panels.borrow().len().checked_sub(1);
    let horizontal = group.direction == Orientation::Horizontal;
    let active = !props.disabled && (use_focus(id).is_focused() || dragging.get());

    let value = primary.and_then(|i| {
        let panels = group.panels.borrow();
        let size = group.current.borrow().get(i).copied();
        size.or(panels[i].default_size)
    });
    let primary_constraints = primary.map(|i| group.panels.borrow()[i].constraints);
    let semantics = Semantics {
        value: value.map(f64::from),
        value_min: primary_constraints.map(|c| f64::from(c.min)),
        value_max: primary_constraints.map(|c| f64::from(c.max)),
        orientation: Some(if horizontal {
            Orientation::Vertical
        } else {
            Orientation::Horizontal
        }),
        controls: primary.map(|i| group.panels.borrow()[i].id),
        ..Semantics::new(Role::Separator, props.label.clone().unwrap_or_default())
    };

    let on_pointer = {
        let group = group.clone();
        let drag = drag.clone();
        move |event: PointerEvent| match event.phase {
            PointerPhase::Down => {
                *drag.borrow_mut() = Some(Drag {
                    origin: (event.x, event.y),
                    sizes: group.current.borrow().clone(),
                });
                dragging.set(true);
            }
            PointerPhase::Drag => {
                let (Some(start), Some(index), Some(area)) =
                    (drag.borrow().clone(), primary, use_layout(group.id))
                else {
                    return;
                };
                let length = if horizontal { area.width } else { area.height };
                let handles = group.panels.borrow().len().saturating_sub(1) as u16;
                let usable = length.saturating_sub(handles);
                if usable == 0 {
                    return;
                }
                let cells = if horizontal {
                    f32::from(event.x) - f32::from(start.origin.0)
                } else {
                    f32::from(event.y) - f32::from(start.origin.1)
                };
                let next = resize(
                    &start.sizes,
                    &constraints_of(&group),
                    index,
                    cells * 100.0 / f32::from(usable),
                );
                commit(&group, next);
            }
            PointerPhase::Up => {
                *drag.borrow_mut() = None;
                dragging.set(false);
            }
        }
    };

    let on_key = {
        let group = group.clone();
        move |key: KeyEvent, _: &mut EventContext| {
            let Some(index) = primary else {
                return EventResult::Ignored;
            };
            let (shrink, grow) = if horizontal {
                (KeyCode::Left, KeyCode::Right)
            } else {
                (KeyCode::Up, KeyCode::Down)
            };
            let sizes = group.current.borrow().clone();
            let constraints = constraints_of(&group);
            let step = if key.modifiers.contains(KeyModifiers::SHIFT) {
                group.step * 5.0
            } else {
                group.step
            };
            let next = match key.code {
                c if c == shrink => resize(&sizes, &constraints, index, -step),
                c if c == grow => resize(&sizes, &constraints, index, step),
                KeyCode::Home => resize(&sizes, &constraints, index, -100.0),
                KeyCode::End => resize(&sizes, &constraints, index, 100.0),
                KeyCode::Enter => {
                    let restore = group.restore.borrow().get(index).copied().flatten();
                    match toggle_collapse(&sizes, &constraints, index, restore) {
                        Some((next, _, _)) => next,
                        None => return EventResult::Handled,
                    }
                }
                _ => return EventResult::Ignored,
            };
            commit(&group, next);
            EventResult::Handled
        }
    };

    let color = if props.disabled {
        Color::DarkGrey
    } else if active {
        Color::White
    } else {
        Color::DarkGrey
    };
    let glyph = match (horizontal, active, props.with_handle) {
        (true, true, _) => "║",
        (true, false, true) => "┃",
        (true, false, false) => "│",
        (false, true, _) => "═",
        (false, false, true) => "━",
        (false, false, false) => "─",
    };
    // The line is drawn from an absolutely positioned child, so however
    // long it is it adds nothing to the handle's size: the handle takes the
    // group's width or height and clips the line to it.
    let rule = |text: String| {
        ViewElement::new()
            .position(Position::Absolute)
            .inset(Inset::new().top(0).left(0))
            .child(TextElement::new(text).fg(color).bold(active))
    };
    let line = if horizontal {
        // Longer than any screen.
        ViewElement::new()
            .width(Dimension::Length(1.0))
            .flex_shrink(0.0)
            .overflow(Overflow::Hidden)
            .child(rule(vec![glyph; 512].join("\n")))
    } else {
        ViewElement::new()
            .height(Dimension::Length(1.0))
            .flex_shrink(0.0)
            .overflow(Overflow::Hidden)
            .child(rule(glyph.repeat(512)))
    };

    // The resize shape stays through a drag; a grabbing hand would hide
    // which way the handle moves.
    let shape = if horizontal {
        PointerShape::ColResize
    } else {
        PointerShape::RowResize
    };
    PressableElement::new()
        .focus_id(id)
        .tab_stop(true)
        .disabled(props.disabled)
        .semantics(semantics)
        .pointer(shape)
        .on_pointer(on_pointer)
        .on_key(on_key)
        .child(line)
        .into_node()
}
