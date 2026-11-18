//! The operators: for each, the type it gives and the value it computes, with the profile's `int` and `double`
//! mixing (spec §8.2). `&&`, `||` and `?:` are in the checker and the evaluator, since they decide what to evaluate.

use super::time;
use super::types::Type;
use super::value::{Key, Value};
use crate::named::named;

named! {
    /// A prefix operator.
    pub(crate) enum UnaryOp {
        Not = "!",
        Negate = "-",
    }
}

named! {
    /// An infix operator, other than `&&` and `||`.
    pub(crate) enum BinaryOp {
        Add = "+",
        Subtract = "-",
        Multiply = "*",
        Divide = "/",
        Remainder = "%",
        Less = "<",
        LessOrEqual = "<=",
        Greater = ">",
        GreaterOrEqual = ">=",
        Equal = "==",
        NotEqual = "!=",
        In = "in",
    }
}

impl UnaryOp {
    /// The type it gives an operand of type `operand`, or `None` when it doesn't take it.
    pub(crate) fn result_type(self, operand: &Type) -> Option<Type> {
        match self {
            Self::Not => operand.fits(&Type::Bool).then_some(Type::Bool),
            Self::Negate => matches!(
                operand,
                Type::Int | Type::Double | Type::Duration | Type::Dyn
            )
            .then(|| operand.clone()),
        }
    }

    /// Applies it to `operand`.
    pub(crate) fn apply(self, operand: &Value) -> Result<Value, String> {
        match (self, operand) {
            (Self::Not, Value::Bool(b)) => Ok(Value::Bool(!b)),
            (Self::Negate, Value::Int(i)) => i
                .checked_neg()
                .map(Value::Int)
                .ok_or_else(|| "int overflow".into()),
            (Self::Negate, Value::Double(d)) => Ok(Value::Double(-d)),
            (Self::Negate, Value::Duration(d)) => duration(-d),
            (_, other) => Err(format!(
                "'{}' can't be applied to {}",
                self.name(),
                other.kind()
            )),
        }
    }
}

impl BinaryOp {
    /// The type it gives operands of these types, or `None` when it doesn't take them.
    pub(crate) fn result_type(self, a: &Type, b: &Type) -> Option<Type> {
        match self {
            Self::Add | Self::Subtract | Self::Multiply | Self::Divide | Self::Remainder => {
                self.arithmetic_type(a, b)
            }
            Self::Less | Self::LessOrEqual | Self::Greater | Self::GreaterOrEqual => {
                ordered(a, b).then_some(Type::Bool)
            }
            Self::Equal | Self::NotEqual => equatable(a, b).then_some(Type::Bool),
            Self::In => contained(a, b).then_some(Type::Bool),
        }
    }

    /// Applies it to `a` and `b`.
    pub(crate) fn apply(self, a: &Value, b: &Value) -> Result<Value, String> {
        match self {
            Self::Add | Self::Subtract | Self::Multiply | Self::Divide | Self::Remainder => {
                self.arithmetic(a, b)
            }
            Self::Less | Self::LessOrEqual | Self::Greater | Self::GreaterOrEqual => {
                self.compare(a, b)
            }
            Self::Equal => Ok(Value::Bool(a.equals(b))),
            Self::NotEqual => Ok(Value::Bool(!a.equals(b))),
            Self::In => contains(a, b),
        }
    }

    fn no_overload(self, a: &Value, b: &Value) -> String {
        format!(
            "'{}' can't be applied to {} and {}",
            self.name(),
            a.kind(),
            b.kind()
        )
    }

    /// Arithmetic's type: an `int` next to a `double` makes a `double`.
    fn arithmetic_type(self, a: &Type, b: &Type) -> Option<Type> {
        use Type::{Double, Duration, Dyn, Grammar, Int, List, String, Timestamp, Uint};
        match (self, a, b) {
            (_, Int, Int) => Some(Int),
            (_, Uint, Uint) => Some(Uint),
            (_, Dyn, other) | (_, other, Dyn) => self.takes(other).then_some(Dyn),
            (Self::Remainder, ..) => None,
            (_, Int | Double, Int | Double) => Some(Double),
            (Self::Add, String | Grammar(_), String | Grammar(_)) => Some(String),
            (Self::Add, List(x), List(y)) => Some(Type::list(match (&**x, &**y) {
                (Dyn, other) | (other, Dyn) => other.clone(),
                _ => (**x).clone().join((**y).clone()),
            })),
            (Self::Add | Self::Subtract, Timestamp, Duration)
            | (Self::Add, Duration, Timestamp) => Some(Timestamp),
            (Self::Add | Self::Subtract, Duration, Duration)
            | (Self::Subtract, Timestamp, Timestamp) => Some(Duration),
            _ => None,
        }
    }

