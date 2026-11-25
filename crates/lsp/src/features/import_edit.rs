//! The edit that adds an import to a document's frontmatter, in the frontmatter's own syntax (spec §2.1): a
//! `[[md.import]]` table in TOML, an item of `md.import` in YAML, or a YAML frontmatter when the document has none.

use compmd::tree::{Node, Root};
use lsp_types::{Position, Range, TextEdit};

/// The edit that imports `name` from `from` into the document whose text is `text` and tree `tree`; `None` when its
/// frontmatter writes `md` or `md.import` on one line (`md: {…}`), which an insertion can't extend.
pub(crate) fn adding_import(text: &str, tree: &Root, from: &str, name: &str) -> Option<TextEdit> {
    let (line, new_text) = match tree.children.first() {
        Some(Node::Toml(frontmatter)) => {
            let close = frontmatter.position?.end.line - 1;
            (
                close,
                format!("[[md.import]]\nfrom = \"{from}\"\nnames = [\"{name}\"]\n"),
            )
        }
        Some(Node::Yaml(frontmatter)) => {
            let position = frontmatter.position?;
            let lines: Vec<&str> = text.split('\n').collect();
            yaml(
                &lines,
                position.start.line,
                position.end.line - 1,
                from,
                name,
            )?
        }
        _ => (
            0,
            format!("---\nmd:\n  import:\n    - from: \"{from}\"\n      names: [{name}]\n---\n\n"),
        ),
    };
    let at = Position {
        line: u32::try_from(line).ok()?,
        character: 0,
    };
    Some(TextEdit {
        range: Range { start: at, end: at },
        new_text,
    })
}

/// How many spaces `line` starts with.
fn indent(line: &str) -> usize {
    line.len() - line.trim_start_matches(' ').len()
}

/// Where to insert the import in a YAML frontmatter whose lines (from 0) run past `open` to the closing `close`, and
/// what.
fn yaml(
    lines: &[&str],
    open: usize,
    close: usize,
    from: &str,
    name: &str,
) -> Option<(usize, String)> {
    let item = |indent: &str| format!("{indent}- from: \"{from}\"\n{indent}  names: [{name}]\n");
    let Some(md) = (open..close).find(|&line| lines[line].starts_with("md:")) else {
        return Some((close, format!("md:\n  import:\n{}", item("    "))));
    };
    if !lines[md]["md:".len()..].trim().is_empty() {
        return None;
    }
    let inside = |line: usize| lines[line].trim().is_empty() || indent(lines[line]) > 0;
    let end = (md + 1..close).find(|&line| !inside(line)).unwrap_or(close);
    let children = (md + 1..end).find(|&line| !lines[line].trim().is_empty());
    let key_indent = children.map_or(2, |line| indent(lines[line]));
    let pad = " ".repeat(key_indent);
    let Some(import) = (md + 1..end).find(|&line| {
        indent(lines[line]) == key_indent && lines[line].trim_start().starts_with("import:")
    }) else {
        return Some((
            md + 1,
            format!("{pad}import:\n{}", item(&format!("{pad}  "))),
        ));
    };
    if !lines[import].trim_start()["import:".len()..]
        .trim()
        .is_empty()
    {
        return None;
    }
    // A list's items are deeper than its key, or at its indentation with their `-`.
    let in_list = |line: &str| {
        line.trim().is_empty()
            || indent(line) > key_indent
            || (indent(line) == key_indent && line.trim_start().starts_with('-'))
    };
    let items: Vec<usize> = (import + 1..end)
        .take_while(|&line| in_list(lines[line]))
        .filter(|&line| !lines[line].trim().is_empty())
        .collect();
    let item_indent = items
        .iter()
        .find(|&&line| lines[line].trim_start().starts_with('-'))
        .map_or(key_indent + 2, |&line| indent(lines[line]));
    let after = items.last().map_or(import + 1, |last| last + 1);
    Some((after, item(&" ".repeat(item_indent))))
}

#[cfg(test)]
mod tests;
