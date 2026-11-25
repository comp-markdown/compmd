//! The profile's functions from CEL's standard definitions and its strings extension (spec §8.2), each with its
//! documentation, its examples and its overloads.

#![allow(
    clippy::unnecessary_wraps,
    reason = "every implementation has an overload's signature, which can fail"
)]

use super::library::text;
use super::value::Value;
use super::{Functions, getters, strings, time};
use compmd_macros::library;

/// A function's implementation: it takes the render, for time zones, and the receiver, if any, then the arguments.
pub(crate) type Call = fn(&dyn Functions, &[Value]) -> Result<Value, String>;

/// A function of the profile from CEL. The `md.` functions come from outside.
#[library(prefix = "", call = Call)]
pub(crate) enum Function {
    /// The length of a string, in characters, a list or a map.
    ///
    /// ```cel
    /// size("héllo") // 5
    /// [1, 2, 3].size() // 3
    /// ```
    #[overload(fn(value: Sized) -> Int = size)]
    #[overload(fn(self: Sized) -> Int = size)]
    Size,

    /// An int: a number without its fraction, a string read, a timestamp's seconds.
    ///
    /// ```cel
    /// int(3.7) // 3
    /// int("42") // 42
    /// ```
    #[overload(fn(value: Int) -> Int = to_int)]
    #[overload(fn(value: Uint) -> Int = to_int)]
    #[overload(fn(value: Double) -> Int = to_int)]
    #[overload(fn(value: String) -> Int = to_int)]
    #[overload(fn(value: Timestamp) -> Int = to_int)]
    Int,

    /// A double from a number or a string.
    ///
    /// ```cel
    /// double("2.5") // 2.5
    /// ```
    #[overload(fn(value: Number) -> Double = to_double)]
    #[overload(fn(value: String) -> Double = to_double)]
    Double,

    /// The value written as a string, as interpolation writes it.
    ///
    /// ```cel
    /// string(1e21) // "1e+21"
    /// string(true) // "true"
    /// ```
    #[overload(fn(value: Number) -> String = to_string)]
    #[overload(fn(value: String) -> String = to_string)]
    #[overload(fn(value: Bool) -> String = to_string)]
    #[overload(fn(value: Timestamp) -> String = to_string)]
    #[overload(fn(value: Duration) -> String = to_string)]
    String,

    /// A timestamp from an RFC 3339 string.
    ///
    /// ```cel
    /// timestamp("2026-10-21T10:00:00Z") // "2026-10-21T10:00:00Z"
    /// ```
    #[overload(fn(text: String) -> Timestamp = to_timestamp)]
    #[overload(fn(value: Timestamp) -> Timestamp = itself)]
    Timestamp,

    /// A duration from a string such as `1h30m`.
    ///
    /// ```cel
    /// duration("1h30m").getMinutes() // 90
    /// ```
    #[overload(fn(text: String) -> Duration = to_duration)]
    #[overload(fn(value: Duration) -> Duration = itself)]
    Duration,

    /// The year, in `md.tz` or the time zone given.
    ///
    /// ```cel
    /// timestamp("2026-10-21T23:30:00Z").getFullYear() // 2026
    /// ```
    #[overload(fn(self: Timestamp) -> Int = getters::full_year)]
    #[overload(fn(self: Timestamp, timeZone: String) -> Int = getters::full_year)]
    GetFullYear,

    /// The month, from 0 for January, in `md.tz` or the time zone given.
    ///
    /// ```cel
    /// timestamp("2026-10-21T23:30:00Z").getMonth() // 9
    /// ```
    #[overload(fn(self: Timestamp) -> Int = getters::month)]
    #[overload(fn(self: Timestamp, timeZone: String) -> Int = getters::month)]
    GetMonth,

    /// The day of the month, from 1, in `md.tz` or the time zone given.
    ///
    /// ```cel
    /// timestamp("2026-10-21T23:30:00Z").getDate() // 21
    /// ```
    #[overload(fn(self: Timestamp) -> Int = getters::date)]
    #[overload(fn(self: Timestamp, timeZone: String) -> Int = getters::date)]
    GetDate,

    /// The day of the month, from 0, in `md.tz` or the time zone given.
    ///
    /// ```cel
    /// timestamp("2026-10-21T23:30:00Z").getDayOfMonth() // 20
    /// ```
    #[overload(fn(self: Timestamp) -> Int = getters::day_of_month)]
    #[overload(fn(self: Timestamp, timeZone: String) -> Int = getters::day_of_month)]
    GetDayOfMonth,

    /// The day of the week, from 0 for Sunday, in `md.tz` or the time zone given.
    ///
    /// ```cel
    /// timestamp("2026-10-21T23:30:00Z").getDayOfWeek() // 3
    /// ```
    #[overload(fn(self: Timestamp) -> Int = getters::day_of_week)]
    #[overload(fn(self: Timestamp, timeZone: String) -> Int = getters::day_of_week)]
    GetDayOfWeek,