    /// Whether some overload of this arithmetic operator takes a value of type `t`.
    fn takes(self, t: &Type) -> bool {
        use Type::{Double, Duration, Dyn, Grammar, Int, List, String, Timestamp, Uint};
        match self {
            Self::Add => matches!(
                t,
                Int | Uint | Double | String | Grammar(_) | List(_) | Timestamp | Duration | Dyn
            ),
            Self::Subtract => matches!(t, Int | Uint | Double | Timestamp | Duration | Dyn),
            Self::Remainder => matches!(t, Int | Uint | Dyn),
            _ => matches!(t, Int | Uint | Double | Dyn),
        }
    }

    fn arithmetic(self, a: &Value, b: &Value) -> Result<Value, String> {
        match (self, a, b) {
            (_, Value::Int(x), Value::Int(y)) => self
                .integers(i128::from(*x), i128::from(*y), "int")
                .map(Value::Int),
            (_, Value::Uint(x), Value::Uint(y)) => self
                .integers(i128::from(*x), i128::from(*y), "uint")
                .map(Value::Uint),
            (Self::Add, Value::String(x), Value::String(y)) => {
                Ok(Value::string(&format!("{x}{y}")))
            }
            (Self::Add, Value::List(x), Value::List(y)) => {
                Ok(Value::list(x.iter().chain(y.iter()).cloned().collect()))
            }
            (Self::Add, Value::Timestamp(t), Value::Duration(d))
            | (Self::Add, Value::Duration(d), Value::Timestamp(t)) => timestamp(t + d),
            (Self::Subtract, Value::Timestamp(t), Value::Duration(d)) => timestamp(t - d),
            (Self::Subtract, Value::Timestamp(t), Value::Timestamp(u)) => duration(t - u),
            (Self::Add, Value::Duration(d), Value::Duration(e)) => duration(d + e),
            (Self::Subtract, Value::Duration(d), Value::Duration(e)) => duration(d - e),
            _ => match (float(a), float(b), self) {
                (_, _, Self::Remainder) | (None, ..) | (_, None, _) => Err(self.no_overload(a, b)),
                (Some(x), Some(y), Self::Add) => Ok(Value::Double(x + y)),
                (Some(x), Some(y), Self::Subtract) => Ok(Value::Double(x - y)),
                (Some(x), Some(y), Self::Multiply) => Ok(Value::Double(x * y)),
                (Some(x), Some(y), _) => Ok(Value::Double(x / y)),
            },
        }
    }

    /// Integer arithmetic on `int`s or `uint`s (`T`, called `name`), in an `i128`, where it can't overflow, and an
    /// overflow error when the result doesn't fit `T`.
    fn integers<T: TryFrom<i128>>(self, x: i128, y: i128, name: &str) -> Result<T, String> {
        let overflow = || format!("{name} overflow");
        let n = match self {
            Self::Add => x + y,
            Self::Subtract => x - y,
            Self::Multiply => x * y,
            _ if y == 0 => return Err("division by zero".into()),
            Self::Divide => x / y,
            // CEL fails `%` where `/` overflows, as for the smallest int and -1.
            _ => {
                T::try_from(x / y).map_err(|_| overflow())?;
                x % y
            }
        };
        T::try_from(n).map_err(|_| overflow())
    }

    fn compare(self, a: &Value, b: &Value) -> Result<Value, String> {
        let ordering = a.order(b).map_err(|()| self.no_overload(a, b))?;
        // A `NaN` is neither less than, equal to nor greater than anything.
        Ok(Value::Bool(ordering.is_some_and(|ordering| match self {
            Self::Less => ordering.is_lt(),
            Self::LessOrEqual => ordering.is_le(),
            Self::Greater => ordering.is_gt(),
            _ => ordering.is_ge(),
        })))
    }
}

/// An `int` or a `double` as a `double`, for the profile's mixed arithmetic.
#[allow(
    clippy::cast_precision_loss,
    reason = "CEL converts an int to the nearest double"
)]
fn float(value: &Value) -> Option<f64> {
    match value {
        Value::Int(i) => Some(*i as f64),
        Value::Double(d) => Some(*d),
        _ => None,
    }
}

fn timestamp(nanos: i128) -> Result<Value, String> {
    if time::timestamp_in_range(nanos) {
        Ok(Value::Timestamp(nanos))
    } else {
        Err("timestamp out of range".into())
    }
}

fn duration(nanos: i128) -> Result<Value, String> {
    if time::duration_in_range(nanos) {
        Ok(Value::Duration(nanos))
    } else {
        Err("duration out of range".into())
    }
}

fn contains(element: &Value, collection: &Value) -> Result<Value, String> {
    match collection {
        Value::List(items) => Ok(Value::Bool(items.iter().any(|item| element.equals(item)))),
        Value::Map(entries) => Ok(Value::Bool(
            Key::to_find(element).is_some_and(|key| entries.contains_key(&key)),
        )),
        other => Err(format!("'in' needs a list or a map, not {}", other.kind())),
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
