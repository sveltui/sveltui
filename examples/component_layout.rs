use sveltui::prelude::*;
use sveltui_components::*;

#[cfg(test)]
#[path = "__tests__/layout_tests.rs"]
mod layout_tests;

#[component]
fn LayoutDemo() -> impl IntoNode {
    let direction = state(FlexDirection::Row);
    let gap = state(2u16);

    view! {
        <View width=Size::Fill height=Size::Fill align_items=Align::Center justify_content=Justify::Center>
            <View
                width=Size::Fill
                max_width=100
                height=Size::Fill
                max_height=50
                border=BorderStyle::Single
                border_color=Color::DarkGrey
            >

                <View flex_direction=FlexDirection::Row gap=1>
                    <Button
                        on_click={move || direction.set(FlexDirection::Row)}
                    >
                        "Row Direction"
                    </Button>
                    <Button
                        on_click={move || direction.set(FlexDirection::Column)}
                    >
                        "Column Direction"
                    </Button>
                    <Button
                        on_click={move || gap.update(|g| *g = if *g >= 4 { 0 } else { *g + 1 })}
                    >
                        {format!("Cycle Gap ({})", gap.get())}
                    </Button>
                </View>

                <View height=1 />

                // Interactive Flex Container
                <View
                    width=Size::Fill
                    height=Size::Fill
                    flex_direction=direction.get()
                    gap=gap.get()
                    overflow=Overflow::Scroll
                >
                    <View
                        width=Size::Fill
                        height=Size::Fill
                        min_height=6
                        background_color=Color::Red
                    >
                    </View>

                    <View
                        width=Size::Fill
                        height=Size::Fill
                        min_height=6
                        background_color=Color::Green
                    >
                    </View>

                    <View
                        width=Size::Fill
                        height=Size::Fill
                        min_height=6
                        background_color=Color::Blue
                    >
                    </View>
                </View>

                <Divider />
                <View justify_content=Justify::Center align_items=AlignItems::Center>
                    <TextSpan>"Tab: Switch controls | Space/Enter: Click | Q/Esc: Quit"</TextSpan>
                </View>
            </View>
        </View>
    }
}

fn main() -> std::io::Result<()> {
    App::new(|| view! { <LayoutDemo /> })
        .on_key(sveltui::app::exit_shortcuts)
        .run()
}
