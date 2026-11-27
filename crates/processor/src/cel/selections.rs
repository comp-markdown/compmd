//! Where an expression reads a field under a path: by dot, `title` in `md.doc.title`, or by index, `md.doc["title"]`.
//! For a caller that rewrites the names of the fields an expression reads, such as one that stores them by an ID and
//! shows them by their name, or one that renames a key.

use super::ast::{Expr, Literal};
use std::ops::Range;

/// A field an expression selects by dot under a path: `title` in `md.doc.title`, or in `has(md.doc.title)`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Selection {
    /// The field's name.
    pub name: String,
    /// Where the name is written, in code points from the start of the expression.
    pub range: Range<usize>,
}

/// An index an expression takes of a path: `md.doc["title"]`, or `md.doc[name]`, whose key is computed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Indexing {
    /// The key, when the index is a string literal; `None` for one computed when the expression runs.
    pub key: Option<String>,
    /// Where the index is written, in code points from the start of the expression: a string literal's opening quote
    /// (or its prefix), else the `[`.
    pub at: usize,
}

/// What an expression reads under a path, each in the order it is written.
#[derive(Debug, Default)]
pub(super) struct Reads {
    pub(super) selections: Vec<Selection>,
    pub(super) indexings: Vec<Indexing>,
}

/// What `expr` reads under `path` (`["md", "doc"]`), by dot and by index, in the order they are written. A
/// comprehension's own variable that has `path`'s first name hides it, so what is read of that variable isn't counted.
/// An index by a literal that isn't a string (`[0]`) names no field, and isn't counted.
pub(super) fn reads(expr: &Expr, path: &[&str]) -> Reads {
    let mut found = Reads::default();
    walk(expr, path, &mut Vec::new(), &mut found);
    found
        .selections
        .sort_by_key(|selection| selection.range.start);
    found.indexings.sort_by_key(|indexing| indexing.at);
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

fn walk(expr: &Expr, path: &[&str], bound: &mut Vec<String>, found: &mut Reads) {
    match expr {
        Expr::Literal(..) | Expr::Ident(..) => {}
        Expr::Select(operand, field, at) | Expr::Has(operand, field, _, at) => {
            if is_path(operand, path, bound) {
                found.selections.push(Selection {
                    name: field.clone(),
                    range: *at..*at + field.chars().count(),
                });
            }
            walk(operand, path, bound, found);
        }
        Expr::Unary(_, operand, _) => walk(operand, path, bound, found),
        Expr::Index(left, right, at) => {
            if is_path(left, path, bound) {
                match &**right {
                    Expr::Literal(Literal::String(key), literal) => {
                        found.indexings.push(Indexing {
                            key: Some(key.clone()),
                            at: *literal,
                        });
                    }
                    Expr::Literal(..) => {}
                    _ => found.indexings.push(Indexing { key: None, at: *at }),
                }
            }
            walk(left, path, bound, found);
            walk(right, path, bound, found);
        }
        Expr::Binary(_, left, right, _) | Expr::And(left, right, _) | Expr::Or(left, right, _) => {
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

    /// Each index `source` takes of `md.doc`: its key, and what is written from where it is.
    fn indexed(source: &str) -> Vec<(Option<String>, String)> {
        Program::parse(source)
            .expect("an expression")
            .indexings("md.doc")
            .into_iter()
            .map(|indexing| (indexing.key, source.chars().skip(indexing.at).collect()))
            .collect()
    }

    #[test]
    fn each_index_of_the_path_with_its_key_when_it_is_a_string() {
        assert_eq!(
            indexed(r#"md.doc["due"] < md.doc[r'note'] + md.doc[name]"#),
            [
                (
                    Some("due".into()),
                    r#""due"] < md.doc[r'note'] + md.doc[name]"#.into()
                ),
                (Some("note".into()), "r'note'] + md.doc[name]".into()),
                (None, "[name]".into()),
            ]
        );
        assert_eq!(
            indexed("md.data[0] + md.doc.a[1] + [md.doc][0]"),
            [],
            "not the path's, or a number"
        );
        assert_eq!(
            indexed("[1].map(md, md.doc['a'])"),
            [],
            "a comprehension's own `md`"
        );
        assert_eq!(
            indexed("md.doc[md.doc['k']]"),
            [
                (None, "[md.doc['k']]".into()),
                (Some("k".into()), "'k']]".into())
            ],
            "an index within an index"
        );
    }
}
