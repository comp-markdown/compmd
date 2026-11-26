//! A document's registry packs ([the language server], decision 6): the releases fetched for it, the packs on their
//! way, which it renders pending (spec §6.4), and those the size cap refuses, for its policy. Each import is asked
//! once: one that can't be had stays missing, F026, until the project is read again.
//!
//! [the language server]: https://github.com/comp-markdown/compmd/blob/main/docs/language-server.md

use std::collections::{BTreeMap, HashMap, HashSet};
use std::rc::Rc;

use compmd::host::{LoadedRelease, NeededPack, Packs, Release, WithPending};
use compmd_host::pack::PackName;
use compmd_host::registry::{Needed, Problem, Resolved};

/// The releases fetched, by pack, with the packs on their way.
type Held = WithPending<HashMap<String, Vec<LoadedRelease>>>;

/// A document's registry packs.
#[derive(Default)]
pub(crate) struct FetchedPacks {
    held: Rc<Held>,
    /// How many asks are on their way for each pack.
    on_their_way: HashMap<String, usize>,
    refused: BTreeMap<PackName, String>,
    /// Each import asked, by its pack, range and integrity.
    asked: HashSet<String>,
}

/// An import as one key: its pack, range and pinned integrity.
fn key_of(needed: &NeededPack) -> String {
    let integrity = needed
        .integrity
        .as_ref()
        .map(ToString::to_string)
        .unwrap_or_default();
    format!("{}@{}#{integrity}", needed.pack, needed.range)
}

/// A release the registry rules gave, as the processor holds one.
pub(crate) fn loaded(release: compmd_host::registry::Release) -> LoadedRelease {
    LoadedRelease {
        release: Release {
            version: release.version,
            integrity: release
                .integrity
                .and_then(|integrity| integrity.parse().ok()),
            broken: release.broken,
        },
        files: release.files.into_iter().collect(),
    }
}

impl FetchedPacks {
    /// The packs, as a source the processor reads, after the project's own.
    pub(crate) fn source(&self) -> Box<dyn Packs> {
        Box::new(Rc::clone(&self.held))
    }

    /// The packs the size cap refuses, each with why, for the policy's `refused`.
    pub(crate) fn refused(&self) -> &BTreeMap<PackName, String> {
        &self.refused
    }

    /// The imports of `needed` not asked yet, as the registry rules take them: each is now asked, and its pack on
    /// its way.
    pub(crate) fn ask(&mut self, needed: Vec<NeededPack>) -> Vec<Needed> {
        let new: Vec<NeededPack> = needed
            .into_iter()
            .filter(|needed| self.asked.insert(key_of(needed)))
            .collect();
        for needed in &new {
            *self
                .on_their_way
                .entry(needed.pack.as_str().to_owned())
                .or_default() += 1;
        }
        self.update_pending();
        new.into_iter()
            .map(|needed| Needed {
                pack: needed.pack,
                range: needed.range,
                integrity: needed.integrity,
                library: needed.library,
            })
            .collect()
    }

    /// What an ask for `packs` got: its releases are held, its refusals kept, and the packs no longer on their way.
    /// Gives what couldn't be got, for the log.
    pub(crate) fn add(&mut self, packs: &[PackName], resolved: Resolved) -> Vec<Problem> {
        for pack in packs {
            if let Some(count) = self.on_their_way.get_mut(pack.as_str()) {
                *count -= 1;
                if *count == 0 {
                    self.on_their_way.remove(pack.as_str());
                }
            }
        }
        let held = Rc::make_mut(&mut self.held);
        for (pack, releases) in resolved.releases {
            let kept = held.held.entry(pack.as_str().to_owned()).or_default();
            for release in releases {
                if !kept
                    .iter()
                    .any(|loaded| loaded.release.version == release.version)
                {
                    kept.push(loaded(release));
                }
            }
        }
        self.refused.extend(resolved.refused);
        self.update_pending();
        resolved.problems
    }

    /// Forgets what was asked, so an import still missing is asked again: after the project is read again.
    pub(crate) fn ask_again(&mut self) {
        self.asked.clear();
    }

    fn update_pending(&mut self) {
        let pending: HashSet<String> = self.on_their_way.keys().cloned().collect();
        if self.held.pending != pending {
            Rc::make_mut(&mut self.held).pending = pending;
        }
    }
}

#[cfg(test)]
mod tests;
