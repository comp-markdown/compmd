//! Hovers ([the language server], *Hover*): what the processor's hint at a place says, as Markdown, with the
//! diagnostics there.
//!
//! [the language server]: https://github.com/comp-markdown/compmd/blob/main/docs/language-server.md#completion-hover-signature-help-go-to-definition

use std::fmt::Write as _;

use compmd::hints::{PositionalHint, Subject};
use lsp_types::{Hover, HoverContents, MarkupContent, MarkupKind};

use super::markdown;
use crate::encoding::Encoding;

/// The hover of `hint`, in the document whose text is `text`; `None` when it says nothing.
pub(crate) fn hover(hint: &PositionalHint, text: &str, encoding: Encoding) -> Option<Hover> {
    let mut parts: Vec<String> = hint.subject.as_ref().map(about).into_iter().collect();
    parts.extend(
        hint.diagnostics
            .iter()
            .map(|diagnostic| format!("**{}** {}", diagnostic.code, diagnostic.message)),
    );
    if parts.is_empty() {
        return None;
    }
    Some(Hover {
        contents: HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value: parts.join("\n\n---\n\n"),
        }),
        range: Some(encoding.range(text, &hint.range)),
    })
}

/// What `subject` is, as Markdown.
fn about(subject: &Subject) -> String {
    match subject {
        Subject::Component {
            name,
            component: Some(component),
        } => markdown::component(name, component),
        Subject::Component {
            name,
            component: None,
        } => format!("**`{name}`** stands for no component here"),
        Subject::Language {
            renderer: Some(renderer),
            component: Some(component),
            ..
        } => markdown::component(renderer, component),
        Subject::Language { language, .. } if language.is_empty() => {
            "A code block's language selects the code renderer that draws it".to_owned()
        }
        Subject::Language { language, .. } => {
            format!("No renderer here renders `{language}`: the block shows as code")
        }
        Subject::Params { component, params } => {
            let lines: Vec<String> = params
                .iter()
                .map(|param| format!("- {}", markdown::param(param)))
                .collect();
            format!("The params `{component}` takes:\n{}", lines.join("\n"))
        }
        Subject::Param {
            component, param, ..
        } => format!("On `{component}`: {}", markdown::param(param)),
        Subject::Slot {
            component,
            slot,
            slots,
        } => {
            let names: Vec<String> = slots
                .iter()
                .map(|slot| format!("`{}`", markdown::slot_name(slot)))
                .collect();
            let declared = if names.is_empty() {
                "none".to_owned()
            } else {
                names.join(", ")
            };
            format!("**`{slot}`**, a slot of `{component}`, which declares {declared}")
        }
        Subject::Path { path, name } => {
            let mut text = format!("`{path}`");
            if let Some(ty) = &name.ty {
                let _ = write!(text, ": `{ty}`");
            }
            if let Some(about) = &name.about {
                let _ = write!(text, "\n\n{about}");
            }
            if !name.fields.is_empty() {
                let fields: Vec<String> = name
                    .fields
                    .iter()
                    .map(|field| format!("`{field}`"))
                    .collect();
                let _ = write!(text, "\n\nFields: {}", fields.join(", "));
            }
            text
        }
        Subject::Function { name, function } => match function {
            Some(function) => format!("**`{name}`**\n\n{}", markdown::function(function)),
            None => format!("**`{name}`** is no function"),
        },
        Subject::Frontmatter { path, about } => {
            let key = path.join(".");
            about.as_ref().map_or_else(
                || format!("`{key}` isn't a key CompMD reads"),
                |about| format!("`{key}`: {about}"),
            )
        }
        Subject::Type { name, takes } => match takes {
            Some(takes) => format!("**`{name}`**: {}", takes.description),
            None => format!("**`{name}`** is no type"),
        },
    }
}

#[cfg(test)]
mod tests;
