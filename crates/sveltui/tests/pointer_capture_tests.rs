use std::{cell::RefCell, rc::Rc};
use sveltui::{
    Buffer,
    element::{PointerEvent, PointerPhase, render_node_interactive, reset_focus_counter},
    layout::Rect,
    prelude::*,
};

fn render(manager: &mut FocusManager, build: &impl Fn() -> VNode) -> HitMap {
    for _ in 0..8 {
        reset_focus_counter();
        begin_frame();
        let node = manager.with_context(build);
        end_frame();
        let mut hits = HitMap::new();
        let mut buffer = Buffer::new(40, 10);
        manager.with_context(|| {
            render_node_interactive(&node, Rect::new(0, 0, 40, 10), &mut buffer, &mut hits)
        });
        if !manager.reconcile(hits.interactions.clone()) {
            return hits;
        }
    }
    panic!("focus did not stabilize");
}

fn handle(log: &Rc<RefCell<Vec<PointerEvent>>>) -> impl Fn() -> VNode {
    let log = log.clone();
    move || {
        let log = log.clone();
        PressableElement::new()
            .focus_id(FocusId(1))
            .tab_stop(true)
            .on_pointer(move |event| log.borrow_mut().push(event))
            .child(
                ViewElement::new()
                    .width(Dimension::Length(3.0))
                    .height(Dimension::Length(1.0)),
            )
            .into_node()
    }
}

#[test]
fn press_captures_so_drag_and_release_follow_outside_the_target() {
    let log = Rc::new(RefCell::new(Vec::new()));
    let build = handle(&log);
    let mut manager = FocusManager::new();
    let hits = render(&mut manager, &build);

    manager.pointer_down_at(&hits, 1, 0);
    assert!(manager.pointer_captured());
    manager.pointer_drag(&hits, 30, 8);
    manager.pointer_up(&hits, 35, 9);
    assert!(!manager.pointer_captured());

    let phases: Vec<_> = log.borrow().iter().map(|e| (e.phase, e.x, e.y)).collect();
    assert_eq!(
        phases,
        [
            (PointerPhase::Down, 1, 0),
            (PointerPhase::Drag, 30, 8),
            (PointerPhase::Up, 35, 9),
        ]
    );
    assert_eq!(manager.focused(), Some(FocusId(1)));
}

#[test]
fn drag_without_a_capture_falls_through() {
    let log = Rc::new(RefCell::new(Vec::new()));
    let build = handle(&log);
    let mut manager = FocusManager::new();
    let hits = render(&mut manager, &build);
    assert!(manager.pointer_drag(&hits, 1, 0).is_none());
    assert!(manager.pointer_up(&hits, 1, 0).is_none());
    assert!(log.borrow().is_empty());
}

#[test]
fn disabled_targets_do_not_capture() {
    let log = Rc::new(RefCell::new(Vec::new()));
    let build = {
        let log = log.clone();
        move || {
            let log = log.clone();
            PressableElement::new()
                .focus_id(FocusId(1))
                .disabled(true)
                .on_pointer(move |event| log.borrow_mut().push(event))
                .child(
                    ViewElement::new()
                        .width(Dimension::Length(3.0))
                        .height(Dimension::Length(1.0)),
                )
                .into_node()
        }
    };
    let mut manager = FocusManager::new();
    let hits = render(&mut manager, &build);
    manager.pointer_down_at(&hits, 1, 0);
    assert!(!manager.pointer_captured());
    assert!(log.borrow().is_empty());
}

#[test]
fn use_layout_reports_the_last_frames_rect() {
    let log = Rc::new(RefCell::new(Vec::new()));
    let build = handle(&log);
    let mut manager = FocusManager::new();
    assert_eq!(manager.with_context(|| use_layout(FocusId(1))), None);
    render(&mut manager, &build);
    assert_eq!(
        manager.with_context(|| use_layout(FocusId(1))),
        Some(Rect::new(0, 0, 3, 1))
    );
}

#[test]
fn pointer_shape_follows_the_target_and_the_capture() {
    let build = || {
        PressableElement::new()
            .focus_id(FocusId(1))
            .tab_stop(true)
            .pointer(PointerShape::ColResize)
            .on_pointer(|_| {})
            .child(
                ViewElement::new()
                    .width(Dimension::Length(3.0))
                    .height(Dimension::Length(1.0)),
            )
            .into_node()
    };
    let mut manager = FocusManager::new();
    let hits = render(&mut manager, &build);
    assert_eq!(manager.pointer_shape(&hits, 1, 0), PointerShape::ColResize);
    assert_eq!(manager.pointer_shape(&hits, 20, 5), PointerShape::Default);
    manager.pointer_down_at(&hits, 1, 0);
    assert_eq!(manager.pointer_shape(&hits, 20, 5), PointerShape::ColResize);
    manager.pointer_up(&hits, 20, 5);
    assert_eq!(manager.pointer_shape(&hits, 20, 5), PointerShape::Default);
}

#[test]
fn disabled_targets_show_not_allowed() {
    let build = || {
        PressableElement::new()
            .focus_id(FocusId(1))
            .tab_stop(true)
            .disabled(true)
            .child(
                ViewElement::new()
                    .width(Dimension::Length(3.0))
                    .height(Dimension::Length(1.0)),
            )
            .into_node()
    };
    let mut manager = FocusManager::new();
    let hits = render(&mut manager, &build);
    assert_eq!(manager.pointer_shape(&hits, 1, 0), PointerShape::NotAllowed);
}
