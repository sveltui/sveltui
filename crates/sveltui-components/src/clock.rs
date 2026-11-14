//! The time components measure delays against. Tests set it by hand, so
//! a delay never needs real waiting.

use std::time::Instant;

#[cfg(test)]
thread_local! {
    pub(crate) static TEST_NOW: std::cell::Cell<Option<Instant>> =
        const { std::cell::Cell::new(None) };
    pub(crate) static TEST_TODAY: std::cell::Cell<Option<jiff::civil::Date>> =
        const { std::cell::Cell::new(None) };
    pub(crate) static TEST_TIME: std::cell::Cell<Option<jiff::civil::Time>> =
        const { std::cell::Cell::new(None) };
}

pub(crate) fn now() -> Instant {
    #[cfg(test)]
    if let Some(now) = TEST_NOW.with(std::cell::Cell::get) {
        return now;
    }
    Instant::now()
}

/// Today's date where the user is.
pub(crate) fn today() -> jiff::civil::Date {
    #[cfg(test)]
    if let Some(today) = TEST_TODAY.with(std::cell::Cell::get) {
        return today;
    }
    jiff::Zoned::now().date()
}

/// The time of day where the user is.
pub(crate) fn time_of_day() -> jiff::civil::Time {
    #[cfg(test)]
    if let Some(time) = TEST_TIME.with(std::cell::Cell::get) {
        return time;
    }
    jiff::Zoned::now().time()
}
