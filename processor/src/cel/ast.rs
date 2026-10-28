//! The expression tree. Every node knows where it is, in code points from the start of the expression.

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

/// The comprehension macros of the profile.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Macro {
    All,
    Exists,
    ExistsOne,
    Filter,
    Map,
}

impl Macro {
    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::All => "all()",
            Self::Exists => "exists()",
            Self::ExistsOne => "exists_one()",
            Self::Filter => "filter()",
            Self::Map => "map()",
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
    /// A function or an operator: `target.name(args)` or `name(args)`. Operators are named `_+_`, `!_`, `-_` and
    /// `@in`, never like a function. At the name or the operator.
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

    /// Whether this is `md.<name>(…)`, a function of the `md.` namespace rather than a method on the `md` variable.
    pub(crate) fn is_md_call(target: Option<&Self>, name: &str, names: &[&str]) -> bool {
        matches!(target, Some(Self::Ident(namespace, _)) if namespace == "md")
            && names.contains(&name)
    }
}
