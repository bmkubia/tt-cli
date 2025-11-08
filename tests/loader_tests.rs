use std::time::Duration;
use tt::loader::format_elapsed;

#[test]
fn sub_second_values_use_millis() {
    assert_eq!(format_elapsed(Duration::from_millis(450)), "450ms");
}

#[test]
fn seconds_minutes_hours_are_formatted() {
    assert_eq!(format_elapsed(Duration::from_secs(12)), "12s");
    assert_eq!(format_elapsed(Duration::from_secs(125)), "2m 5s");
    assert_eq!(
        format_elapsed(Duration::from_secs(3 * 3600 + 42 * 60 + 9)),
        "3h 42m 9s"
    );
}
