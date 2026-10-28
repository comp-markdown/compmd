//! The `md.` functions of spec §8.2: dates in the render's time zone, formatting by the document's language, and the
//! list functions. [`Signatures`] gives their types to the checker; [`Md`], made for one render, evaluates them.

mod lists;
mod locale;

use crate::cel::overload::{self, Library, Overload, Param, Returns};
use crate::cel::time::{self, DAY, NANOS};
use crate::cel::{FunctionTypes, Functions, Type, Value};
use crate::named::{choices, named};
use icu::calendar::types::Weekday;
use icu::calendar::{Date, Iso};
use locale::{DateStyle, Language, ListKind};
use std::rc::Rc;

/// The time zones of the host: the browser's `Intl`, an app's own zone data. The crate carries none.
pub trait TimeZones {
    /// The offset of `zone` from UTC at `at`, in seconds east of UTC, or `None` when the zone is unknown. `zone` is
    /// `md.tz`: an IANA name such as `Europe/Berlin`, or `UTC`; `at` is in seconds since 1970-01-01T00:00:00Z.
    fn offset(&self, zone: &str, at: i64) -> Option<i32>;
}

/// An `md.` function's implementation: it takes the render and the arguments.
type Call = for<'m> fn(&Md<'m>, &[Value]) -> Result<Value, String>;

named! {
    /// An `md.` function, named without `md.`.
    enum MdFunction {
        Currency = "currency",
        Number = "number",
        Date = "date",
        Plural = "plural",
        Today = "today",
        Now = "now",
        DateOf = "dateOf",
        AddDays = "addDays",
        DaysBetween = "daysBetween",
        StartOf = "startOf",
        Sort = "sort",
        SortBy = "sortBy",
        Reverse = "reverse",
        Slice = "slice",
        Sum = "sum",
        Min = "min",
        Max = "max",
        Unique = "unique",
        GroupBy = "groupBy",
        List = "list",
    }
}

impl Library for MdFunction {
    type Call = Call;
    const PREFIX: &'static str = "md.";

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
            Self::Currency => {
                const {
                    &[Overload::global(
                        &[P::Number, P::String],
                        R::String,
                        currency,
                    )]
                }
            }
            Self::Number => {
                const { &[Overload::global(&[P::Number, P::Number], R::String, number)] }
            }
            Self::Date => {
                const {
                    &[
                        Overload::global(&[P::Timestamp, P::String], R::String, format_timestamp),
                        Overload::global(&[P::String, P::String], R::String, format_date),
                    ]
                }
            }
            Self::Plural => {
                const {
                    &[
                        Overload::global(
                            &[P::Number, P::String, P::String],
                            R::String,
                            plural_of_two,
                        ),
                        Overload::global(&[P::Number, P::Map], R::Values(1), plural),
                    ]
                }
            }
            Self::Today => const { &[Overload::global(&[], R::String, today)] },
            Self::Now => const { &[Overload::global(&[], R::Timestamp, now)] },
            Self::DateOf => const { &[Overload::global(&[P::Timestamp], R::String, date_of)] },
            Self::AddDays => {
                const {
                    &[Overload::global(
                        &[P::String, P::Number],
                        R::String,
                        add_days,
                    )]
                }
            }
            Self::DaysBetween => {
                const {
                    &[Overload::global(
                        &[P::String, P::String],
                        R::Int,
                        days_between,
                    )]
                }
            }
            Self::StartOf => {
                const {
                    &[Overload::global(
                        &[P::String, P::String],
                        R::String,
                        start_of,
                    )]
                }
            }
            Self::Sort => const { &[Overload::global(&[P::List], R::Same(0), lists::sort)] },
            Self::SortBy => {
                const {
                    &[Overload::global(
                        &[P::List, P::String],
                        R::Same(0),
                        lists::sort_by,
                    )]
                }
            }
            Self::Reverse => const { &[Overload::global(&[P::List], R::Same(0), lists::reverse)] },
            Self::Slice => {
                const {
                    &[Overload::global(
                        &[P::List, P::Number, P::Number],
                        R::Same(0),
                        lists::slice,
                    )]
                }
            }
            Self::Sum => const { &[Overload::global(&[P::Numbers], R::Double, lists::sum)] },
            Self::Min => const { &[Overload::global(&[P::Numbers], R::Double, lists::min)] },
            Self::Max => const { &[Overload::global(&[P::Numbers], R::Double, lists::max)] },
            Self::Unique => const { &[Overload::global(&[P::List], R::Same(0), lists::unique)] },
            Self::GroupBy => {
                const {
                    &[Overload::global(
                        &[P::List, P::String],
                        R::Groups(0),
                        lists::group_by,
                    )]
                }
            }
            Self::List => {
                const {
                    &[
                        Overload::global(&[P::List], R::String, list),
                        Overload::global(&[P::List, P::String], R::String, list),
                    ]
                }
            }
        }
    }
}

