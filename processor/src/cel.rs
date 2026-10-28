//! The CompMD CEL profile (spec §8.2): a parser, a type-checker that runs before evaluation, and an evaluator with a
//! work budget. Every error is an F024, with the column it is at.
//!
//! An expression is parsed once into a [`Program`], checked against the types of the variables it sees (a
//! [`Scope`]), and evaluated with their values ([`Values`]). The `md.` functions come from outside, through
//! [`FunctionTypes`] and [`Functions`]: [`crate::md`] has the spec's.
//!
//! ```
//! use compmd::cel::{Program, Scope, Type, Value, Values};
//! use compmd::md::{Md, Signatures, TimeZones};
//!
//! struct Utc;
//!
//! impl TimeZones for Utc {
//!     fn offset(&self, _zone: &str, _at: i64) -> Option<i32> {
//!         Some(0)
//!     }
//! }
//!
//! let program = Program::parse("md.plural(count, 'item', 'items') + ' in ' + md.date(md.today(), 'long')")?;
//! let scope = Scope::default().with("count", Type::Double);
//! assert_eq!(program.check(&scope, &Signatures)?, Type::String);
//!
//! let now = 1_792_576_800 * 1_000_000_000; // 2026-10-21T10:00:00Z
//! let md = Md::new(now, "en", "UTC", &Utc);
//! let values = Values::default().with("count", Value::Double(3.0));
//! let text = program.evaluate(&values, &md)?;
//! assert_eq!(text.written(), Ok(Some("items in October 21, 2026".into())));
//! # Ok::<(), compmd::cel::Error>(())
//! ```

mod ast;
mod check;
mod eval;
mod functions;
mod getters;
mod lexer;
mod ops;
pub(crate) mod overload;
mod parser;
mod strings;
pub(crate) mod time;
mod types;
mod value;

pub use eval::BUDGET;
pub use parser::MAX_DEPTH;
pub use types::Type;
pub use value::{Key, Value};

use std::collections::BTreeMap;

/// An error in an expression, reported as F024. Each kind has the code point it is at, from the start of the
/// expression, 0-based, and says what is wrong: where it helps, with the nearest known name or the way out.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    /// Not an expression of the profile, or nested deeper than [`MAX_DEPTH`].
    #[error("column {}: {message}", at + 1)]
    Syntax {
        /// Where it is.
        at: usize,
        /// What is wrong.
        message: String,
    },
    /// Found by the checker: an unknown name, field or function, or a type that doesn't fit.
    #[error("column {}: {message}", at + 1)]
    Type {
        /// Where it is.
        at: usize,
        /// What is wrong.
        message: String,
    },
    /// Found while evaluating: division by zero, an index out of range, an overflow, an `md.` function's error.
    #[error("column {}: {message}", at + 1)]
    Evaluation {
        /// Where it is.
        at: usize,
        /// What is wrong.
        message: String,
    },
    /// The evaluation took more than [`BUDGET`] steps.
    #[error("column {}: the expression takes too much work", at + 1)]
    Budget {
        /// The node being evaluated when the budget ran out.
        at: usize,
    },
}

impl Error {
    pub(crate) fn syntax(at: usize, message: impl Into<String>) -> Self {
        Self::Syntax {
            at,
            message: message.into(),
        }
    }

    pub(crate) fn type_error(at: usize, message: impl Into<String>) -> Self {
        Self::Type {
            at,
            message: message.into(),
        }
    }

    pub(crate) fn evaluation(at: usize, message: impl Into<String>) -> Self {
        Self::Evaluation {
            at,
            message: message.into(),
        }
    }

    /// The code point it is at, from the start of the expression, 0-based.
    #[must_use]
    pub fn at(&self) -> usize {
        match self {
            Self::Syntax { at, .. }
            | Self::Type { at, .. }
            | Self::Evaluation { at, .. }
            | Self::Budget { at } => *at,
        }
    }

    /// The 1-based column, as F024 reports it.
    #[must_use]
    pub fn column(&self) -> usize {
        self.at() + 1
    }

