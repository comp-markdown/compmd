//! The `md.` functions of spec §8.2: dates in the render's time zone, formatting by the document's language, and the
//! list functions. [`Signatures`] gives their types to the checker; [`Md`], made for one render, evaluates them.

mod lists;
mod locale;
#[cfg(test)]
mod tests;

use crate::cel::library;
use crate::cel::time::{self, DAY, NANOS};
use crate::cel::{FunctionTypes, Functions, Type, Value};
use crate::host::Time;
use crate::named::{choices, named};
use compmd_macros::library;
use icu::calendar::types::Weekday;
use icu::calendar::{Date, Iso};
use locale::{DateStyle, Language, ListKind};
use std::rc::Rc;

/// An `md.` function's implementation: it takes the render and the arguments.
type Call = for<'m> fn(&Md<'m>, &[Value]) -> Result<Value, String>;

/// The data the examples of the list functions read, as `md.data`.
const SALES: &str = r#"{"sales": [{"region": "eu", "amount": 30}, {"region": "us", "amount": 10}, {"region": "eu", "amount": 20}]}"#;

/// An `md.` function, named without `md.`. Its examples are the spec's (§8.2), and its params' names too.
#[library(prefix = "md.", call = Call)]
pub(crate) enum MdFunction {
    /// An amount written in a currency, by `md.lang`.
    ///
    /// ```cel
    /// let price = 12
    /// md.currency(price * 10, "USD") // "$120.00"
    /// ```
    #[overload(fn(n: Number, code: String) -> String = currency)]
    Currency,

    /// A number written with the digits given, by `md.lang`.
    ///
    /// ```cel
    /// md.number(0.183 * 100, 0) // "18"
    /// ```
    #[overload(fn(n: Number, digits: Number) -> String = number)]
    Number,

    /// A date or timestamp written in a style, `short`, `medium`, `long` or `full`, by `md.lang`.
    ///
    /// ```cel
    /// let md.doc = {"released": "2026-10-21"}
    /// md.date(md.doc.released, "long") // "October 21, 2026"
    /// ```
    #[overload(fn(d: Timestamp, style: String) -> String = format_timestamp)]
    #[overload(fn(d: String, style: String) -> String = format_date)]
    Date,

    /// The plural form for a number, by `md.lang`: one of two, or by CLDR's plural category, `other` when there is
    /// no form for it.
    ///
    /// ```cel
    /// let count = 3
    /// md.plural(count, "item", "items") // "items"
    /// let n = 1
    /// md.plural(n, {"one": "файл", "few": "файла", "many": "файлов", "other": "файла"}) // "файл"
    /// ```
    #[overload(fn(n: Number, one: String, other: String) -> String = plural_of_two)]
    #[overload(fn(n: Number, forms: Map) -> Values(1) = plural)]
    Plural,

    /// The render date, a `date`, in the render's time zone (`md.tz`).
    ///
    /// ```cel
    /// md.today() // "2026-10-21"
    /// ```
    #[overload(fn() -> Date = today)]
    Today,

    /// The render time, a `timestamp`.
    ///
    /// ```cel
    /// md.now() // "2026-10-21T10:00:00Z"
    /// ```
    #[overload(fn() -> Timestamp = now)]
    Now,

    /// The `date` of a `timestamp` in the render's time zone (`md.tz`).
    ///
    /// ```cel
    /// let md.doc = {"created": "2026-10-01T08:00:00Z"}
    /// md.daysBetween(md.dateOf(timestamp(md.doc.created)), md.today()) // 20
    /// ```
    #[overload(fn(t: Timestamp) -> Date = date_of)]
    DateOf,

    /// A `date` some days after another; before it for a negative number.
    ///
    /// ```cel
    /// md.addDays(md.today(), -7) // "2026-10-14"
    /// ```
    #[overload(fn(d: String, n: Number) -> Date = add_days)]
    AddDays,

    /// The days from one `date` to another, an `int`, negative when it is before.
    ///
    /// ```cel
    /// let md.doc = {"launch": "2026-11-01"}
    /// md.daysBetween(md.today(), md.doc.launch) // 11
    /// ```
    #[overload(fn(a: String, b: String) -> Int = days_between)]
    DaysBetween,

    /// The first `date` of its `"week"` (by `md.lang`'s first day), `"month"` or `"year"`.
    ///
    /// ```cel
    /// md.startOf(md.today(), "week") // "2026-10-18"
    /// ```
    #[overload(fn(d: String, unit: String) -> Date = start_of)]
    StartOf,

