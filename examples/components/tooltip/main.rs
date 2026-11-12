use sveltui::prelude::*;
use sveltui_components::*;

#[cfg(test)]
mod tooltip_tests;

#[component]
fn TooltipDemo() -> impl IntoNode {
    let last = state(String::from("Nothing yet."));
    view! {
        <View padding=2 gap=1>
            <View flex_direction=FlexDirection::Row gap=1 margin={Margin::only(3, 0, 0, 0)}>
                <Tooltip::Root>
                    <Tooltip::Trigger>
                        <Button label="Save" on_click={move || last.set("Saved.".into())} />
                    </Tooltip::Trigger>
                    <Tooltip::Content>"Write the buffer to disk (Ctrl+S)"</Tooltip::Content>
                </Tooltip::Root>
                <Tooltip::Root>
                    <Tooltip::Trigger>
                        <Button label="Undo" on_click={move || last.set("Undone.".into())} />
                    </Tooltip::Trigger>
                    <Tooltip::Content>"Undo the last change (Ctrl+Z)"</Tooltip::Content>
                </Tooltip::Root>
                <Tooltip::Root>
                    <Tooltip::Trigger>
                        <Button label="Find" on_click={move || last.set("Searching.".into())} />
                    </Tooltip::Trigger>
                    <Tooltip::Content>"Search this file (Ctrl+F)"</Tooltip::Content>
                </Tooltip::Root>
            </View>
            <TextSpan foreground_color=Color::DarkGrey>{last.get()}</TextSpan>
            <View flex_direction=FlexDirection::Row>
                <TextSpan>"Reviewed by "</TextSpan>
                <HoverCard::Root>
                    <HoverCard::Trigger tab_stop=true>
                        <TextSpan underline=true>"@ada"</TextSpan>
                    </HoverCard::Trigger>
                    <HoverCard::Content width=40>
                        <TextSpan bold=true>"Ada Lovelace"</TextSpan>
                        <TextSpan foreground_color=Color::DarkGrey>
                            "Analyst. Wrote the first published program."
                        </TextSpan>
                    </HoverCard::Content>
                </HoverCard::Root>
            </View>
            <TextSpan foreground_color=Color::DarkGrey>
                "Hover or Tab to a button · Esc hides a tip · move onto the card to keep it · q quits"
            </TextSpan>
        </View>
    }
}

fn main() -> std::io::Result<()> {
    App::new(|| view! { <TooltipDemo /> })
        .on_key(sveltui::app::exit_shortcuts)
        .run()
}
