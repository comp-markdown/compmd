//! The type-checker: every expression is checked before it is evaluated (spec §8.2), each branch of `&&`, `||` and
//! `?:` too, so an error in a branch that isn't taken is still found.

use super::ast::{Expr, Literal, Macro};
use super::types::Type;
use super::{Error, FunctionTypes, Scope};
use crate::nearest::nearest;

/// The functions of the profile from CEL's standard definitions and its strings extension. The `md.` ones come
/// from outside.
pub(crate) const FUNCTIONS: [&str; 19] = [
    "size",
    "int",
    "double",
    "string",
    "timestamp",
    "duration",
    "contains",
    "startsWith",
    "endsWith",
    "charAt",
    "indexOf",
    "lastIndexOf",
    "lowerAscii",
    "upperAscii",
    "replace",
    "split",
    "substring",
    "trim",
    "join",
];

/// CEL functions the profile leaves out, named as such when used.
const LEFT_OUT: [&str; 18] = [
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

/// The functions whose `int` arguments are positions or counts.
const COUNTING: [&str; 6] = [
    "charAt",
    "indexOf",
    "lastIndexOf",
    "replace",
    "split",
    "substring",
];

/// The way out when a `double` is given where an `int` is needed: numbers from data are `double`s (spec §8.2).
const INT_HINT: &str = "; wrap the number in int(…) to drop its fraction";

/// Checks `expr` against the types of the variables in `scope`, and gives its type.
pub(crate) fn check(
    expr: &Expr,
    scope: &Scope,
    functions: &dyn FunctionTypes,
) -> Result<Type, Error> {
    Checker {
        scope,
        functions,
        locals: Vec::new(),
        guarded: Vec::new(),
    }
    .check(expr)
}

/// The names of a field path such as `md.doc.title`, or `None` for any other expression.
fn path(expr: &Expr) -> Option<Vec<&str>> {
    match expr {
        Expr::Ident(name, _) => Some(vec![name]),
        Expr::Select(operand, field, _) => {
            let mut names = path(operand)?;
            names.push(field);
            Some(names)
        }
        _ => None,
    }
}

/// The fields `condition` proves present when it is true: `has(a.b)`, and those of either side of `&&`.
fn proven(condition: &Expr) -> Vec<Vec<&str>> {
    match condition {
        Expr::Has(operand, field, _) => path(operand)
            .map(|mut names| {
                names.push(field);
                names
            })
            .into_iter()
            .collect(),
        Expr::And(left, right, _) => proven(left).into_iter().chain(proven(right)).collect(),
        _ => Vec::new(),
    }
}

struct Checker<'a> {
    scope: &'a Scope,
    functions: &'a dyn FunctionTypes,
    /// Comprehension variables, innermost last.
    locals: Vec<(&'a str, Type)>,
    /// Fields a `has()` proves present where they are checked, so a field a record may lack can be read there.
    guarded: Vec<Vec<&'a str>>,
}

impl<'a> Checker<'a> {
    fn check(&mut self, expr: &'a Expr) -> Result<Type, Error> {
        match expr {
            Expr::Literal(literal, _) => Ok(literal_type(literal)),
            Expr::Ident(name, at) => self.lookup(name).ok_or_else(|| {
                let names = self
                    .locals
                    .iter()
                    .map(|(n, _)| *n)
                    .chain(self.scope.names());
                unknown(*at, "name", name, names)
            }),
            Expr::Select(operand, field, at) => {
                let operand_type = self.check(operand)?;
                self.select(expr, &operand_type, field, *at)
            }
            Expr::Index(operand, index, at) => {
                let (operand, index_type) = (self.check(operand)?, self.check(index)?);
                self::index(&operand, &index_type, index, *at)
            }
            Expr::Call {
                target,
                name,
                args,
                at,
            } => self.call(target.as_deref(), name, args, *at),
            Expr::And(left, right, at) | Expr::Or(left, right, at) => {
                let operator = if matches!(expr, Expr::And(..)) {
                    "&&"
                } else {
                    "||"
                };
                self.boolean(left, *at, operator)?;
                let guards = if operator == "&&" {
                    proven(left)
                } else {
                    Vec::new()
                };
                self.guarded_by(guards, |checker| checker.boolean(right, *at, operator))?;
                Ok(Type::Bool)
            }
            Expr::Conditional(condition, then, otherwise, at) => {
                let condition_type = self.check(condition)?;
                if !condition_type.fits(&Type::Bool) {
                    return Err(Error::new(
                        *at,
                        format!("the condition of '?:' must be a bool, not {condition_type}"),
                    ));
                }
                let then = self.guarded_by(proven(condition), |checker| checker.check(then))?;
                Ok(then.join(self.check(otherwise)?))
            }
            Expr::List(items, _) => {
                let types = items
                    .iter()
                    .map(|item| self.check(item))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(Type::list(Type::joined(types)))
            }
            Expr::Map(entries, _) => self.map(entries),
            Expr::Has(operand, _, at) => match self.check(operand)? {
                Type::Record(_) | Type::Map(..) | Type::Dyn => Ok(Type::Bool),
                other => Err(Error::new(*at, format!("has() needs a map, not {other}"))),
            },
            Expr::Comprehension {
                kind,
                range,
                var,
                filter,
                body,
                at,
            } => {
                let element = match self.check(range)? {
                    Type::List(element) => (*element).clone(),
                    Type::Map(key, _) => (*key).clone(),
                    Type::Record(_) => Type::String,
                    Type::Dyn => Type::Dyn,
                    other => {
                        return Err(Error::new(
                            *at,
                            format!("{} needs a list or a map, not {other}", kind.name()),
                        ));
                    }
                };
                self.locals.push((var, element.clone()));
                let result = self.comprehension(*kind, element, filter.as_deref(), body, *at);
                self.locals.pop();
                result
            }
        }
    }

    /// A map literal: CEL's map keys are `int`, `uint`, `bool` and `string`.
    fn map(&mut self, entries: &'a [(Expr, Expr)]) -> Result<Type, Error> {
        let (mut keys, mut values) = (Vec::new(), Vec::new());
        for (key, value) in entries {
            let key_type = self.check(key)?;
            if !matches!(
                key_type,
                Type::Int | Type::Uint | Type::Bool | Type::String | Type::Dyn
            ) {
                return Err(Error::new(
                    key.at(),
                    format!("a map key must be an int, uint, bool or string, not {key_type}"),
                ));
            }
            keys.push(key_type);
            values.push(self.check(value)?);
        }
        Ok(Type::map(Type::joined(keys), Type::joined(values)))
    }

    fn lookup(&self, name: &str) -> Option<Type> {
        let local = self
            .locals
            .iter()
            .rev()
            .find(|(n, _)| *n == name)
            .map(|(_, t)| t);
        local.or_else(|| self.scope.get(name)).cloned()
    }

    /// Checks `check` with the fields in `guards` proven present.
    fn guarded_by<T>(
        &mut self,
        guards: Vec<Vec<&'a str>>,
        check: impl FnOnce(&mut Self) -> T,
    ) -> T {
        let count = guards.len();
        self.guarded.extend(guards);
        let result = check(self);
        self.guarded.truncate(self.guarded.len() - count);
        result
    }

    fn boolean(&mut self, side: &'a Expr, at: usize, operator: &str) -> Result<(), Error> {
        let side = self.check(side)?;
        if side.fits(&Type::Bool) {
            Ok(())
        } else {
            Err(Error::new(
                at,
                format!("'{operator}' takes bools, not {side}"),
            ))
        }
    }

    fn select(&self, expr: &Expr, operand: &Type, field: &str, at: usize) -> Result<Type, Error> {
        match operand {
            Type::Record(fields) => match fields.get(field) {
                Some(ty) => Ok(ty.clone()),
                None if path(expr).is_some_and(|path| self.guarded.contains(&path)) => {
                    Ok(Type::Dyn)
                }
                None => Err(unknown(
                    at,
                    "field",
                    field,
                    fields.keys().map(String::as_str),
                )),
            },
            Type::Map(key, value) if Type::String.fits(key) => Ok((**value).clone()),
            Type::Dyn => Ok(Type::Dyn),
            other => Err(Error::new(at, format!("{other} has no fields"))),
        }
    }

    fn comprehension(
        &mut self,
        kind: Macro,
        element: Type,
        filter: Option<&'a Expr>,
        body: &'a Expr,
        at: usize,
    ) -> Result<Type, Error> {
        let predicate = |checker: &mut Self, expr: &'a Expr| -> Result<(), Error> {
            let ty = checker.check(expr)?;
            if ty.fits(&Type::Bool) {
                Ok(())
            } else {
                Err(Error::new(
                    at,
                    format!("{} needs a bool expression, not {ty}", kind.name()),
                ))
            }
        };
        if let Some(filter) = filter {
            predicate(self, filter)?;
        }
        match kind {
            Macro::Map => Ok(Type::list(self.check(body)?)),
            Macro::Filter => {
                predicate(self, body)?;
                Ok(Type::list(element))
            }
            Macro::All | Macro::Exists | Macro::ExistsOne => {
                predicate(self, body)?;
                Ok(Type::Bool)
            }
        }
    }

    fn call(
        &mut self,
        target: Option<&'a Expr>,
        name: &str,
        args: &'a [Expr],
        at: usize,
    ) -> Result<Type, Error> {
        if matches!(target, Some(Expr::Ident(namespace, _)) if namespace == "md") {
            return self.md_call(name, args, at);
        }
        let target_type = target.map(|t| self.check(t)).transpose()?;
        let arg_types = args
            .iter()
            .map(|a| self.check(a))
            .collect::<Result<Vec<_>, _>>()?;
        let listed = |separator: &str| {
            let types: Vec<String> = arg_types.iter().map(ToString::to_string).collect();
            types.join(separator)
        };
        if OPERATORS.contains(&name) {
            return operator(name, &arg_types).ok_or_else(|| {
                let hint = if name == "_%_" {
                    int_hint(&arg_types)
                } else {
                    ""
                };
                let types = listed(" and ");
                Error::new(
                    at,
                    format!("'{}' can't be applied to {types}{hint}", symbol(name)),
                )
            });
        }
        if LEFT_OUT.contains(&name) {
            return Err(Error::new(
                at,
                format!("{name}() isn't in the CompMD CEL profile"),
            ));
        }
        if !FUNCTIONS.contains(&name) {
            return Err(unknown(at, "function", name, FUNCTIONS.iter().copied()));
        }
        function_type(name, target_type.as_ref(), &arg_types).ok_or_else(|| {
            let hint = if COUNTING.contains(&name) {
                int_hint(&arg_types)
            } else {
                ""
            };
            let receiver = target_type
                .as_ref()
                .map(|t| format!("{t}."))
                .unwrap_or_default();
            Error::new(
                at,
                format!("{receiver}{name}() doesn't take ({}){hint}", listed(", ")),
            )
        })
    }

    fn md_call(&mut self, name: &str, args: &'a [Expr], at: usize) -> Result<Type, Error> {
        let names = self.functions.names();
        if !names.contains(&name) {
            let known: Vec<String> = names.iter().map(|n| format!("md.{n}")).collect();
            return Err(unknown(
                at,
                "function",
                &format!("md.{name}"),
                known.iter().map(String::as_str),
            ));
        }
        let arg_types = args
            .iter()
            .map(|a| self.check(a))
            .collect::<Result<Vec<_>, _>>()?;
        self.functions.result_type(name, &arg_types).ok_or_else(|| {
            let types: Vec<String> = arg_types.iter().map(ToString::to_string).collect();
            Error::new(
                at,
                format!("md.{name}() doesn't take ({})", types.join(", ")),
            )
        })
    }
}

