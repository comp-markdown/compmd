//! `compmd-lsp`: the [CompMD](https://compmd.dev/) language server ([the language server]), on the processor. One
//! binary, synchronous, on threads: a main loop reads the client's messages and hands each document's work to its
//! own worker, which owns the document's processor and posts its diagnostics (decision 2).
//!
//! A document's root is the nearest folder above it with a configuration, else the editor's workspace folder, else
//! its own (decision 3); its configuration and packs are read as a host reads them (`compmd-project`), so the editor
//! sees what the site will render (decision 4), and its components are analysed (W104, `compmd-components`).
//!
//! [the language server]: https://github.com/comp-markdown/compmd/blob/main/docs/language-server.md

mod add_registry;
mod browser;
mod diagnostics;
mod document_preview;
mod encoding;
mod error;
mod features;
mod fetched_packs;
mod files;
mod json_object;
mod markdown_file;
mod own_packs;
mod pack_components;
mod places;
mod preview;
mod processor_reads;
mod project;
mod registry_packs;
mod server;
mod settings;
mod suggested_packs;
mod watched_files;
mod worker;

pub use error::ServerError;
pub use server::serve;
