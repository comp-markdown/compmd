//! The evaluator: CEL's semantics for the profile, with errors at their node's column and a work budget.

use super::ast::{Expr, Macro};
use super::time;
use super::value::{Key, Value};
use super::{Error, Functions, Values, strings};
use indexmap::IndexMap;
use std::rc::Rc;

/// The steps an evaluation may take: one for each node evaluated, and one for each character or element of a string
/// or list it builds. A document's expressions take far fewer (a filter takes about six steps a record), and a
/// runaway one is stopped.
pub const BUDGET: u64 = 100_000;

/// Evaluates `expr` within `budget` steps.
pub(crate) fn evaluate(
    expr: &Expr,
    values: &Values,
    functions: &dyn Functions,
    budget: u64,
) -> Result<Value, Error> {
    Evaluator {
        values,
        functions,
        locals: Vec::new(),
        budget,
    }
    .eval(expr)
}

/// Integer arithmetic that fails on overflow and on division by zero.
macro_rules! checked {
    ($op:expr, $x:expr, $y:expr, $at:expr, $wrap:path, $type:literal) => {{
        let (x, y) = ($x, $y);
        let result = match $op {
            "_+_" => x.checked_add(y),
            "_-_" => x.checked_sub(y),
            "_*_" => x.checked_mul(y),
            "_/_" | "_%_" if y == 0 => return Err(Error::new($at, "division by zero")),
            "_/_" => x.checked_div(y),
            _ => x.checked_rem(y),
        };
        result
            .map($wrap)
            .ok_or_else(|| Error::new($at, concat!($type, " overflow")))
    }};
}

struct Evaluator<'a> {
    values: &'a Values,
    functions: &'a dyn Functions,
    /// Comprehension variables, innermost last.
    locals: Vec<(&'a str, Value)>,
    budget: u64,
}

impl<'a> Evaluator<'a> {
    fn spend(&mut self, at: usize, steps: usize) -> Result<(), Error> {
        let steps = u64::try_from(steps).unwrap_or(u64::MAX);
        let Some(rest) = self.budget.checked_sub(steps) else {
            self.budget = 0;
            return Err(Error::new(at, "the expression takes too much work"));
        };
        self.budget = rest;
        Ok(())
    }

    /// Spends a step for each character or element of a string or list that was built.
    fn built(&mut self, at: usize, value: Value) -> Result<Value, Error> {
        match &value {
            Value::String(s) => self.spend(at, s.chars().count())?,
            Value::List(items) => self.spend(at, items.len())?,
            _ => {}
        }
        Ok(value)
    }

    fn eval(&mut self, expr: &'a Expr) -> Result<Value, Error> {
        self.spend(expr.at(), 1)?;
        match expr {
            Expr::Literal(literal, _) => Ok(Value::from(literal)),
            Expr::Ident(name, at) => self
                .lookup(name)
                .ok_or_else(|| Error::new(*at, format!("unknown name '{name}'"))),
            Expr::Select(operand, field, at) => match self.eval(operand)? {
                Value::Map(entries) => entries
                    .get(&Key::String(Rc::from(field.as_str())))
                    .cloned()
                    .ok_or_else(|| Error::new(*at, format!("no field '{field}'"))),
                other => Err(Error::new(*at, format!("{} has no fields", other.kind()))),
            },
            Expr::Index(operand, index, at) => {
                let (operand, index) = (self.eval(operand)?, self.eval(index)?);
                self::index(&operand, &index, *at)
            }
            Expr::Call {
                target,
                name,
                args,
                at,
            } => self.call(target.as_deref(), name, args, *at),
            Expr::And(left, right, at) => self.logic(left, right, false, *at),
            Expr::Or(left, right, at) => self.logic(left, right, true, *at),
            Expr::Conditional(condition, then, otherwise, at) => {
                if self.boolean(condition, *at)? {
                    self.eval(then)
                } else {
                    self.eval(otherwise)
                }
            }
            Expr::List(items, at) => {
                let items = items
                    .iter()
                    .map(|item| self.eval(item))
                    .collect::<Result<_, _>>()?;
                self.built(*at, Value::list(items))
            }
            Expr::Map(entries, _) => {
                let mut map = IndexMap::new();
                for (key, value) in entries {
                    let key_value = self.eval(key)?;
                    let Some(map_key) = Key::of(&key_value) else {
                        return Err(Error::new(
                            key.at(),
                            format!("a map key can't be a {}", key_value.kind()),
                        ));
                    };
                    let value = self.eval(value)?;
                    if map.insert(map_key, value).is_some() {
                        return Err(Error::new(key.at(), "the same key twice"));
                    }
                }
                Ok(Value::Map(Rc::new(map)))
            }
            Expr::Has(operand, field, at) => match self.eval(operand)? {
                Value::Map(entries) => Ok(Value::Bool(
                    entries.contains_key(&Key::String(Rc::from(field.as_str()))),
                )),
                other => Err(Error::new(
                    *at,
                    format!("has() needs a map, not {}", other.kind()),
                )),
            },
            Expr::Comprehension {
                kind,
                range,
                var,
                filter,
                body,
                at,
            } => self.comprehension(*kind, range, var, filter.as_deref(), body, *at),
        }
    }

