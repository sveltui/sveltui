use sveltui::prelude::*;
use sveltui_components::date::{self, Date, Weekday};
use sveltui_components::*;

#[cfg(test)]
mod calendar_tests;

#[component]
fn CalendarDemo() -> impl IntoNode {
    let day = state(None::<Date>);
    let picked = day.get().map_or_else(
        || "No date picked.".to_owned(),
        |d| format!("Picked {}.", date::format_date(d)),
    );
    // Weekends can be looked at but not booked.
    let weekend = |d: Date| matches!(d.weekday(), Weekday::Saturday | Weekday::Sunday);
    view! {
        <View padding=1 gap=1>
            <View border=BorderStyle::Single border_color=Color::DarkGrey title=" Book a slot "
                padding={Padding::only(0, 1, 0, 1)} width=26>
                <Calendar bind:value={day} label="Booking date"
                    min={Some(date::today())} is_date_unavailable=weekend />
            </View>
            <TextSpan>{picked}</TextSpan>
            <TextSpan foreground_color=Color::DarkGrey>
                "Tab to the grid · arrows move · PgUp/PgDn months (Shift: years) · Enter picks · q quits"
            </TextSpan>
        </View>
    }
}

fn main() -> std::io::Result<()> {
    App::new(|| view! { <CalendarDemo /> })
        .on_key(sveltui::app::exit_shortcuts)
        .run()
}
