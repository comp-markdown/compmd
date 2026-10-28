//! A recursive-descent parser for CEL's grammar (cel-spec's `langdef.md`), one function per precedence level, with
//! the profile's macros.

use super::Error;
use super::ast::{Expr, Literal, Macro};
use super::lexer::{Token, tokens};

/// How deeply an expression may nest. Deeper nesting is refused, so a hostile expression can't exhaust the stack of
/// the parser, the checker or the evaluator. Parentheses, each operator of a chain, signs and member accesses count
/// one each.
pub const MAX_DEPTH: usize = 64;

/// Parses `source` into a tree.
///
/// # Errors
///
/// A lexical or syntax error, at its column.
pub(crate) fn parse(source: &str) -> Result<Expr, Error> {
    let mut parser = Parser {
        tokens: tokens(source)?,
        at: 0,
        depth: 0,
    };
    let expr = parser.expr()?;
    match parser.peek() {
        Token::End => Ok(expr),
        _ => Err(parser.unexpected()),
    }
}

struct Parser {
    tokens: Vec<(Token, usize)>,
    at: usize,
    depth: usize,
}

impl Parser {
    fn peek(&self) -> &Token {
        &self.tokens[self.at].0
    }

    fn position(&self) -> usize {
        self.tokens[self.at].1
    }

    fn next(&mut self) -> (Token, usize) {
        let token = self.tokens[self.at].clone();
        if self.at + 1 < self.tokens.len() {
            self.at += 1;
        }
        token
    }

    fn eat(&mut self, punct: &str) -> Option<usize> {
        match self.peek() {
            Token::Punct(p) if *p == punct => Some(self.next().1),
            _ => None,
        }
    }

    fn expect(&mut self, punct: &str) -> Result<usize, Error> {
        self.eat(punct).ok_or_else(|| {
            let found = match self.peek() {
                Token::End => "the expression ends".to_string(),
                other => format!("found {}", describe(other)),
            };
            Error::new(self.position(), format!("expected '{punct}' but {found}"))
        })
    }

    fn unexpected(&self) -> Error {
        unexpected(self.peek(), self.position())
    }

    /// One level deeper, refused past [`MAX_DEPTH`].
    fn deeper(&mut self) -> Result<(), Error> {
        self.depth += 1;
        if self.depth > MAX_DEPTH {
            return Err(Error::new(
                self.position(),
                "the expression is nested too deeply",
            ));
        }
        Ok(())
    }

    /// Parses with `parse`, then returns to the depth before it: a chain's operators count while it is parsed.
    fn level(
        &mut self,
        parse: impl FnOnce(&mut Self) -> Result<Expr, Error>,
    ) -> Result<Expr, Error> {
        let depth = self.depth;
        let expr = parse(self);
        self.depth = depth;
        expr
    }

    fn expr(&mut self) -> Result<Expr, Error> {
        self.level(|parser| {
            parser.deeper()?;
            let condition = parser.or()?;
            let Some(at) = parser.eat("?") else {
                return Ok(condition);
            };
            let then = parser.or()?;
            parser.expect(":")?;
            let otherwise = parser.expr()?;
            Ok(Expr::Conditional(
                Box::new(condition),
                Box::new(then),
                Box::new(otherwise),
                at,
            ))
        })
    }

    fn or(&mut self) -> Result<Expr, Error> {
        self.level(|parser| {
            let mut left = parser.and()?;
            while let Some(at) = parser.eat("||") {
                parser.deeper()?;
                left = Expr::Or(Box::new(left), Box::new(parser.and()?), at);
            }
            Ok(left)
        })
    }

    fn and(&mut self) -> Result<Expr, Error> {
        self.level(|parser| {
            let mut left = parser.relation()?;
            while let Some(at) = parser.eat("&&") {
                parser.deeper()?;
                left = Expr::And(Box::new(left), Box::new(parser.relation()?), at);
            }
            Ok(left)
        })
    }

