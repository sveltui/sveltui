use super::*;

fn minutes(value: Option<i32>, step: i32) -> Segment {
    Segment {
        name: "minute",
        value,
        start: 41,
        min: 0,
        max: 59,
        step,
        page_step: 15,
        digits: 2,
        text: String::new(),
        value_text: String::new(),
        letters: None,
    }
}

#[test]
fn steps_land_on_multiples_of_the_step() {
    assert_eq!(step(&minutes(Some(7), 5), 5), 10);
    assert_eq!(step(&minutes(Some(7), 5), -5), 5);
    assert_eq!(step(&minutes(Some(10), 5), -5), 5);
    assert_eq!(step(&minutes(Some(10), 5), 5), 15);
}

#[test]
fn steps_wrap_round_both_ends() {
    assert_eq!(step(&minutes(Some(59), 1), 1), 0);
    assert_eq!(step(&minutes(Some(55), 5), 5), 0);
    assert_eq!(step(&minutes(Some(0), 1), -1), 59);
    assert_eq!(
        step(&minutes(Some(0), 15), -15),
        45,
        "the last one that fits"
    );
}

#[test]
fn an_empty_segment_starts_where_it_is_told() {
    assert_eq!(step(&minutes(None, 5), 5), 41);
    assert_eq!(step(&minutes(None, 5), -5), 41);
}
