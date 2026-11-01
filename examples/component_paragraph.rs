use sveltui::prelude::*;
use sveltui_components::*;

#[path = "__fixtures__/the_raven.rs"]
mod the_raven;

use the_raven::THE_RAVEN;

#[cfg(test)]
#[path = "__tests__/paragraph_tests.rs"]
mod paragraph_tests;

#[component]
fn ParagraphDemo() -> impl IntoNode {
    let scroll_id = use_hook(FocusId::new);
    let poem = THE_RAVEN.join("\n\n");
    view! {
        <View width=Size::Fill height=Size::Fill
            align_items=Align::Center justify_content=Justify::Center>
            <Pressable focus_id=scroll_id tab_stop=true>
                <View width=Size::Fill max_width=68 height=Size::Fill max_height=40
                    scroll_id=scroll_id
                    border=BorderStyle::Single
                    border_color=Color::DarkGrey padding=Padding::all(1) gap=1
                    overflow_y=Overflow::Scroll>
                    <TextSpan foreground_color=Color::White bold=true>"The Raven"</TextSpan>
                    <TextSpan foreground_color=Color::DarkGrey>"Edgar Allan Poe"</TextSpan>
                    <Divider />

                    <Paragraph foreground_color=Color::DarkGrey max_lines=2>
                        {poem.clone()}
                    </Paragraph>

                    <Divider title="Full poem" />
                    <Paragraph foreground_color=Color::White content=poem />

                    <Divider />
                    <Paragraph foreground_color=Color::DarkGrey>"Tab: Focus | Up / Down / Wheel: Scroll | Q / Esc: Quit"</Paragraph>
                </View>
            </Pressable>
        </View>
    }
}

fn main() -> std::io::Result<()> {
    App::new(|| view! { <ParagraphDemo /> })
        .on_key(sveltui::app::exit_shortcuts)
        .run()
}