    /// The day of the year, from 0, in `md.tz` or the time zone given.
    ///
    /// ```cel
    /// timestamp("2026-10-21T23:30:00Z").getDayOfYear() // 293
    /// ```
    #[overload(fn(self: Timestamp) -> Int = getters::day_of_year)]
    #[overload(fn(self: Timestamp, timeZone: String) -> Int = getters::day_of_year)]
    GetDayOfYear,

    /// A timestamp's hour, in `md.tz` or the time zone given, or a duration's whole hours.
    ///
    /// ```cel
    /// timestamp("2026-10-21T23:30:00Z").getHours("+02:00") // 1
    /// duration("1h30m").getHours() // 1
    /// ```
    #[overload(fn(self: Timestamp) -> Int = getters::hours)]
    #[overload(fn(self: Timestamp, timeZone: String) -> Int = getters::hours)]
    #[overload(fn(self: Duration) -> Int = getters::duration_hours)]
    GetHours,

    /// A timestamp's minute, in `md.tz` or the time zone given, or a duration's whole minutes.
    ///
    /// ```cel
    /// duration("1h30m").getMinutes() // 90
    /// ```
    #[overload(fn(self: Timestamp) -> Int = getters::minutes)]
    #[overload(fn(self: Timestamp, timeZone: String) -> Int = getters::minutes)]
    #[overload(fn(self: Duration) -> Int = getters::duration_minutes)]
    GetMinutes,

    /// A timestamp's second, in `md.tz` or the time zone given, or a duration's whole seconds.
    ///
    /// ```cel
    /// timestamp("2026-10-21T10:00:05Z").getSeconds() // 5
    /// ```
    #[overload(fn(self: Timestamp) -> Int = getters::seconds)]
    #[overload(fn(self: Timestamp, timeZone: String) -> Int = getters::seconds)]
    #[overload(fn(self: Duration) -> Int = getters::duration_seconds)]
    GetSeconds,

    /// A timestamp's millisecond, in `md.tz` or the time zone given, or a duration's whole milliseconds.
    ///
    /// ```cel
    /// duration("1.5s").getMilliseconds() // 1500
    /// ```
    #[overload(fn(self: Timestamp) -> Int = getters::milliseconds)]
    #[overload(fn(self: Timestamp, timeZone: String) -> Int = getters::milliseconds)]
    #[overload(fn(self: Duration) -> Int = getters::duration_milliseconds)]
    GetMilliseconds,

    /// Whether the string holds the other.
    ///
    /// ```cel
    /// "rayplay".contains("play") // true
    /// ```
    #[overload(fn(self: String, substring: String) -> Bool = strings::contains)]
    Contains,

    /// Whether the string starts with the other.
    ///
    /// ```cel
    /// "rayplay".startsWith("ray") // true
    /// ```
    #[overload(fn(self: String, prefix: String) -> Bool = strings::starts_with)]
    StartsWith,

    /// Whether the string ends with the other.
    ///
    /// ```cel
    /// "rayplay".endsWith("play") // true
    /// ```
    #[overload(fn(self: String, suffix: String) -> Bool = strings::ends_with)]
    EndsWith,

    /// The character at a position, from 0.
    ///
    /// ```cel
    /// "héllo".charAt(1) // "é"
    /// ```
    #[overload(fn(self: String, index: Int) -> String = strings::char_at)]
    #[counts]
    CharAt,

    /// The first position of a substring, or -1; from the position given.
    ///
    /// ```cel
    /// "banana".indexOf("an") // 1
    /// "banana".indexOf("an", 2) // 3
    /// ```
    #[overload(fn(self: String, substring: String) -> Int = strings::index_of)]
    #[overload(fn(self: String, substring: String, start: Int) -> Int = strings::index_of)]
    #[counts]
    IndexOf,

    /// The last position of a substring, or -1; up to the position given.
    ///
    /// ```cel
    /// "banana".lastIndexOf("an") // 3
    /// ```
    #[overload(fn(self: String, substring: String) -> Int = strings::last_index_of)]
    #[overload(fn(self: String, substring: String, start: Int) -> Int = strings::last_index_of)]
    #[counts]
    LastIndexOf,

    /// The string with ASCII letters in lower case.
    ///
    /// ```cel
    /// "Hello".lowerAscii() // "hello"
    /// ```
    #[overload(fn(self: String) -> String = strings::lower_ascii)]
    LowerAscii,

    /// The string with ASCII letters in upper case.
    ///
    /// ```cel
    /// "Hello".upperAscii() // "HELLO"
    /// ```
    #[overload(fn(self: String) -> String = strings::upper_ascii)]
    UpperAscii,

