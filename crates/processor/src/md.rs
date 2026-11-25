//! The `md.` functions of spec §8.2: dates in the render's time zone, formatting by the document's language, and the
//! list functions. [`Signatures`] gives their types to the checker; [`Md`], made for one render, evaluates them.

mod lists;
mod locale;
#[cfg(test)]
mod tests;

use crate::cel::overload::{self, Entry, Library, Overload, Param, Returns, entry, example, given};
use crate::cel::time::{self, DAY, NANOS};
use crate::cel::{FunctionTypes, Functions, Type, Value};
use crate::host::Time;
use crate::named::{choices, named};
use icu::calendar::types::Weekday;
use icu::calendar::{Date, Iso};
use locale::{DateStyle, Language, ListKind};
use std::rc::Rc;

/// An `md.` function's implementation: it takes the render and the arguments.
type Call = for<'m> fn(&Md<'m>, &[Value]) -> Result<Value, String>;

named! {
    /// An `md.` function, named without `md.`.
    pub(crate) enum MdFunction {
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
            Self::Today => const { &[Overload::global(&[], R::Date, today)] },
            Self::Now => const { &[Overload::global(&[], R::Timestamp, now)] },
            Self::DateOf => const { &[Overload::global(&[P::Timestamp], R::Date, date_of)] },
            Self::AddDays => {
                const { &[Overload::global(&[P::String, P::Number], R::Date, add_days)] }
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
                const { &[Overload::global(&[P::String, P::String], R::Date, start_of)] }
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
                        Overload::global(&[P::List], R::String, list as Call)
                            .sized(lists::written_size),
                        Overload::global(&[P::List, P::String], R::String, list as Call)
                            .sized(lists::written_size),
                    ]
                }
            }
        }
    }

    /// The spec's examples (§8.2), with values for what they read; the params' names are the spec's.
    #[allow(
        clippy::too_many_lines,
        reason = "one table of every function's documentation"
    )]
    fn entry(self) -> Entry {
        const SALES: &str = r#"{"sales": [{"region": "eu", "amount": 30}, {"region": "us", "amount": 10}, {"region": "eu", "amount": 20}]}"#;
        match self {
            Self::Currency => entry(
                "an amount written in a currency, by `md.lang`",
                &[&["n", "code"]],
                const {
                    &[given(
                        r#"md.currency(price * 10, "USD")"#,
                        &[("price", "12")],
                        r#""$120.00""#,
                    )]
                },
            ),
            Self::Number => entry(
                "a number written with the digits given, by `md.lang`",
                &[&["n", "digits"]],
                const { &[example("md.number(0.183 * 100, 0)", r#""18""#)] },
            ),
            Self::Date => entry(
                "a date or timestamp written in a style, `short`, `medium`, `long` or `full`, by `md.lang`",
                &[&["d", "style"]],
                const {
                    &[given(
                        r#"md.date(md.doc.released, "long")"#,
                        &[("md.doc", r#"{"released": "2026-10-21"}"#)],
                        r#""October 21, 2026""#,
                    )]
                },
            ),
            Self::Plural => entry(
                "the plural form for a number, by `md.lang`: one of two, or by CLDR's plural category, `other` when there is no form for it",
                &[&["n", "one", "other"], &["n", "forms"]],
                const {
                    &[
                        given(
                            r#"md.plural(count, "item", "items")"#,
                            &[("count", "3")],
                            r#""items""#,
                        ),
                        given(
                            r#"md.plural(n, {"one": "файл", "few": "файла", "many": "файлов", "other": "файла"})"#,
                            &[("n", "1")],
                            r#""файл""#,
                        ),
                    ]
                },
            ),
            Self::Today => entry(
                "the render date, a `date`, in the render's time zone (`md.tz`)",
                &[],
                const { &[example("md.today()", r#""2026-10-21""#)] },
            ),
            Self::Now => entry(
                "the render time, a `timestamp`",
                &[],
                const { &[example("md.now()", r#""2026-10-21T10:00:00Z""#)] },
            ),
            Self::DateOf => entry(
                "the `date` of a `timestamp` in the render's time zone (`md.tz`)",
                &[&["t"]],
                const {
                    &[given(
                        "md.daysBetween(md.dateOf(timestamp(md.doc.created)), md.today())",
                        &[("md.doc", r#"{"created": "2026-10-01T08:00:00Z"}"#)],
                        "20",
                    )]
                },
            ),
            Self::AddDays => entry(
                "a `date` some days after another; before it for a negative number",
                &[&["d", "n"]],
                const { &[example("md.addDays(md.today(), -7)", r#""2026-10-14""#)] },
            ),
            Self::DaysBetween => entry(
                "the days from one `date` to another, an `int`, negative when it is before",
                &[&["a", "b"]],
                const {
                    &[given(
                        "md.daysBetween(md.today(), md.doc.launch)",
                        &[("md.doc", r#"{"launch": "2026-11-01"}"#)],
                        "11",
                    )]
                },
            ),
            Self::StartOf => entry(
                "the first `date` of its `\"week\"` (by `md.lang`'s first day), `\"month\"` or `\"year\"`",
                &[&["d", "unit"]],
                const {
                    &[example(
                        r#"md.startOf(md.today(), "week")"#,
                        r#""2026-10-18""#,
                    )]
                },
            ),
            Self::Sort => entry(
                "the list in ascending order; strings by code point",
                &[&["list"]],
                const {
                    &[given(
                        "md.sort(md.doc.tags)",
                        &[("md.doc", r#"{"tags": ["b", "c", "a"]}"#)],
                        r#"["a", "b", "c"]"#,
                    )]
                },
            ),
            Self::SortBy => entry(
                "the list of records in ascending order of a field",
                &[&["list", "field"]],
                const {
                    &[given(
                        r#"md.sortBy(md.data.sales, "amount")"#,
                        &[("md.data", SALES)],
                        r#"[{"region": "us", "amount": 10}, {"region": "eu", "amount": 20}, {"region": "eu", "amount": 30}]"#,
                    )]
                },
            ),
            Self::Reverse => entry(
                "the list reversed",
                &[&["list"]],
                const {
                    &[given(
                        r#"md.reverse(md.sortBy(md.data.sales, "amount"))"#,
                        &[("md.data", SALES)],
                        r#"[{"region": "eu", "amount": 30}, {"region": "eu", "amount": 20}, {"region": "us", "amount": 10}]"#,
                    )]
                },
            ),
            Self::Slice => entry(
                "the elements from a position up to another",
                &[&["list", "start", "end"]],
                const {
                    &[given(
                        "md.slice(rows, 0, 5)",
                        &[("rows", "[1, 2, 3, 4, 5, 6, 7]")],
                        "[1, 2, 3, 4, 5]",
                    )]
                },
            ),
            Self::Sum => entry(
                "the sum of the numbers; `0` for an empty list",
                &[&["list"]],
                const {
                    &[given(
                        "md.sum(md.data.sales.map(s, s.amount))",
                        &[("md.data", SALES)],
                        "60",
                    )]
                },
            ),
            Self::Min => entry(
                "the smallest number, or `null` for an empty list",
                &[&["list"]],
                const {
                    &[given(
                        "md.min(md.data.sales.map(s, s.amount))",
                        &[("md.data", SALES)],
                        "10",
                    )]
                },
            ),
            Self::Max => entry(
                "the largest number, or `null` for an empty list",
                &[&["list"]],
                const {
                    &[given(
                        "md.max(md.data.sales.map(s, s.amount))",
                        &[("md.data", SALES)],
                        "30",
                    )]
                },
            ),
            Self::Unique => entry(
                "each element's first occurrence, in order",
                &[&["list"]],
                const {
                    &[given(
                        "md.unique(md.data.sales.map(s, s.region))",
                        &[("md.data", SALES)],
                        r#"["eu", "us"]"#,
                    )]
                },
            ),
            Self::GroupBy => entry(
                "a map from each value of a field to its records, in order",
                &[&["list", "field"]],
                const {
                    &[given(
                        r#"md.groupBy(md.data.sales, "region")"#,
                        &[("md.data", SALES)],
                        r#"{"eu": [{"region": "eu", "amount": 30}, {"region": "eu", "amount": 20}], "us": [{"region": "us", "amount": 10}]}"#,
                    )]
                },
            ),
            Self::List => entry(
                "the elements written as a list, by `md.lang`'s list pattern; with `\"or\"`, as alternatives",
                &[&["list", "kind"]],
                const {
                    &[given(
                        "md.list(md.doc.contributors)",
                        &[(
                            "md.doc",
                            r#"{"contributors": ["Ana Lima", "Kenji Sato", "Priya Nair"]}"#,
                        )],
                        r#""Ana Lima, Kenji Sato, and Priya Nair""#,
                    )]
                },
            ),
        }
    }
}

/// The types of the `md.` functions, for checking expressions without a render, as a definition's are when it is
/// loaded.
#[derive(Clone, Copy, Debug, Default)]
pub struct Signatures;

impl FunctionTypes for Signatures {
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
    time: &'a dyn Time,
}

impl Functions for Md<'_> {
    fn names(&self) -> &[&'static str] {
        MdFunction::NAMES
    }

    fn call(&self, name: &str, args: &[Value]) -> Result<Value, String> {
        (overload::select::<MdFunction>(name, None, args)?.call)(self, args)
    }

    fn result_size(&self, name: &str, args: &[Value]) -> Option<usize> {
        overload::select::<MdFunction>(name, None, args)
            .ok()?
            .result_size(args)
    }

    fn offset(&self, zone: Option<&str>, at: i64) -> Result<i32, String> {
        let zone = zone.unwrap_or(&self.tz);
        time::parse_offset(zone)
            .or_else(|| self.time.offset(zone, at))
            .ok_or_else(|| format!("the time zone '{zone}' is unknown"))
    }
}

impl<'a> Md<'a> {
    /// The functions for a render at `now` (nanoseconds since 1970-01-01T00:00:00Z), with `md.lang` (a BCP 47 tag)
    /// and `md.tz` (a time zone the host's `time` knows).
    #[must_use]
    pub fn new(now: i128, lang: &str, tz: &str, time: &'a dyn Time) -> Self {
        Self {
            now,
            language: Language::new(lang),
            tz: tz.to_string(),
            time,
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
