//! The timestamp boundary.
//!
//! A capsule's validity window is a security boundary: if one reader of a
//! timestamp and another reader disagree about it, they disagree about
//! whether a stale capsule may run. The parser therefore accepts one form
//! and refuses everything else, and these tests pin that form.
//!
//! The Python reference model has its own parser and its own tests. The two
//! are separate implementations of the same contract, so a case added here
//! should be added there too.

use senvoryn_capsule::capsule::parse_utc_millis;

#[test]
fn a_utc_timestamp_is_the_expected_number_of_milliseconds() {
    // The epoch itself.
    assert_eq!(parse_utc_millis("1970-01-01T00:00:00Z"), Ok(0));
    // One second, one millisecond.
    assert_eq!(parse_utc_millis("1970-01-01T00:00:01Z"), Ok(1_000));
    assert_eq!(parse_utc_millis("1970-01-01T00:00:01.001Z"), Ok(1_001));
    // A leap day, and the day after it.
    assert_eq!(parse_utc_millis("2024-02-29T00:00:00Z"), Ok(1_709_164_800_000));
    assert_eq!(parse_utc_millis("2024-03-01T00:00:00Z"), Ok(1_709_251_200_000));
    // The sample capsule's issue time.
    assert_eq!(parse_utc_millis("2026-01-01T00:00:00Z"), Ok(1_767_225_600_000));
}

#[test]
fn a_fraction_is_read_as_milliseconds() {
    assert_eq!(parse_utc_millis("1970-01-01T00:00:01.123Z"), Ok(1_123));
    assert_eq!(parse_utc_millis("1970-01-01T00:00:01.1Z"), Ok(1_100));
    assert_eq!(parse_utc_millis("1970-01-01T00:00:01.12Z"), Ok(1_120));
    // A fourth fractional digit is beyond the millisecond the profile
    // carries, so the timestamp is not in the form the profile accepts.
    assert!(parse_utc_millis("1970-01-01T00:00:01.1239Z").is_err());
}

#[test]
fn a_timestamp_with_an_offset_is_refused_rather_than_converted() {
    // Converting an offset needs a time-zone database, and a capsule whose
    // clock is stated in UTC is not ambiguous.
    assert!(parse_utc_millis("2026-01-01T00:00:00+00:00").is_err());
    assert!(parse_utc_millis("2026-01-01T05:30:00+05:30").is_err());
    assert!(parse_utc_millis("2026-01-01T00:00:00-08:00").is_err());
}

#[test]
fn a_timestamp_that_is_not_in_the_strict_form_is_refused() {
    assert!(parse_utc_millis("2026-01-01T00:00:00").is_err());
    assert!(parse_utc_millis("2026-01-01 00:00:00Z").is_err());
    assert!(parse_utc_millis("2026-01-01").is_err());
    assert!(parse_utc_millis("").is_err());
    assert!(parse_utc_millis("not a timestamp").is_err());
    // A two-digit year, and a year that is not four digits.
    assert!(parse_utc_millis("26-01-01T00:00:00Z").is_err());
    assert!(parse_utc_millis("20260-01-01T00:00:00Z").is_err());
    // A leap second, which the profile does not accept.
    assert!(parse_utc_millis("2026-01-01T00:00:60Z").is_err());
    // An hour out of range.
    assert!(parse_utc_millis("2026-01-01T24:00:00Z").is_err());
}

#[test]
fn a_date_that_does_not_exist_is_refused_rather_than_rolled_over() {
    // The thirtieth of February, in a year that is not a leap year.
    assert!(parse_utc_millis("2026-02-30T00:00:00Z").is_err());
    // The twenty-ninth of February, in a year that is not a leap year.
    assert!(parse_utc_millis("2026-02-29T00:00:00Z").is_err());
    // The thirty-first of a month that has thirty days.
    assert!(parse_utc_millis("2026-04-31T00:00:00Z").is_err());
    // The zeroth of a month.
    assert!(parse_utc_millis("2026-01-00T00:00:00Z").is_err());
    // The thirteenth of a month.
    assert!(parse_utc_millis("2026-13-01T00:00:00Z").is_err());
    // A year of zero.
    assert!(parse_utc_millis("0000-01-01T00:00:00Z").is_err());
    // The leap day of a year that is not a leap year, by the century rule.
    assert!(parse_utc_millis("1900-02-29T00:00:00Z").is_err());
    // The leap day of a year that is, by the four-hundred-year rule.
    assert!(parse_utc_millis("2000-02-29T00:00:00Z").is_ok());
}
