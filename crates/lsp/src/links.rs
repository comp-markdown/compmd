//! Links and headings ([the language server], decision 15): what a document's links lead to and the headings they
//! reach, read from the processor's trees. The project index keeps them for every Markdown file of a root.
//!
//! [the language server]: https://github.com/comp-markdown/compmd/blob/main/docs/language-server.md#links-and-headings

pub(crate) mod document_links;
pub(crate) mod headings;
pub(crate) mod link_parts;
pub(crate) mod passage;
pub(crate) mod relative_path;
pub(crate) mod target;
pub(crate) mod text_span;
pub(crate) mod typed_link;
pub(crate) mod wiki_links;
pub(crate) mod wiki_target;
