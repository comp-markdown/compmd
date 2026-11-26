//! The packs the editor suggests from registries ([the language server], decision 7, Q2): the latest release of each
//! pack its suggested scopes list, CompMD's own first, then the project's (`editor.suggest`), then the writer's (the
//! `compmd.packs.suggest` setting). They only suggest: completion and auto-import offer them, and nothing the
//! document is processed with comes from them.
//!
//! [the language server]: https://github.com/comp-markdown/compmd/blob/main/docs/language-server.md

use std::collections::HashMap;
use std::rc::Rc;

use compmd::host::{KnownPack, LoadedRelease, Packs, Release};
use compmd_host::pack::{PackName, Place};
use compmd_host::registry::Resolved;

use crate::fetched_packs::loaded;
use crate::own_packs::SCOPE;

/// The releases fetched for suggesting, by pack.
type Held = HashMap<String, Vec<LoadedRelease>>;

/// A worker's suggested packs, and the scopes they were asked for.
#[derive(Default)]
pub(crate) struct SuggestedPacks {
    held: Rc<Held>,
    /// The scopes asked, in order; `None` until they are, or after the project is read again.
    asked: Option<Vec<String>>,
}

/// The suggested packs as a source: the held releases, listed in their scopes' order.
struct InScopeOrder {
    held: Rc<Held>,
    scopes: Vec<String>,
}

impl Packs for InScopeOrder {
    fn releases(&self, pack: &PackName) -> Vec<Release> {
        self.held.releases(pack)
    }

    fn pack_file(&self, place: &Place) -> Option<String> {
        self.held.pack_file(place)
    }

    /// The held releases, by their scope's place in the order, then as held: by name, then version.
    fn known(&self) -> Vec<KnownPack> {
        let mut known = self.held.known();
        let rank = |known: &KnownPack| {
            let scope = format!("@{}", known.release.pack.scope());
            self.scopes.iter().position(|asked| *asked == scope)
        };
        known.sort_by_key(rank);
        known
    }
}

/// The scopes suggested for a project whose configuration suggests `project`'s and whose writer suggests `writer`'s:
/// CompMD's own, then the project's, then the writer's, each once.
pub(crate) fn scopes_of(project: &[String], writer: &[String]) -> Vec<String> {
    let mut scopes = vec![SCOPE.to_owned()];
    for scope in project.iter().chain(writer) {
        if !scopes.contains(scope) {
            scopes.push(scope.clone());
        }
    }
    scopes
}

impl SuggestedPacks {
    /// Whether `scopes` have to be asked for: they weren't, as they are now. They are then asked.
    pub(crate) fn ask(&mut self, scopes: &[String]) -> bool {
        if self.asked.as_deref() == Some(scopes) {
            return false;
        }
        self.asked = Some(scopes.to_vec());
        true
    }

    /// Forgets the scopes asked, so they are asked again: after the project is read again.
    pub(crate) fn ask_again(&mut self) {
        self.asked = None;
    }

    /// What an ask got: its releases, in place of those held before.
    pub(crate) fn add(&mut self, resolved: Resolved) {
        let mut held = Held::new();
        for (pack, releases) in resolved.releases {
            held.insert(
                pack.as_str().to_owned(),
                releases.into_iter().map(loaded).collect(),
            );
        }
        self.held = Rc::new(held);
    }

    /// The suggested packs, as a source the known packs are read from, in their scopes' order.
    pub(crate) fn source(&self) -> Box<dyn Packs> {
        Box::new(InScopeOrder {
            held: Rc::clone(&self.held),
            scopes: self.asked.clone().unwrap_or_default(),
        })
    }
}

#[cfg(test)]
mod tests;