    fn lookup(&self, name: &str) -> Option<Value> {
        let local = self
            .locals
            .iter()
            .rev()
            .find(|(n, _)| *n == name)
            .map(|(_, v)| v);
        local.or_else(|| self.values.get(name)).cloned()
    }

    fn boolean(&mut self, expr: &'a Expr, at: usize) -> Result<bool, Error> {
        match self.eval(expr)? {
            Value::Bool(b) => Ok(b),
            other => Err(Error::new(
                at,
                format!("expected a bool, not {}", other.kind()),
            )),
        }
    }

    /// `&&` (`decisive` false) and `||` (true): either side alone can decide, so an error on one side is absorbed
    /// when the other decides, as in CEL.
    fn logic(
        &mut self,
        left: &'a Expr,
        right: &'a Expr,
        decisive: bool,
        at: usize,
    ) -> Result<Value, Error> {
        let left = self.boolean(left, at);
        if left == Ok(decisive) {
            return Ok(Value::Bool(decisive));
        }
        match (left, self.boolean(right, at)) {
            (_, Ok(b)) if b == decisive => Ok(Value::Bool(decisive)),
            (Err(error), _) | (_, Err(error)) => Err(error),
            _ => Ok(Value::Bool(!decisive)),
        }
    }

    fn comprehension(
        &mut self,
        kind: Macro,
        range: &'a Expr,
        var: &'a str,
        filter: Option<&'a Expr>,
        body: &'a Expr,
        at: usize,
    ) -> Result<Value, Error> {
        let items = match self.eval(range)? {
            Value::List(items) => items,
            Value::Map(entries) => Rc::new(entries.keys().map(Key::to_value).collect()),
            other => {
                return Err(Error::new(
                    at,
                    format!(
                        "{} needs a list or a map, not {}",
                        kind.name(),
                        other.kind()
                    ),
                ));
            }
        };
        let (mut results, mut count, mut error) = (Vec::new(), 0, None);
        for item in items.iter() {
            self.locals.push((var, item.clone()));
            let outcome = self.iteration(kind, filter, body, at);
            self.locals.pop();
            match (kind, outcome) {
                (Macro::Map, Ok(Some(value))) => results.push(value),
                (Macro::Filter, Ok(Some(Value::Bool(true)))) => results.push(item.clone()),
                (Macro::All, Ok(Some(Value::Bool(false)))) => return Ok(Value::Bool(false)),
                (Macro::Exists, Ok(Some(Value::Bool(true)))) => return Ok(Value::Bool(true)),
                (Macro::ExistsOne, Ok(Some(Value::Bool(true)))) => count += 1,
                // `all` and `exists` absorb an error when another element decides, as `&&` and `||` do.
                (Macro::All | Macro::Exists, Err(e)) => {
                    error.get_or_insert(e);
                }
                (_, Err(e)) => return Err(e),
                _ => {}
            }
        }
        if let Some(error) = error {
            return Err(error);
        }
        match kind {
            Macro::All => Ok(Value::Bool(true)),
            Macro::Exists => Ok(Value::Bool(false)),
            Macro::ExistsOne => Ok(Value::Bool(count == 1)),
            Macro::Filter | Macro::Map => self.built(at, Value::list(results)),
        }
    }