    /// What is wrong, without the column.
    #[must_use]
    pub fn message(&self) -> &str {
        match self {
            Self::Syntax { message, .. }
            | Self::Type { message, .. }
            | Self::Evaluation { message, .. } => message,
            Self::Budget { .. } => "the expression takes too much work",
        }
    }
}

/// The types of the `md.` functions, for the checker.
pub trait FunctionTypes {
    /// The function names, without `md.`: `["today", "sum", …]`.
    fn names(&self) -> &[&'static str];

    /// The type `md.<name>` gives for arguments of these types.
    ///
    /// # Errors
    ///
    /// An unknown function, or arguments it doesn't take, reported at the call.
    fn result_type(&self, name: &str, args: &[Type]) -> Result<Type, String>;
}

/// The render an expression is evaluated for: the `md.` functions, and time zone offsets for the timestamp getters.
pub trait Functions: FunctionTypes {
    /// Calls `md.<name>`.
    ///
    /// # Errors
    ///
    /// What went wrong, reported at the call.
    fn call(&self, name: &str, args: &[Value]) -> Result<Value, String>;

    /// The offset from UTC, in seconds east, of `zone` at `at` (seconds since 1970-01-01T00:00:00Z). `zone` is an
    /// IANA name or an offset such as `"+02:00"`; `None` is `md.tz`.
    ///
    /// # Errors
    ///
    /// An unknown zone.
    fn offset(&self, zone: Option<&str>, at: i64) -> Result<i32, String>;
}

/// The variables an expression sees, by their types: `md`, a definition's params, `each` names. Types share their
/// parts, so extending a scope is cheap.
#[derive(Clone, Debug, Default)]
pub struct Scope(BTreeMap<String, Type>);

impl Scope {
    /// This scope with `name` added, or its type replaced.
    #[must_use]
    pub fn with(mut self, name: impl Into<String>, ty: Type) -> Self {
        self.0.insert(name.into(), ty);
        self
    }

    /// The type of `name`.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&Type> {
        self.0.get(name)
    }

    fn names(&self) -> impl Iterator<Item = &str> {
        self.0.keys().map(String::as_str)
    }
}

/// The variables an expression sees, by their values. Values share their parts, so extending them is cheap.
#[derive(Clone, Debug, Default)]
pub struct Values(BTreeMap<String, Value>);

impl Values {
    /// These values with `name` added, or its value replaced.
    #[must_use]
    pub fn with(mut self, name: impl Into<String>, value: Value) -> Self {
        self.0.insert(name.into(), value);
        self
    }

    /// The value of `name`.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&Value> {
        self.0.get(name)
    }
}

/// A parsed expression: checked against the types of its variables, then evaluated with their values. Parse it
/// once, and check and evaluate it as often as needed.
#[derive(Clone, Debug, PartialEq)]
pub struct Program {
    expr: ast::Expr,
}

impl Program {
    /// Parses `source`, an expression without its parentheses.
    ///
    /// # Errors
    ///
    /// A syntax error, or nesting deeper than [`MAX_DEPTH`].
    pub fn parse(source: &str) -> Result<Self, Error> {
        Ok(Self {
            expr: parser::parse(source)?,
        })
    }

    /// Type-checks the program against the types of the variables in `scope` and of the `md.` functions, and gives
    /// its type. Every branch is checked, also one that is never taken.
    ///
    /// # Errors
    ///
    /// The first type error; an unknown name, field or function comes with the nearest known one, and a `double`
    /// where an `int` is needed with the way out.
    pub fn check(&self, scope: &Scope, functions: &dyn FunctionTypes) -> Result<Type, Error> {
        check::check(&self.expr, scope, functions)
    }

    /// Evaluates the program within [`BUDGET`] steps.
    ///
    /// # Errors
    ///
    /// An evaluation error: division by zero, an index out of range, overflow, an error of an `md.` function, the
    /// budget spent.
    pub fn evaluate(&self, values: &Values, functions: &dyn Functions) -> Result<Value, Error> {
        eval::evaluate(&self.expr, values, functions, BUDGET)
    }
}
