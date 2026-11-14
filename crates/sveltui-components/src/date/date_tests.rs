use super::*;
use jiff::civil::{date, time};

#[test]
fn months_keep_the_day_or_use_the_last_one() {
    assert_eq!(add_months(date(2024, 1, 31), 1), date(2024, 2, 29));
    assert_eq!(add_months(date(2023, 1, 31), 1), date(2023, 2, 28));
    assert_eq!(add_months(date(2024, 3, 15), -14), date(2023, 1, 15));
    assert_eq!(add_years(date(2024, 2, 29), 1), date(2025, 2, 28));
}

#[test]
fn moving_stops_at_the_ends_of_time() {
    assert_eq!(add_days(Date::MAX, 1), Date::MAX);
    assert_eq!(add_months(Date::MIN, -1), Date::MIN);
}

#[test]
fn weeks_start_on_the_chosen_day() {
    // 2024-05-15 is a Wednesday.
    let wed = date(2024, 5, 15);
    assert_eq!(start_of_week(wed, Weekday::Monday), date(2024, 5, 13));
    assert_eq!(end_of_week(wed, Weekday::Monday), date(2024, 5, 19));
    assert_eq!(start_of_week(wed, Weekday::Sunday), date(2024, 5, 12));
    assert_eq!(
        start_of_week(date(2024, 5, 13), Weekday::Monday),
        date(2024, 5, 13)
    );
    assert_eq!(weekdays(Weekday::Sunday)[1], Weekday::Monday);
}

#[test]
fn a_month_grid_is_six_weeks_from_the_week_of_the_first() {
    // February 2026 starts on a Sunday.
    let grid = month_grid(date(2026, 2, 10), Weekday::Monday);
    assert_eq!(grid[0][0], date(2026, 1, 26));
    assert_eq!(grid[0][6], date(2026, 2, 1));
    assert_eq!(grid[5][6], date(2026, 3, 8));
    assert!(
        grid.iter()
            .flatten()
            .filter(|d| same_month(**d, date(2026, 2, 1)))
            .count()
            == 28
    );
}

#[test]
fn clamping_keeps_dates_in_range() {
    let (min, max) = (Some(date(2024, 1, 1)), Some(date(2024, 12, 31)));
    assert_eq!(clamp(date(2023, 6, 1), min, max), date(2024, 1, 1));
    assert_eq!(clamp(date(2025, 6, 1), min, max), date(2024, 12, 31));
    assert_eq!(clamp(date(2024, 6, 1), None, None), date(2024, 6, 1));
}

#[test]
fn text_round_trips_in_rfc_3339_form() {
    assert_eq!(parse_date("2024-02-29"), Some(date(2024, 2, 29)));
    assert_eq!(parse_date("2023-02-29"), None);
    assert_eq!(format_date(date(2024, 2, 9)), "2024-02-09");
    assert_eq!(parse_time("09:05"), Some(time(9, 5, 0, 0)));
    assert_eq!(parse_time("23:59:58"), Some(time(23, 59, 58, 0)));
    assert_eq!(format_time(time(9, 5, 7, 0), false), "09:05");
    assert_eq!(format_time(time(9, 5, 7, 0), true), "09:05:07");
}

#[test]
fn today_can_be_set_in_tests() {
    crate::clock::TEST_TODAY.with(|t| t.set(Some(date(2030, 1, 2))));
    assert_eq!(today(), date(2030, 1, 2));
}

#[test]
fn today_reads_the_local_clock() {
    // A fresh thread has no test override, so this is the real clock and
    // time zone.
    let real = std::thread::spawn(today).join().unwrap();
    assert!(real.year() >= 2024, "{real}");
}
