//! The evaluator: CEL's semantics for the profile, with errors at their node's column and a work budget.

use super::ast::{Expr, Macro};
use super::functions::Function;
use super::overload;
use super::value::{Key, Value};
use super::{Error, Functions, Values};
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
            Expr::Unary(op, operand, at) => {
                let operand = self.eval(operand)?;
                op.apply(&operand)
                    .map_err(|message| Error::new(*at, message))
            }
            Expr::Binary(op, left, right, at) => {
                let (left, right) = (self.eval(left)?, self.eval(right)?);
                let value = op
                    .apply(&left, &right)
                    .map_err(|message| Error::new(*at, message))?;
                self.built(*at, value)
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
                        "{}() needs a list or a map, not {}",
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
        let receiver = if md {
            None
        } else {
            target.map(|t| self.eval(t)).transpose()?
        };
        let args = args
            .iter()
            .map(|a| self.eval(a))
            .collect::<Result<Vec<_>, _>>()?;
        let result = if md {
            self.functions.call(name, &args)
        } else {
            call(name, receiver.as_ref(), &args)
        };
        let value = result.map_err(|message| Error::new(at, message))?;
        self.built(at, value)
    }
}

/// Calls one of the profile's functions from CEL, by the overload its arguments' values select.
fn call(name: &str, receiver: Option<&Value>, args: &[Value]) -> Result<Value, String> {
    let function = Function::from_name(name).ok_or_else(|| format!("unknown function '{name}'"))?;
    let Some(overload) = overload::select(function.overloads(), receiver, args) else {
        let kinds: Vec<&str> = args.iter().map(Value::kind).collect();
        let receiver = receiver
            .map(|r| format!("{}.", r.kind()))
            .unwrap_or_default();
        return Err(format!(
            "{receiver}{name}() doesn't take ({})",
            kinds.join(", ")
        ));
    };
    let all: Vec<Value> = receiver.into_iter().chain(args).cloned().collect();
    (overload.call)(&all)
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
