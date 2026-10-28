//! The type-checker: every expression is checked before it is evaluated (spec §8.2), each branch of `&&`, `||` and
//! `?:` too, so an error in a branch that isn't taken is still found.

use super::ast::{Expr, Literal, Macro};
use super::functions::{Function, LEFT_OUT};
use super::ops::BinaryOp;
use super::overload;
use super::types::Type;
use super::{Error, FunctionTypes, Scope};
use crate::nearest::nearest;

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
            Expr::Unary(op, operand, at) => {
                let operand = self.check(operand)?;
                op.result_type(&operand).ok_or_else(|| {
                    Error::new(
                        *at,
                        format!("'{}' can't be applied to {operand}", op.name()),
                    )
                })
            }
            Expr::Binary(op, left, right, at) => {
                let (left, right) = (self.check(left)?, self.check(right)?);
                op.result_type(&left, &right).ok_or_else(|| {
                    let hint = if *op == BinaryOp::Remainder {
                        int_hint(&[left.clone(), right.clone()])
                    } else {
                        ""
                    };
                    Error::new(
                        *at,
                        format!(
                            "'{}' can't be applied to {left} and {right}{hint}",
                            op.name()
                        ),
                    )
                })
            }
            Expr::Call {
                target,
                name,
                args,
                at,
            } => self.call(target.as_deref(), name, args, *at),
            Expr::And(left, right, at) => {
                self.boolean(left, *at, "&&")?;
                self.guarded_by(proven(left), |checker| checker.boolean(right, *at, "&&"))?;
                Ok(Type::Bool)
            }
            Expr::Or(left, right, at) => {
                self.boolean(left, *at, "||")?;
                self.boolean(right, *at, "||")?;
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
                let element = self.element(*kind, range, *at)?;
                self.locals.push((var, element.clone()));
                let result = self.comprehension(*kind, element, filter.as_deref(), body, *at);
                self.locals.pop();
                result
            }
        }
    }

    /// The type of the elements a macro ranges over: a list's elements, or a map's keys.
    fn element(&mut self, kind: Macro, range: &'a Expr, at: usize) -> Result<Type, Error> {
        match self.check(range)? {
            Type::List(element) => Ok((*element).clone()),
            Type::Map(key, _) => Ok((*key).clone()),
            Type::Record(_) => Ok(Type::String),
            Type::Dyn => Ok(Type::Dyn),
            other => Err(Error::new(
                at,
                format!("{}() needs a list or a map, not {other}", kind.name()),
            )),
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
                    format!("{}() needs a bool expression, not {ty}", kind.name()),
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
        let receiver = target.map(|t| self.check(t)).transpose()?;
        let arg_types = args
            .iter()
            .map(|a| self.check(a))
            .collect::<Result<Vec<_>, _>>()?;
        if LEFT_OUT.contains(&name) {
            return Err(Error::new(
                at,
                format!("{name}() isn't in the CompMD CEL profile"),
            ));
        }
        let Some(function) = Function::from_name(name) else {
            return Err(unknown(
                at,
                "function",
                name,
                Function::NAMES.iter().copied(),
            ));
        };
        overload::result_type(function.overloads(), receiver.as_ref(), &arg_types).ok_or_else(
            || {
                let hint = if function.counts() {
                    int_hint(&arg_types)
                } else {
                    ""
                };
                let receiver = receiver.map(|t| format!("{t}.")).unwrap_or_default();
                Error::new(
                    at,
                    format!(
                        "{receiver}{name}() doesn't take ({}){hint}",
                        listed(&arg_types)
                    ),
                )
            },
        )
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
            Error::new(
                at,
                format!("md.{name}() doesn't take ({})", listed(&arg_types)),
            )
        })
    }
}

fn listed(types: &[Type]) -> String {
    types
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(", ")
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
