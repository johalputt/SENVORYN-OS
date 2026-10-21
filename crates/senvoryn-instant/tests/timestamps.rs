//! The timestamp boundary.
//!
//! A capsule's validity window is a security boundary: if one reader of a
//! timestamp and another reader disagree about it, they disagree about
//! whether a stale capsule may run. The parser therefore accepts one form
//! and refuses everything else, and these tests pin that form.
//!
//! Each guard is pinned by its own test, and each test names the guard it
//! pins. The parser has several ways to say no to a malformed timestamp —
//! the length, the separators, the zone marker, the fraction, the date, the
//! time, the year — and a test that only says "some refusal happened"
//! cannot tell which of them is load-bearing. A guard that is only ever
//! reached together with another one cannot be tested: it can be removed and
//! nothing would change. So where a case could be refused by two guards, the
//! case here is chosen so that only one of them can refuse it.
//!
//! Every guard below is mutation-tested, in this file and in the Python
//! reference model.
//!
//! The Python reference model has its own parser and its own tests. The two
//! are separate implementations of the same contract, so a case added here
//! should be added there too.

use senvoryn_instant::parse_utc_millis;

#[test]
fn a_utc_timestamp_is_the_expected_number_of_milliseconds() {
    // The epoch itself.
    assert_eq!(parse_utc_millis("1970-01-01T00:00:00Z"), Ok(0), "the epoch is zero");
    // One second, one millisecond.
    assert_eq!(
        parse_utc_millis("1970-01-01T00:00:01Z"),
        Ok(1_000),
        "a second is a thousand milliseconds"
    );
    assert_eq!(
        parse_utc_millis("1970-01-01T00:00:01.001Z"),
        Ok(1_001),
        "a millisecond is a millisecond"
    );
    // The sample capsule's issue time.
    assert_eq!(
        parse_utc_millis("2026-01-01T00:00:00Z"),
        Ok(1_767_225_600_000),
        "the sample capsule's issue time is a number"
    );
}

#[test]
fn a_leap_day_in_a_leap_year_is_a_real_instant() {
    assert_eq!(
        parse_utc_millis("2024-02-29T00:00:00Z"),
        Ok(1_709_164_800_000),
        "a leap day in a leap year is a real instant"
    );
    assert_eq!(
        parse_utc_millis("2024-03-01T00:00:00Z"),
        Ok(1_709_251_200_000),
        "the day after a leap day is the next day"
    );
    // The four-hundred-year rule: 2000 is a leap year and 1900 is not.
    assert_eq!(
        parse_utc_millis("2000-02-29T00:00:00Z"),
        Ok(951_782_400_000),
        "a leap day in a leap year is a real instant"
    );
}

#[test]
fn a_timestamp_that_is_not_in_the_strict_form_is_refused() {
    // The date and the time are separated by a T, and the separators are
    // fixed: a form that differs by one character is a different form.
    assert!(
        parse_utc_millis("2026-01-01 00:00:00Z").is_err(),
        "the strict form is one form: the date and the time are separated by a T"
    );
    assert!(
        parse_utc_millis("2026-01-01T00:00:00").is_err(),
        "the strict form is one form: a timestamp without a zone marker is not in it"
    );
    assert!(
        parse_utc_millis("2026-01-01").is_err(),
        "the strict form is one form: a date alone is not a timestamp"
    );
    assert!(
        parse_utc_millis("").is_err(),
        "the strict form is one form: nothing is not a timestamp"
    );
    assert!(
        parse_utc_millis("not a timestamp").is_err(),
        "the strict form is one form: text is not a timestamp"
    );
    // A two-digit year, and a year that is not four digits.
    assert!(
        parse_utc_millis("26-01-01T00:00:00Z").is_err(),
        "the strict form is one form: the year is four digits"
    );
    assert!(
        parse_utc_millis("20260-01-01T00:00:00Z").is_err(),
        "the strict form is one form: the year is four digits"
    );
}

#[test]
fn a_timestamp_that_is_not_in_utc_is_refused() {
    // A numeric offset is refused rather than converted: converting one
    // needs a time-zone database, and a capsule that states its clock in UTC
    // is not ambiguous.
    assert!(
        parse_utc_millis("2026-01-01T00:00:00+00:00").is_err(),
        "a timestamp that is not in UTC is refused rather than converted"
    );
    assert!(
        parse_utc_millis("2026-01-01T05:30:00+05:30").is_err(),
        "a timestamp that is not in UTC is refused rather than converted"
    );
    assert!(
        parse_utc_millis("2026-01-01T00:00:00-08:00").is_err(),
        "a timestamp that is not in UTC is refused rather than converted"
    );
    // A lower-case zone marker. The length, the separators and the fraction
    // are all in place here, so the zone marker is the only guard that can
    // refuse this one — which is what makes the guard testable.
    assert!(
        parse_utc_millis("2026-01-01T00:00:00z").is_err(),
        "a timestamp that is not in UTC is refused rather than converted"
    );
}

#[test]
fn a_malformed_fraction_is_refused() {
    // A fourth fractional digit is beyond the millisecond the profile
    // carries, so the timestamp is not in the form the profile accepts.
    assert!(
        parse_utc_millis("1970-01-01T00:00:01.1239Z").is_err(),
        "a malformed fraction is refused: three digits at most"
    );
    // The fraction follows a full stop. The length, the separators, the zone
    // marker and the digits are all in place here, so the full stop is the
    // only guard that can refuse this one.
    assert!(
        parse_utc_millis("2026-01-01T00:00:00+1Z").is_err(),
        "a malformed fraction is refused: the fraction follows a full stop"
    );
}

#[test]
fn a_date_that_does_not_exist_is_refused_rather_than_rolled_over() {
    // The thirtieth of February, in a year that is not a leap year.
    assert!(
        parse_utc_millis("2026-02-30T00:00:00Z").is_err(),
        "a date that does not exist is refused rather than rolled over"
    );
    // The twenty-ninth of February, in a year that is not a leap year.
    assert!(
        parse_utc_millis("2026-02-29T00:00:00Z").is_err(),
        "a date that does not exist is refused rather than rolled over"
    );
    // The thirty-first of a month that has thirty days.
    assert!(
        parse_utc_millis("2026-04-31T00:00:00Z").is_err(),
        "a date that does not exist is refused rather than rolled over"
    );
    // The zeroth of a month.
    assert!(
        parse_utc_millis("2026-01-00T00:00:00Z").is_err(),
        "a date that does not exist is refused rather than rolled over"
    );
    // The thirteenth of a month.
    assert!(
        parse_utc_millis("2026-13-01T00:00:00Z").is_err(),
        "a date that does not exist is refused rather than rolled over"
    );
    // The leap day of a year that is not a leap year, by the century rule.
    assert!(
        parse_utc_millis("1900-02-29T00:00:00Z").is_err(),
        "a date that does not exist is refused rather than rolled over"
    );
}

#[test]
fn a_time_that_does_not_exist_is_refused() {
    // An hour out of range.
    assert!(
        parse_utc_millis("2026-01-01T24:00:00Z").is_err(),
        "a time that does not exist is refused"
    );
    // A leap second, which the profile does not accept.
    assert!(
        parse_utc_millis("2026-01-01T00:00:60Z").is_err(),
        "a time that does not exist is refused"
    );
}

#[test]
fn a_year_before_one_is_refused() {
    // A year of zero. The proleptic Gregorian calendar has no year zero, so
    // the conversion from a civil date to a number of days is not defined
    // there.
    assert!(parse_utc_millis("0000-01-01T00:00:00Z").is_err(), "the year must be one or later");
}