fn int_hint(given: &[Type]) -> &'static str {
    if given.contains(&Type::Double) {
        INT_HINT
    } else {
        ""
    }
}

fn literal_type(literal: &Literal) -> Type {
    match literal {
        Literal::Int(_) => Type::Int,
        Literal::Uint(_) => Type::Uint,
        Literal::Double(_) => Type::Double,
        Literal::String(_) => Type::String,
        Literal::Bool(_) => Type::Bool,
        Literal::Null => Type::Null,
    }
}

fn index(operand: &Type, index_type: &Type, index: &Expr, at: usize) -> Result<Type, Error> {
    match operand {
        Type::List(element) if index_type.fits(&Type::Int) => Ok((**element).clone()),
        Type::List(_) => Err(Error::new(
            at,
            format!(
                "a list index must be an int, not {index_type}{}",
                int_hint(std::slice::from_ref(index_type))
            ),
        )),
        // Number keys match by value, so any number finds an `int` or `uint` key.
        Type::Map(key, value)
            if index_type.fits(key) || (key.is_numeric() && index_type.is_numeric()) =>
        {
            Ok((**value).clone())
        }
        Type::Record(fields) if index_type.fits(&Type::String) => match index {
            Expr::Literal(Literal::String(field), _) => fields
                .get(field)
                .cloned()
                .ok_or_else(|| unknown(at, "field", field, fields.keys().map(String::as_str))),
            _ => Ok(Type::Dyn),
        },
        Type::Dyn => Ok(Type::Dyn),
        other => Err(Error::new(
            at,
            format!("{other} can't be indexed with {index_type}"),
        )),
    }
}

