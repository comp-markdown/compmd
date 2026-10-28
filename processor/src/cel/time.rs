//! Timestamps and durations in nanoseconds, and dates as ICU4X's ISO dates, which count days since 1970-01-01 here.

use icu::calendar::types::RataDie;
use icu::calendar::{Date, Iso};

/// Nanoseconds in a second.
pub(crate) const NANOS: i128 = 1_000_000_000;
/// Seconds in a day.
pub(crate) const DAY: i64 = 86_400;
/// CEL's range of timestamps: 0001-01-01T00:00:00Z to 9999-12-31T23:59:59.999999999Z, in seconds.
const FIRST_SECOND: i128 = -62_135_596_800;
const LAST_SECOND: i128 = 253_402_300_799;
/// CEL's range of durations: about ten thousand years either way.
const LONGEST_DURATION: i128 = 315_576_000_000 * NANOS;

pub(crate) fn timestamp_in_range(nanos: i128) -> bool {
    (FIRST_SECOND * NANOS..(LAST_SECOND + 1) * NANOS).contains(&nanos)
}

pub(crate) fn duration_in_range(nanos: i128) -> bool {
    (-LONGEST_DURATION..=LONGEST_DURATION).contains(&nanos)
}

/// 1970-01-01, the day timestamps count from, as a day number of the proleptic Gregorian calendar.
const EPOCH: RataDie = RataDie::new(719_163);

/// The date `days` after 1970-01-01.
pub(crate) fn date(days: i64) -> Date<Iso> {
    Date::from_rata_die(EPOCH + days, Iso)
}

/// The days from 1970-01-01 to `date`.
pub(crate) fn days(date: Date<Iso>) -> i64 {
    date.to_rata_die() - EPOCH
}

/// The number in `text`, which must be all ASCII digits.
fn digits(text: Option<&str>) -> Option<i64> {
    let text = text?;
    if text.is_empty() || !text.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    text.parse().ok()
}

/// A `YYYY-MM-DD` date that exists.
pub(crate) fn parse_date(text: &str) -> Option<Date<Iso>> {
    let bytes = text.as_bytes();
    if bytes.len() != 10 || bytes[4] != b'-' || bytes[7] != b'-' {
        return None;
    }
    let part = |range| digits(text.get(range));
    let year = i32::try_from(part(0..4)?).ok()?;
    let (month, day) = (
        u8::try_from(part(5..7)?).ok()?,
        u8::try_from(part(8..10)?).ok()?,
    );
    Date::try_new_iso(year, month, day).ok()
}

/// A date as `YYYY-MM-DD`.
pub(crate) fn format_date(date: Date<Iso>) -> String {
    let (year, month, day) = (
        date.year().extended_year(),
        date.month().ordinal,
        date.day_of_month().0,
    );
    format!("{year:04}-{month:02}-{day:02}")
}

/// An RFC 3339 timestamp, such as `2026-10-21T10:00:00Z` or `2026-10-21T12:00:00.5+02:00`, in CEL's range.
pub(crate) fn parse_timestamp(text: &str) -> Option<i128> {
    let days = days(parse_date(text.get(..10)?)?);
    let bytes = text.as_bytes();
    if bytes.get(10) != Some(&b'T') || bytes.get(13) != Some(&b':') || bytes.get(16) != Some(&b':')
    {
        return None;
    }
    let (hour, minute, second) = (
        digits(text.get(11..13))?,
        digits(text.get(14..16))?,
        digits(text.get(17..19))?,
    );
    if hour > 23 || minute > 59 || second > 59 {
        return None;
    }
    let mut rest = text.get(19..)?;
    let mut nanos = 0;
    if let Some(fraction) = rest.strip_prefix('.') {
        let count = fraction.bytes().take_while(u8::is_ascii_digit).count();
        if !(1..=9).contains(&count) {
            return None;
        }
        let scale = 10_i128.pow(9 - u32::try_from(count).ok()?);
        nanos = i128::from(digits(fraction.get(..count))?) * scale;
        rest = &fraction[count..];
    }
    let offset = if rest == "Z" { 0 } else { offset(rest)? };
    let seconds = i128::from(days * DAY + hour * 3600 + minute * 60 + second - offset);
    let total = seconds * NANOS + nanos;
    timestamp_in_range(total).then_some(total)
}

