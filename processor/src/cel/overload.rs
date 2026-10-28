//! A function's overloads: for each way to call it, what it takes, what it gives and what it does, together, so the
//! checker and the evaluator can't disagree.

use super::types::Type;
use super::value::Value;
use crate::nearest::nearest;

/// What an argument, or a method's receiver, must be.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Param {
    Int,
    Uint,
    Double,
    /// An `int`, a `uint` or a `double`.
    Number,
    String,
    Bool,
    Timestamp,
    Duration,
    List,
    /// A list of numbers.
    Numbers,
    /// A list of strings.
    Strings,
    /// A map, or a record.
    Map,
    /// A string, a list or a map: what has a size.
    Sized,
}

impl Param {
    /// Whether an expression of type `ty` can be given: `dyn` can always, and is checked when evaluated.
    pub(crate) fn takes_type(self, ty: &Type) -> bool {
        match (self, ty) {
            (_, Type::Dyn)
            | (Self::Int, Type::Int)
            | (Self::Uint, Type::Uint)
            | (Self::Double, Type::Double)
            | (Self::String, Type::String)
            | (Self::Bool, Type::Bool)
            | (Self::Timestamp, Type::Timestamp)
            | (Self::Duration, Type::Duration)
            | (Self::List, Type::List(_))
            | (Self::Map, Type::Map(..) | Type::Record(_))
            | (Self::Sized, Type::String | Type::List(_) | Type::Map(..) | Type::Record(_)) => true,
            (Self::Number, ty) => ty.is_numeric(),
            (Self::Numbers, Type::List(element)) => element.is_numeric(),
            (Self::Strings, Type::List(element)) => element.fits(&Type::String),
            _ => false,
        }
    }

    /// Whether `value` can be given. A list's elements are checked by the function.
    pub(crate) fn takes_value(self, value: &Value) -> bool {
        matches!(
            (self, value),
            (Self::Int, Value::Int(_))
                | (Self::Uint, Value::Uint(_))
                | (Self::Double, Value::Double(_))
                | (
                    Self::Number,
                    Value::Int(_) | Value::Uint(_) | Value::Double(_)
                )
                | (Self::String, Value::String(_))
                | (Self::Bool, Value::Bool(_))
                | (Self::Timestamp, Value::Timestamp(_))
                | (Self::Duration, Value::Duration(_))
                | (Self::List | Self::Numbers | Self::Strings, Value::List(_))
                | (Self::Map, Value::Map(_))
                | (
                    Self::Sized,
                    Value::String(_) | Value::List(_) | Value::Map(_)
                )
        )
    }
}

/// The type a call gives, from its arguments' types.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Returns {
    Int,
    Double,
    String,
    Bool,
    Timestamp,
    Duration,
    /// `list(string)`.
    Strings,
    /// The type of the argument at this index, such as a sorted list's.
    Same(usize),
    /// The values' type of the map at this index, such as a plural form's.
    Values(usize),
    /// A map from any key to the type of the argument at this index, such as groups of a list.
    Groups(usize),
}

impl Returns {
    fn type_for(self, args: &[Type]) -> Type {
        let arg = |i: usize| args.get(i).cloned().unwrap_or(Type::Dyn);
        match self {
            Self::Int => Type::Int,
            Self::Double => Type::Double,
            Self::String => Type::String,
            Self::Bool => Type::Bool,
            Self::Timestamp => Type::Timestamp,
            Self::Duration => Type::Duration,
            Self::Strings => Type::list(Type::String),
            Self::Same(i) => arg(i),
            Self::Values(i) => match arg(i) {
                Type::Map(_, value) => (*value).clone(),
                Type::Record(fields) => Type::joined(fields.values().cloned()),
                _ => Type::Dyn,
            },
            Self::Groups(i) => Type::map(Type::Dyn, arg(i)),
        }
    }
}

/// One way to call a function. `Call` is its implementation's type, which takes the receiver, if any, first.
pub(crate) struct Overload<Call> {
    pub(crate) receiver: Option<Param>,
    pub(crate) params: &'static [Param],
    pub(crate) returns: Returns,
    pub(crate) call: Call,
}

impl<Call> Overload<Call> {
    /// A function called as `f(args)`.
    pub(crate) const fn global(params: &'static [Param], returns: Returns, call: Call) -> Self {
        Self {
            receiver: None,
            params,
            returns,
            call,
        }
    }

