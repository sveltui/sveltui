use sveltui::prelude::*;
use sveltui_components::date::{self, Time};
use sveltui_components::*;

#[cfg(test)]
mod time_field_tests;

fn shown(time: Option<Time>) -> String {
    time.map_or_else(|| "none".to_owned(), |t| date::format_time(t, false))
}

#[component]
fn TimeFieldDemo() -> impl IntoNode {
    let alarm = state(None::<Time>);
    let meeting = state(None::<Time>);
    let summary = format!(
        "Alarm: {} · Meeting: {}",
        shown(alarm.get()),
        shown(meeting.get())
    );
    view! {
        <View padding=1 gap=1>
            <View flex_direction=FlexDirection::Row gap=2>
                <View width=10><TextSpan>"Alarm"</TextSpan></View>
                <TimeField bind:value={alarm} label="Alarm" />
            </View>
            <View flex_direction=FlexDirection::Row gap=2>
                <View width=10><TextSpan>"Meeting"</TextSpan></View>
                <TimeField bind:value={meeting} label="Meeting" hour_cycle=HourCycle::H12
                    minute_step=15 />
            </View>
            <TextSpan>{summary}</TextSpan>
            <TextSpan foreground_color=Color::DarkGrey>
                "Type digits or use ↑/↓ (PgUp/PgDn for big steps) · ←/→ between parts · a/p for AM/PM · q quits"
            </TextSpan>
        </View>
    }
}

fn main() -> std::io::Result<()> {
    App::new(|| view! { <TimeFieldDemo /> })
        .on_key(sveltui::app::exit_shortcuts)
        .run()
}