    /// One element: the body's value, or `None` when `map`'s filter leaves the element out.
    fn iteration(
        &mut self,
        kind: Macro,
        filter: Option<&'a Expr>,
        body: &'a Expr,
        at: usize,
    ) -> Result<Option<Value>, Error> {
        if let Some(filter) = filter
            && !self.boolean(filter, at)?
        {
            return Ok(None);
        }
        match kind {
            Macro::Map => self.eval(body).map(Some),
            _ => self.boolean(body, at).map(|b| Some(Value::Bool(b))),
        }
    }

    fn call(
        &mut self,
        target: Option<&'a Expr>,
        name: &'a str,
        args: &'a [Expr],
        at: usize,
    ) -> Result<Value, Error> {
        let md = Expr::is_md_call(target, name, self.functions.names());
        let target = if md {
            None
        } else {
            target.map(|t| self.eval(t)).transpose()?
        };
        let args = args
            .iter()
            .map(|a| self.eval(a))
            .collect::<Result<Vec<_>, _>>()?;
        let result = if md {
            self.functions
                .call(name, &args)
                .map_err(|message| Error::new(at, message))?
        } else if let Some(result) = operator(name, &args, at) {
            result?
        } else {
            function(name, target.as_ref(), &args, at)?
        };
        self.built(at, result)
    }
}

/// An operator's value: `None` when `name` isn't an operator.
fn operator(name: &str, args: &[Value], at: usize) -> Option<Result<Value, Error>> {
    Some(match (name, args) {
        ("!_", [Value::Bool(b)]) => Ok(Value::Bool(!b)),
        ("!_", [other]) => Err(Error::new(
            at,
            format!("'!' can't be applied to {}", other.kind()),
        )),
        ("-_", [operand]) => negate(operand, at),
        ("_+_" | "_-_" | "_*_" | "_/_" | "_%_", [a, b]) => arithmetic(name, a, b, at),
        ("_<_" | "_<=_" | "_>_" | "_>=_", [a, b]) => compare(name, a, b, at),
        ("_==_", [a, b]) => Ok(Value::Bool(a.equals(b))),
        ("_!=_", [a, b]) => Ok(Value::Bool(!a.equals(b))),
        ("@in", [a, b]) => contains(a, b, at),
        _ => return None,
    })
}

/// Arithmetic, with the profile's `int` and `double` mixing: an `int` next to a `double` makes a `double`.
fn arithmetic(op: &str, a: &Value, b: &Value, at: usize) -> Result<Value, Error> {
    match (op, a, b) {
        (_, Value::Int(x), Value::Int(y)) => checked!(op, *x, *y, at, Value::Int, "int"),
        (_, Value::Uint(x), Value::Uint(y)) => checked!(op, *x, *y, at, Value::Uint, "uint"),
        ("_+_", Value::String(x), Value::String(y)) => Ok(Value::string(&format!("{x}{y}"))),
        ("_+_", Value::List(x), Value::List(y)) => {
            Ok(Value::list(x.iter().chain(y.iter()).cloned().collect()))
        }
        ("_+_", Value::Timestamp(t), Value::Duration(d))
        | ("_+_", Value::Duration(d), Value::Timestamp(t)) => timestamp(t + d, at),
        ("_-_", Value::Timestamp(t), Value::Duration(d)) => timestamp(t - d, at),
        ("_-_", Value::Timestamp(t), Value::Timestamp(u)) => duration(t - u, at),
        ("_+_", Value::Duration(d), Value::Duration(e)) => duration(d + e, at),
        ("_-_", Value::Duration(d), Value::Duration(e)) => duration(d - e, at),
        _ => match (float(a), float(b)) {
            (Some(x), Some(y)) if op != "_%_" => Ok(Value::Double(match op {
                "_+_" => x + y,
                "_-_" => x - y,
                "_*_" => x * y,
                _ => x / y,
            })),
            _ => Err(Error::new(
                at,
                format!(
                    "'{}' can't be applied to {} and {}",
                    op.trim_matches('_'),
                    a.kind(),
                    b.kind()
                ),
            )),
        },
    }
}