/// The types of the `md.` functions, for checking expressions without a render, as a definition's are when it is
/// loaded.
#[derive(Clone, Copy, Debug, Default)]
pub struct Signatures;

impl FunctionTypes for Signatures {
    fn names(&self) -> &[&'static str] {
        MdFunction::NAMES
    }

    fn result_type(&self, name: &str, args: &[Type]) -> Result<Type, String> {
        overload::type_of::<MdFunction>(name, None, args)
    }
}

/// The `md.` functions for one render (spec §8.5): every call to `md.today()` and `md.now()` within it gives the
/// same value.
pub struct Md<'a> {
    /// The render time, in nanoseconds since 1970-01-01T00:00:00Z.
    now: i128,
    language: Language,
    tz: String,
    zones: &'a dyn TimeZones,
}

impl FunctionTypes for Md<'_> {
    fn names(&self) -> &[&'static str] {
        Signatures.names()
    }

    fn result_type(&self, name: &str, args: &[Type]) -> Result<Type, String> {
        Signatures.result_type(name, args)
    }
}

impl Functions for Md<'_> {
    fn call(&self, name: &str, args: &[Value]) -> Result<Value, String> {
        (overload::select::<MdFunction>(name, None, args)?.call)(self, args)
    }

    fn offset(&self, zone: Option<&str>, at: i64) -> Result<i32, String> {
        let zone = zone.unwrap_or(&self.tz);
        time::parse_offset(zone)
            .or_else(|| self.zones.offset(zone, at))
            .ok_or_else(|| format!("the time zone '{zone}' is unknown"))
    }
}

impl<'a> Md<'a> {
    /// The functions for a render at `now` (nanoseconds since 1970-01-01T00:00:00Z), with `md.lang` (a BCP 47 tag)
    /// and `md.tz` (a time zone the host's `zones` know).
    #[must_use]
    pub fn new(now: i128, lang: &str, tz: &str, zones: &'a dyn TimeZones) -> Self {
        Self {
            now,
            language: Language::new(lang),
            tz: tz.to_string(),
            zones,
        }
    }

    /// The date in `md.tz` at `nanos`.
    fn local_date(&self, nanos: i128) -> Result<Date<Iso>, String> {
        let seconds =
            i64::try_from(nanos.div_euclid(NANOS)).map_err(|_| "the timestamp is out of range")?;
        let offset = self.offset(None, seconds)?;
        Ok(time::date((seconds + i64::from(offset)).div_euclid(DAY)))
    }

    /// A date in the style the argument after it names.
    fn formatted_date(&self, date: Date<Iso>, args: &[Value]) -> Result<Value, String> {
        let style = overload::text(args, 1);
        let style = DateStyle::from_name(style).ok_or_else(|| {
            format!(
                "md.date() takes the style {}, not \"{style}\"",
                choices(DateStyle::NAMES)
            )
        })?;
        Ok(Value::string(&self.language.date(date, style)?))
    }
}

fn today(md: &Md, _: &[Value]) -> Result<Value, String> {
    Ok(text(md.local_date(md.now)?))
}

#[allow(
    clippy::unnecessary_wraps,
    reason = "it has an overload's signature, which can fail"
)]
fn now(md: &Md, _: &[Value]) -> Result<Value, String> {
    Ok(Value::Timestamp(md.now))
}

fn date_of(md: &Md, args: &[Value]) -> Result<Value, String> {
    Ok(text(md.local_date(timestamp(args))?))
}

fn add_days(_: &Md, args: &[Value]) -> Result<Value, String> {
    let date = date_arg(args, 0)?;
    Ok(text(time::date(time::days(date) + whole(&args[1])?)))
}

fn days_between(_: &Md, args: &[Value]) -> Result<Value, String> {
    Ok(Value::Int(
        time::days(date_arg(args, 1)?) - time::days(date_arg(args, 0)?),
    ))
}