    fn relation(&mut self) -> Result<Expr, Error> {
        self.level(|parser| {
            let mut left = parser.addition()?;
            loop {
                let name = match parser.peek() {
                    Token::Punct(p @ ("<" | "<=" | ">" | ">=" | "==" | "!=")) => format!("_{p}_"),
                    Token::Ident(word) if word == "in" => "@in".to_string(),
                    _ => return Ok(left),
                };
                parser.deeper()?;
                let at = parser.next().1;
                left = binary(name, left, parser.addition()?, at);
            }
        })
    }

    fn addition(&mut self) -> Result<Expr, Error> {
        self.level(|parser| {
            let mut left = parser.multiplication()?;
            while let &Token::Punct(p @ ("+" | "-")) = parser.peek() {
                parser.deeper()?;
                let at = parser.next().1;
                left = binary(format!("_{p}_"), left, parser.multiplication()?, at);
            }
            Ok(left)
        })
    }

    fn multiplication(&mut self) -> Result<Expr, Error> {
        self.level(|parser| {
            let mut left = parser.unary(true)?;
            while let &Token::Punct(p @ ("*" | "/" | "%")) = parser.peek() {
                parser.deeper()?;
                let at = parser.next().1;
                left = binary(format!("_{p}_"), left, parser.unary(true)?, at);
            }
            Ok(left)
        })
    }

    /// A unary expression. A number right after a single `-` is one negative literal, so `-9223372036854775808`
    /// fits; after a second `-` it is unsigned again, as in CEL's grammar, so `--9223372036854775808` doesn't
    /// (`fold`).
    fn unary(&mut self, fold: bool) -> Result<Expr, Error> {
        self.level(|parser| {
            if let Some(at) = parser.eat("!") {
                parser.deeper()?;
                return Ok(unary("!_", parser.unary(true)?, at));
            }
            let Some(at) = parser.eat("-") else {
                return parser.member();
            };
            parser.deeper()?;
            let fold = fold && parser.number_stands_alone();
            let literal = match parser.peek() {
                Token::Int(n) if fold => Literal::Int(-n),
                Token::MinIntMagnitude if fold => Literal::Int(i64::MIN),
                Token::Double(n) if fold => Literal::Double(-n),
                _ => return Ok(unary("-_", parser.unary(false)?, at)),
            };
            parser.next();
            Ok(Expr::Literal(literal, at))
        })
    }

    /// Whether the token after the current one isn't a member access, which binds tighter than a sign.
    fn number_stands_alone(&self) -> bool {
        !matches!(
            self.tokens.get(self.at + 1).map(|(token, _)| token),
            Some(Token::Punct("." | "["))
        )
    }

    fn member(&mut self) -> Result<Expr, Error> {
        let mut expr = self.primary()?;
        loop {
            if let Some(dot) = self.eat(".") {
                self.deeper()?;
                let (Token::Ident(name), at) = self.next() else {
                    return Err(Error::new(dot, "expected a field or method name after '.'"));
                };
                expr = if self.eat("(").is_some() {
                    let args = self.list_until(")")?;
                    macro_or_call(expr, name, args, at)?
                } else {
                    Expr::Select(Box::new(expr), name, dot)
                };
            } else if let Some(at) = self.eat("[") {
                self.deeper()?;
                let index = self.expr()?;
                self.expect("]")?;
                expr = Expr::Index(Box::new(expr), Box::new(index), at);
            } else {
                return Ok(expr);
            }
        }
    }

    fn primary(&mut self) -> Result<Expr, Error> {
        let (token, at) = self.next();
        Ok(match token {
            Token::Int(n) => Expr::Literal(Literal::Int(n), at),
            Token::Uint(n) => Expr::Literal(Literal::Uint(n), at),
            Token::Double(n) => Expr::Literal(Literal::Double(n), at),
            Token::String(s) => Expr::Literal(Literal::String(s), at),
            Token::MinIntMagnitude => return Err(Error::new(at, "int literal out of range")),
            Token::Ident(word) => match word.as_str() {
                "true" => Expr::Literal(Literal::Bool(true), at),
                "false" => Expr::Literal(Literal::Bool(false), at),
                "null" => Expr::Literal(Literal::Null, at),
                "in" => return Err(Error::new(at, "'in' isn't expected here")),
                _ if self.eat("(").is_some() => {
                    let args = self.list_until(")")?;
                    if word == "has" {
                        return has(args, at);
                    }
                    Expr::Call {
                        target: None,
                        name: word,
                        args,
                        at,
                    }
                }
                _ => Expr::Ident(word, at),
            },
            Token::Punct("(") => {
                let inner = self.expr()?;
                self.expect(")")?;
                inner
            }
            Token::Punct("[") => Expr::List(self.list_until("]")?, at),
            Token::Punct("{") => Expr::Map(self.entries()?, at),
            other => return Err(unexpected(&other, at)),
        })
    }

