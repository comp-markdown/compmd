//! *Open preview* ([the language server], decision 13): on any line, the command that opens the document's preview in
//! the browser, `compmd.preview`, with the document's URI.
//!
//! [the language server]: https://github.com/comp-markdown/compmd/blob/main/docs/language-server.md

use lsp_types::{CodeAction, CodeActionKind, Command};
use serde_json::Value;

use super::Asked;

/// The command that opens a document's preview, its URI its one argument.
pub(crate) const COMMAND: &str = "compmd.preview";

/// The action's kind: a source action of CompMD's own.
pub(super) const KIND: &str = "source.compmd.preview";

/// *Open preview*, for the document `asked` is about.
pub(super) fn open_preview(asked: &Asked) -> CodeAction {
    let title = "Open preview".to_owned();
    CodeAction {
        title: title.clone(),
        kind: Some(CodeActionKind::new(KIND)),
        command: Some(Command {
            title,
            command: COMMAND.to_owned(),
            arguments: Some(vec![Value::String(asked.uri.as_str().to_owned())]),
        }),
        ..CodeAction::default()
    }
}
