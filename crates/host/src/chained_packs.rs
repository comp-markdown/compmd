//! Pack sources chained ([the language server], decision 5): each a `compmd::host::Packs` (folder packs, installed
//! packs, a registry's cache, a bundle), asked in the host's order, so one abstraction holds every source.
//!
//! [the language server]: https://github.com/comp-markdown/compmd/blob/main/docs/language-server.md

use compmd::host::{Packs, Release};
use compmd_pack::{PackName, Place};

/// Pack sources asked in order: a pack's releases are the first source's that has any, and a file is the first that
/// has it, so a source earlier in the chain hides a pack later ones have.
///
/// ```
/// use compmd_host::ChainedPacks;
///
/// let packs = ChainedPacks::new(vec![Box::new(()), Box::new(())]);
/// # drop(packs);
/// ```
pub struct ChainedPacks {
    sources: Vec<Box<dyn Packs>>,
}

impl ChainedPacks {
    /// The chain of `sources`, the first asked first.
    #[must_use]
    pub fn new(sources: Vec<Box<dyn Packs>>) -> Self {
        Self { sources }
    }
}

impl Packs for ChainedPacks {
    fn releases(&self, pack: &PackName) -> Vec<Release> {
        self.sources
            .iter()
            .map(|source| source.releases(pack))
            .find(|releases| !releases.is_empty())
            .unwrap_or_default()
    }

    fn pack_file(&self, place: &Place) -> Option<String> {
        self.sources.iter().find_map(|source| {
            let has_release = source
                .releases(&place.release.pack)
                .iter()
                .any(|release| release.version == place.release.version);
            has_release.then(|| source.pack_file(place)).flatten()
        })
    }

    fn pending(&self, pack: &PackName) -> bool {
        self.sources.iter().any(|source| source.pending(pack))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use compmd::host::LoadedRelease;

    use super::*;

    /// A source holding `pack` at `version`, its document saying `text`.
    fn source(pack: &str, version: &str, text: &str) -> Box<dyn Packs> {
        let release: LoadedRelease = serde_json::from_value(serde_json::json!({
            "version": version,
            "integrity": null,
            "files": { "index.comp.md": text }
        }))
        .expect("a release");
        Box::new(HashMap::from([(pack.to_owned(), vec![release])]))
    }

    /// A source still getting `@kit/late`.
    struct Getting;

    impl Packs for Getting {
        fn releases(&self, _pack: &PackName) -> Vec<Release> {
            Vec::new()
        }

        fn pack_file(&self, _place: &Place) -> Option<String> {
            None
        }

        fn pending(&self, pack: &PackName) -> bool {
            *pack == "@kit/late"
        }
    }

    #[test]
    fn the_first_source_with_a_pack_gives_its_releases_and_files() {
        let packs = ChainedPacks::new(vec![
            source("@kit/notes", "1.0.0", "# Folder"),
            source("@kit/notes", "2.0.0", "# Installed"),
            source("@kit/mermaid", "1.0.0", "# Mermaid"),
            Box::new(Getting),
        ]);
        let name = |text: &str| text.parse::<PackName>().expect("a name");
        let place = |text: &str| text.parse::<Place>().expect("a place");
        let versions: Vec<String> = packs
            .releases(&name("@kit/notes"))
            .iter()
            .map(|release| release.version.to_string())
            .collect();
        assert_eq!(versions, ["1.0.0"], "the folder's hides the installed one");
        assert_eq!(
            packs
                .pack_file(&place("@kit/notes@1.0.0/index.comp.md"))
                .as_deref(),
            Some("# Folder")
        );
        assert_eq!(
            packs
                .pack_file(&place("@kit/notes@2.0.0/index.comp.md"))
                .as_deref(),
            Some("# Installed")
        );
        assert_eq!(
            packs
                .pack_file(&place("@kit/mermaid@1.0.0/index.comp.md"))
                .as_deref(),
            Some("# Mermaid")
        );
        assert_eq!(
            packs.pack_file(&place("@kit/mermaid@9.0.0/index.comp.md")),
            None
        );
        assert!(packs.pending(&name("@kit/late")));
        assert!(!packs.pending(&name("@kit/notes")));
        assert_eq!(packs.releases(&name("@kit/none")), []);
    }
}
