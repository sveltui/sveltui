use crate::binding::CodeBinding;
use crate::buffer::{with_code_buffer, with_code_buffer_mut};
use crate::highlight::highlight_text;
use crossterm::event::KeyEvent;
use crossterm::style::Color;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use sveltui::element::scroll::{get_scroll, set_scroll};
use sveltui::layout::Dimension;
use sveltui::prelude::*;

thread_local! {
    static LAST_SYNCED: RefCell<HashMap<FocusId, String>> = RefCell::new(HashMap::new());
}

#[derive(Props)]
pub struct CodeEditorProps {
    #[prop(optional)]
    pub id: Option<FocusId>,
    #[prop(optional)]
    pub value: Option<CodeBinding>,
    #[prop(optional)]
    pub set_value: Option<WriteSignal<String>>,
    #[prop(default = String::from("// Type code here..."))]
    pub placeholder: String,
    #[prop(optional)]
    pub language: Option<String>,
    #[prop(optional)]
    pub theme: Option<String>,
    #[prop(default = true)]
    pub show_line_numbers: bool,
    #[prop(default = Dimension::Auto)]
    pub width: Dimension,
    #[prop(default = Dimension::Auto)]
    pub height: Dimension,
    #[prop(optional)]
    pub title: Option<String>,
    #[prop(default = 4)]
    pub tab_size: usize,
    #[prop(default = true)]
    pub autofocus: bool,
}

