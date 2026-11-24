//! Where an expression selects a field by dot under a path, `title` in `md.doc.title`: for a caller that rewrites the
//! names of the fields an expression reads, such as one that stores them by an ID and shows them by their name.

use super::ast::Expr;
use std::ops::Range;

/// A field an expression selects by dot under a path: `title` in `md.doc.title`, or in `has(md.doc.title)`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Selection {
    /// The field's name.
    pub name: String,
    /// Where the name is written, in code points from the start of the expression.
    pub range: Range<usize>,
}

/// The fields `expr` selects by dot under `path` (`["md", "doc"]`), in the order they are written. A comprehension's
/// own variable that has `path`'s first name hides it, so what is selected from that variable isn't counted.
pub(super) fn selections(expr: &Expr, path: &[&str]) -> Vec<Selection> {
    let mut found = Vec::new();
    walk(expr, path, &mut Vec::new(), &mut found);
    found.sort_by_key(|selection| selection.range.start);
    found
}

/// Whether `expr` is `path`, its first name not one a comprehension binds.
fn is_path(expr: &Expr, path: &[&str], bound: &[String]) -> bool {
    match (expr, path.split_last()) {
        (Expr::Ident(name, _), Some((last, []))) => name == last && !bound.contains(name),
        (Expr::Select(operand, field, _), Some((last, rest))) if !rest.is_empty() => {
            field == last && is_path(operand, rest, bound)
        }
        _ => false,
    }
}

fn walk(expr: &Expr, path: &[&str], bound: &mut Vec<String>, found: &mut Vec<Selection>) {
    match expr {
        Expr::Literal(..) | Expr::Ident(..) => {}
        Expr::Select(operand, field, at) | Expr::Has(operand, field, _, at) => {
            if is_path(operand, path, bound) {
                found.push(Selection {
                    name: field.clone(),
                    range: *at..*at + field.chars().count(),
                });
            }
            walk(operand, path, bound, found);
        }
        Expr::Unary(_, operand, _) => walk(operand, path, bound, found),
        Expr::Index(left, right, _)
        | Expr::Binary(_, left, right, _)
        | Expr::And(left, right, _)
        | Expr::Or(left, right, _) => {
            walk(left, path, bound, found);
            walk(right, path, bound, found);
        }
        Expr::Conditional(condition, then, otherwise, _) => {
            for expr in [condition, then, otherwise] {
                walk(expr, path, bound, found);
            }
        }
        Expr::Call { target, args, .. } => {
            for expr in target.iter().map(AsRef::as_ref).chain(args) {
                walk(expr, path, bound, found);
            }
        }
        Expr::List(items, _) => {
            for item in items {
                walk(item, path, bound, found);
            }
        }
        Expr::Map(entries, _) => {
            for (key, value) in entries {
                walk(key, path, bound, found);
                walk(value, path, bound, found);
            }
        }
        Expr::Comprehension {
            range,
            var,
            filter,
            body,
            ..
        } => {
            walk(range, path, bound, found);
            bound.push(var.clone());
            for expr in filter.iter().map(AsRef::as_ref).chain([body.as_ref()]) {
                walk(expr, path, bound, found);
            }
            bound.pop();
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::cel::Program;

    /// Each field `source` selects under `md.doc`, with what its range covers in `source`.
    fn selected(source: &str) -> Vec<(String, String)> {
        let chars: Vec<char> = source.chars().collect();
        Program::parse(source)
            .expect("an expression")
            .selections("md.doc")
            .into_iter()
            .map(|selection| {
                let written: String = chars[selection.range.clone()].iter().collect();
                (selection.name, written)
            })
            .collect()
    }

    fn names(source: &str) -> Vec<String> {
        selected(source)
            .into_iter()
            .map(|(name, written)| {
                assert_eq!(name, written, "the range covers the name");
                name
            })
            .collect()
    }

    #[test]
    fn each_field_selected_by_dot_with_its_range() {
        assert_eq!(names("md.doc.price * 2 + md.doc.tax"), ["price", "tax"]);
        assert_eq!(
            names("has(md.doc.due) ? md.doc.due : md.today()"),
            ["due", "due"]
        );
        assert_eq!(
            names("md.doc.title.size() > 0 && md.doc . note != ''"),
            ["title", "note"]
        );
        // Code points, not bytes: `é` before the selection is one.
        assert_eq!(
            selected("'é' + md.doc.name"),
            [("name".into(), "name".into())]
        );
    }

    #[test]
    fn what_isn_t_a_field_of_the_path_selected_by_dot_is_left_out() {
        // In strings, of all kinds.
        let none: [String; 0] = [];
        assert_eq!(
            names(r#""md.doc.a" + 'md.doc.b' + r"md.doc.c" + """md.doc.d""""#),
            none
        );
        // By index, under another path, the path itself, a method of it.
        assert_eq!(
            names("md.doc['a'] + md.data.b + string(md.doc) + string(md.doc.size())"),
            none
        );
        assert_eq!(names("doc.a + x.md.doc.b"), none);
        // A comprehension's own `md` hides the document's.
        assert_eq!(names("[1].map(md, md.doc.a) + [md.doc.b]"), ["b"]);
    }
}
