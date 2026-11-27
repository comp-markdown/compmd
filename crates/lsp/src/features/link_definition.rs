//! Go to definition on a link ([the language server], *Links and headings*): to its file, to the heading it names, to
//! the first match of its passage, and a reference to its definition.
//!
//! [the language server]: https://github.com/comp-markdown/compmd/blob/main/docs/language-server.md#links-and-headings

use lsp_types::{Location, Position};

use super::Asked;
use super::link_targets::{definition_of, link_at, location, reached, text_of, uri_of};
use crate::links::document_links::LinkKind;
use crate::links::text_span::range_of;

/// Where the link at `position` leads; `None` off a link, or for one that leads nowhere in the project.
pub(crate) fn link_definition(position: Position, asked: &Asked) -> Option<Location> {
    let link = link_at(position, asked)?;
    if let LinkKind::Reference { identifier } = &link.kind {
        let definition = definition_of(identifier, asked)?;
        return location(asked.path, Some(&definition.position), asked);
    }
    let reached = reached(&link, asked)?;
    if let Some(passage) = reached.passage {
        let text = text_of(&reached.file, asked)?;
        return Some(Location {
            uri: uri_of(&reached.file)?,
            range: range_of(&text, asked.encoding, passage),
        });
    }
    location(
        &reached.file,
        reached.heading.as_ref().map(|heading| &heading.position),
        asked,
    )
}
