//! The variables an expression reads: the names it doesn't bind itself, in the order it reads them.

use super::ast::Expr;

/// The variables `expr` reads, each once, in order: a comprehension's own variable is left out of its body.
pub(super) fn variables(expr: &Expr) -> Vec<String> {
    let mut found = Vec::new();
    read(expr, &mut Vec::new(), &mut found);
    found
}

fn read(expr: &Expr, bound: &mut Vec<String>, found: &mut Vec<String>) {
    match expr {
        Expr::Literal(..) => {}
        Expr::Ident(name, _) => {
            if !bound.contains(name) && !found.contains(name) {
                found.push(name.clone());
            }
        }
        Expr::Select(operand, ..) | Expr::Unary(_, operand, _) | Expr::Has(operand, ..) => {
            read(operand, bound, found);
        }
        Expr::Index(left, right, _)
        | Expr::Binary(_, left, right, _)
        | Expr::And(left, right, _)
        | Expr::Or(left, right, _) => {
            read(left, bound, found);
            read(right, bound, found);
        }
        Expr::Conditional(condition, then, otherwise, _) => {
            for expr in [condition, then, otherwise] {
                read(expr, bound, found);
            }
        }
        Expr::Call { target, args, .. } => {
            for expr in target.iter().map(AsRef::as_ref).chain(args) {
                read(expr, bound, found);
            }
        }
        Expr::List(items, _) => {
            for item in items {
                read(item, bound, found);
            }
        }
        Expr::Map(entries, _) => {
            for (key, value) in entries {
                read(key, bound, found);
                read(value, bound, found);
            }
        }
        Expr::Comprehension {
            range,
            var,
            filter,
            body,
            ..
        } => {
            read(range, bound, found);
            bound.push(var.clone());
            for expr in filter.iter().map(AsRef::as_ref).chain([body.as_ref()]) {
                read(expr, bound, found);
            }
            bound.pop();
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::cel::Program;

    fn variables(source: &str) -> Vec<String> {
        Program::parse(source).expect("an expression").variables()
    }

    #[test]
    fn an_expression_reads_its_free_names_in_order() {
        assert_eq!(variables("card.href"), ["card"]);
        assert_eq!(
            variables("flag ? rows[i] : md.doc.rows"),
            ["flag", "rows", "i", "md"]
        );
        assert_eq!(
            variables("-a + size(b) && !has(c.d) || e"),
            ["a", "b", "c", "e"]
        );
        assert_eq!(variables("[x, {k: y}, \"s\", 1]"), ["x", "k", "y"]);
        assert_eq!(variables("x.trim()"), ["x"]);
    }

    #[test]
    fn a_comprehension_binds_its_own_variable() {
        assert_eq!(variables("rows.map(r, r.image)"), ["rows"]);
        assert_eq!(variables("rows.map(r, r > n, r + m)"), ["rows", "n", "m"]);
        assert_eq!(variables("rows.all(rows, rows)"), ["rows"]);
    }
}
