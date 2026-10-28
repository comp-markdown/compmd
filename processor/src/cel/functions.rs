//! The profile's functions from CEL's standard definitions and its strings extension (spec §8.2), each with its
//! overloads.

#![allow(
    clippy::unnecessary_wraps,
    reason = "every implementation has an overload's signature, which can fail"
)]

use super::overload::{Library, Overload, Param, Returns, text};
use super::value::Value;
use super::{Functions, getters, strings, time};
use crate::named::named;

/// A function's implementation: it takes the render, for time zones, and the receiver, if any, then the arguments.
pub(crate) type Call = fn(&dyn Functions, &[Value]) -> Result<Value, String>;

named! {
    /// A function of the profile from CEL. The `md.` functions come from outside.
    pub(crate) enum Function {
        Size = "size",
        Int = "int",
        Double = "double",
        String = "string",
        Timestamp = "timestamp",
        Duration = "duration",
        GetFullYear = "getFullYear",
        GetMonth = "getMonth",
        GetDate = "getDate",
        GetDayOfMonth = "getDayOfMonth",
        GetDayOfWeek = "getDayOfWeek",
        GetDayOfYear = "getDayOfYear",
        GetHours = "getHours",
        GetMinutes = "getMinutes",
        GetSeconds = "getSeconds",
        GetMilliseconds = "getMilliseconds",
        Contains = "contains",
        StartsWith = "startsWith",
        EndsWith = "endsWith",
        CharAt = "charAt",
        IndexOf = "indexOf",
        LastIndexOf = "lastIndexOf",
        LowerAscii = "lowerAscii",
        UpperAscii = "upperAscii",
        Replace = "replace",
        Split = "split",
        Substring = "substring",
        Trim = "trim",
        Join = "join",
    }
}

/// CEL functions the profile leaves out, named as such when used.
pub(crate) const LEFT_OUT: [&str; 8] = [
    "matches", "uint", "bytes", "type", "dyn", "format", "quote", "reverse",
];

/// A method of strings.
const fn on_string(params: &'static [Param], returns: Returns, call: Call) -> Overload<Call> {
    Overload::method(Param::String, params, returns, call)
}

/// A getter of timestamps, read in `md.tz` or in the zone given.
const fn on_timestamp(call: Call) -> [Overload<Call>; 2] {
    [
        Overload::method(Param::Timestamp, &[], Returns::Int, call),
        Overload::method(Param::Timestamp, &[Param::String], Returns::Int, call),
    ]
}

impl Library for Function {
    type Call = Call;
    const PREFIX: &'static str = "";

    fn from_name(name: &str) -> Option<Self> {
        Self::from_name(name)
    }

