//! Library packs (the companion, *CompMD packs on a registry*, section 5; `docs/sharing-library-packs.md`): the
//! libraries the packs a document loads name in `peerDependencies`, one version of each per document, and W121 for
//! one that can't be had.

use super::source::range;
use super::{DocumentId, LibraryModule, Origin, Store};
use crate::diagnostic::Location;
use crate::host::{NeededPack, Release};
use crate::policy::Policy;
use compmd_pack::{FilePath, PackName, Place, ReleaseId};
use semver::VersionReq;
use serde_json::Value;
use std::collections::{BTreeMap, HashMap, HashSet};

/// The most rounds of choosing: a library's own libraries are asked for once it is chosen, a level a round.
const ROUNDS: usize = 16;

/// A `peerDependencies` entry that names a library pack.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Ask {
    /// The pack or library that names it.
    asker: ReleaseId,
    library: PackName,
    /// The range, as written and as read.
    written: String,
    range: VersionReq,
    /// The import it is reported on: of the pack that names it, or of the pack whose library names it.
    at: (DocumentId, Location),
}

impl Ask {
    /// The entry `library: written` of `asker`'s `peerDependencies`, reported at `at`; `None` for a name that isn't
    /// a pack's or a range the processor can't read, which aren't checked.
    pub(super) fn new(
        asker: ReleaseId,
        library: &str,
        written: &str,
        at: (DocumentId, Location),
    ) -> Option<Self> {
        let (range, _) = range(written)?;
        Some(Self {
            asker,
            library: PackName::new(library).ok()?,
            written: written.to_string(),
            range,
            at,
        })
    }
}

/// The libraries chosen for a document: what each pack's and library's imports of a library name, each the place of
/// the library's module.
#[derive(Clone, Debug, Default)]
pub(crate) struct Libraries(HashMap<ReleaseId, BTreeMap<String, Place>>);

impl Libraries {
    /// The libraries `release`, a pack's or a library's, imports, by name, each with its own.
    pub(crate) fn of(&self, release: &ReleaseId) -> BTreeMap<String, LibraryModule> {
        self.0
            .get(release)
            .into_iter()
            .flatten()
            .map(|(import, place)| {
                let module = LibraryModule {
                    place: place.clone(),
                    libraries: self.of(&place.release),
                };
                (import.clone(), module)
            })
            .collect()
    }
}

/// A library a pack or library can't have (W121): where it is reported, who names it, and what is missing.
pub(super) struct Unmet {
    pub(super) at: (DocumentId, Location),
    pub(super) asker: ReleaseId,
    pub(super) missing: String,
}

/// What choosing gave: the libraries, those that can't be had, and those to fetch.
pub(super) struct Chosen {
    pub(super) libraries: Libraries,
    pub(super) unmet: Vec<Unmet>,
    pub(super) needed: Vec<NeededPack>,
}

/// Why an ask has no library.
#[derive(Clone, Debug)]
enum Unhad {
    /// No release matches its range: the host may fetch one.
    Absent,
    /// What the message says after the library and its range.
    Why(String),
}

type Outcome = Result<Place, Unhad>;

/// Chooses a version of each library the packs' `asks` name, in the order they were loaded, from the releases in
/// `store`, under `policy`; then those the chosen libraries name, until none is new.
pub(super) fn choose(
    store: Store<'_>,
    policy: &Policy,
    ambient: &[PackName],
    asks: &[Ask],
) -> Chosen {
    let chooser = Chooser {
        store,
        policy,
        ambient,
    };
    let mut all = asks.to_vec();
    let mut outcomes = chooser.round(&all);
    for _ in 0..ROUNDS {
        let next: Vec<Ask> = asks
            .iter()
            .cloned()
            .chain(chooser.asks_of_libraries(&all, &outcomes))
            .collect();
        if next == all {
            break;
        }
        outcomes = chooser.round(&next);
        all = next;
    }
    chooser.chosen(&all, &outcomes)
}

struct Chooser<'s, 'h> {
    store: Store<'s>,
    policy: &'h Policy,
    ambient: &'h [PackName],
}

impl Chooser<'_, '_> {
    /// Each ask's library, as one round chooses it: each library's asks together.
    fn round(&self, asks: &[Ask]) -> Vec<Outcome> {
        let mut outcomes = vec![Err(Unhad::Absent); asks.len()];
        let mut names: Vec<&PackName> = Vec::new();
        for ask in asks {
            if !names.contains(&&ask.library) {
                names.push(&ask.library);
            }
        }
        for name in names {
            let indices: Vec<usize> = (0..asks.len())
                .filter(|index| asks[*index].library == *name)
                .collect();
            let theirs: Vec<&Ask> = indices.iter().map(|index| &asks[*index]).collect();
            for (index, outcome) in indices.into_iter().zip(self.library(name, &theirs)) {
                outcomes[index] = outcome;
            }
        }
        outcomes
    }