    /// Comma-separated expressions up to `close`, which is consumed; a trailing comma is allowed.
    fn list_until(&mut self, close: &str) -> Result<Vec<Expr>, Error> {
        let mut items = Vec::new();
        while self.eat(close).is_none() {
            items.push(self.expr()?);
            if self.eat(",").is_none() {
                self.expect(close)?;
                break;
            }
        }
        Ok(items)
    }

    /// A map's entries up to `}`, which is consumed.
    fn entries(&mut self) -> Result<Vec<(Expr, Expr)>, Error> {
        let mut entries = Vec::new();
        while self.eat("}").is_none() {
            let key = self.expr()?;
            self.expect(":")?;
            entries.push((key, self.expr()?));
            if self.eat(",").is_none() {
                self.expect("}")?;
                break;
            }
        }
        Ok(entries)
    }
}

fn unexpected(token: &Token, at: usize) -> Error {
    let message = match token {
        Token::End => "the expression ends too early".to_string(),
        other => format!("{} isn't expected here", describe(other)),
    };
    Error::new(at, message)
}

/// A token other than the end, for messages.
fn describe(token: &Token) -> String {
    match token {
        Token::Punct(p) => format!("'{p}'"),
        Token::Ident(word) => format!("'{word}'"),
        Token::String(_) => "a string".into(),
        _ => "a number".into(),
    }
}

fn binary(name: String, left: Expr, right: Expr, at: usize) -> Expr {
    Expr::Call {
        target: None,
        name,
        args: vec![left, right],
        at,
    }
}

fn unary(name: &str, operand: Expr, at: usize) -> Expr {
    Expr::Call {
        target: None,
        name: name.into(),
        args: vec![operand],
        at,
    }
}

/// `has(operand.field)`.
fn has(mut args: Vec<Expr>, at: usize) -> Result<Expr, Error> {
    match (args.pop(), args.is_empty()) {
        (Some(Expr::Select(operand, field, _)), true) => Ok(Expr::Has(operand, field, at)),
        _ => Err(Error::new(
            at,
            "has() takes one field selection, such as has(md.doc.title)",
        )),
    }
}