    /// The list in ascending order; strings by code point.
    ///
    /// ```cel
    /// let md.doc = {"tags": ["b", "c", "a"]}
    /// md.sort(md.doc.tags) // ["a", "b", "c"]
    /// ```
    #[overload(fn(list: List) -> Same(0) = lists::sort)]
    Sort,

    /// The list of records in ascending order of a field.
    ///
    /// ```cel
    /// let md.data = SALES
    /// md.sortBy(md.data.sales, "amount") // [{"region": "us", "amount": 10}, {"region": "eu", "amount": 20}, {"region": "eu", "amount": 30}]
    /// ```
    #[overload(fn(list: List, field: String) -> Same(0) = lists::sort_by)]
    SortBy,

    /// The list reversed.
    ///
    /// ```cel
    /// let md.data = SALES
    /// md.reverse(md.sortBy(md.data.sales, "amount")) // [{"region": "eu", "amount": 30}, {"region": "eu", "amount": 20}, {"region": "us", "amount": 10}]
    /// ```
    #[overload(fn(list: List) -> Same(0) = lists::reverse)]
    Reverse,

    /// The elements from a position up to another.
    ///
    /// ```cel
    /// let rows = [1, 2, 3, 4, 5, 6, 7]
    /// md.slice(rows, 0, 5) // [1, 2, 3, 4, 5]
    /// ```
    #[overload(fn(list: List, start: Number, end: Number) -> Same(0) = lists::slice)]
    Slice,

    /// The sum of the numbers; `0` for an empty list.
    ///
    /// ```cel
    /// let md.data = SALES
    /// md.sum(md.data.sales.map(s, s.amount)) // 60
    /// ```
    #[overload(fn(list: Numbers) -> Double = lists::sum)]
    Sum,

    /// The smallest number, or `null` for an empty list.
    ///
    /// ```cel
    /// let md.data = SALES
    /// md.min(md.data.sales.map(s, s.amount)) // 10
    /// ```
    #[overload(fn(list: Numbers) -> Double = lists::min)]
    Min,

    /// The largest number, or `null` for an empty list.
    ///
    /// ```cel
    /// let md.data = SALES
    /// md.max(md.data.sales.map(s, s.amount)) // 30
    /// ```
    #[overload(fn(list: Numbers) -> Double = lists::max)]
    Max,

    /// Each element's first occurrence, in order.
    ///
    /// ```cel
    /// let md.data = SALES
    /// md.unique(md.data.sales.map(s, s.region)) // ["eu", "us"]
    /// ```
    #[overload(fn(list: List) -> Same(0) = lists::unique)]
    Unique,

    /// A map from each value of a field to its records, in order.
    ///
    /// ```cel
    /// let md.data = SALES
    /// md.groupBy(md.data.sales, "region") // {"eu": [{"region": "eu", "amount": 30}, {"region": "eu", "amount": 20}], "us": [{"region": "us", "amount": 10}]}
    /// ```
    #[overload(fn(list: List, field: String) -> Groups(0) = lists::group_by)]
    GroupBy,

    /// The elements written as a list, by `md.lang`'s list pattern; with `"or"`, as alternatives.
    ///
    /// ```cel
    /// let md.doc = {"contributors": ["Ana Lima", "Kenji Sato", "Priya Nair"]}
    /// md.list(md.doc.contributors) // "Ana Lima, Kenji Sato, and Priya Nair"
    /// ```
    #[overload(fn(list: List) -> String = list, size = lists::written_size)]
    #[overload(fn(list: List, kind: String) -> String = list, size = lists::written_size)]
    List,
}

/// The types of the `md.` functions, for checking expressions without a render, as a definition's are when it is
/// loaded.
#[derive(Clone, Copy, Debug, Default)]
pub struct Signatures;

impl FunctionTypes for Signatures {
    fn result_type(&self, name: &str, args: &[Type]) -> Result<Type, String> {
        library::type_of::<MdFunction>(name, None, args)
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
        (library::select::<MdFunction>(name, None, args)?.call)(self, args)
    }

    fn result_size(&self, name: &str, args: &[Value]) -> Option<usize> {
        library::select::<MdFunction>(name, None, args)
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
        let style = library::text(args, 1);
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
    let unit = library::text(args, 1);
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
            .currency(number_arg(args, 0), library::text(args, 1))?,
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
        let kind = library::text(args, 1);
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
            .list(&lists::written(library::list(args, 0))?, kind)?,
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
    let s = library::text(args, i);
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