/// The operators, as the parser names them.
const OPERATORS: [&str; 14] = [
    "!_", "-_", "_+_", "_-_", "_*_", "_/_", "_%_", "_<_", "_<=_", "_>_", "_>=_", "_==_", "_!=_",
    "@in",
];

/// An operator's type, or `None` when it doesn't take these types.
fn operator(name: &str, args: &[Type]) -> Option<Type> {
    use Type::{Bool, Double, Duration, Dyn, Int};
    match (name, args) {
        ("!_", [operand]) => operand.fits(&Bool).then_some(Bool),
        ("-_", [operand]) => {
            matches!(operand, Int | Double | Duration | Dyn).then(|| operand.clone())
        }
        ("_<_" | "_<=_" | "_>_" | "_>=_", [a, b]) => ordered(a, b).then_some(Bool),
        ("_==_" | "_!=_", [a, b]) => equatable(a, b).then_some(Bool),
        ("@in", [a, b]) => contained(a, b).then_some(Bool),
        (_, [a, b]) => arithmetic(name, a, b),
        _ => None,
    }
}

/// How an operator is written.
fn symbol(name: &str) -> &str {
    match name {
        "@in" => "in",
        _ => name.trim_matches('_'),
    }
}

/// Arithmetic, with the profile's `int` and `double` mixing: an `int` next to a `double` makes a `double`.
fn arithmetic(op: &str, a: &Type, b: &Type) -> Option<Type> {
    use Type::{Double, Duration, Dyn, Grammar, Int, List, String, Timestamp, Uint};
    match (op, a, b) {
        (_, Int, Int) => Some(Int),
        (_, Uint, Uint) => Some(Uint),
        (_, Dyn, other) | (_, other, Dyn) => takes(op, other).then_some(Dyn),
        ("_+_" | "_-_" | "_*_" | "_/_", Int | Double, Int | Double) => Some(Double),
        ("_+_", String | Grammar(_), String | Grammar(_)) => Some(String),
        ("_+_", List(x), List(y)) => Some(Type::list(match (&**x, &**y) {
            (Dyn, other) | (other, Dyn) => other.clone(),
            _ => (**x).clone().join((**y).clone()),
        })),
        ("_+_" | "_-_", Timestamp, Duration) | ("_+_", Duration, Timestamp) => Some(Timestamp),
        ("_+_" | "_-_", Duration, Duration) | ("_-_", Timestamp, Timestamp) => Some(Duration),
        _ => None,
    }
}