/// The first day of the week, month or year the date is in; a week starts on the locale's first day.
fn start_of(md: &Md, args: &[Value]) -> Result<Value, String> {
    let date = date_arg(args, 0)?;
    let unit = overload::text(args, 1);
    let (year, month) = (date.year().extended_year(), date.month().ordinal);
    let start = match Unit::from_name(unit) {
        Some(Unit::Week) => {
            let weekday = |day: Weekday| i64::from(day as u8);
            let since =
                (weekday(date.weekday()) - weekday(md.language.first_weekday()?)).rem_euclid(7);
            time::date(time::days(date) - since)
        }
        Some(Unit::Month) => first_day(year, month),
        Some(Unit::Year) => first_day(year, 1),
        None => {
            return Err(format!(
                "md.startOf() takes the unit {}, not \"{unit}\"",
                choices(Unit::NAMES)
            ));
        }
    };
    Ok(text(start))
}

fn format_date(md: &Md, args: &[Value]) -> Result<Value, String> {
    md.formatted_date(date_arg(args, 0)?, args)
}

fn format_timestamp(md: &Md, args: &[Value]) -> Result<Value, String> {
    md.formatted_date(md.local_date(timestamp(args))?, args)
}

fn number(md: &Md, args: &[Value]) -> Result<Value, String> {
    Ok(Value::string(
        &md.language.number(number_arg(args, 0), whole(&args[1])?)?,
    ))
}

fn currency(md: &Md, args: &[Value]) -> Result<Value, String> {
    Ok(Value::string(
        &md.language
            .currency(number_arg(args, 0), overload::text(args, 1))?,
    ))
}

/// The shorthand for `{"one": one, "other": other}`.
fn plural_of_two(md: &Md, args: &[Value]) -> Result<Value, String> {
    let one = md.language.is_one(number_arg(args, 0))?;
    Ok(args[if one { 1 } else { 2 }].clone())
}

fn plural(md: &Md, args: &[Value]) -> Result<Value, String> {
    let empty = Rc::default();
    let forms = match &args[1] {
        Value::Map(forms) => forms,
        _ => &empty,
    };
    md.language.plural_form(number_arg(args, 0), forms)
}

/// The items joined as the locale writes a list: with "and", or the kind the argument after them names.
fn list(md: &Md, args: &[Value]) -> Result<Value, String> {
    let kind = if args.len() > 1 {
        let kind = overload::text(args, 1);
        ListKind::from_name(kind).ok_or_else(|| {
            format!(
                "md.list() takes {}, not \"{kind}\"",
                choices(ListKind::NAMES)
            )
        })?
    } else {
        ListKind::And
    };
    Ok(Value::string(
        &md.language
            .list(&lists::written(overload::list(args, 0))?, kind)?,
    ))
}

named! {
    /// What `md.startOf` finds the start of.
    enum Unit {
        Week = "week",
        Month = "month",
        Year = "year",
    }
}

fn first_day(year: i32, month: u8) -> Date<Iso> {
    Date::try_new_iso(year, month, 1).unwrap_or_else(|_| time::date(0))
}

fn text(date: Date<Iso>) -> Value {
    Value::string(&time::format_date(date))
}

/// The timestamp argument, the first.
fn timestamp(args: &[Value]) -> i128 {
    match args.first() {
        Some(Value::Timestamp(t)) => *t,
        _ => 0,
    }
}

/// The date argument at `i`, a `YYYY-MM-DD` string (spec §8.2).
fn date_arg(args: &[Value], i: usize) -> Result<Date<Iso>, String> {
    let s = overload::text(args, i);
    time::parse_date(s).ok_or_else(|| format!("'{s}' is not a date (YYYY-MM-DD)"))
}

/// A number value as a `double`, or `None` for another value.
#[allow(
    clippy::cast_precision_loss,
    reason = "the md. functions take every number as a double"
)]
pub(crate) fn number_of(value: &Value) -> Option<f64> {
    match value {
        Value::Int(i) => Some(*i as f64),
        Value::Uint(u) => Some(*u as f64),
        Value::Double(d) => Some(*d),
        _ => None,
    }
}

/// The number argument at `i`. Overloads are selected by their arguments' values, so it is one.
fn number_arg(args: &[Value], i: usize) -> f64 {
    args.get(i).and_then(number_of).unwrap_or(f64::NAN)
}

/// A whole number value, such as a count of days or digits: numbers from data are `double`s.
#[allow(
    clippy::cast_possible_truncation,
    reason = "the double is whole and far inside the i64 range"
)]
pub(crate) fn whole(value: &Value) -> Result<i64, String> {
    let n = number_of(value).unwrap_or(f64::NAN);
    if n.fract() == 0.0 && n.abs() < 9.0e15 {
        Ok(n as i64)
    } else {
        Err(format!("{} is not a whole number", value.text()))
    }
}