/// A `+hh:mm` or `-hh:mm` offset, in seconds.
fn offset(text: &str) -> Option<i64> {
    let sign = match text.as_bytes().first() {
        Some(b'+') => 1,
        Some(b'-') => -1,
        _ => return None,
    };
    if text.len() != 6 || text.as_bytes()[3] != b':' {
        return None;
    }
    Some(sign * (digits(text.get(1..3))? * 3600 + digits(text.get(4..6))? * 60))
}

/// RFC 3339 in UTC, with as many fractional digits as it needs.
pub(crate) fn format_timestamp(nanos: i128) -> String {
    let seconds = i64::try_from(nanos.div_euclid(NANOS)).unwrap_or_default();
    let (days, second_of_day) = (seconds.div_euclid(DAY), seconds.rem_euclid(DAY));
    let (hour, minute, second) = (
        second_of_day / 3600,
        second_of_day / 60 % 60,
        second_of_day % 60,
    );
    let fraction = fraction(nanos.rem_euclid(NANOS));
    format!(
        "{}T{hour:02}:{minute:02}:{second:02}{fraction}Z",
        format_date(date(days))
    )
}

/// `.5` for half a second, nothing for none.
fn fraction(nanos: i128) -> String {
    if nanos == 0 {
        return String::new();
    }
    format!(".{}", format!("{nanos:09}").trim_end_matches('0'))
}

/// A duration as Go writes them, as CEL takes them: `1h`, `1h30m`, `1.5s`, `-90ms`, `0`.
pub(crate) fn parse_duration(text: &str) -> Option<i128> {
    let (sign, mut rest) = match text.as_bytes().first() {
        Some(b'-') => (-1, &text[1..]),
        Some(b'+') => (1, &text[1..]),
        _ => (1, text),
    };
    if rest == "0" {
        return Some(0);
    }
    if rest.is_empty() {
        return None;
    }
    let mut total: i128 = 0;
    while !rest.is_empty() {
        let whole_len = rest.bytes().take_while(u8::is_ascii_digit).count();
        let whole = &rest[..whole_len];
        rest = &rest[whole_len..];
        let mut fraction = "";
        if let Some(after) = rest.strip_prefix('.') {
            let len = after.bytes().take_while(u8::is_ascii_digit).count();
            fraction = &after[..len];
            rest = &after[len..];
        }
        if whole.is_empty() && fraction.is_empty() {
            return None;
        }
        let unit_len: usize = rest
            .chars()
            .take_while(|c| !c.is_ascii_digit() && *c != '.')
            .map(char::len_utf8)
            .sum();
        let unit = match &rest[..unit_len] {
            "ns" => 1,
            "us" | "\u{b5}s" | "\u{3bc}s" => 1_000,
            "ms" => 1_000_000,
            "s" => NANOS,
            "m" => 60 * NANOS,
            "h" => 3600 * NANOS,
            _ => return None,
        };
        rest = &rest[unit_len..];
        let whole: i128 = if whole.is_empty() {
            0
        } else {
            whole.parse().ok()?
        };
        // Digits past the eighteenth can't change the nanoseconds of any unit.
        let fraction = fraction.get(..18).unwrap_or(fraction);
        let part = if fraction.is_empty() {
            0
        } else {
            fraction.parse::<i128>().ok()? * unit / 10_i128.pow(u32::try_from(fraction.len()).ok()?)
        };
        total = total.checked_add(whole.checked_mul(unit)?.checked_add(part)?)?;
    }
    let total = sign * total;
    duration_in_range(total).then_some(total)
}

