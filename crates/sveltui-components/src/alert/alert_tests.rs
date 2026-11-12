use super::*;
use crate::AlertKind;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::cell::Cell;
use std::rc::Rc;
use sveltui::element::{HitMap, InteractionTree, render_node_interactive};
use sveltui::layout::Rect;
use sveltui::prelude::*;
use sveltui::terminal::Buffer;

fn render(node: &VNode, focus: &mut FocusManager) -> (InteractionTree, String) {
    let mut buffer = Buffer::new(40, 4);
    let mut hits = HitMap::new();
    focus.with_context(|| {
        render_node_interactive(node, Rect::new(0, 0, 40, 4), &mut buffer, &mut hits)
    });
    focus.reconcile(hits.interactions.clone());
    let screen = (0..4)
        .map(|y| {
            (0..40)
                .map(|x| buffer.get(x, y).unwrap().ch)
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n");
    (hits.interactions, screen)
}

fn alert_semantics(tree: &InteractionTree) -> Semantics {
    tree.nodes()
        .iter()
        .map(|n| n.interaction.semantics.clone())
        .find(|s| matches!(s.role, Role::Alert | Role::Status))
        .expect("an alert")
}

#[test]
fn an_alert_announces_at_once_and_whole() {
    let node = view! {
        <Alert kind=AlertKind::Warning title="Disk almost full">"1 GB left."</Alert>
    }
    .into_node();
    let mut focus = FocusManager::new();
    let (tree, screen) = render(&node, &mut focus);
    let semantics = alert_semantics(&tree);
    assert_eq!(semantics.role, Role::Alert);
    assert_eq!(semantics.label, "Disk almost full");
    assert_eq!(semantics.live, Some(Live::Assertive));
    assert_eq!(semantics.atomic, Some(true));
    assert!(screen.contains("! Disk almost full"), "{screen}");
    assert!(screen.contains("1 GB left."), "{screen}");
    assert!(
        focus.tab_order().is_empty(),
        "an alert without a dismiss button takes no focus"
    );
}

#[test]
fn a_polite_alert_is_a_status() {
    let node = view! { <Alert polite=true title="Synced" /> }.into_node();
    let (tree, _) = render(&node, &mut FocusManager::new());
    let semantics = alert_semantics(&tree);
    assert_eq!(semantics.role, Role::Status);
    assert_eq!(semantics.live, Some(Live::Polite));
}

#[test]
fn the_dismiss_button_is_a_tab_stop() {
    let dismissed = Rc::new(Cell::new(false));
    let flag = dismissed.clone();
    let node = view! {
        <Alert title="Update ready" on_dismiss={move || flag.set(true)} />
    }
    .into_node();
    let mut focus = FocusManager::new();
    let (_, screen) = render(&node, &mut focus);
    assert!(screen.contains("[×]"), "{screen}");
    assert_eq!(focus.tab_order().len(), 1);

    focus.dispatch(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE));
    focus.dispatch(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
    assert!(dismissed.get());
}
