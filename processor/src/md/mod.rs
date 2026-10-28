//! The `md.` functions of spec §8.2: dates in the render's time zone, formatting by the document's language, and the
//! list functions. [`Signatures`] gives their types to the checker; [`Md`], made for one render, evaluates them.

mod lists;
mod locale;

use crate::cel::time::{DAY, NANOS, civil_from_days, days_from_civil, format_date, parse_date};
use crate::cel::{FunctionTypes, Functions, Key, Type, Value};
use locale::Language;
use std::rc::Rc;

/// The time zones of the host: the browser's `Intl`, an app's own zone data. The crate carries none.
pub trait TimeZones {
    /// The offset of `zone` from UTC at `at`, in seconds east of UTC, or `None` when the zone is unknown. `zone` is
    /// `md.tz`: an IANA name such as `Europe/Berlin`, or `UTC`; `at` is in seconds since 1970-01-01T00:00:00Z.
    fn offset(&self, zone: &str, at: i64) -> Option<i32>;
}

/// The `md.` functions, without `md.`.
const NAMES: [&str; 20] = [
    "currency",
    "number",
    "date",
    "plural",
    "today",
    "now",
    "dateOf",
    "addDays",
    "daysBetween",
    "startOf",
    "sort",
    "sortBy",
    "reverse",
    "slice",
    "sum",
    "min",
    "max",
    "unique",
    "groupBy",
    "list",
];

/// The types of the `md.` functions, for checking expressions without a render, as a definition's are when it is
/// loaded.
#[derive(Clone, Copy, Debug, Default)]
pub struct Signatures;

impl FunctionTypes for Signatures {
    fn names(&self) -> &[&'static str] {
        &NAMES
    }

    fn result_type(&self, name: &str, args: &[Type]) -> Option<Type> {
        use Type::{Double, Dyn, Int, List, Map, Record, String, Timestamp};
        let text = |t: &Type| t.fits(&String);
        let number = Type::is_numeric;
        let list = |t: &Type| matches!(t, List(_) | Dyn);
        match (name, args) {
            ("today", []) => Some(String),
            ("now", []) => Some(Timestamp),
            ("dateOf", [t]) if t.fits(&Timestamp) => Some(String),
            ("addDays", [d, n]) if text(d) && number(n) => Some(String),
            ("daysBetween", [a, b]) if text(a) && text(b) => Some(Int),
            ("startOf", [d, unit]) if text(d) && text(unit) => Some(String),
            ("date", [d, style]) if (text(d) || d.fits(&Timestamp)) && text(style) => Some(String),
            ("number", [n, digits]) if number(n) && number(digits) => Some(String),
            ("currency", [n, code]) if number(n) && text(code) => Some(String),
            ("plural", [n, one, other]) if number(n) && text(one) && text(other) => Some(String),
            ("plural", [n, forms]) if number(n) => match forms {
                Map(_, form) => Some((**form).clone()),
                Record(fields) => Some(Type::joined(fields.values().cloned())),
                Dyn => Some(Dyn),
                _ => None,
            },
            ("list", [items]) if list(items) => Some(String),
            ("list", [items, kind]) if list(items) && text(kind) => Some(String),
            ("sort" | "reverse" | "unique", [items]) if list(items) => Some(items.clone()),
            ("sortBy", [items, field]) if list(items) && text(field) => Some(items.clone()),
            ("slice", [items, start, end]) if list(items) && number(start) && number(end) => {
                Some(items.clone())
            }
            ("sum" | "min" | "max", [items]) if numbers(items) => Some(Double),
            ("groupBy", [items, field]) if list(items) && text(field) => {
                Some(Type::map(Dyn, items.clone()))
            }
            _ => None,
        }
    }
}