/// Whether some overload of the arithmetic operator `op` takes a value of type `t`.
fn takes(op: &str, t: &Type) -> bool {
    use Type::{Double, Duration, Dyn, Grammar, Int, List, String, Timestamp, Uint};
    match op {
        "_+_" => matches!(
            t,
            Int | Uint | Double | String | Grammar(_) | List(_) | Timestamp | Duration | Dyn
        ),
        "_-_" => matches!(t, Int | Uint | Double | Timestamp | Duration | Dyn),
        "_%_" => matches!(t, Int | Uint | Dyn),
        _ => matches!(t, Int | Uint | Double | Dyn),
    }
}

/// Whether `<` and the like compare these types: numbers across their types, and strings, bools, timestamps and
/// durations each with their own.
fn ordered(a: &Type, b: &Type) -> bool {
    use Type::{Bool, Duration, Dyn, String, Timestamp};
    let orderable = |t: &Type| t.is_numeric() || matches!(t, String | Bool | Timestamp | Duration);
    match (a, b) {
        (Dyn, other) | (other, Dyn) => orderable(other),
        _ => (a.is_numeric() && b.is_numeric()) || (a == b && orderable(a)),
    }
}

/// Whether `==` compares these types: one fits the other, both are numbers, or one is `null`.
fn equatable(a: &Type, b: &Type) -> bool {
    a.fits(b)
        || b.fits(a)
        || (a.is_numeric() && b.is_numeric())
        || *a == Type::Null
        || *b == Type::Null
}

