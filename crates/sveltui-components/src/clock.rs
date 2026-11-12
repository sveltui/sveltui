//! The time components measure delays against. Tests set it by hand, so
//! a delay never needs real waiting.

use std::time::Instant;

#[cfg(test)]
thread_local! {
    pub(crate) static TEST_NOW: std::cell::Cell<Option<Instant>> =
        const { std::cell::Cell::new(None) };
}

pub(crate) fn now() -> Instant {
    #[cfg(test)]
    if let Some(now) = TEST_NOW.with(std::cell::Cell::get) {
        return now;
    }
    Instant::now()
}
