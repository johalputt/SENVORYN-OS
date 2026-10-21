//! The strict UTC instant form the system's documents use.
//!
//! An instant that crosses a boundary has to be read the same way by every
//! reader, or two of them will disagree about whether a window is open. This
//! crate is that one arithmetic: it parses the strict UTC form
//! `YYYY-MM-DDTHH:MM:SS[.fff]Z` into milliseconds since the Unix epoch, and
//! refuses everything else.
//!
//! The form is specified in ADR-012. It is deliberately narrower than RFC
//! 3339: a numeric offset is refused rather than converted, because
//! converting one needs a time-zone database, and a document that states its
//! clock in UTC is not ambiguous.
//!
//! The parser is total: there is no input for which it panics, and no input
//! for which it returns a wrong number without saying so. Every guard in it
//! is mutation-tested, in Rust and in the Python reference model.

#![deny(missing_docs)]

/// Parses a strict UTC RFC 3339 timestamp into milliseconds since the epoch.
///
/// Only `YYYY-MM-DDTHH:MM:SSZ` and `YYYY-MM-DDTHH:MM:SS.sssZ` are accepted.
/// A numeric offset is refused rather than converted: converting one needs a
/// time-zone database, and a capsule that states its clock in UTC is not
/// ambiguous. The parser is total — there is no input for which it panics and
/// no input for which it returns a wrong number without saying so.
pub fn parse_utc_millis(text: &str) -> Result<i64, String> {
    let bytes = text.as_bytes();
    if bytes.len() < 20 || bytes.len() > 24 {
        return Err(format!("{text:?} is not a UTC timestamp"));
    }
    if bytes[4] != b'-'
        || bytes[7] != b'-'
        || bytes[10] != b'T'
        || bytes[13] != b':'
        || bytes[16] != b':'
    {
        return Err(format!("{text:?} is not a UTC timestamp"));
    }
    // The `Z` is the last byte. A fractional part, if there is one, sits
    // between the seconds and the `Z`, so the `Z` is not at a fixed offset.
    // Reading it at a fixed offset rejects every timestamp that carries
    // fractional seconds, and the fraction-handling code below would never
    // be reached.
    if bytes[bytes.len() - 1] != b'Z' {
        return Err(format!("{text:?} is not in UTC"));
    }
    let year = number(&bytes[0..4]).ok_or_else(|| format!("{text:?} has no year"))?;
    if year < 1 {
        return Err(format!("{text:?} is not a date"));
    }
    let month = number(&bytes[5..7]).ok_or_else(|| format!("{text:?} has no month"))?;
    let day = number(&bytes[8..10]).ok_or_else(|| format!("{text:?} has no day"))?;
    let hour = number(&bytes[11..13]).ok_or_else(|| format!("{text:?} has no hour"))?;
    let minute = number(&bytes[14..16]).ok_or_else(|| format!("{text:?} has no minute"))?;
    let second = number(&bytes[17..19]).ok_or_else(|| format!("{text:?} has no second"))?;
    if !(1..=12).contains(&month) || day < 1 || day > days_in_month(year, month) {
        return Err(format!("{text:?} is not a date"));
    }
    if hour > 23 || minute > 59 || second > 59 {
        return Err(format!("{text:?} is not a time"));
    }
    let mut millis = 0i64;
    if bytes.len() > 20 {
        if bytes[19] != b'.' {
            return Err(format!("{text:?} is not a UTC timestamp"));
        }
        let fraction = &text[20..text.len() - 1];
        if fraction.is_empty() || !fraction.bytes().all(|byte| byte.is_ascii_digit()) {
            return Err(format!("{text:?} has no fractional seconds"));
        }
        let mut scaled = String::with_capacity(3);
        scaled.push_str(&fraction[..fraction.len().min(3)]);
        while scaled.len() < 3 {
            scaled.push('0');
        }
        millis = number(scaled.as_bytes()).ok_or_else(|| format!("{text:?} has no fraction"))?;
    }
    let days = days_from_civil(year, month, day);
    let seconds = days * 86_400 + hour * 3_600 + minute * 60 + second;
    Ok(seconds * 1_000 + millis)
}

/// How many days a month has, in the proleptic Gregorian calendar.
///
/// A date that does not exist — the thirtieth of February, or the
/// twenty-ninth of February in a year that is not a leap year — is refused
/// rather than rolled over into the next month. A capsule that names a time
/// which never happened is malformed, and silently moving it would mean the
/// executor compared the capsule against an instant its issuer never stated.
fn days_in_month(year: i64, month: i64) -> i64 {
    const LENGTHS: [i64; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    if month == 2 && is_leap_year(year) {
        return 29;
    }
    LENGTHS[(month - 1) as usize]
}

/// Whether a year has a twenty-ninth of February.
fn is_leap_year(year: i64) -> bool {
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}

/// The number of days since 1970-01-01 for a civil date.
///
/// Howard Hinnant's `days_from_civil`, which is exact for the whole proleptic
/// Gregorian calendar and needs no lookup table.
fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let year = if month <= 2 { year - 1 } else { year };
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let year_of_era = year - era * 400;
    let month_position = if month > 2 { month - 3 } else { month + 9 };
    let day_of_year = (153 * month_position + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
}

fn number(bytes: &[u8]) -> Option<i64> {
    if bytes.is_empty() || !bytes.iter().all(u8::is_ascii_digit) {
        return None;
    }
    let mut value = 0i64;
    for byte in bytes {
        value = value * 10 + i64::from(byte - b'0');
    }
    Some(value)
}