    /// The release of `name` each of `asks` gets: the newest that matches every range; else each its own newest,
    /// unless the library is `single`, when the first to get one decides.
    fn library(&self, name: &PackName, asks: &[&Ask]) -> Vec<Outcome> {
        let refusal = if name.is_reserved() {
            Some("which is reserved, so no host fetches it".to_string())
        } else {
            self.policy
                .refusal(name, self.ambient)
                .map(|why| format!("but {why}"))
        };
        if let Some(why) = refusal {
            return asks.iter().map(|_| Err(Unhad::Why(why.clone()))).collect();
        }
        let releases = self.store.releases(name);
        let newest = |ranges: &[&VersionReq]| {
            releases
                .iter()
                .filter(|release| ranges.iter().all(|range| range.matches(&release.version)))
                .max_by(|a, b| a.version.cmp(&b.version))
        };
        let ranges: Vec<&VersionReq> = asks.iter().map(|ask| &ask.range).collect();
        if let Some(release) = newest(&ranges) {
            return asks.iter().map(|_| self.checked(name, release)).collect();
        }
        let own: Vec<Option<&Release>> = asks.iter().map(|ask| newest(&[&ask.range])).collect();
        let first = asks
            .iter()
            .zip(&own)
            .find_map(|(ask, release)| release.map(|release| (*ask, release)));
        let single = first.filter(|(_, release)| self.is_single(name, &release.version));
        asks.iter()
            .zip(own)
            .map(|(ask, release)| match (release, single) {
                (None, _) => Err(Unhad::Absent),
                (Some(_), Some((first, decided))) if !ask.range.matches(&decided.version) => Err(Unhad::Why(format!(
                    "which can't have two copies in one sandbox, and this document has {}, for `{}`",
                    decided.version, first.written
                ))),
                (Some(_), Some((_, decided))) => self.checked(name, decided),
                (Some(release), None) => self.checked(name, release),
            })
            .collect()
    }

    /// The module of `name`'s `release`, if it can be used: not denied or broken, a library pack, with its file.
    fn checked(&self, name: &PackName, release: &Release) -> Outcome {
        let version = &release.version;
        let id = ReleaseId::new(name.clone(), version.clone());
        if let Some(why) = self.policy.denial(name, version) {
            return Err(Unhad::Why(format!("but {why}")));
        }
        if let Some(why) = &release.broken {
            return Err(Unhad::Why(format!("but `{id}`: {why}")));
        }
        let Some(path) = self.library_path(&id) else {
            return Err(Unhad::Why(format!("but `{id}` isn't a library pack")));
        };
        let module = Place::new(id, path);
        if self.store.read(&Origin::Pack(module.clone())).is_none() {
            return Err(Unhad::Why(format!(
                "but `{}` doesn't have its module `{}`",
                module.release, module.path
            )));
        }
        Ok(module)
    }

    /// The `package.json` of `release`, when there is one and it is JSON.
    fn manifest(&self, release: &ReleaseId) -> Option<Value> {
        let path = FilePath::new("package.json").ok()?;
        let text = self
            .store
            .read(&Origin::Pack(Place::new(release.clone(), path)))?;
        serde_json::from_str(&text).ok()
    }

    /// The module a library pack's `compmd` field names, `{ "library": path }`; `None` for another package, or a
    /// path that isn't one in the pack.
    fn library_path(&self, release: &ReleaseId) -> Option<FilePath> {
        let manifest = self.manifest(release)?;
        let path = manifest.get("compmd")?.get("library")?.as_str()?;
        FilePath::new(path.trim_start_matches("./")).ok()
    }

    /// Whether the library pack says it can't have two copies in one sandbox: `"single": true` in its `compmd` field.
    fn is_single(&self, name: &PackName, version: &semver::Version) -> bool {
        self.manifest(&ReleaseId::new(name.clone(), version.clone()))
            .and_then(|manifest| manifest.get("compmd")?.get("single")?.as_bool())
            .unwrap_or(false)
    }

    /// The `peerDependencies` of `release`, a pack's or a library's, in order.
    fn needs(&self, release: &ReleaseId) -> Vec<(String, String)> {
        let manifest = self.manifest(release);
        let needs = manifest
            .as_ref()
            .and_then(|manifest| manifest.get("peerDependencies")?.as_object());
        needs
            .into_iter()
            .flatten()
            .filter_map(|(need, range)| Some((need.clone(), range.as_str()?.to_string())))
            .collect()
    }

    /// The asks of each library chosen, once, reported where the first ask that chose it is.
    fn asks_of_libraries(&self, asks: &[Ask], outcomes: &[Outcome]) -> Vec<Ask> {
        let mut found = Vec::new();
        for (named, at) in chosen_once(asks, outcomes) {
            for (need, range) in self.needs(&named) {
                if self.policy.provides(&need) {
                    continue;
                }
                if let Some(ask) = Ask::new(named.clone(), &need, &range, at) {
                    found.push(ask);
                }
            }
        }
        found
    }