    /// A method called as `receiver.f(args)`.
    pub(crate) const fn method(
        receiver: Param,
        params: &'static [Param],
        returns: Returns,
        call: Call,
    ) -> Self {
        Self {
            receiver: Some(receiver),
            params,
            returns,
            call,
        }
    }
}

/// A family of named, overloaded functions: the profile's functions from CEL, or the `md.` functions.
pub(crate) trait Library: Copy + 'static {
    /// The type of its implementations.
    type Call: 'static;

    /// What its names are written with in messages: `""`, or `"md."`.
    const PREFIX: &'static str;

    /// The function written `name`.
    fn from_name(name: &str) -> Option<Self>;

    /// Every function's name.
    fn names() -> &'static [&'static str];

    /// Its overloads.
    fn overloads(self) -> &'static [Overload<Self::Call>];

    /// Whether its `int` arguments are positions or counts, which a `double` from data is often given for.
    fn counts(self) -> bool {
        false
    }
}

/// The way out when a `double` is given where an `int` is needed: numbers from data are `double`s (spec §8.2).
pub(crate) const INT_HINT: &str = "; wrap the number in int(…) to drop its fraction";

/// The type a call of `L`'s function `name` gives, from its first overload that takes arguments of these types.
///
/// # Errors
///
/// An unknown function, with the nearest known one, or arguments no overload takes.
pub(crate) fn type_of<L: Library>(
    name: &str,
    receiver: Option<&Type>,
    args: &[Type],
) -> Result<Type, String> {
    let function = resolve::<L>(name)?;
    let overload = function
        .overloads()
        .iter()
        .find(|overload| fits(overload, receiver, args, Param::takes_type));
    let Some(overload) = overload else {
        let hint = if function.counts() && args.contains(&Type::Double) {
            INT_HINT
        } else {
            ""
        };
        let receiver = receiver.map(ToString::to_string);
        let args: Vec<String> = args.iter().map(ToString::to_string).collect();
        return Err(no_overload::<L>(name, receiver, &args, hint));
    };
    let all: Vec<Type> = receiver.into_iter().chain(args).cloned().collect();
    Ok(overload.returns.type_for(&all))
}

/// The overload of `L`'s function `name` that takes these values.
///
/// # Errors
///
/// An unknown function, or values no overload takes.
pub(crate) fn select<L: Library>(
    name: &str,
    receiver: Option<&Value>,
    args: &[Value],
) -> Result<&'static Overload<L::Call>, String> {
    let function = resolve::<L>(name)?;
    function
        .overloads()
        .iter()
        .find(|overload| fits(overload, receiver, args, Param::takes_value))
        .ok_or_else(|| {
            let receiver = receiver.map(|r| r.kind().to_string());
            let args: Vec<String> = args.iter().map(|a| a.kind().to_string()).collect();
            no_overload::<L>(name, receiver, &args, "")
        })
}

fn resolve<L: Library>(name: &str) -> Result<L, String> {
    L::from_name(name).ok_or_else(|| {
        let written = format!("{}{name}", L::PREFIX);
        let known: Vec<String> = L::names()
            .iter()
            .map(|n| format!("{}{n}", L::PREFIX))
            .collect();
        match nearest(&written, known.iter().map(String::as_str)) {
            Some(near) => format!("unknown function '{written}'; did you mean '{near}'?"),
            None => format!("unknown function '{written}'"),
        }
    })
}

fn no_overload<L: Library>(
    name: &str,
    receiver: Option<String>,
    args: &[String],
    hint: &str,
) -> String {
    let receiver = receiver.map(|r| format!("{r}.")).unwrap_or_default();
    format!(
        "{receiver}{}{name}() doesn't take ({}){hint}",
        L::PREFIX,
        args.join(", ")
    )
}

/// The string argument at `i`. Overloads are selected by their arguments' values, so a call finds the type it
/// declares; the empty string is never used.
pub(crate) fn text(args: &[Value], i: usize) -> &str {
    match args.get(i) {
        Some(Value::String(s)) => s,
        _ => "",
    }
}

