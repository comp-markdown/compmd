//! CompMD's own packs with no configuration ([the language server], decision 4 and its exception): when a site's
//! configuration names no registry for `@comp-markdown`, the editor resolves its packs from npm all the same, and
//! each import of one the site can't load gets a warning saying so, with the fix that adds the registry (Q4).
//!
//! [the language server]: https://github.com/comp-markdown/compmd/blob/main/docs/language-server.md

use std::collections::BTreeMap;

use compmd::Processor;
use compmd::pack::PackName;
use compmd::processor::Expanded;
use lsp_types::{Diagnostic as LspDiagnostic, DiagnosticSeverity, Range};

use crate::add_registry::AddRegistry;
use crate::diagnostics::SOURCE;
use crate::encoding::Encoding;
use crate::places::Places;

/// The scope of CompMD's own packs.
pub(crate) const SCOPE: &str = "@comp-markdown";

/// The registry the editor resolves them from when the site names none: npm's.
pub(crate) const NPM: &str = "https://registry.npmjs.org/";

/// Whether a site's `registries` name one for CompMD's own packs: their scope's, or every other scope's.
pub(crate) fn site_fetches(registries: &BTreeMap<String, String>) -> bool {
    registries.contains_key(SCOPE) || registries.contains_key("*")
}

/// Whether `pack` is one of CompMD's own.
fn is_own(pack: &PackName) -> bool {
    SCOPE.strip_prefix('@') == Some(pack.scope())
}

/// A document's imports of CompMD's own packs that its site can't load, when its configuration names no registry for
/// them.
#[derive(Debug, Default)]
pub(crate) struct Unregistered {
    /// The edit that adds npm for them, when the configuration's file can be edited.
    pub(crate) add: Option<AddRegistry>,
    /// Each import of one whose release the editor fetched, the site having none on the disk, as a warning.
    pub(crate) warnings: Vec<LspDiagnostic>,
}

impl Unregistered {
    /// Finds the imports of `processor`'s document, whose text is `text`, that took a release of one of CompMD's own
    /// packs that isn't one of its project's (`places`).
    pub(crate) fn find(
        &mut self,
        processor: &Processor<Expanded>,
        places: &Places,
        text: &str,
        encoding: Encoding,
    ) {
        self.warnings = processor
            .imported_packs()
            .into_iter()
            .filter(|(_, release)| is_own(&release.pack) && !places.holds(release))
            .map(|(at, release)| {
                let at = encoding.position(text, at.line, at.column);
                LspDiagnostic {
                    range: Range { start: at, end: at },
                    severity: Some(DiagnosticSeverity::WARNING),
                    source: Some(SOURCE.to_owned()),
                    message: format!(
                        "your site can't load `{}`: install it, or add a registry",
                        release.pack
                    ),
                    ..LspDiagnostic::default()
                }
            })
            .collect();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_site_fetches_compmds_packs_from_their_scopes_registry_or_every_scopes() {
        let named = |scope: &str| BTreeMap::from([(scope.to_owned(), NPM.to_owned())]);
        assert!(site_fetches(&named("@comp-markdown")));
        assert!(site_fetches(&named("*")));
        assert!(!site_fetches(&named("@kit")));
        assert!(!site_fetches(&BTreeMap::new()));
    }

    #[test]
    fn compmds_own_packs_are_those_of_its_scope() {
        let pack = |name: &str| name.parse::<PackName>().expect("a name");
        assert!(is_own(&pack("@comp-markdown/mermaid")));
        assert!(!is_own(&pack("@kit/notes")));
    }
}
