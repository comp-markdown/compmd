//! The expression tree. Every node knows where it is, in code points from the start of the expression.

use super::ops::{BinaryOp, UnaryOp};
use crate::named::named;

/// Where a node is: the code point its errors are reported at (an operator, a call's name, a `[`, a `.`), 0-based.
pub(crate) type At = usize;

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Literal {
    Int(i64),
    Uint(u64),
    Double(f64),
    String(String),
    Bool(bool),
    Null,
}

named! {
    /// The comprehension macros of the profile.
    pub(crate) enum Macro {
        All = "all",
        Exists = "exists",
        ExistsOne = "exists_one",
        Filter = "filter",
        Map = "map",
    }
}

impl Macro {
    /// How it is written, for an editor's hint.
    pub(crate) fn signature(self) -> &'static str {
        match self {
            Self::All => "list.all(x, predicate) → bool",
            Self::Exists => "list.exists(x, predicate) → bool",
            Self::ExistsOne => "list.exists_one(x, predicate) → bool",
            Self::Filter => "list.filter(x, predicate) → list",
            Self::Map => "list.map(x, expression) → list",
        }
    }

    /// What it gives, in one line, for an editor's hint.
    pub(crate) fn doc(self) -> &'static str {
        match self {
            Self::All => "true when every element satisfies the predicate",
            Self::Exists => "true when any element satisfies the predicate",
            Self::ExistsOne => "true when exactly one element satisfies the predicate",
            Self::Filter => "the elements that satisfy the predicate",
            Self::Map => {
                "each element made into another; `map(x, filter, expression)` keeps those the filter keeps"
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Expr {
    Literal(Literal, At),
    Ident(String, At),
    /// `operand.field`, at the `.`.
    Select(Box<Expr>, String, At),
    /// `operand[index]`, at the `[`.
    Index(Box<Expr>, Box<Expr>, At),
    /// `!operand` or `-operand`, at the operator.
    Unary(UnaryOp, Box<Expr>, At),
    /// `left op right`, at the operator.
    Binary(BinaryOp, Box<Expr>, Box<Expr>, At),
    /// A function: `target.name(args)` or `name(args)`, at the name.
    Call {
        target: Option<Box<Expr>>,
        name: String,
        args: Vec<Expr>,
        at: At,
    },
    And(Box<Expr>, Box<Expr>, At),
    Or(Box<Expr>, Box<Expr>, At),
    Conditional(Box<Expr>, Box<Expr>, Box<Expr>, At),
    List(Vec<Expr>, At),
    Map(Vec<(Expr, Expr)>, At),
    /// `has(operand.field)`.
    Has(Box<Expr>, String, At),
    /// `range.macro(var, body)`, or `range.map(var, filter, body)`.
    Comprehension {
        kind: Macro,
        range: Box<Expr>,
        var: String,
        filter: Option<Box<Expr>>,
        body: Box<Expr>,
        at: At,
    },
}

impl Expr {
    pub(crate) fn at(&self) -> At {
        match self {
            Self::Literal(_, at)
            | Self::Ident(_, at)
            | Self::Select(_, _, at)
            | Self::Index(_, _, at)
            | Self::Unary(_, _, at)
            | Self::Binary(_, _, _, at)
            | Self::Call { at, .. }
            | Self::And(_, _, at)
            | Self::Or(_, _, at)
            | Self::Conditional(_, _, _, at)
            | Self::List(_, at)
            | Self::Map(_, at)
            | Self::Has(_, _, at)
            | Self::Comprehension { at, .. } => *at,
        }
    }

    /// Where the expression's text starts: at its leftmost operand, for an operator, a field, an index or a method.
    pub(crate) fn start(&self) -> At {
        match self {
            Self::Select(operand, ..)
            | Self::Index(operand, ..)
            | Self::Binary(_, operand, ..)
            | Self::And(operand, ..)
            | Self::Or(operand, ..)
            | Self::Conditional(operand, ..)
            | Self::Call {
                target: Some(operand),
                ..
            } => operand.start(),
            _ => self.at(),
        }
    }

    /// Whether this is `md.<name>(…)`, a function of the `md.` namespace rather than a method on the `md` variable.
    pub(crate) fn is_md_call(target: Option<&Self>, name: &str, names: &[&str]) -> bool {
        matches!(target, Some(Self::Ident(namespace, _)) if namespace == "md")
            && names.contains(&name)
    }
}