/// A method call, or one of the comprehension macros when it is given a variable name and the right number of
/// arguments.
fn macro_or_call(target: Expr, name: String, args: Vec<Expr>, at: usize) -> Result<Expr, Error> {
    let kind = match name.as_str() {
        "all" => Macro::All,
        "exists" => Macro::Exists,
        "exists_one" => Macro::ExistsOne,
        "filter" => Macro::Filter,
        "map" => Macro::Map,
        _ => {
            return Ok(Expr::Call {
                target: Some(Box::new(target)),
                name,
                args,
                at,
            });
        }
    };
    let mut args = args.into_iter();
    let (var, filter, body) = match (kind, args.next(), args.next(), args.next(), args.next()) {
        (_, Some(Expr::Ident(var, _)), Some(body), None, None) => (var, None, body),
        (Macro::Map, Some(Expr::Ident(var, _)), Some(filter), Some(body), None) => {
            (var, Some(Box::new(filter)), body)
        }
        _ => {
            let filter = if kind == Macro::Map {
                ", optionally a filter,"
            } else {
                ""
            };
            return Err(Error::new(
                at,
                format!("{name}() takes a variable name{filter} and an expression"),
            ));
        }
    };
    Ok(Expr::Comprehension {
        kind,
        range: Box::new(target),
        var,
        filter,
        body: Box::new(body),
        at,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The tree as an S-expression, without positions.
    fn shape(source: &str) -> String {
        fn walk(expr: &Expr) -> String {
            let all = |args: &[Expr]| args.iter().map(walk).collect::<Vec<_>>().join(" ");
            match expr {
                Expr::Literal(Literal::Int(i), _) => i.to_string(),
                Expr::Literal(Literal::Double(d), _) => format!("{d:?}"),
                Expr::Literal(other, _) => format!("{other:?}"),
                Expr::Ident(name, _) => name.clone(),
                Expr::Select(operand, field, _) => format!("{}.{field}", walk(operand)),
                Expr::Index(operand, index, _) => format!("{}[{}]", walk(operand), walk(index)),
                Expr::Call {
                    target: Some(target),
                    name,
                    args,
                    ..
                } => format!("({}.{name} {})", walk(target), all(args)),
                Expr::Call {
                    target: None,
                    name,
                    args,
                    ..
                } => format!("({name} {})", all(args)),
                Expr::And(a, b, _) => format!("(&& {} {})", walk(a), walk(b)),
                Expr::Or(a, b, _) => format!("(|| {} {})", walk(a), walk(b)),
                Expr::Conditional(c, t, o, _) => format!("(? {} {} {})", walk(c), walk(t), walk(o)),
                Expr::List(items, _) => format!("[{}]", all(items)),
                Expr::Map(entries, _) => {
                    let entries: Vec<String> = entries
                        .iter()
                        .map(|(k, v)| format!("{}: {}", walk(k), walk(v)))
                        .collect();
                    format!("{{{}}}", entries.join(", "))
                }
                Expr::Has(operand, field, _) => format!("(has {}.{field})", walk(operand)),
                Expr::Comprehension {
                    kind,
                    range,
                    var,
                    filter,
                    body,
                    ..
                } => {
                    let filter = filter
                        .as_ref()
                        .map(|f| format!(" {}", walk(f)))
                        .unwrap_or_default();
                    format!(
                        "({} {} {var}{filter} {})",
                        kind.name(),
                        walk(range),
                        walk(body)
                    )
                }
            }
        }
        parse(source).map_or_else(
            |e| format!("error at {}: {}", e.at, e.message),
            |expr| walk(&expr),
        )
    }

    #[test]
    fn precedence_and_associativity_follow_cel() {
        assert_eq!(shape("1 - 2 - 3"), "(_-_ (_-_ 1 2) 3)");
        assert_eq!(
            shape("1 + 2 * 3 < 4 && a || b"),
            "(|| (&& (_<_ (_+_ 1 (_*_ 2 3)) 4) a) b)"
        );
        assert_eq!(shape("a ? b : c ? d : e"), "(? a b (? c d e))");
        assert_eq!(shape("-a.size()"), "(-_ (a.size ))");
        assert_eq!(shape("!a in b"), "(@in (!_ a) b)");
        assert_eq!(shape("a.b[0].c"), "a.b[0].c");
        assert_eq!(shape("f(1, 2,)"), "(f 1 2)");
    }

    #[test]
    fn literals_lists_and_maps() {
        assert_eq!(
            shape("[1, 'a', true, null, 2.5, 3u]"),
            "[1 String(\"a\") Bool(true) Null 2.5 Uint(3)]"
        );
        assert_eq!(shape("{'a': 1, 2: [],}"), "{String(\"a\"): 1, 2: []}");
        assert_eq!(shape("{}"), "{}");
    }

    #[test]
    fn signs_fold_into_number_literals() {
        assert_eq!(shape("-1"), "-1");
        assert_eq!(shape("-9223372036854775808"), "-9223372036854775808");
        assert_eq!(shape("-2.5"), "-2.5");
        assert_eq!(shape("--1"), "(-_ (-_ 1))");
        assert_eq!(shape("-1.size()"), "(-_ (1.size ))");
        assert_eq!(
            shape("--9223372036854775808"),
            "error at 2: int literal out of range"
        );
        assert_eq!(
            shape("9223372036854775808"),
            "error at 0: int literal out of range"
        );
        assert_eq!(
            shape("1 - -9223372036854775808"),
            "(_-_ 1 -9223372036854775808)"
        );
        assert_eq!(shape("!!true"), "(!_ (!_ Bool(true)))");
    }

    #[test]
    fn macros_need_a_variable_name() {
        assert_eq!(shape("l.map(x, x * 2)"), "(map() l x (_*_ x 2))");
        assert_eq!(shape("l.map(x, x > 0, x)"), "(map() l x (_>_ x 0) x)");
        assert_eq!(shape("l.exists_one(x, x)"), "(exists_one() l x x)");
        assert_eq!(shape("l.all(x, x)"), "(all() l x x)");
        assert_eq!(shape("l.exists(x, x)"), "(exists() l x x)");
        assert_eq!(shape("l.filter(x, x)"), "(filter() l x x)");
        assert_eq!(
            shape("l.all(1, true)"),
            "error at 2: all() takes a variable name and an expression"
        );
        assert_eq!(
            shape("l.map(x)"),
            "error at 2: map() takes a variable name, optionally a filter, and an expression"
        );
        assert_eq!(shape("has(a.b)"), "(has a.b)");
        assert_eq!(
            shape("has(a)"),
            "error at 0: has() takes one field selection, such as has(md.doc.title)"
        );
    }

    #[test]
    fn nodes_are_at_their_operator() {
        let expr = parse("a.b * c[1]");
        assert!(matches!(&expr, Ok(Expr::Call { name, at: 4, .. }) if name == "_*_"));
        let Ok(Expr::Call { args, .. }) = expr else {
            return;
        };
        assert!(matches!(&args[0], Expr::Select(_, field, 1) if field == "b"));
        assert!(matches!(&args[1], Expr::Index(_, _, 7)));
    }

    #[test]
    fn deep_nesting_is_refused() {
        let nested = |n: usize| format!("{}1{}", "(".repeat(n), ")".repeat(n));
        assert!(parse(&nested(MAX_DEPTH - 1)).is_ok());
        assert!(parse(&nested(200)).is_err());
        assert!(parse(&vec!["1"; 200].join(" + ")).is_err());
        assert!(parse(&vec!["1"; 60].join(" + ")).is_ok());
        assert!(parse(&format!("{}true", "!".repeat(200))).is_err());
        assert!(parse(&format!("{}1", "-".repeat(200))).is_err());
        assert!(parse(&format!("a{}", ".b".repeat(200))).is_err());
        assert!(parse(&format!("a{}", "[0]".repeat(200))).is_err());
        for chain in ["||", "&&", "==", "*"] {
            let source = vec!["a"; 200].join(chain);
            assert_eq!(
                parse(&source).map_err(|e| e.message),
                Err("the expression is nested too deeply".into()),
                "{chain}"
            );
        }
    }

    #[test]
    fn syntax_errors_say_what_was_found() {
        assert_eq!(shape("1 +"), "error at 3: the expression ends too early");
        assert_eq!(
            shape("(1"),
            "error at 2: expected ')' but the expression ends"
        );
        assert_eq!(shape("1 2"), "error at 2: a number isn't expected here");
        assert_eq!(shape(")"), "error at 0: ')' isn't expected here");
        assert_eq!(
            shape("a.'x'"),
            "error at 1: expected a field or method name after '.'"
        );
        assert_eq!(shape("a.1"), "error at 1: a number isn't expected here");
        assert_eq!(shape("in"), "error at 0: 'in' isn't expected here");
        assert_eq!(
            shape("{'a' 1}"),
            "error at 5: expected ':' but found a number"
        );
        assert_eq!(
            shape("[1 2]"),
            "error at 3: expected ']' but found a number"
        );
        assert_eq!(
            shape("{'a': 1 'b'}"),
            "error at 8: expected '}' but found a string"
        );
        assert_eq!(shape("a b"), "error at 2: 'b' isn't expected here");
        assert_eq!(shape("'"), "error at 0: unterminated string");
    }
}
