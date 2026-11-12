//! A one-line button for tight spots like a toast or an alert, where a
//! bordered `Button` would take three rows.

use crossterm::event::{KeyCode, KeyEvent};
use crossterm::style::Color;
use std::rc::Rc;
use sveltui::prelude::*;

/// What a press does. The keyboard path gets the event context, so it can
/// move focus (closing a toast sends focus back where it came from); a
/// click doesn't need to.
pub(crate) struct Press {
    pub(crate) by_key: Rc<dyn Fn(&mut EventContext)>,
    pub(crate) by_click: Rc<dyn Fn()>,
}

/// `[label]`, shown inverted while focused. `content` replaces the label
/// when given; `name` is what assistive tech calls it.
pub(crate) fn inline_button(
    id: FocusId,
    name: String,
    label: String,
    content: Option<VNode>,
    press: Press,
) -> VNode {
    let focused = use_focus(id).is_focused();
    let (fg, bg) = if focused {
        (Color::Black, Some(Color::White))
    } else {
        (Color::White, None)
    };
    let by_key = press.by_key;
    let on_key = move |key: KeyEvent, ctx: &mut EventContext| match key.code {
        KeyCode::Enter | KeyCode::Char(' ') => {
            by_key(ctx);
            EventResult::Handled
        }
        _ => EventResult::Ignored,
    };
    let by_click = press.by_click;
    let content = content.unwrap_or_else(|| view! { <TextSpan>{label}</TextSpan> }.into_node());
    view! {
        <Pressable focus_id=id tab_stop=true semantics={Semantics::new(Role::Button, name)}
            on_key=on_key on_click={move || by_click()}>
            <View flex_direction=FlexDirection::Row foreground_color=fg background_color=bg>
                <TextSpan>"["</TextSpan>
                {content}
                <TextSpan>"]"</TextSpan>
            </View>
        </Pressable>
    }
    .into_node()
}