/// The `int` argument at `i`, if there is one.
pub(crate) fn int(args: &[Value], i: usize) -> Option<i64> {
    match args.get(i) {
        Some(Value::Int(n)) => Some(*n),
        _ => None,
    }
}

/// The list argument at `i`.
pub(crate) fn list(args: &[Value], i: usize) -> &[Value] {
    match args.get(i) {
        Some(Value::List(items)) => items,
        _ => &[],
    }
}

fn fits<Call, T>(
    overload: &Overload<Call>,
    receiver: Option<&T>,
    args: &[T],
    takes: impl Fn(Param, &T) -> bool,
) -> bool {
    let receiver_fits = match (overload.receiver, receiver) {
        (None, None) => true,
        (Some(param), Some(receiver)) => takes(param, receiver),
        _ => false,
    };
    receiver_fits
        && overload.params.len() == args.len()
        && overload
            .params
            .iter()
            .zip(args)
            .all(|(param, arg)| takes(*param, arg))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A library of one function, `f`: `f(int)` and `string.f(list(string))`.
    #[derive(Clone, Copy)]
    struct Test;

    impl Library for Test {
        type Call = u8;
        const PREFIX: &'static str = "t.";

        fn from_name(name: &str) -> Option<Self> {
            (name == "f").then_some(Self)
        }

        fn names() -> &'static [&'static str] {
            &["f"]
        }

        fn overloads(self) -> &'static [Overload<u8>] {
            const {
                &[
                    Overload::global(&[Param::Int], Returns::Int, 1),
                    Overload::method(Param::String, &[Param::Strings], Returns::Same(1), 2),
                ]
            }
        }
    }

    #[test]
    fn the_first_overload_that_takes_the_arguments_decides() {
        let type_of = |receiver: Option<&Type>, args: &[Type]| type_of::<Test>("f", receiver, args);
        assert_eq!(type_of(None, &[Type::Int]), Ok(Type::Int));
        assert_eq!(
            type_of(None, &[Type::Double]),
            Err("t.f() doesn't take (double)".into())
        );
        assert_eq!(
            type_of(Some(&Type::Int), &[Type::Int]),
            Err("int.t.f() doesn't take (int)".into())
        );
        let strings = Type::list(Type::String);
        assert_eq!(
            type_of(Some(&Type::String), std::slice::from_ref(&strings)),
            Ok(strings)
        );
        assert_eq!(type_of(Some(&Type::Dyn), &[Type::Dyn]), Ok(Type::Dyn));
        assert_eq!(
            select::<Test>("f", None, &[Value::Int(1)]).map(|o| o.call),
            Ok(1)
        );
        assert_eq!(
            select::<Test>("f", None, &[Value::Int(1), Value::Int(2)]).map(|o| o.call),
            Err("t.f() doesn't take (int, int)".into())
        );
        assert!(!Test.counts());
    }

    #[test]
    fn unknown_functions_come_with_the_nearest() {
        assert_eq!(
            type_of::<Test>("g", None, &[]),
            Err("unknown function 't.g'; did you mean 't.f'?".into())
        );
        assert_eq!(
            select::<Test>("nothing", None, &[]).map(|o| o.call),
            Err("unknown function 't.nothing'".into())
        );
    }

    #[test]
    fn overloads_are_functions_or_methods() {
        let function = Overload::global(&[Param::Int], Returns::Int, ());
        let method = Overload::method(Param::String, &[], Returns::Int, ());
        assert_eq!(
            (function.receiver, function.params),
            (None, &[Param::Int][..])
        );
        assert_eq!(
            (method.receiver, method.returns),
            (Some(Param::String), Returns::Int)
        );
    }

    #[test]
    fn returned_types_follow_the_arguments() {
        let forms = Type::of_json(&serde_json::json!({ "one": "a", "other": "b" }));
        assert_eq!(Returns::Values(0).type_for(&[forms]), Type::String);
        assert_eq!(
            Returns::Values(0).type_for(&[Type::map(Type::Dyn, Type::Int)]),
            Type::Int
        );
        assert_eq!(Returns::Values(0).type_for(&[Type::Dyn]), Type::Dyn);
        assert_eq!(
            Returns::Groups(0).type_for(&[Type::Int]),
            Type::map(Type::Dyn, Type::Int)
        );
        assert_eq!(Returns::Same(3).type_for(&[]), Type::Dyn);
    }
}