/// An `int` or a `double` as a `double`, for the profile's mixed arithmetic.
#[allow(clippy::cast_precision_loss)]
fn float(value: &Value) -> Option<f64> {
    match value {
        Value::Int(i) => Some(*i as f64),
        Value::Double(d) => Some(*d),
        _ => None,
    }
}

fn timestamp(nanos: i128, at: usize) -> Result<Value, Error> {
    if time::timestamp_in_range(nanos) {
        Ok(Value::Timestamp(nanos))
    } else {
        Err(Error::new(at, "timestamp out of range"))
    }
}

fn duration(nanos: i128, at: usize) -> Result<Value, Error> {
    if time::duration_in_range(nanos) {
        Ok(Value::Duration(nanos))
    } else {
        Err(Error::new(at, "duration out of range"))
    }
}

fn negate(operand: &Value, at: usize) -> Result<Value, Error> {
    match operand {
        Value::Int(i) => i
            .checked_neg()
            .map(Value::Int)
            .ok_or_else(|| Error::new(at, "int overflow")),
        Value::Double(d) => Ok(Value::Double(-d)),
        Value::Duration(d) => duration(-d, at),
        other => Err(Error::new(
            at,
            format!("'-' can't be applied to {}", other.kind()),
        )),
    }
}

fn compare(op: &str, a: &Value, b: &Value, at: usize) -> Result<Value, Error> {
    let ordering = a.order(b).map_err(|()| {
        Error::new(
            at,
            format!(
                "'{}' can't be applied to {} and {}",
                op.trim_matches('_'),
                a.kind(),
                b.kind()
            ),
        )
    })?;
    // A `NaN` is neither less, equal nor greater.
    Ok(Value::Bool(ordering.is_some_and(|ordering| match op {
        "_<_" => ordering.is_lt(),
        "_<=_" => ordering.is_le(),
        "_>_" => ordering.is_gt(),
        _ => ordering.is_ge(),
    })))
}

fn contains(element: &Value, collection: &Value, at: usize) -> Result<Value, Error> {
    match collection {
        Value::List(items) => Ok(Value::Bool(items.iter().any(|item| element.equals(item)))),
        Value::Map(entries) => Ok(Value::Bool(
            Key::to_find(element).is_some_and(|key| entries.contains_key(&key)),
        )),
        other => Err(Error::new(
            at,
            format!("'in' needs a list or a map, not {}", other.kind()),
        )),
    }
}

fn index(operand: &Value, index: &Value, at: usize) -> Result<Value, Error> {
    match (operand, index) {
        (Value::List(items), Value::Int(i)) => usize::try_from(*i)
            .ok()
            .and_then(|i| items.get(i))
            .cloned()
            .ok_or_else(|| {
                Error::new(
                    at,
                    format!("index {i} is out of range for a list of {}", items.len()),
                )
            }),
        (Value::List(_), other) => Err(Error::new(
            at,
            format!("a list index must be an int, not {}", other.kind()),
        )),
        (Value::Map(entries), key) => Key::to_find(key)
            .and_then(|k| entries.get(&k))
            .cloned()
            .ok_or_else(|| Error::new(at, format!("no key {}", key.to_json()))),
        (other, _) => Err(Error::new(at, format!("{} can't be indexed", other.kind()))),
    }
}

