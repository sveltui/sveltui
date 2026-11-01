use super::*;
use crate::{
    Buffer,
    element::{FocusId, get_scroll},
    prelude::*,
};
use crossterm::event::MouseEvent;
use std::collections::VecDeque;

fn wheel(kind: MouseEventKind) -> Event {
    Event::Mouse(MouseEvent {
        kind,
        column: 2,
        row: 1,
        modifiers: KeyModifiers::NONE,
    })
}

#[test]
fn wheel_backlog_is_drained_before_the_next_render() {
    let mut pending = VecDeque::from(vec![wheel(MouseEventKind::ScrollDown); 1000]);
    pending.push_back(wheel(MouseEventKind::ScrollUp));
    let batch = read_event_batch(
        wheel(MouseEventKind::ScrollDown),
        || Ok(pending.pop_front()),
    )
    .unwrap();
    assert!(pending.is_empty());
    assert_eq!(batch.len(), 1002);

    let id = FocusId::new();
    let node = view! {
        <View width=10 height=3 overflow_y=Overflow::Scroll scroll_id=id>
            {#each (0..10) as i}<TextSpan>{i.to_string()}</TextSpan>{/each}
        </View>
    };
    let mut buffer = Buffer::new(10, 3);
    let mut hits = HitMap::new();
    render_node_interactive(&node, Rect::new(0, 0, 10, 3), &mut buffer, &mut hits);
    let mut manager = FocusManager::new();
    manager.reconcile(hits.interactions.clone());
    for event in batch {
        let Event::Mouse(mouse) = event else {
            panic!("expected wheel event")
        };
        let dy = if mouse.kind == MouseEventKind::ScrollUp {
            -1
        } else {
            1
        };
        manager.scroll(hits.scroll_target(mouse.column, mouse.row).unwrap(), 0, dy);
    }
    // Down events clamp at seven; the final Up takes effect in this same frame.
    assert_eq!(get_scroll(id), (0, 6));
}

#[test]
fn wheel_batch_stops_at_non_scroll_events_without_reordering() {
    for barrier in [
        Event::Key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE)),
        Event::Resize(40, 12),
        Event::Mouse(MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: 2,
            row: 1,
            modifiers: KeyModifiers::NONE,
        }),
    ] {
        let mut pending = VecDeque::from([barrier.clone(), wheel(MouseEventKind::ScrollUp)]);
        let batch = read_event_batch(
            wheel(MouseEventKind::ScrollDown),
            || Ok(pending.pop_front()),
        )
        .unwrap();
        assert_eq!(batch, vec![wheel(MouseEventKind::ScrollDown), barrier]);
        assert_eq!(pending.pop_front(), Some(wheel(MouseEventKind::ScrollUp)));
    }
}

#[test]
fn non_scroll_event_does_not_drain_the_queue() {
    let first = Event::Resize(40, 12);
    let batch = read_event_batch(first.clone(), || {
        panic!("must not poll after a non-scroll event")
    })
    .unwrap();
    assert_eq!(batch, vec![first]);
}

#[test]
fn continuous_wheel_input_yields_after_a_bounded_batch() {
    let mut reads = 0;
    let batch = read_event_batch(wheel(MouseEventKind::ScrollDown), || {
        reads += 1;
        Ok(Some(wheel(MouseEventKind::ScrollDown)))
    })
    .unwrap();
    assert!(batch.len() > 1);
    assert!(batch.len() <= MAX_SCROLL_BATCH);
    assert_eq!(reads + 1, batch.len());
}

#[test]
fn wheel_queue_read_errors_are_propagated() {
    let result = read_event_batch(wheel(MouseEventKind::ScrollDown), || {
        Err(io::Error::other("input failed"))
    });
    assert!(result.is_err());
}