#[component]
pub fn CodeEditor(props: CodeEditorProps) -> impl IntoNode {
    let id = props.id.unwrap_or_default();
    let focused = is_focused(id);

    let border_style = if focused {
        BorderStyle::Double
    } else {
        BorderStyle::Single
    };

    let border_color = if focused {
        Color::White
    } else {
        Color::DarkGrey
    };

    let set_value = props
        .set_value
        .or_else(|| props.value.as_ref().and_then(|b| b.write_signal()));

    let total_lines = with_code_buffer_mut(id, |buf| {
        buf.tab_size = props.tab_size;
        let last = LAST_SYNCED.with(|m| m.borrow().get(&id).cloned());

        if let Some(ref binding) = props.value {
            let current = binding.get_text();
            match last {
                None => {
                    if !current.is_empty() {
                        buf.set_content(&current);
                    }
                    LAST_SYNCED.with(|m| m.borrow_mut().insert(id, current));
                }
                Some(ref prev) if prev != &current => {
                    buf.set_content(&current);
                    LAST_SYNCED.with(|m| m.borrow_mut().insert(id, current));
                }
                _ => {}
            }
        }
        buf.line_count()
    });

    let digits = format!("{}", total_lines.max(1)).len().max(2);
    let gutter_width = if props.show_line_numbers {
        digits + 3
    } else {
        0
    };

    let visible_height = match props.height {
        Dimension::Length(h) => ((h.round() as u16).saturating_sub(2)).max(1) as usize,
        _ => 20,
    };
    let visible_width = match props.width {
        Dimension::Length(w) => {
            ((w.round() as u16).saturating_sub(4 + gutter_width as u16)).max(1) as usize
        }
        _ => 60,
    };

    let (is_empty, full_content) = with_code_buffer_mut(id, |buf| {
        buf.update_viewport(visible_width, visible_height);
        (buf.is_empty(), buf.content())
    });

    let all_line_spans = highlight_text(
        &full_content,
        props.language.as_deref(),
        props.theme.as_deref(),
    );

    let show_line_numbers = props.show_line_numbers;

    let rendered_line_nodes: Vec<VNode> = if is_empty {
        let mut row = ViewProps::builder()
            .flex_direction(FlexDirection::Row)
            .align_items(AlignItems::Center)
            .justify_content(JustifyContent::FlexStart);
        if show_line_numbers {
            row = row.child(TextSpan(
                TextSpanProps::builder()
                    .foreground_color(Color::DarkGrey)
                    .content(format!("{:>width$} │ ", 1, width = digits))
                    .build(),
            ));
        }
        row = row.child(TextSpan(
            TextSpanProps::builder()
                .foreground_color(Color::DarkGrey)
                .content(props.placeholder.clone())
                .build(),
        ));
        vec![View(row.build()).into_node()]
    } else {
        all_line_spans
            .into_iter()
            .enumerate()
            .map(|(offset, spans)| {
                let line_num = offset + 1;
                let mut row = ViewProps::builder()
                    .flex_direction(FlexDirection::Row)
                    .align_items(AlignItems::Center)
                    .justify_content(JustifyContent::FlexStart);
                if show_line_numbers {
                    row = row.child(TextSpan(
                        TextSpanProps::builder()
                            .foreground_color(Color::DarkGrey)
                            .content(format!("{:>width$} │ ", line_num, width = digits))
                            .build(),
                    ));
                }
                if spans.is_empty() {
                    row = row.child(TextSpan(
                        TextSpanProps::builder()
                            .foreground_color(Color::DarkGrey)
                            .content(" ")
                            .build(),
                    ));
                } else {
                    for span in spans {
                        row = row.child(TextSpan(
                            TextSpanProps::builder()
                                .foreground_color(span.fg)
                                .bold(span.bold)
                                .italic(span.italic)
                                .underline(span.underline)
                                .content(span.text)
                                .build(),
                        ));
                    }
                }
                View(row.build()).into_node()
            })
            .collect()
    };

    let on_click_at: Rc<dyn Fn(u16, u16)> = Rc::new(move |rel_x: u16, rel_y: u16| {
        let (scroll_x, scroll_y) = get_scroll(id);
        let click_col =
            (rel_x as i32 + scroll_x as i32 - (2 + gutter_width as i32)).max(0) as usize;
        let click_row = (rel_y as i32 + scroll_y as i32 - 1).max(0) as usize;
        with_code_buffer_mut(id, |buf| {
            buf.set_cursor_pos(click_row, click_col);
            buf.update_viewport(visible_width, visible_height);
            set_scroll(id, (buf.scroll_col as u16, buf.scroll_row as u16));
        });
    });

    let on_scroll: Rc<dyn Fn(i16, i16)> = Rc::new(move |_dx: i16, dy: i16| {
        with_code_buffer_mut(id, |buf| {
            if dy > 0 {
                let max_row = buf.line_count().saturating_sub(1);
                buf.scroll_row = (buf.scroll_row + dy as usize).min(max_row);
            } else if dy < 0 {
                buf.scroll_row = buf.scroll_row.saturating_sub((-dy) as usize);
            }
            set_scroll(id, (buf.scroll_col as u16, buf.scroll_row as u16));
        });
    });

    let on_key: KeyHandler = Rc::new(move |key: KeyEvent, _ctx: &mut EventContext| {
        if !with_code_buffer(id, |buf| buf.accepts_key(&key)) {
            return EventResult::Ignored;
        }

        with_code_buffer_mut(id, |buf| {
            let (sx, sy) = get_scroll(id);
            buf.scroll_row = sy as usize;
            buf.scroll_col = sx as usize;

            let modified = buf.handle_key(&key);
            buf.update_viewport(visible_width, visible_height);
            set_scroll(id, (buf.scroll_col as u16, buf.scroll_row as u16));

            if modified {
                let text = buf.content();
                LAST_SYNCED.with(|m| m.borrow_mut().insert(id, text.clone()));
                if let Some(sv) = set_value {
                    sv.set(text);
                }
            }
        });
        EventResult::Handled
    });

    let (scroll_x, scroll_y) = get_scroll(id);
    let (cur_row, cur_col) = with_code_buffer(id, |buf| (buf.cursor_row, buf.cursor_col));
    let rel_row = (cur_row as i32) - (scroll_y as i32);
    let rel_col = (gutter_width as i32 + cur_col as i32) - (scroll_x as i32);

    let cursor_offset = if focused && rel_row >= 0 && rel_col >= 0 {
        Some((rel_col as u16, rel_row as u16))
    } else {
        None
    };

    let title_str = props
        .title
        .clone()
        .unwrap_or_else(|| props.placeholder.clone());

    let focus_scope = if props.autofocus {
        Some(FocusScope {
            contain_tab: false,
            initial_focus: InitialFocus::Container,
        })
    } else {
        None
    };

    view! {
        <Pressable
            focus_id=id
            tab_stop=true
            opt_focus_scope=focus_scope
            semantics=Semantics::new(Role::Textbox, title_str)
            on_click_at=on_click_at
            on_key=on_key
        >
            <View
                border=border_style
                border_color=border_color
                padding=Padding::symmetric(1, 0)
                width=props.width
                height=props.height
                flex_direction=FlexDirection::Column
                align_items=AlignItems::FlexStart
                justify_content=JustifyContent::FlexStart
                overflow_x=Overflow::Scroll
                overflow_y=Overflow::Scroll
                scroll_id=id
                on_scroll=on_scroll
                opt_title=props.title
                opt_cursor_offset=cursor_offset
            >
                {#each rendered_line_nodes as line_node}
                    {line_node}
                {/each}
            </View>
        </Pressable>
    }
}