/// Whether `t` is a list of numbers, or may be one.
fn numbers(t: &Type) -> bool {
    match t {
        Type::List(element) => element.is_numeric(),
        other => *other == Type::Dyn,
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

    /// The date in `md.tz` at `nanos`, in days since 1970-01-01.
    fn local_days(&self, nanos: i128) -> Result<i64, String> {
        let seconds =
            i64::try_from(nanos.div_euclid(NANOS)).map_err(|_| "the timestamp is out of range")?;
        let offset = self
            .zones
            .offset(&self.tz, seconds)
            .ok_or_else(|| format!("the time zone '{}' is unknown", self.tz))?;
        Ok((seconds + i64::from(offset)).div_euclid(DAY))
    }

    /// The first day of the week, month or year that `days` is in; a week starts on the locale's first day.
    fn start_of(&self, days: i64, unit: &str) -> Result<i64, String> {
        let (year, month, _) = civil_from_days(days);
        match unit {
            "week" => {
                // 1970-01-01 was a Thursday: weekday 4, counting Monday as 1.
                let weekday = (days + 3).rem_euclid(7) + 1;
                Ok(days - (weekday - self.language.first_weekday()?).rem_euclid(7))
            }
            "month" => Ok(days_from_civil(year, month, 1)),
            "year" => Ok(days_from_civil(year, 1, 1)),
            other => Err(format!(
                "md.startOf() takes the unit \"week\", \"month\" or \"year\", not \"{other}\""
            )),
        }
    }

    fn plural(&self, n: f64, forms: &Value) -> Result<Value, String> {
        let category = self.language.plural(n)?;
        let Value::Map(forms) = forms else {
            return Err(format!(
                "md.plural() takes a map of forms, not {}",
                forms.kind()
            ));
        };
        let form = |name: &str| forms.get(&Key::String(Rc::from(name))).cloned();
        form(category)
            .or_else(|| form("other"))
            .ok_or_else(|| "md.plural() needs an \"other\" form".to_string())
    }
}

impl FunctionTypes for Md<'_> {
    fn names(&self) -> &[&'static str] {
        Signatures.names()
    }

    fn result_type(&self, name: &str, args: &[Type]) -> Option<Type> {
        Signatures.result_type(name, args)
    }
}

impl Functions for Md<'_> {
    fn call(&self, name: &str, args: &[Value]) -> Result<Value, String> {
        let text = |s: String| Ok(Value::string(&s));
        match (name, args) {
            ("today", []) => text(format_date(self.local_days(self.now)?)),
            ("now", []) => Ok(Value::Timestamp(self.now)),
            ("dateOf", [Value::Timestamp(t)]) => text(format_date(self.local_days(*t)?)),
            ("addDays", [d, n]) => text(format_date(days_of(d)? + whole(n)?)),
            ("daysBetween", [a, b]) => Ok(Value::Int(days_of(b)? - days_of(a)?)),
            ("startOf", [d, Value::String(unit)]) => {
                text(format_date(self.start_of(days_of(d)?, unit)?))
            }
            ("date", [Value::Timestamp(t), Value::String(style)]) => text(
                self.language
                    .date(civil_from_days(self.local_days(*t)?), style)?,
            ),
            ("date", [d, Value::String(style)]) => {
                text(self.language.date(civil_from_days(days_of(d)?), style)?)
            }
            ("number", [n, digits]) => text(self.language.number(number_of(n)?, whole(digits)?)?),
            ("currency", [n, Value::String(code)]) => {
                text(self.language.currency(number_of(n)?, code)?)
            }
            // The shorthand for {"one": one, "other": other}.
            ("plural", [n, one, other]) => {
                let category = self.language.plural(number_of(n)?)?;
                Ok(if category == "one" { one } else { other }.clone())
            }
            ("plural", [n, forms]) => self.plural(number_of(n)?, forms),
            ("list", [Value::List(items)]) => {
                text(self.language.list(&lists::written(items)?, false)?)
            }
            ("list", [Value::List(items), Value::String(kind)]) => match &**kind {
                "and" => text(self.language.list(&lists::written(items)?, false)?),
                "or" => text(self.language.list(&lists::written(items)?, true)?),
                other => Err(format!(
                    "md.list() takes \"and\" or \"or\", not \"{other}\""
                )),
            },
            (_, [Value::List(items), rest @ ..]) => lists::call(name, items, rest),
            _ => {
                let kinds: Vec<&str> = args.iter().map(Value::kind).collect();
                Err(format!("md.{name}() doesn't take ({})", kinds.join(", ")))
            }
        }
    }
}

/// Days since 1970-01-01 of a date value, a `YYYY-MM-DD` string (spec §8.2).
fn days_of(value: &Value) -> Result<i64, String> {
    match value {
        Value::String(s) => {
            parse_date(s).ok_or_else(|| format!("'{s}' is not a date (YYYY-MM-DD)"))
        }
        other => Err(format!("expected a date, not {}", other.kind())),
    }
}

/// A number value as a `double`.
#[allow(clippy::cast_precision_loss)]
pub(crate) fn number_of(value: &Value) -> Result<f64, String> {
    match value {
        Value::Int(i) => Ok(*i as f64),
        Value::Uint(u) => Ok(*u as f64),
        Value::Double(d) => Ok(*d),
        other => Err(format!("expected a number, not {}", other.kind())),
    }
}

/// A whole number value, such as a count of days or digits: numbers from data are `double`s.
#[allow(clippy::cast_possible_truncation)]
pub(crate) fn whole(value: &Value) -> Result<i64, String> {
    let n = number_of(value)?;
    if n.fract() == 0.0 && n.abs() < 9.0e15 {
        Ok(n as i64)
    } else {
        Err(format!("{} is not a whole number", value.text()))
    }
}
