use sveltui::prelude::*;
use sveltui_components::date::{self, Date};
use sveltui_components::*;

#[cfg(test)]
mod date_picker_tests;

fn shown(day: Option<Date>) -> String {
    day.map_or_else(|| "none".to_owned(), date::format_date)
}

#[component]
fn DatePickerDemo() -> impl IntoNode {
    let check_in = state(None::<Date>);
    let birthday = state(None::<Date>);
    let summary = format!(
        "Check-in: {} · Birthday: {}",
        shown(check_in.get()),
        shown(birthday.get())
    );
    view! {
        <View padding=1 gap=1>
            <View flex_direction=FlexDirection::Row gap=2>
                <View width=10><TextSpan>"Check-in"</TextSpan></View>
                <DatePicker bind:value={check_in} label="Check-in" min={Some(date::today())} />
            </View>
            <View flex_direction=FlexDirection::Row gap=2>
                <View width=10><TextSpan>"Birthday"</TextSpan></View>
                <DateField bind:value={birthday} label="Birthday" order=DateOrder::Dmy
                    max={Some(date::today())} />
            </View>
            <TextSpan>{summary}</TextSpan>
            <TextSpan foreground_color=Color::DarkGrey>
                "Type digits or use ↑/↓ in a field · ←/→ between parts · Alt+↓ or [▾] opens the calendar · q quits"
            </TextSpan>
        </View>
    }
}

fn main() -> std::io::Result<()> {
    App::new(|| view! { <DatePickerDemo /> })
        .on_key(sveltui::app::exit_shortcuts)
        .run()
}
