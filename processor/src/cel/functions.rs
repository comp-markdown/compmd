//! The profile's functions from CEL's standard definitions and its strings extension (spec §8.2), each with its
//! overloads.

#![allow(
    clippy::unnecessary_wraps,
    reason = "every implementation has an overload's signature, which can fail"
)]

use super::overload::{Overload, Param, Returns};
use super::strings;
use super::time;
use super::value::Value;
use crate::named::named;

/// A function's implementation: it takes the receiver, if any, then the arguments.
pub(crate) type Call = fn(&[Value]) -> Result<Value, String>;

named! {
    /// A function of the profile from CEL. The `md.` functions come from outside.
    pub(crate) enum Function {
        Size = "size",
        Int = "int",
        Double = "double",
        String = "string",
        Timestamp = "timestamp",
        Duration = "duration",
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
pub(crate) const LEFT_OUT: [&str; 18] = [
    "matches",
    "uint",
    "bytes",
    "type",
    "dyn",
    "format",
    "quote",
    "reverse",
    "getDate",
    "getDayOfMonth",
    "getDayOfWeek",
    "getDayOfYear",
    "getFullYear",
    "getHours",
    "getMilliseconds",
    "getMinutes",
    "getMonth",
    "getSeconds",
];

impl Function {
    /// Its overloads.
    #[allow(
        clippy::too_many_lines,
        reason = "one table of every function's overloads"
    )]
    pub(crate) fn overloads(self) -> &'static [Overload<Call>] {
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
            Self::Contains => {
                const {
                    &[Overload::method(
                        P::String,
                        &[P::String],
                        R::Bool,
                        strings::contains,
                    )]
                }
            }
            Self::StartsWith => {
                const {
                    &[Overload::method(
                        P::String,
                        &[P::String],
                        R::Bool,
                        strings::starts_with,
                    )]
                }
            }
            Self::EndsWith => {
                const {
                    &[Overload::method(
                        P::String,
                        &[P::String],
                        R::Bool,
                        strings::ends_with,
                    )]
                }
            }
            Self::CharAt => {
                const {
                    &[Overload::method(
                        P::String,
                        &[P::Int],
                        R::String,
                        strings::char_at,
                    )]
                }
            }
            Self::IndexOf => {
                const {
                    &[
                        Overload::method(P::String, &[P::String], R::Int, strings::index_of),
                        Overload::method(
                            P::String,
                            &[P::String, P::Int],
                            R::Int,
                            strings::index_of,
                        ),
                    ]
                }
            }
            Self::LastIndexOf => {
                const {
                    &[
                        Overload::method(P::String, &[P::String], R::Int, strings::last_index_of),
                        Overload::method(
                            P::String,
                            &[P::String, P::Int],
                            R::Int,
                            strings::last_index_of,
                        ),
                    ]
                }
            }
            Self::LowerAscii => {
                const {
                    &[Overload::method(
                        P::String,
                        &[],
                        R::String,
                        strings::lower_ascii,
                    )]
                }
            }
            Self::UpperAscii => {
                const {
                    &[Overload::method(
                        P::String,
                        &[],
                        R::String,
                        strings::upper_ascii,
                    )]
                }
            }
            Self::Trim => const { &[Overload::method(P::String, &[], R::String, strings::trim)] },
            Self::Replace => {
                const {
                    &[
                        Overload::method(
                            P::String,
                            &[P::String, P::String],
                            R::String,
                            strings::replace,
                        ),
                        Overload::method(
                            P::String,
                            &[P::String, P::String, P::Int],
                            R::String,
                            strings::replace,
                        ),
                    ]
                }
            }
            Self::Split => {
                const {
                    &[
                        Overload::method(P::String, &[P::String], R::Strings, strings::split),
                        Overload::method(
                            P::String,
                            &[P::String, P::Int],
                            R::Strings,
                            strings::split,
                        ),
                    ]
                }
            }
            Self::Substring => {
                const {
                    &[
                        Overload::method(P::String, &[P::Int], R::String, strings::substring),
                        Overload::method(
                            P::String,
                            &[P::Int, P::Int],
                            R::String,
                            strings::substring,
                        ),
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

    /// Whether its `int` arguments are positions or counts, which a `double` from data is often given for.
    pub(crate) fn counts(self) -> bool {
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

fn size(args: &[Value]) -> Result<Value, String> {
    let n = match args.first() {
        Some(Value::String(s)) => s.chars().count(),
        Some(Value::List(items)) => items.len(),
        Some(Value::Map(entries)) => entries.len(),
        _ => 0,
    };
    Ok(Value::Int(i64::try_from(n).unwrap_or(i64::MAX)))
}

fn itself(args: &[Value]) -> Result<Value, String> {
    Ok(args.first().cloned().unwrap_or(Value::Null))
}

#[allow(
    clippy::cast_possible_truncation,
    reason = "the double is in range, and CEL truncates it"
)]
fn to_int(args: &[Value]) -> Result<Value, String> {
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
fn to_double(args: &[Value]) -> Result<Value, String> {
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

fn to_string(args: &[Value]) -> Result<Value, String> {
    Ok(Value::string(
        &args.first().map(Value::text).unwrap_or_default(),
    ))
}

fn to_timestamp(args: &[Value]) -> Result<Value, String> {
    let s = super::overload::text(args, 0);
    time::parse_timestamp(s)
        .map(Value::Timestamp)
        .ok_or_else(|| format!("'{s}' is not an RFC 3339 timestamp in CEL's range"))
}

fn to_duration(args: &[Value]) -> Result<Value, String> {
    let s = super::overload::text(args, 0);
    time::parse_duration(s)
        .map(Value::Duration)
        .ok_or_else(|| format!("'{s}' is not a duration such as '1h30m'"))
}
