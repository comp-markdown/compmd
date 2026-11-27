//! Go to definition on a link ([the language server], *Links and headings*): to its file, to the heading it names,
//! and a reference to its definition.
//!
//! [the language server]: https://github.com/comp-markdown/compmd/blob/main/docs/language-server.md#links-and-headings

use lsp_types::{Location, Position};

use super::Asked;
use super::link_targets::{definition_of, link_at, location, reached};
use crate::links::document_links::LinkKind;

/// Where the link at `position` leads; `None` off a link, or for one that leads nowhere in the project.
pub(crate) fn link_definition(position: Position, asked: &Asked) -> Option<Location> {
    let link = link_at(position, asked)?;
    if let LinkKind::Reference { identifier } = &link.kind {
        let definition = definition_of(identifier, asked)?;
        return location(asked.path, Some(&definition.position), asked);
    }
    let reached = reached(&link, asked)?;
    location(
        &reached.file,
        reached.heading.as_ref().map(|heading| &heading.position),
        asked,
    )
}