    fn names() -> &'static [&'static str] {
        Self::NAMES
    }

    #[allow(
        clippy::too_many_lines,
        reason = "one table of every function's overloads"
    )]
    fn overloads(self) -> &'static [Overload<Call>] {
        use Param as P;
        use Returns as R;
        match self {
            Self::Size => {
                const {
                    &[
                        Overload::global(&[P::Sized], R::Int, size),
                        Overload::method(P::Sized, &[], R::Int, size),
                    ]
                }
            }
            Self::Int => {
                const {
                    &[
                        Overload::global(&[P::Int], R::Int, to_int),
                        Overload::global(&[P::Uint], R::Int, to_int),
                        Overload::global(&[P::Double], R::Int, to_int),
                        Overload::global(&[P::String], R::Int, to_int),
                        Overload::global(&[P::Timestamp], R::Int, to_int),
                    ]
                }
            }
            Self::Double => {
                const {
                    &[
                        Overload::global(&[P::Number], R::Double, to_double),
                        Overload::global(&[P::String], R::Double, to_double),
                    ]
                }
            }
            Self::String => {
                const {
                    &[
                        Overload::global(&[P::Number], R::String, to_string),
                        Overload::global(&[P::String], R::String, to_string),
                        Overload::global(&[P::Bool], R::String, to_string),
                        Overload::global(&[P::Timestamp], R::String, to_string),
                        Overload::global(&[P::Duration], R::String, to_string),
                    ]
                }
            }
            Self::Timestamp => {
                const {
                    &[
                        Overload::global(&[P::String], R::Timestamp, to_timestamp),
                        Overload::global(&[P::Timestamp], R::Timestamp, itself),
                    ]
                }
            }
            Self::Duration => {
                const {
                    &[
                        Overload::global(&[P::String], R::Duration, to_duration),
                        Overload::global(&[P::Duration], R::Duration, itself),
                    ]
                }
            }
            Self::GetFullYear => const { &on_timestamp(getters::full_year) },
            Self::GetMonth => const { &on_timestamp(getters::month) },
            Self::GetDate => const { &on_timestamp(getters::date) },
            Self::GetDayOfMonth => const { &on_timestamp(getters::day_of_month) },
            Self::GetDayOfWeek => const { &on_timestamp(getters::day_of_week) },
            Self::GetDayOfYear => const { &on_timestamp(getters::day_of_year) },
            Self::GetHours => {
                const { &with_duration(on_timestamp(getters::hours), getters::duration_hours) }
            }
            Self::GetMinutes => {
                const { &with_duration(on_timestamp(getters::minutes), getters::duration_minutes) }
            }
            Self::GetSeconds => {
                const { &with_duration(on_timestamp(getters::seconds), getters::duration_seconds) }
            }
            Self::GetMilliseconds => {
                const {
                    &with_duration(
                        on_timestamp(getters::milliseconds),
                        getters::duration_milliseconds,
                    )
                }
            }
            Self::Contains => const { &[on_string(&[P::String], R::Bool, strings::contains)] },
            Self::StartsWith => const { &[on_string(&[P::String], R::Bool, strings::starts_with)] },
            Self::EndsWith => const { &[on_string(&[P::String], R::Bool, strings::ends_with)] },
            Self::CharAt => const { &[on_string(&[P::Int], R::String, strings::char_at)] },
            Self::IndexOf => {
                const {
                    &[
                        on_string(&[P::String], R::Int, strings::index_of),
                        on_string(&[P::String, P::Int], R::Int, strings::index_of),
                    ]
                }
            }
            Self::LastIndexOf => {
                const {
                    &[
                        on_string(&[P::String], R::Int, strings::last_index_of),
                        on_string(&[P::String, P::Int], R::Int, strings::last_index_of),
                    ]
                }
            }
            Self::LowerAscii => const { &[on_string(&[], R::String, strings::lower_ascii)] },
            Self::UpperAscii => const { &[on_string(&[], R::String, strings::upper_ascii)] },
            Self::Trim => const { &[on_string(&[], R::String, strings::trim)] },
            Self::Replace => {
                const {
                    &[
                        on_string(&[P::String, P::String], R::String, strings::replace),
                        on_string(&[P::String, P::String, P::Int], R::String, strings::replace),
                    ]
                }
            }
            Self::Split => {
                const {
                    &[
                        on_string(&[P::String], R::Strings, strings::split),
                        on_string(&[P::String, P::Int], R::Strings, strings::split),
                    ]
                }
            }
            Self::Substring => {
                const {
                    &[
                        on_string(&[P::Int], R::String, strings::substring),
                        on_string(&[P::Int, P::Int], R::String, strings::substring),
                    ]
                }
            }
            Self::Join => {
                const {
                    &[
                        Overload::method(P::Strings, &[], R::String, strings::join),
                        Overload::method(P::Strings, &[P::String], R::String, strings::join),
                    ]
                }
            }
        }
    }

    fn counts(self) -> bool {
        matches!(
            self,
            Self::CharAt
                | Self::IndexOf
                | Self::LastIndexOf
                | Self::Replace
                | Self::Split
                | Self::Substring
        )
    }
}

/// A timestamp getter's two overloads, and the duration getter of the same name.
const fn with_duration(timestamp: [Overload<Call>; 2], duration: Call) -> [Overload<Call>; 3] {
    let [in_md_tz, in_zone] = timestamp;
    [
        in_md_tz,
        in_zone,
        Overload::method(Param::Duration, &[], Returns::Int, duration),
    ]
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_table_helpers_build_methods() {
        let method = on_string(&[Param::Int], Returns::String, strings::char_at);
        assert_eq!(
            (method.receiver, method.params),
            (Some(Param::String), &[Param::Int][..])
        );
        let [in_md_tz, in_zone, duration] =
            with_duration(on_timestamp(getters::hours), getters::duration_hours);
        assert_eq!(in_md_tz.params, &[]);
        assert_eq!(in_zone.params, &[Param::String]);
        assert_eq!(duration.receiver, Some(Param::Duration));
    }
}