    /// The string with a substring replaced, every time or as many times as given.
    ///
    /// ```cel
    /// "a-b-c".replace("-", " ") // "a b c"
    /// "a-b-c".replace("-", " ", 1) // "a b-c"
    /// ```
    #[overload(fn(self: String, old: String, new: String) -> String = strings::replace, size = strings::replace_size)]
    #[overload(fn(self: String, old: String, new: String, count: Int) -> String = strings::replace, size = strings::replace_size)]
    #[counts]
    Replace,

    /// The parts between each separator; at most as many as given.
    ///
    /// ```cel
    /// "a,b,c".split(",") // ["a", "b", "c"]
    /// ```
    #[overload(fn(self: String, separator: String) -> Strings = strings::split, size = strings::split_size)]
    #[overload(fn(self: String, separator: String, limit: Int) -> Strings = strings::split, size = strings::split_size)]
    #[counts]
    Split,

    /// The characters from a position up to another, or to the end.
    ///
    /// ```cel
    /// "rayplay".substring(3) // "play"
    /// "rayplay".substring(0, 3) // "ray"
    /// ```
    #[overload(fn(self: String, start: Int) -> String = strings::substring)]
    #[overload(fn(self: String, start: Int, end: Int) -> String = strings::substring)]
    #[counts]
    Substring,

    /// The string without white space at either end.
    ///
    /// ```cel
    /// "  hi  ".trim() // "hi"
    /// ```
    #[overload(fn(self: String) -> String = strings::trim)]
    Trim,

    /// The strings of the list joined, with the separator given.
    ///
    /// ```cel
    /// ["a", "b"].join(", ") // "a, b"
    /// ```
    #[overload(fn(self: Strings) -> String = strings::join, size = strings::join_size)]
    #[overload(fn(self: Strings, separator: String) -> String = strings::join, size = strings::join_size)]
    Join,
}

/// CEL functions the profile leaves out, named as such when used.
pub(crate) const LEFT_OUT: [&str; 8] = [
    "matches", "uint", "bytes", "type", "dyn", "format", "quote", "reverse",
];

fn size(_: &dyn Functions, args: &[Value]) -> Result<Value, String> {
    let n = match args.first() {
        Some(Value::String(s)) => s.chars().count(),
        Some(Value::List(items)) => items.len(),
        Some(Value::Map(entries)) => entries.len(),
        _ => 0,
    };
    Ok(Value::Int(i64::try_from(n).unwrap_or(i64::MAX)))
}

fn itself(_: &dyn Functions, args: &[Value]) -> Result<Value, String> {
    Ok(args.first().cloned().unwrap_or(Value::Null))
}

#[allow(
    clippy::cast_possible_truncation,
    reason = "the double is in range, and CEL truncates it"
)]
fn to_int(_: &dyn Functions, args: &[Value]) -> Result<Value, String> {
    let Some(value) = args.first() else {
        return Ok(Value::Null);
    };
    let too_big = || format!("{} doesn't fit an int", value.text());
    match value {
        Value::Uint(u) => i64::try_from(*u).map(Value::Int).map_err(|_| too_big()),
        // -2^63 fits; 2^63 doesn't.
        Value::Double(d)
            if (-9_223_372_036_854_775_808.0..9_223_372_036_854_775_808.0).contains(d) =>
        {
            Ok(Value::Int(d.trunc() as i64))
        }
        Value::Double(_) => Err(too_big()),
        Value::String(s) => s
            .parse()
            .map(Value::Int)
            .map_err(|_| format!("'{s}' is not an int")),
        Value::Timestamp(t) => Ok(Value::Int(
            i64::try_from(t.div_euclid(time::NANOS)).unwrap_or_default(),
        )),
        other => Ok(other.clone()),
    }
}

#[allow(
    clippy::cast_precision_loss,
    reason = "CEL converts an integer to the nearest double"
)]
fn to_double(_: &dyn Functions, args: &[Value]) -> Result<Value, String> {
    match args.first() {
        Some(Value::Int(i)) => Ok(Value::Double(*i as f64)),
        Some(Value::Uint(u)) => Ok(Value::Double(*u as f64)),
        Some(Value::String(s)) => s
            .parse()
            .map(Value::Double)
            .map_err(|_| format!("'{s}' is not a double")),
        other => Ok(other.cloned().unwrap_or(Value::Null)),
    }
}

fn to_string(_: &dyn Functions, args: &[Value]) -> Result<Value, String> {
    Ok(Value::string(
        &args.first().map(Value::text).unwrap_or_default(),
    ))
}

fn to_timestamp(_: &dyn Functions, args: &[Value]) -> Result<Value, String> {
    let s = text(args, 0);
    time::parse_timestamp(s)
        .map(Value::Timestamp)
        .ok_or_else(|| format!("'{s}' is not an RFC 3339 timestamp in CEL's range"))
}

fn to_duration(_: &dyn Functions, args: &[Value]) -> Result<Value, String> {
    let s = text(args, 0);
    time::parse_duration(s)
        .map(Value::Duration)
        .ok_or_else(|| format!("'{s}' is not a duration such as '1h30m'"))
}
