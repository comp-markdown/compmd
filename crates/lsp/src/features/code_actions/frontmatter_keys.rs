//! The common frontmatter keys a document lacks, offered one by one and never added unasked ([the language server],
//! Q7): `title`, `description`, `date` and `tags`, the author's, and `md.lang`. Each is added with an empty value (the
//! language `en`, its default) at the top of the frontmatter, or of `md`; a document without a frontmatter gets one.
//!
//! [the language server]: https://github.com/comp-markdown/compmd/blob/main/docs/language-server.md#:~:text=Q7.

use compmd::frontmatter::Syntax;
use lsp_types::{CodeAction, CodeActionKind, Position, Range, TextEdit};
use serde_json::Value;

use super::frontmatter::Frontmatter;
use super::{Asked, editing};

/// The kind of these actions: a source action, apart from organizing, so that organizing on save adds nothing.
pub(crate) const KIND: &str = "source.addFrontmatterKey.compmd";

/// The author's common keys, each with the value it gets, written alike in YAML and TOML.
const AUTHOR_KEYS: [(&str, &str); 4] = [
    ("title", "\"\""),
    ("description", "\"\""),
    ("date", "\"\""),
    ("tags", "[]"),
];

/// The actions that add the common keys the document lacks, when `range` starts in its frontmatter or, without one,
/// on its first line.
pub(super) fn offered(range: Range, asked: &Asked) -> Vec<CodeAction> {
    let Some(frontmatter) = Frontmatter::of(asked) else {
        return without_frontmatter(range, asked);
    };
    if !frontmatter.holds(range.start) {
        return Vec::new();
    }
    let mut actions: Vec<CodeAction> = AUTHOR_KEYS
        .iter()
        .filter(|(name, _)| !frontmatter.value.contains_key(*name))
        .map(|(name, value)| {
            let entry = match frontmatter.syntax {
                Syntax::Yaml => format!("{name}: {value}\n"),
                Syntax::Toml => format!("{name} = {value}\n"),
            };
            adding(name, inserted(frontmatter.first_line(), entry), asked)
        })
        .collect();
    let md = frontmatter.value.get("md");
    if md.and_then(|md| md.get("lang")).is_none()
        && let Some(edit) = adding_lang(&frontmatter, md, asked)
    {
        actions.push(adding("md.lang", edit, asked));
    }
    actions
}

/// The keys offered to a document without a frontmatter, each in a YAML frontmatter of its own.
fn without_frontmatter(range: Range, asked: &Asked) -> Vec<CodeAction> {
    if range.start.line != 0 || asked.processor.is_none() {
        return Vec::new();
    }
    let author = AUTHOR_KEYS
        .iter()
        .map(|(name, value)| (*name, format!("{name}: {value}\n")));
    author
        .chain([("md.lang", "md:\n  lang: en\n".to_owned())])
        .map(|(name, entry)| adding(name, inserted(0, format!("---\n{entry}---\n\n")), asked))
        .collect()
}

/// The edit that adds `md.lang`: under `md`'s line or its `[md]` table, a new `md` at the end, or the frontmatter
/// rewritten when `md` is written on one line.
fn adding_lang(frontmatter: &Frontmatter, md: Option<&Value>, asked: &Asked) -> Option<TextEdit> {
    let lines: Vec<&str> = asked.text.split('\n').collect();
    let md_line = frontmatter.key_lines.get("/md").copied();
    let text_of = |line: u32| lines.get(line as usize).copied().unwrap_or_default();
    match (frontmatter.syntax, md_line) {
        (Syntax::Yaml, None) => Some(inserted(
            frontmatter.closing_line(),
            "md:\n  lang: en\n".into(),
        )),
        (Syntax::Yaml, Some(line)) if text_of(line).trim_end() == "md:" => {
            let indent = first_child_indent(frontmatter, &lines).unwrap_or(2);
            Some(inserted(
                line + 1,
                format!("{}lang: en\n", " ".repeat(indent)),
            ))
        }
        (Syntax::Toml, _) => {
            let header = (frontmatter.first_line()..frontmatter.closing_line())
                .find(|line| text_of(*line).trim() == "[md]");
            match header {
                Some(line) => Some(inserted(line + 1, "lang = \"en\"\n".into())),
                None if md_line
                    .is_none_or(|line| !text_of(line).trim_start().starts_with("md =")) =>
                {
                    Some(inserted(
                        frontmatter.first_line(),
                        "md.lang = \"en\"\n".into(),
                    ))
                }
                None => rewritten_with_lang(frontmatter, md),
            }
        }
        (Syntax::Yaml, Some(_)) => rewritten_with_lang(frontmatter, md),
    }
}

/// The indentation of the first key under `md`, as written.
fn first_child_indent(frontmatter: &Frontmatter, lines: &[&str]) -> Option<usize> {
    let line = frontmatter
        .key_lines
        .iter()
        .filter(|(pointer, _)| pointer.starts_with("/md/") && pointer[4..].find('/').is_none())
        .map(|(_, line)| *line)
        .min()?;
    let text = lines.get(line as usize)?;
    Some(text.len() - text.trim_start_matches(' ').len())
}

/// The frontmatter rewritten with `md.lang`, for an `md` an insertion can't extend.
fn rewritten_with_lang(frontmatter: &Frontmatter, md: Option<&Value>) -> Option<TextEdit> {
    let mut value = frontmatter.value.clone();
    let mut md = md.and_then(Value::as_object).cloned().unwrap_or_default();
    md.insert("lang".into(), Value::String("en".into()));
    value.insert("md".into(), Value::Object(md));
    frontmatter.rewritten(frontmatter.syntax, &value).ok()
}

/// The edit that inserts `text` at the start of `line`.
fn inserted(line: u32, text: String) -> TextEdit {
    let at = Position { line, character: 0 };
    TextEdit {
        range: Range { start: at, end: at },
        new_text: text,
    }
}

/// The action that adds the key `name` with `edit`.
fn adding(name: &str, edit: TextEdit, asked: &Asked) -> CodeAction {
    CodeAction {
        title: format!("Add `{name}` to the frontmatter"),
        kind: Some(CodeActionKind::new(KIND)),
        edit: Some(editing(asked, vec![edit])),
        ..CodeAction::default()
    }
}

#[cfg(test)]
mod tests;