fn contained(element: &Type, collection: &Type) -> bool {
    match collection {
        Type::List(item) => equatable(element, item),
        Type::Map(key, _) => equatable(element, key),
        Type::Record(_) => element.fits(&Type::String),
        Type::Dyn => true,
        _ => false,
    }
}

/// A function's result type, or `None` when it doesn't take these arguments, or isn't called this way.
fn function_type(name: &str, target: Option<&Type>, args: &[Type]) -> Option<Type> {
    use Type::{Bool, Double, Duration, Dyn, Int, List, Map, Record, String, Timestamp, Uint};
    // `size` and the conversions are called as `f(x)`, the string functions as `s.f(…)`.
    let global = |params: &[Type]| matches!((target, args), (None, [arg]) if params.iter().any(|p| arg.fits(p)));
    let on_string = |params: &[Type], required: usize| {
        target.is_some_and(|t| t.fits(&String)) && accepts(args, params, required)
    };
    match name {
        "size" => match (target, args) {
            (None, [x]) | (Some(x), []) => {
                matches!(x, String | List(_) | Map(..) | Record(_) | Dyn).then_some(Int)
            }
            _ => None,
        },
        "int" => global(&[Int, Uint, Double, String, Timestamp]).then_some(Int),
        "double" => global(&[Int, Uint, Double, String]).then_some(Double),
        "string" => {
            global(&[Int, Uint, Double, String, Bool, Timestamp, Duration]).then_some(String)
        }
        "timestamp" => global(&[String, Timestamp]).then_some(Timestamp),
        "duration" => global(&[String, Duration]).then_some(Duration),
        "contains" | "startsWith" | "endsWith" => on_string(&[String], 1).then_some(Bool),
        "charAt" => on_string(&[Int], 1).then_some(String),
        "indexOf" | "lastIndexOf" => on_string(&[String, Int], 1).then_some(Int),
        "lowerAscii" | "upperAscii" | "trim" => on_string(&[], 0).then_some(String),
        "replace" => on_string(&[String, String, Int], 2).then_some(String),
        "split" => on_string(&[String, Int], 1).then(|| Type::list(String)),
        "substring" => on_string(&[Int, Int], 1).then_some(String),
        // "join"
        _ => (target.is_some_and(|t| t.fits(&Type::list(String))) && accepts(args, &[String], 0))
            .then_some(String),
    }
}

/// Whether `args` fit `params`, of which the first `required` must be given.
fn accepts(args: &[Type], params: &[Type], required: usize) -> bool {
    (required..=params.len()).contains(&args.len())
        && args.iter().zip(params).all(|(a, p)| a.fits(p))
}

/// An unknown name, field or function, with the nearest known one when one is close enough.
fn unknown<'n>(
    at: usize,
    what: &str,
    name: &str,
    known: impl IntoIterator<Item = &'n str>,
) -> Error {
    match nearest(name, known) {
        Some(near) => Error::new(
            at,
            format!("unknown {what} '{name}'; did you mean '{near}'?"),
        ),
        None => Error::new(at, format!("unknown {what} '{name}'")),
    }
}
