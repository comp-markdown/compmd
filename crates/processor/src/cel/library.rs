//! A library of functions: each one's definition, with its overloads (for each way to call it, what it takes, what it
//! gives and what it does, together, so the checker and the evaluator can't disagree), and how a call picks one.

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
        match (self, ty.cel()) {
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
    /// A `date` (spec §8.2): a string the checker knows is one.
    Date,
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
            Self::Date => Type::Date,
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

/// The size of a call's result, from its arguments (the receiver first), in the evaluator's steps: a string's
/// characters or a list's elements. Worked out without building the result.
pub(crate) type Size = fn(&[Value]) -> usize;

/// One way to call a function. `Call` is its implementation's type, which takes the receiver, if any, first. A
/// library's are written with `#[overload(…)]` (`compmd_macros::library`).
pub(crate) struct Overload<Call> {
    /// A method's receiver: `None` for a function called as `f(args)`.
    pub(crate) receiver: Option<Param>,
    pub(crate) params: &'static [Param],
    /// The params' names, one for each of `params`.
    pub(crate) names: &'static [&'static str],
    pub(crate) returns: Returns,
    pub(crate) call: Call,
    /// The size of its result, for a function whose result can be far larger than its arguments, which the
    /// evaluator spends before the call; `None` for one whose result is no larger than they are.
    pub(crate) size: Option<Size>,
}

impl<Call> Overload<Call> {
    /// The size of its result for `args` (the receiver first), when it declares one.
    pub(crate) fn result_size(&self, args: &[Value]) -> Option<usize> {
        self.size.map(|size| size(args))
    }
}

/// A function, as its variant of a library writes it (`compmd_macros::library`): what the catalog says of it
/// (`crate::functions`), and its overloads.
pub(crate) struct Definition<Call: 'static> {
    /// Its name, without the library's prefix: `currency`.
    pub(crate) name: &'static str,
    /// What it gives: its doc's first paragraph.
    pub(crate) description: &'static str,
    /// What calls of it give, each evaluated by a test of its own.
    pub(crate) examples: &'static [Example],
    pub(crate) overloads: &'static [Overload<Call>],
    /// Whether its `int` arguments are positions or counts, which a `double` from data is often given for.
    pub(crate) counts: bool,
}

/// A call of a function and what it gives, at the catalog's render (`crate::functions::examples`).
pub(crate) struct Example {
    /// The expression.
    pub(crate) expression: &'static str,
    /// The variables it reads, each a name (`price`, `md.doc`) and its value as JSON.
    pub(crate) given: &'static [(&'static str, &'static str)],
    /// What it gives, as JSON.
    pub(crate) result: &'static str,
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

    /// The function's definition.
    fn definition(self) -> &'static Definition<Self::Call>;
}

impl Param {
    /// How a signature writes it.
    pub(crate) fn written(self) -> &'static str {
        match self {
            Self::Int => "int",
            Self::Uint => "uint",
            Self::Double => "double",
            Self::Number => "number",
            Self::String => "string",
            Self::Bool => "bool",
            Self::Timestamp => "timestamp",
            Self::Duration => "duration",
            Self::List => "list",
            Self::Numbers => "list(number)",
            Self::Strings => "list(string)",
            Self::Map => "map",
            Self::Sized => "string | list | map",
        }
    }
}

impl<Call> Overload<Call> {
    /// How a signature writes what it gives.
    pub(crate) fn returns_written(&self) -> &'static str {
        let all: Vec<Param> = self
            .receiver
            .into_iter()
            .chain(self.params.iter().copied())
            .collect();
        match self.returns {
            Returns::Int => "int",
            Returns::Double => "double",
            Returns::String => "string",
            Returns::Date => "date",
            Returns::Bool => "bool",
            Returns::Timestamp => "timestamp",
            Returns::Duration => "duration",
            Returns::Strings => "list(string)",
            Returns::Same(i) => all.get(i).map_or("dyn", |param| param.written()),
            Returns::Values(_) => "dyn",
            Returns::Groups(_) => "map",
        }
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
    let function = resolve::<L>(name)?.definition();
    let overload = function
        .overloads
        .iter()
        .find(|overload| fits(overload, receiver, args, Param::takes_type));
    let Some(overload) = overload else {
        let hint = if function.counts && args.contains(&Type::Double) {
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
    resolve::<L>(name)?
        .definition()
        .overloads
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

        fn definition(self) -> &'static Definition<u8> {
            const {
                &Definition {
                    name: "f",
                    description: "A test.",
                    examples: &[],
                    overloads: &[
                        Overload {
                            receiver: None,
                            params: &[Param::Int],
                            names: &["n"],
                            returns: Returns::Int,
                            call: 1,
                            size: None,
                        },
                        Overload {
                            receiver: Some(Param::String),
                            params: &[Param::Strings],
                            names: &["m"],
                            returns: Returns::Same(1),
                            call: 2,
                            size: None,
                        },
                    ],
                    counts: false,
                }
            }
        }
    }

    #[test]
    fn an_overload_writes_what_it_gives() {
        let [global, method] = Test.definition().overloads else {
            panic!("two overloads")
        };
        assert_eq!(
            (global.returns_written(), method.returns_written()),
            ("int", "list(string)")
        );
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
