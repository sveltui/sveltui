//! Dates and times of day, with no time zone: what the calendar, date
//! picker and time picker read and write. The types are `jiff`'s civil
//! ones, so they work with the rest of `jiff` too.

use jiff::ToSpan;
pub use jiff::civil::{Date, Time, Weekday};

/// Today's date where the user is.
pub fn today() -> Date {
    crate::clock::today()
}

/// `date` moved by `days`, stopping at the first and last dates there are.
pub fn add_days(date: Date, days: i64) -> Date {
    date.saturating_add(days.days())
}

/// `date` moved by `months`, keeping the day where the month allows and
/// otherwise using its last day: Jan 31 plus a month is the end of
/// February.
pub fn add_months(date: Date, months: i64) -> Date {
    date.saturating_add(months.months())
}

/// `date` moved by `years`; Feb 29 becomes Feb 28 in a year without one.
pub fn add_years(date: Date, years: i64) -> Date {
    date.saturating_add(years.years())
}

/// The first day of the week `date` is in, for weeks that start on
/// `first`.
pub fn start_of_week(date: Date, first: Weekday) -> Date {
    let offset =
        (date.weekday().to_monday_zero_offset() - first.to_monday_zero_offset()).rem_euclid(7);
    add_days(date, -i64::from(offset))
}

/// The last day of the week `date` is in.
pub fn end_of_week(date: Date, first: Weekday) -> Date {
    add_days(start_of_week(date, first), 6)
}

/// The weekdays in order, starting from `first`.
pub fn weekdays(first: Weekday) -> [Weekday; 7] {
    std::array::from_fn(|i| first.wrapping_add(i as i64))
}

/// The six weeks a month grid shows for the month `date` is in, starting
/// on the week of the 1st. Six weeks always fit a month, and keeping the
/// count fixed stops the grid changing height between months.
pub fn month_grid(date: Date, first: Weekday) -> [[Date; 7]; 6] {
    let start = start_of_week(date.first_of_month(), first);
    std::array::from_fn(|week| std::array::from_fn(|day| add_days(start, (week * 7 + day) as i64)))
}

/// Whether `a` and `b` fall in the same month of the same year.
pub fn same_month(a: Date, b: Date) -> bool {
    a.year() == b.year() && a.month() == b.month()
}

/// `date` kept within `min` and `max`, either of which may be absent.
pub fn clamp(date: Date, min: Option<Date>, max: Option<Date>) -> Date {
    let date = min.map_or(date, |min| date.max(min));
    max.map_or(date, |max| date.min(max))
}

/// Reads `YYYY-MM-DD` (RFC 3339 full-date).
pub fn parse_date(text: &str) -> Option<Date> {
    text.trim().parse().ok()
}

/// Writes `YYYY-MM-DD`.
pub fn format_date(date: Date) -> String {
    date.to_string()
}

/// Reads `HH:MM` or `HH:MM:SS` (RFC 3339 partial-time, without fractions).
pub fn parse_time(text: &str) -> Option<Time> {
    text.trim().parse().ok()
}

/// Writes `HH:MM`, or `HH:MM:SS` when `seconds` is set.
pub fn format_time(time: Time, seconds: bool) -> String {
    if seconds {
        format!(
            "{:02}:{:02}:{:02}",
            time.hour(),
            time.minute(),
            time.second()
        )
    } else {
        format!("{:02}:{:02}", time.hour(), time.minute())
    }
}

#[cfg(test)]
mod date_tests;
