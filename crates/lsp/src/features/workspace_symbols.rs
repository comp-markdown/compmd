//! Workspace symbols (`workspace/symbol`, [the language server], *Links and headings*): every indexed file's
//! headings, those whose text holds the query's characters in order, any case.
//!
//! [the language server]: https://github.com/comp-markdown/compmd/blob/main/docs/language-server.md#links-and-headings

use lsp_types::{Location, OneOf, Position, Range, SymbolKind, WorkspaceSymbol};

use super::link_targets::{line_of, uri_of};
use crate::project_index::ProjectIndex;

/// The headings of `indexes` that `query` matches, each at the start of its line.
pub(crate) fn workspace_symbols<'a>(
    query: &str,
    indexes: impl IntoIterator<Item = &'a ProjectIndex>,
) -> Vec<WorkspaceSymbol> {
    let mut found = Vec::new();
    for index in indexes {
        for (file, indexed) in index.files().iter() {
            let Some(uri) = uri_of(file) else {
                continue;
            };
            let container = file
                .strip_prefix(index.root())
                .unwrap_or(file)
                .to_string_lossy()
                .replace('\\', "/");
            for heading in indexed
                .headings
                .iter()
                .filter(|heading| matches(query, &heading.text))
            {
                let start = Position {
                    line: line_of(&heading.position),
                    character: 0,
                };
                found.push(WorkspaceSymbol {
                    name: heading.text.clone(),
                    kind: SymbolKind::STRING,
                    tags: None,
                    container_name: Some(container.clone()),
                    location: OneOf::Left(Location {
                        uri: uri.clone(),
                        range: Range { start, end: start },
                    }),
                    data: None,
                });
            }
        }
    }
    found
}

/// Whether `text` holds `query`'s characters in order, any case: `hn` matches `Harbour notes`.
fn matches(query: &str, text: &str) -> bool {
    let mut characters = text.chars().flat_map(char::to_lowercase);
    query
        .chars()
        .flat_map(char::to_lowercase)
        .filter(|character| !character.is_whitespace())
        .all(|wanted| characters.any(|character| character == wanted))
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::project_index::IndexedFile;

    #[test]
    fn a_query_matches_its_characters_in_order_any_case() {
        assert!(matches("hn", "Harbour notes"));
        assert!(matches("", "Anything"));
        assert!(matches("TIDE t", "High tide tables"));
        assert!(!matches("nh", "Harbour"));
    }

    #[test]
    fn every_indexed_files_headings_with_their_file() {
        let index = ProjectIndex::new(PathBuf::from("/site"));
        let file = PathBuf::from("/site/posts/harbour.md");
        index.set(
            file.clone(),
            IndexedFile::read(&file, "# Harbour notes\n\n## Tides\n", false),
        );
        let found = workspace_symbols("tid", [index.as_ref()]);
        assert_eq!(found.len(), 1);
        assert_eq!(
            (found[0].name.as_str(), found[0].container_name.as_deref()),
            ("Tides", Some("posts/harbour.md"))
        );
        let OneOf::Left(location) = &found[0].location else {
            panic!("a location");
        };
        assert_eq!(location.range.start.line, 2);
        assert_eq!(workspace_symbols("", [index.as_ref()]).len(), 2);
    }
}