/// Seconds with as many fractional digits as they need, as CEL writes durations: `3600s`, `1.5s`, `-0.25s`.
pub(crate) fn format_duration(nanos: i128) -> String {
    let sign = if nanos < 0 { "-" } else { "" };
    let nanos = nanos.abs();
    format!("{sign}{}{}s", nanos / NANOS, fraction(nanos % NANOS))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dates_go_both_ways() {
        let day = |text: &str| parse_date(text).map(days);
        let text = |text: &str| parse_date(text).map(format_date);
        assert_eq!(day("1970-01-01"), Some(0));
        assert_eq!(day("1969-12-31"), Some(-1));
        assert_eq!(
            day("2026-10-21").zip(day("2026-10-14")).map(|(a, b)| a - b),
            Some(7)
        );
        assert_eq!(format_date(date(20_747)), "2026-10-21");
        assert_eq!(text("2024-02-29").as_deref(), Some("2024-02-29"));
        assert_eq!(text("0001-01-01").as_deref(), Some("0001-01-01"));
        assert_eq!(parse_date("2026-02-29"), None);
        assert_eq!(parse_date("2026-1-01"), None);
        assert_eq!(parse_date("2026/10/21"), None);
        assert_eq!(parse_date("2026-10-2x"), None);
        assert_eq!(parse_date("20260-10-2"), None);
    }

    #[test]
    fn timestamps_go_both_ways() {
        let one = parse_timestamp("2026-10-21T10:00:00Z");
        assert_eq!(
            one.map(format_timestamp).as_deref(),
            Some("2026-10-21T10:00:00Z")
        );
        assert_eq!(parse_timestamp("2026-10-21T12:00:00+02:00"), one);
        assert_eq!(parse_timestamp("2026-10-21T08:30:00-01:30"), one);
        assert_eq!(
            parse_timestamp("1969-12-31T23:59:59.5Z")
                .map(format_timestamp)
                .as_deref(),
            Some("1969-12-31T23:59:59.5Z")
        );
        assert_eq!(
            parse_timestamp("2026-10-21T10:00:00.123456789Z")
                .map(format_timestamp)
                .as_deref(),
            Some("2026-10-21T10:00:00.123456789Z")
        );
        assert_eq!(
            parse_timestamp("0001-01-01T00:00:00Z"),
            Some(FIRST_SECOND * NANOS)
        );
        for bad in [
            "2026-10-21 10:00:00Z",
            "2026-10-21t10:00:00Z",
            "2026-10-21T24:00:00Z",
            "2026-10-21T10:60:00Z",
            "2026-10-21T10:00:60Z",
            "2026-10-21T10:00:00",
            "2026-10-21T10:00:00.Z",
            "2026-10-21T10:00:00.1234567890Z",
            "2026-10-21T10:00:00+0200",
            "2026-10-21T10:00:00*02:00",
            "2026-10-21T10-00:00Z",
            "2026-10-21",
        ] {
            assert_eq!(parse_timestamp(bad), None, "{bad}");
        }
    }

    #[test]
    fn durations_go_both_ways() {
        assert_eq!(parse_duration("1h"), parse_duration("60m"));
        assert_eq!(parse_duration("1h30m"), Some(5400 * NANOS));
        assert_eq!(parse_duration("+1s"), Some(NANOS));
        assert_eq!(
            parse_duration("1.5s").map(format_duration).as_deref(),
            Some("1.5s")
        );
        assert_eq!(
            parse_duration("-90ms").map(format_duration).as_deref(),
            Some("-0.09s")
        );
        assert_eq!(parse_duration(".5s"), Some(NANOS / 2));
        assert_eq!(parse_duration("1us"), parse_duration("1\u{b5}s"));
        assert_eq!(parse_duration("1\u{3bc}s"), Some(1000));
        assert_eq!(parse_duration("2ns"), Some(2));
        assert_eq!(
            parse_duration("0.1234567890123456789012s"),
            Some(123_456_789)
        );
        assert_eq!(parse_duration("0"), Some(0));
        assert_eq!(format_duration(0), "0s");
        for bad in [
            "1d",
            "h",
            "",
            "-",
            "1",
            "1.s.",
            "99999999999999999999999999999999999999h",
            "400000000h",
        ] {
            assert_eq!(parse_duration(bad), None, "{bad}");
        }
    }

    #[test]
    fn ranges_are_cels() {
        assert!(timestamp_in_range(LAST_SECOND * NANOS + NANOS - 1));
        assert!(!timestamp_in_range((LAST_SECOND + 1) * NANOS));
        assert!(duration_in_range(-LONGEST_DURATION));
        assert!(!duration_in_range(LONGEST_DURATION + 1));
    }
}