/// The profile's functions from CEL: `size`, the conversions, and the string functions.
fn function(name: &str, target: Option<&Value>, args: &[Value], at: usize) -> Result<Value, Error> {
    match (name, target, args) {
        ("size", None, [x]) | ("size", Some(x), []) => size(x, at),
        ("int", None, [x]) => to_int(x, at),
        ("double", None, [x]) => to_double(x, at),
        ("string", None, [x]) => to_string(x, at),
        ("timestamp", None, [Value::String(s)]) => time::parse_timestamp(s)
            .map(Value::Timestamp)
            .ok_or_else(|| {
                Error::new(
                    at,
                    format!("'{s}' is not an RFC 3339 timestamp in CEL's range"),
                )
            }),
        ("duration", None, [Value::String(s)]) => time::parse_duration(s)
            .map(Value::Duration)
            .ok_or_else(|| Error::new(at, format!("'{s}' is not a duration such as '1h30m'"))),
        ("timestamp", None, [t @ Value::Timestamp(_)])
        | ("duration", None, [t @ Value::Duration(_)]) => Ok(t.clone()),
        ("join", Some(Value::List(items)), []) => join(items, "", at),
        ("join", Some(Value::List(items)), [Value::String(separator)]) => {
            join(items, separator, at)
        }
        (_, Some(Value::String(s)), _) => strings::call(name, s, args, at),
        _ => {
            let kinds: Vec<&str> = target.into_iter().chain(args).map(Value::kind).collect();
            Err(Error::new(
                at,
                format!("{name}() doesn't take ({})", kinds.join(", ")),
            ))
        }
    }
}

fn count(n: usize) -> Value {
    Value::Int(i64::try_from(n).unwrap_or(i64::MAX))
}

fn size(value: &Value, at: usize) -> Result<Value, Error> {
    match value {
        Value::String(s) => Ok(count(s.chars().count())),
        Value::List(items) => Ok(count(items.len())),
        Value::Map(entries) => Ok(count(entries.len())),
        other => Err(Error::new(
            at,
            format!("size() doesn't take {}", other.kind()),
        )),
    }
}

fn to_int(value: &Value, at: usize) -> Result<Value, Error> {
    let too_big = |what: String| Error::new(at, format!("{what} doesn't fit an int"));
    match value {
        Value::Int(i) => Ok(Value::Int(*i)),
        Value::Uint(u) => i64::try_from(*u)
            .map(Value::Int)
            .map_err(|_| too_big(u.to_string())),
        // -2^63 fits; 2^63 doesn't.
        #[allow(clippy::cast_possible_truncation)]
        Value::Double(d)
            if (-9_223_372_036_854_775_808.0..9_223_372_036_854_775_808.0).contains(d) =>
        {
            Ok(Value::Int(d.trunc() as i64))
        }
        Value::Double(_) => Err(too_big(value.text())),
        Value::String(s) => s
            .parse()
            .map(Value::Int)
            .map_err(|_| Error::new(at, format!("'{s}' is not an int"))),
        Value::Timestamp(t) => Ok(Value::Int(
            i64::try_from(t.div_euclid(time::NANOS)).unwrap_or_default(),
        )),
        other => Err(Error::new(
            at,
            format!("int() doesn't take {}", other.kind()),
        )),
    }
}

#[allow(clippy::cast_precision_loss)]
fn to_double(value: &Value, at: usize) -> Result<Value, Error> {
    match value {
        Value::Int(i) => Ok(Value::Double(*i as f64)),
        Value::Uint(u) => Ok(Value::Double(*u as f64)),
        Value::Double(d) => Ok(Value::Double(*d)),
        Value::String(s) => s
            .parse()
            .map(Value::Double)
            .map_err(|_| Error::new(at, format!("'{s}' is not a double"))),
        other => Err(Error::new(
            at,
            format!("double() doesn't take {}", other.kind()),
        )),
    }
}

fn to_string(value: &Value, at: usize) -> Result<Value, Error> {
    match value {
        Value::Null | Value::List(_) | Value::Map(_) => Err(Error::new(
            at,
            format!("string() doesn't take {}", value.kind()),
        )),
        scalar => Ok(Value::string(&scalar.text())),
    }
}

fn join(items: &[Value], separator: &str, at: usize) -> Result<Value, Error> {
    let parts = items
        .iter()
        .map(|item| match item {
            Value::String(s) => Ok(&**s),
            other => Err(Error::new(
                at,
                format!("join() needs strings, not {}", other.kind()),
            )),
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Value::string(&parts.join(separator)))
}
