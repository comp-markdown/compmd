//! The files whose changes reload the open documents ([the language server], decision 8): the configuration, the
//! packages and their lock files, and what a document can import (CompMD documents and data files, spec §6). A client
//! only sends changes of the files a server registers for.
//!
//! [the language server]: https://github.com/comp-markdown/compmd/blob/main/docs/language-server.md

use lsp_types::notification::{DidChangeWatchedFiles, Notification as _};
use lsp_types::{
    DidChangeWatchedFilesRegistrationOptions, FileSystemWatcher, GlobPattern, Registration,
};

/// The globs watched: `compmd.config.json`, `package.json` and the npm and pnpm lock files are among the JSON and YAML
/// files.
const GLOBS: [&str; 3] = ["**/*.md", "**/*.{json,yaml,yml,csv,tsv}", "**/yarn.lock"];

/// The registration of the watched files, for `client/registerCapability`.
pub(crate) fn registration() -> Registration {
    let watchers = GLOBS
        .iter()
        .map(|glob| FileSystemWatcher {
            glob_pattern: GlobPattern::String((*glob).to_owned()),
            kind: None,
        })
        .collect();
    Registration {
        id: DidChangeWatchedFiles::METHOD.to_owned(),
        method: DidChangeWatchedFiles::METHOD.to_owned(),
        register_options: serde_json::to_value(DidChangeWatchedFilesRegistrationOptions {
            watchers,
        })
        .ok(),
    }
}