    /// What the last round chose: each ask's library, or why it has none; with the runtimes and libraries the host
    /// provides that a chosen library needs, and libraries that name each other, as W121.
    fn chosen(&self, asks: &[Ask], outcomes: &[Outcome]) -> Chosen {
        let mut libraries: HashMap<ReleaseId, BTreeMap<String, Place>> = HashMap::new();
        let mut unmet = Vec::new();
        let mut needed: Vec<NeededPack> = Vec::new();
        for (ask, outcome) in asks.iter().zip(outcomes) {
            let why = match outcome {
                Ok(place) => {
                    libraries
                        .entry(ask.asker.clone())
                        .or_default()
                        .insert(ask.library.to_string(), place.clone());
                    continue;
                }
                Err(Unhad::Absent) => {
                    let pack = NeededPack {
                        pack: ask.library.clone(),
                        range: ask.written.clone(),
                        integrity: None,
                        library: true,
                    };
                    if !needed.contains(&pack) {
                        needed.push(pack);
                    }
                    "which this host couldn't get"
                }
                Err(Unhad::Why(why)) => why,
            };
            unmet.push(Unmet {
                at: ask.at,
                asker: ask.asker.clone(),
                missing: format!("`{}` `{}`, {why}", ask.library, ask.written),
            });
        }
        for (asker, import, cycle) in cut_cycles(&mut libraries) {
            let Some(ask) = asks
                .iter()
                .find(|ask| ask.asker == asker && ask.library == import.as_str())
            else {
                continue;
            };
            let cycle = cycle
                .iter()
                .map(|name| format!("`{name}`"))
                .collect::<Vec<_>>()
                .join(" → ");
            unmet.push(Unmet {
                at: ask.at,
                asker,
                missing: format!(
                    "`{import}` `{}`, but libraries name each other: {cycle}",
                    ask.written
                ),
            });
        }
        for (named, at) in chosen_once(asks, outcomes) {
            for (need, range) in self.needs(&named) {
                let missing = self
                    .policy
                    .provides(&need)
                    .then(|| VersionReq::parse(&range).ok())
                    .flatten()
                    .and_then(|range| self.policy.unmet(&need, &range));
                if let Some(missing) = missing {
                    unmet.push(Unmet {
                        at,
                        asker: named.clone(),
                        missing,
                    });
                }
            }
        }
        Chosen {
            libraries: Libraries(libraries),
            unmet,
            needed,
        }
    }
}

/// Each library `outcomes` chose, once, with where the first ask that chose it is reported.
fn chosen_once(asks: &[Ask], outcomes: &[Outcome]) -> Vec<(ReleaseId, (DocumentId, Location))> {
    let mut seen: Vec<(ReleaseId, (DocumentId, Location))> = Vec::new();
    for (ask, outcome) in asks.iter().zip(outcomes) {
        if let Ok(place) = outcome
            && !seen.iter().any(|(known, _)| *known == place.release)
        {
            seen.push((place.release.clone(), ask.at));
        }
    }
    seen
}

/// Removes each import that closes a cycle of libraries, giving who imports, what, and the cycle's names.
fn cut_cycles(
    libraries: &mut HashMap<ReleaseId, BTreeMap<String, Place>>,
) -> Vec<(ReleaseId, String, Vec<String>)> {
    let mut cut = Vec::new();
    let mut done = HashSet::new();
    let mut starts: Vec<ReleaseId> = libraries.keys().cloned().collect();
    starts.sort();
    for start in starts {
        visit(libraries, &start, &mut Vec::new(), &mut done, &mut cut);
    }
    for (asker, import, _) in &cut {
        if let Some(imports) = libraries.get_mut(asker) {
            imports.remove(import);
        }
    }
    cut
}

fn visit(
    libraries: &HashMap<ReleaseId, BTreeMap<String, Place>>,
    node: &ReleaseId,
    stack: &mut Vec<ReleaseId>,
    done: &mut HashSet<ReleaseId>,
    cut: &mut Vec<(ReleaseId, String, Vec<String>)>,
) {
    if done.contains(node) {
        return;
    }
    stack.push(node.clone());
    for (import, place) in libraries.get(node).into_iter().flatten() {
        let next = place.release.clone();
        if let Some(start) = stack.iter().position(|known| *known == next) {
            let names = stack[start..]
                .iter()
                .map(|release| release.pack.to_string())
                .chain([next.pack.to_string()])
                .collect();
            cut.push((node.clone(), import.clone(), names));
        } else {
            visit(libraries, &next, stack, done, cut);
        }
    }
    stack.pop();
    done.insert(node.clone());
}

#[cfg(test)]
mod tests;
