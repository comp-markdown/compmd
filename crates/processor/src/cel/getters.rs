//! CEL's timestamp and duration getters (spec §8.2). A timestamp's parts are read in a time zone: the one given, an
//! IANA name or an offset such as `"+02:00"`, or else `md.tz`.

#![allow(
    clippy::unnecessary_wraps,
    reason = "every getter has an overload's signature, which can fail"
)]

use super::Functions;
use super::overload::text;
use super::time::{self, DAY, NANOS};
use super::value::Value;
use icu::calendar::{Date, Iso};

/// A timestamp's date, second of the day and nanosecond of the second, in a zone.
struct Local {
    date: Date<Iso>,
    second: i64,
    nanos: i128,
}

/// The timestamp argument, the receiver, in the zone of the argument after it, or in `md.tz`.
fn local(render: &dyn Functions, args: &[Value]) -> Result<Local, String> {
    let nanos = match args.first() {
        Some(Value::Timestamp(t)) => *t,
        _ => 0,
    };
    let zone = (args.len() > 1).then(|| text(args, 1));
    // Timestamps are within years 1 to 9999, so their seconds fit.
    let seconds = i64::try_from(nanos.div_euclid(NANOS)).unwrap_or_default();
    let local = seconds + i64::from(render.offset(zone, seconds)?);
    Ok(Local {
        date: time::date(local.div_euclid(DAY)),
        second: local.rem_euclid(DAY),
        nanos: nanos.rem_euclid(NANOS),
    })
}

fn int(n: impl Into<i64>) -> Result<Value, String> {
    Ok(Value::Int(n.into()))
}

pub(crate) fn full_year(render: &dyn Functions, args: &[Value]) -> Result<Value, String> {
    int(local(render, args)?.date.year().extended_year())
}

/// The month, from 0 for January.
pub(crate) fn month(render: &dyn Functions, args: &[Value]) -> Result<Value, String> {
    int(local(render, args)?.date.month().ordinal - 1)
}

/// The day of the month, from 1.
pub(crate) fn date(render: &dyn Functions, args: &[Value]) -> Result<Value, String> {
    int(local(render, args)?.date.day_of_month().0)
}

/// The day of the month, from 0.
pub(crate) fn day_of_month(render: &dyn Functions, args: &[Value]) -> Result<Value, String> {
    int(local(render, args)?.date.day_of_month().0 - 1)
}

/// The day of the week, from 0 for Sunday.
pub(crate) fn day_of_week(render: &dyn Functions, args: &[Value]) -> Result<Value, String> {
    int(local(render, args)?.date.weekday() as u8 % 7)
}

/// The day of the year, from 0.
pub(crate) fn day_of_year(render: &dyn Functions, args: &[Value]) -> Result<Value, String> {
    int(local(render, args)?.date.day_of_year().0 - 1)
}

pub(crate) fn hours(render: &dyn Functions, args: &[Value]) -> Result<Value, String> {
    int(local(render, args)?.second / 3600)
}

pub(crate) fn minutes(render: &dyn Functions, args: &[Value]) -> Result<Value, String> {
    int(local(render, args)?.second / 60 % 60)
}

pub(crate) fn seconds(render: &dyn Functions, args: &[Value]) -> Result<Value, String> {
    int(local(render, args)?.second % 60)
}

pub(crate) fn milliseconds(render: &dyn Functions, args: &[Value]) -> Result<Value, String> {
    int(i64::try_from(local(render, args)?.nanos / 1_000_000).unwrap_or_default())
}

/// A duration in whole units, toward zero.
fn duration_in(args: &[Value], unit: i128) -> Result<Value, String> {
    let nanos = match args.first() {
        Some(Value::Duration(d)) => *d,
        _ => 0,
    };
    // Durations are within ten thousand years, so any count of them fits.
    int(i64::try_from(nanos / unit).unwrap_or_default())
}

/// The whole hours of a duration.
pub(crate) fn duration_hours(_: &dyn Functions, args: &[Value]) -> Result<Value, String> {
    duration_in(args, 3600 * NANOS)
}

/// The whole minutes of a duration.
pub(crate) fn duration_minutes(_: &dyn Functions, args: &[Value]) -> Result<Value, String> {
    duration_in(args, 60 * NANOS)
}

/// The whole seconds of a duration.
pub(crate) fn duration_seconds(_: &dyn Functions, args: &[Value]) -> Result<Value, String> {
    duration_in(args, NANOS)
}

/// The whole milliseconds of a duration.
pub(crate) fn duration_milliseconds(_: &dyn Functions, args: &[Value]) -> Result<Value, String> {
    duration_in(args, NANOS / 1000)
}
