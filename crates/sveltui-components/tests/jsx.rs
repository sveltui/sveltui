use sveltui::prelude::*;
use sveltui_components::*;

#[test]
fn test_jsx_custom_components_expansion() {
    let (val, set_val) = create_signal(String::from("Hello"));
    let (checked, set_checked) = create_signal(true);

    let _node: VNode = view! {
        <View
            min_width=20
            max_width=80
            min_height=5
            max_height=30
            overflow_y=Overflow::Scroll
            scrollbar_visibility=ScrollbarVisibility::Auto
        >
            <Button>"Click Me"</Button>
            <Switch checked={checked.get()} on_checked_change={move |v: bool| set_checked.set(v)} label="Toggle" />
            <Checkbox checked={checked.get()} on_checked_change={move |v: bool| set_checked.set(v)} label="Accept" />
            <TextInput value={val.get()} on_value_change={move |v: String| set_val.set(v)} placeholder="Type..." title="Input" />
            <Divider title="Separator" />
            <ProgressBar ratio=0.75f32 width=20u16 />
            <Badge bg=Color::Green fg=Color::Black>"Active"</Badge>
        </View>
    };
}

#[test]
fn test_jsx_overlay_components_expansion() {
    let (is_open, set_open) = create_signal(true);
    let (modal_open, set_modal_open) = create_signal(false);

    let _node: VNode = view! {
        <View width=80 height=24>
            <DropdownMenu::Root open={is_open.get()} on_open_change={move |next| set_open.set(next)}>
                <DropdownMenu::Trigger>"Actions"</DropdownMenu::Trigger>
                <DropdownMenu::Content placement=Placement::BottomStart>

                    <DropdownMenu::Item>"Cut"</DropdownMenu::Item>
                    <DropdownMenu::Item>"Copy"</DropdownMenu::Item>
                    <DropdownMenu::Item>"Paste"</DropdownMenu::Item>
                </DropdownMenu::Content>
            </DropdownMenu::Root>

            <Dialog
                open={modal_open.get()}
                title="Confirm Delete"
                on_close={move || set_modal_open.set(false)}
            >
                <TextSpan>"Are you sure you want to delete this file?"</TextSpan>
                <View justify_content=JustifyContent::FlexEnd>
                    <Button on_click={move || set_modal_open.set(false)}>"Cancel"</Button>
                    <Button on_click={move || set_modal_open.set(false)}>"Delete"</Button>
                </View>
            </Dialog>
        </View>
    };
}
