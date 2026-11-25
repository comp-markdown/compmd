//! The CompMD CEL profile (spec §8.2): a parser, a type-checker that runs before evaluation, and an evaluator with a
//! work budget. Every error is an F024, with the column it is at.
//!
//! An expression is parsed once into a [`Program`], checked against the types of the variables it sees (a
//! [`Scope`]), and evaluated with their values ([`Values`]). The `md.` functions come from outside, through
//! [`FunctionTypes`] and [`Functions`]: [`crate::md`] has the spec's.

pub(crate) mod ast;
mod check;
mod eval;
pub(crate) mod functions;
mod getters;
mod lexer;
mod ops;
pub(crate) mod overload;
mod parser;
mod selections;
mod strings;
#[cfg(test)]
mod tests;
pub(crate) mod time;
mod types;
mod value;
mod variables;

pub use eval::BUDGET;
pub use parser::MAX_DEPTH;
pub use selections::Selection;
pub use types::Type;
pub use value::{Key, Value};

use crate::diagnostic::{Code, Diagnostic, Stage};
use crate::line_index::LineIndex;
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

    /// The stage that finds it in a document (spec §11): validating finds parse and type errors, evaluating the rest.
    pub(crate) fn stage(&self) -> Stage {
        match self {
            Self::Syntax { .. } | Self::Type { .. } => Stage::Validate,
            Self::Evaluation { .. } | Self::Budget { .. } => Stage::Evaluate,
        }
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

    /// The F024 a document reports for this error in `expression`, the text it was parsed from: at the error's line
    /// and column in it, found by the stage that finds it in a document (validating for parse and type errors,
    /// evaluating for the rest), so an expression outside a document gets the same diagnostic as one in a body.
    ///
    /// ```
    /// use compmd::cel::Program;
    ///
    /// let error = Program::parse("1 +").expect_err("not an expression");
    /// let diagnostic = error.diagnostic("1 +");
    /// assert_eq!((diagnostic.code.to_string(), diagnostic.range.start.column), ("F024".into(), 4));
    /// ```
    #[must_use]
    pub fn diagnostic(&self, expression: &str) -> Diagnostic {
        let at = LineIndex::new(expression).location(self.at());
        Diagnostic::new(Code::F024, at, self.message()).in_stage(self.stage())
    }
}

/// The profile's functions and macros: those called on a value when `method`, as `s.trim()`, and the others
/// otherwise.
pub(crate) fn function_names(method: bool) -> Vec<&'static str> {
    let functions = functions::Function::NAMES.iter().copied().filter(|name| {
        functions::Function::from_name(name).is_some_and(|function| {
            overload::Library::overloads(function)
                .iter()
                .any(|overload| overload.receiver.is_some() == method)
        })
    });
    let macros = ast::Macro::NAMES.iter().copied().filter(|_| method);
    let has = (!method).then_some("has");
    functions.chain(macros).chain(has).collect()
}

/// The types of the `md.` functions, for the checker.
pub trait FunctionTypes {
    /// The type `md.<name>` gives for arguments of these types.
    ///
    /// # Errors
    ///
    /// An unknown function, or arguments it doesn't take, reported at the call.
    fn result_type(&self, name: &str, args: &[Type]) -> Result<Type, String>;
}

/// The render an expression is evaluated for: the `md.` functions, and time zone offsets for the timestamp getters.
pub trait Functions {
    /// The function names, without `md.`, which tell a call of one from a field of `md`.
    fn names(&self) -> &[&'static str];

    /// Calls `md.<name>`.
    ///
    /// # Errors
    ///
    /// What went wrong, reported at the call.
    fn call(&self, name: &str, args: &[Value]) -> Result<Value, String>;

    /// The size of the value `md.<name>` would give for `args`, in the evaluator's steps (a string's characters, a
    /// list's elements), worked out without building it, so the budget is spent before the call: for a function whose
    /// result can be far larger than its arguments. `None`, the default, for one whose result is no larger than they
    /// are, which is spent for once built.
    fn result_size(&self, name: &str, args: &[Value]) -> Option<usize> {
        let _ = (name, args);
        None
    }

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

    /// Every name, with its type.
    pub(crate) fn names(&self) -> impl Iterator<Item = &str> {
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

    /// Each field the program selects by dot under `path`, `md.doc` say, in the order they are written: its name,
    /// and where the name is in the source, from its parse, so a string, raw or triple-quoted, that looks like one
    /// isn't one. A field read by index (`md.doc["title"]`) isn't selected by dot, and what a comprehension's own
    /// variable of `path`'s first name selects isn't under `path`.
    ///
    /// ```
    /// use compmd::cel::Program;
    ///
    /// let program = Program::parse("md.doc.price * (1 + md.doc.tax) + size('md.doc.x')").expect("an expression");
    /// let fields: Vec<_> = program.selections("md.doc").into_iter().map(|s| (s.name, s.range)).collect();
    /// assert_eq!(fields, [("price".to_string(), 7..12), ("tax".to_string(), 27..30)]);
    /// ```
    #[must_use]
    pub fn selections(&self, path: &str) -> Vec<Selection> {
        let path: Vec<&str> = path.split('.').collect();
        selections::selections(&self.expr, &path)
    }

    /// The variables the program reads, each once, in the order it reads them; a comprehension's own variable is
    /// left out.
    pub(crate) fn variables(&self) -> Vec<String> {
        variables::variables(&self.expr)
    }
}

/// A parsed program is kept and evaluated on other threads: it holds nothing shared.
const _: () = {
    const fn shared_across_threads<T: Send + Sync>() {}
    shared_across_threads::<Program>();
};
