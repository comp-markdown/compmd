//! Resolving a request's packs from registries ([the registry rules]; [resolving registry packs], decisions 1 to 6
//! and 8): ask what is missing, resolve each import once (a library pack's to the version every range asked for it
//! matches), apply the policy before downloading, check each tarball, add the releases, and ask again. A state
//! machine: it says what it needs ([`Step`]), and the host does it and answers ([`Answer`]).
//!
//! [the registry rules]: https://github.com/comp-markdown/compmd/blob/main/docs/registry-rules.md
//! [resolving registry packs]: https://github.com/comp-markdown/compmd/blob/main/docs/resolving-registry-packs.md

use std::collections::{BTreeMap, HashSet};
use std::time::Duration;

use compmd_pack::{PackName, newest_matching};
use semver::Version;

use super::file_types::file_type_of;
use super::integrity::{integrity_of, is_checked};
use super::metadata::VersionInfo;
use super::options::Options;
use super::resolved::{FrameFile, Problem, Release, Resolved};
use super::step::{Answer, Files, KeptRelease, KeptVersions, Needed, Step, StepId};
use super::tar::{gunzip, split, untar};

/// A megabyte, as the size cap's messages count them.
const MB: f64 = 1_000_000.0;

/// What [`Resolution::next_steps`] gives: steps for the host, or what the resolution gave.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Next {
    /// Do each, and give each its answer, before asking again; they don't depend on each other.
    Steps(Vec<Step>),
    /// Nothing is missing, or nothing more can be got.
    Done(Resolved),
}

/// What resolving one import gave.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Outcome {
    /// A release, for the request's packs.
    Release(Release),
    /// The size cap refuses it, for `policy.refused`.
    Refused(String),
    /// It couldn't be got.
    Problem(String),
    /// No release matches: the processor's F026.
    Nothing,
}

/// An import being resolved.
#[derive(Clone, Debug)]
struct Need {
    needed: Needed,
    outcome: Option<Outcome>,
}

/// What a step's answer goes to.
#[derive(Clone, Debug)]
enum Waiting {
    /// The processor's list of what is missing.
    Missing,
    /// A pinned import's files, kept.
    PinnedFiles(usize),
    /// An import's versions, kept.
    KeptVersions(usize),
    /// An import's versions from its registry, with the stale list kept, if any.
    FetchedVersions(usize, Option<Vec<VersionInfo>>),
    /// A release's files, kept, by its key.
    ReleaseFiles(String),
    /// A release's tarball, by its key.
    Tarball(String),
    /// A step that gives nothing back.
    Nothing,
}

/// A release being got, for every import that resolves to it at once.
#[derive(Clone, Debug)]
struct Fetch {
    release: KeptRelease,
    info: VersionInfo,
    /// The imports waiting for it.
    waiters: Vec<usize>,
}

/// One resolution of a request's packs.
#[derive(Debug)]
pub struct Resolution {
    options: Options,
    next_id: StepId,
    started: bool,
    finished: bool,
    queued: Vec<Step>,
    waiting: BTreeMap<StepId, Waiting>,
    /// Every import asked, by its key, so each is resolved once.
    asked: HashSet<String>,
    /// Every import, across rounds.
    needs: Vec<Need>,
    /// The imports of the round being resolved.
    round: Vec<usize>,
    /// Every range asked for each library pack, of which the processor takes the version all match.
    library_ranges: BTreeMap<PackName, Vec<String>>,
    /// The releases being got, by key.
    fetching: BTreeMap<String, Fetch>,
    resolved: Resolved,
}

/// One import's key: its pack, range and pinned integrity.
fn key_of(needed: &Needed) -> String {
    let integrity = needed
        .integrity
        .as_ref()
        .map(ToString::to_string)
        .unwrap_or_default();
    format!("{}@{}#{integrity}", needed.pack, needed.range)
}

/// `info` as a release with no files, for a version the processor refuses before it reads a pack.
fn fileless(info: &VersionInfo) -> Release {
    Release {
        version: info.version.clone(),
        integrity: info.integrity.clone(),
        files: BTreeMap::new(),
        broken: None,
    }
}

impl Resolution {
    /// A resolution with the host's `options`.
    #[must_use]
    pub fn new(options: Options) -> Self {
        Self {
            options,
            next_id: 0,
            started: false,
            finished: false,
            queued: Vec::new(),
            waiting: BTreeMap::new(),
            asked: HashSet::new(),
            needs: Vec::new(),
            round: Vec::new(),
            library_ranges: BTreeMap::new(),
            fetching: BTreeMap::new(),
            resolved: Resolved::default(),
        }
    }

    /// What the host is to do next: the steps ready, or what the resolution gave once nothing is missing.
    pub fn next_steps(&mut self) -> Next {
        if !self.started {
            self.started = true;
            self.ask_missing(Vec::new(), BTreeMap::new());
        }
        if self.finished {
            return Next::Done(std::mem::take(&mut self.resolved));
        }
        Next::Steps(std::mem::take(&mut self.queued))
    }

    /// The host's answer to the step `id`. An answer to no step waiting, or of the wrong kind, changes nothing.
    pub fn give(&mut self, id: StepId, answer: Answer) {
        let Some(waiting) = self.waiting.remove(&id) else {
            return;
        };
        match (waiting, answer) {
            (Waiting::Missing, Answer::Missing { packs }) => self.start_round(packs),
            (Waiting::PinnedFiles(need), Answer::Kept { files }) => self.pinned_kept(need, files),
            (Waiting::KeptVersions(need), Answer::KeptVersions { kept }) => {
                self.versions_kept(need, kept);
            }
            (Waiting::FetchedVersions(need, stale), answer) => {
                self.versions_fetched(need, stale, answer);
            }
            (Waiting::ReleaseFiles(key), Answer::Kept { files }) => self.release_kept(&key, files),
            (Waiting::Tarball(key), answer) => self.tarball_fetched(&key, answer),
            _ => {}
        }
    }

    /// Gives up on `pack`, which hasn't come `after` the host's timeout: each of its imports not resolved yet is a
    /// problem, and stays missing, so a host that shows it as pending meanwhile (spec §6.4) stops, and the processor
    /// reports F026.
    pub fn give_up(&mut self, pack: &PackName, after: Duration) {
        let seconds = after.as_secs_f64();
        let late: Vec<usize> = self
            .round
            .iter()
            .copied()
            .filter(|&need| {
                self.needs[need].needed.pack == *pack && self.needs[need].outcome.is_none()
            })
            .collect();
        for need in late {
            self.finish(
                need,
                Outcome::Problem(format!("`{pack}` didn't come within {seconds} s")),
            );
        }
    }

    /// Queues `step` made from its number, waiting for `waiting`.
    fn queue(&mut self, waiting: Waiting, step: impl FnOnce(StepId) -> Step) {
        let id = self.next_id;
        self.next_id += 1;
        self.waiting.insert(id, waiting);
        self.queued.push(step(id));
    }

    /// Asks the host for what is missing, once it added `releases` and `refused`.
    fn ask_missing(
        &mut self,
        releases: Vec<(PackName, Release)>,
        refused: BTreeMap<PackName, String>,
    ) {
        self.queue(Waiting::Missing, |id| Step::Missing {
            id,
            releases,
            refused,
        });
    }

    /// Starts resolving each of `packs` not asked before; with none, the resolution is done.
    fn start_round(&mut self, packs: Vec<Needed>) {
        let new: Vec<Needed> = packs
            .into_iter()
            .filter(|needed| self.asked.insert(key_of(needed)))
            .collect();
        if new.is_empty() {
            self.finished = true;
            return;
        }
        for needed in new.iter().filter(|needed| needed.library) {
            let ranges = self.library_ranges.entry(needed.pack.clone()).or_default();
            if !ranges.contains(&needed.range) {
                ranges.push(needed.range.clone());
            }
        }
        self.round.clear();
        for needed in new {
            self.needs.push(Need {
                needed,
                outcome: None,
            });
            self.round.push(self.needs.len() - 1);
        }
        for need in self.round.clone() {
            self.start(need);
        }
    }

    /// Starts resolving one import: a pinned one from the cache if it keeps it, else from its registry.
    fn start(&mut self, need: usize) {
        let needed = self.needs[need].needed.clone();
        if self.options.registry_of(&needed.pack).is_none() {
            let message = format!("no registry is configured for `{}`'s scope", needed.pack);
            return self.finish(need, Outcome::Problem(message));
        }
        // A pinned import names an exact version, and its files never change: kept for this pack, they need no
        // registry, and the host's policy holds for them as for a download.
        if let (Some(pinned), Ok(version)) = (&needed.integrity, Version::parse(&needed.range)) {
            let release = KeptRelease {
                pack: needed.pack,
                version,
                integrity: pinned.to_string(),
            };
            return self.queue(Waiting::PinnedFiles(need), |id| Step::Kept { id, release });
        }
        self.ask_versions(need);
    }

    /// Asks the cache for an import's pack's versions.
    fn ask_versions(&mut self, need: usize) {
        let pack = self.needs[need].needed.pack.clone();
        let registry = self
            .options
            .registry_of(&pack)
            .unwrap_or_default()
            .to_owned();
        self.queue(Waiting::KeptVersions(need), |id| Step::KeptVersions {
            id,
            registry,
            pack,
        });
    }

    fn pinned_kept(&mut self, need: usize, files: Option<Files>) {
        if self.needs[need].outcome.is_some() {
            return;
        }
        let Some(files) = files else {
            return self.ask_versions(need);
        };
        let needed = &self.needs[need].needed;
        let info = VersionInfo {
            version: Version::parse(&needed.range).unwrap_or_else(|_| Version::new(0, 0, 0)),
            tarball: String::new(),
            integrity: needed.integrity.as_ref().map(ToString::to_string),
            unpacked_size: Some(files.size()),
        };
        let pack = needed.pack.clone();
        let outcome = self
            .withheld(&pack, &info)
            .unwrap_or_else(|| self.release(&pack, &info, &files));
        self.finish(need, outcome);
    }

    fn versions_kept(&mut self, need: usize, kept: Option<KeptVersions>) {
        if self.needs[need].outcome.is_some() {
            return;
        }
        match kept {
            Some(kept) if kept.fresh => self.choose(need, &kept.versions),
            stale => {
                let pack = self.needs[need].needed.pack.clone();
                let registry = self
                    .options
                    .registry_of(&pack)
                    .unwrap_or_default()
                    .to_owned();
                let stale = stale.map(|kept| kept.versions);
                self.queue(Waiting::FetchedVersions(need, stale), |id| Step::Versions {
                    id,
                    registry,
                    pack,
                });
            }
        }
    }

    fn versions_fetched(&mut self, need: usize, stale: Option<Vec<VersionInfo>>, answer: Answer) {
        if self.needs[need].outcome.is_some() {
            return;
        }
        let pack = self.needs[need].needed.pack.clone();
        let registry = self
            .options
            .registry_of(&pack)
            .unwrap_or_default()
            .to_owned();
        let fetched = match answer {
            Answer::Versions { versions, max_age } => Ok((versions, max_age)),
            Answer::Failed { message } => Err(message),
            _ => return,
        };
        match (fetched, stale) {
            (Ok((versions, max_age)), _) => {
                let kept = versions.clone();
                self.queue(Waiting::Nothing, |id| Step::KeepVersions {
                    id,
                    registry,
                    pack,
                    versions: kept,
                    max_age,
                });
                self.choose(need, &versions);
            }
            (Err(_), Some(stale)) => self.choose(need, &stale),
            (Err(message), None) => self.finish(need, Outcome::Problem(message)),
        }
    }

    /// The newest of `versions` that every range asked for the library `pack` matches.
    fn every_range<'v>(&self, pack: &PackName, versions: &'v [Version]) -> Option<&'v Version> {
        let ranges = self.library_ranges.get(pack)?;
        versions
            .iter()
            .filter(|version| {
                ranges
                    .iter()
                    .all(|range| newest_matching(range, std::slice::from_ref(version)).is_some())
            })
            .max()
    }

    /// Chooses an import's release among `versions`, applies the policy, and gets its files.
    fn choose(&mut self, need: usize, versions: &[VersionInfo]) {
        let Need { needed, .. } = self.needs[need].clone();
        let known: Vec<Version> = versions.iter().map(|info| info.version.clone()).collect();
        let chosen = needed
            .library
            .then(|| self.every_range(&needed.pack, &known))
            .flatten()
            .or_else(|| newest_matching(&needed.range, &known))
            .cloned();
        // None matching stays F026 (no release matches), in the processor.
        let Some(info) =
            chosen.and_then(|version| versions.iter().find(|info| info.version == version))
        else {
            return self.finish(need, Outcome::Nothing);
        };
        // A pinned integrity the registry's doesn't equal is the processor's F035: it checks it before it reads a
        // pack, so the release needs no files.
        if needed
            .integrity
            .as_ref()
            .is_some_and(|pinned| info.integrity.as_deref() != Some(&pinned.to_string()))
        {
            return self.finish(need, Outcome::Release(fileless(info)));
        }
        if let Some(withheld) = self.withheld(&needed.pack, info) {
            return self.finish(need, withheld);
        }
        self.files_of(need, &needed.pack, info.clone());
    }

    /// What the host's policy gives instead of `info`'s files, if anything: a denied version is the processor's F026,
    /// which it checks before it reads a pack, so the release needs no files; a refused one is for `policy.refused`.
    fn withheld(&self, pack: &PackName, info: &VersionInfo) -> Option<Outcome> {
        let denied = self.options.deny.iter().any(|deny| {
            deny.pack == *pack
                && deny.version.as_deref().is_none_or(|range| {
                    newest_matching(range, std::slice::from_ref(&info.version)).is_some()
                })
        });
        if denied {
            return Some(Outcome::Release(fileless(info)));
        }
        self.refusal(pack, info).map(Outcome::Refused)
    }

    /// Why the host's size cap refuses `info` before it is downloaded, or its having no hash to check it by.
    fn refusal(&self, pack: &PackName, info: &VersionInfo) -> Option<String> {
        let version = &info.version;
        #[allow(
            clippy::cast_precision_loss,
            reason = "sizes in messages, to a tenth of a megabyte"
        )]
        let (cap, max) = (
            self.options.max_pack_size as f64 / MB,
            self.options.max_pack_size,
        );
        let Some(size) = info.unpacked_size else {
            return Some(format!(
                "its registry doesn't say how large `{pack}@{version}` is, and this host takes packs up to {cap} MB"
            ));
        };
        if size > max {
            #[allow(
                clippy::cast_precision_loss,
                reason = "sizes in messages, to a tenth of a megabyte"
            )]
            let unpacked = size as f64 / MB;
            return Some(format!(
                "`{pack}@{version}` is {unpacked:.1} MB unpacked, more than this host's {cap} MB"
            ));
        }
        (!is_checked(info.integrity.as_deref()))
            .then(|| format!("its registry publishes no SHA-512 integrity for `{pack}@{version}`, so it can't be checked"))
    }

    /// Gets the checked files of `info`, once in a resolution however many imports resolve to it at once.
    fn files_of(&mut self, need: usize, pack: &PackName, info: VersionInfo) {
        let release = KeptRelease {
            pack: pack.clone(),
            version: info.version.clone(),
            integrity: info.integrity.clone().unwrap_or_default(),
        };
        let key = release.key();
        if let Some(fetch) = self.fetching.get_mut(&key) {
            fetch.waiters.push(need);
            return;
        }
        self.fetching.insert(
            key.clone(),
            Fetch {
                release: release.clone(),
                info,
                waiters: vec![need],
            },
        );
        self.queue(Waiting::ReleaseFiles(key), |id| Step::Kept { id, release });
    }

    fn release_kept(&mut self, key: &str, files: Option<Files>) {
        let Some(fetch) = self.fetching.get(key).cloned() else {
            return;
        };
        if let Some(files) = files {
            return self.fetched(key, Ok(Some(files)));
        }
        let registry = self
            .options
            .registry_of(&fetch.release.pack)
            .unwrap_or_default()
            .to_owned();
        let (version, max_bytes) = (fetch.info.clone(), self.options.max_pack_size);
        self.queue(Waiting::Tarball(key.to_owned()), |id| Step::Tarball {
            id,
            registry,
            version,
            max_bytes,
        });
    }

    fn tarball_fetched(&mut self, key: &str, answer: Answer) {
        let Some(fetch) = self.fetching.get(key).cloned() else {
            return;
        };
        let checked = match answer {
            Answer::Tarball { bytes } => self.intake(&fetch, &bytes),
            Answer::Failed { message } => Err(message),
            _ => return,
        };
        if let Ok(Some(files)) = &checked {
            let (release, files) = (fetch.release.clone(), files.clone());
            self.queue(Waiting::Nothing, |id| Step::Keep { id, release, files });
        }
        self.fetched(key, checked);
    }

    /// `bytes`, `fetch`'s tarball, taken in (audit A9): refused past the size cap, hashed, gunzipped up to the cap,
    /// and read as a tar whose files may take no more than the unpacked size the registry declares. `None` when it
    /// doesn't match its integrity.
    fn intake(&self, fetch: &Fetch, bytes: &[u8]) -> Result<Option<Files>, String> {
        let max = self.options.max_pack_size;
        let cap = usize::try_from(max).unwrap_or(usize::MAX);
        if bytes.len() > cap {
            return Err(format!("{} is larger than {max} bytes", fetch.info.tarball));
        }
        if integrity_of(bytes) != fetch.release.integrity {
            return Ok(None);
        }
        let declared = fetch.info.unpacked_size.map_or(max, |size| size.min(max));
        let files = untar(
            &gunzip(bytes, cap)?,
            usize::try_from(declared).unwrap_or(cap),
        )?;
        let (text, binary) = split(files);
        Ok(Some(Files { text, binary }))
    }

    /// A release got: each import waiting for it resolves to it, or to a broken release, or to the problem.
    fn fetched(&mut self, key: &str, checked: Result<Option<Files>, String>) {
        let Some(fetch) = self.fetching.remove(key) else {
            return;
        };
        let outcome = match checked {
            Ok(Some(files)) => self.release(&fetch.release.pack, &fetch.info, &files),
            Ok(None) => Outcome::Release(Release {
                broken: Some(
                    "its tarball doesn't match the integrity its registry publishes".to_owned(),
                ),
                ..fileless(&fetch.info)
            }),
            Err(message) => Outcome::Problem(message),
        };
        for need in fetch.waiters {
            self.finish(need, outcome.clone());
        }
    }

    /// The release of `info` with its checked files, keeping its built modules, its images and its fonts' files.
    fn release(&mut self, pack: &PackName, info: &VersionInfo, files: &Files) -> Outcome {
        let place_of = |path: &str| format!("{pack}@{}/{path}", info.version);
        #[allow(
            clippy::case_sensitive_file_extension_comparisons,
            reason = "a built module is a `.js` file, as the processor reads one (spec §6.4)"
        )]
        let modules = files.text.iter().filter(|(path, _)| path.ends_with(".js"));
        for (path, code) in modules {
            self.resolved.modules.insert(place_of(path), code.clone());
        }
        let texts = files
            .text
            .iter()
            .map(|(path, text)| (path, text.as_bytes()));
        let binaries = files
            .binary
            .iter()
            .map(|(path, bytes)| (path, bytes.as_slice()));
        for (path, bytes) in texts.chain(binaries) {
            if let Some(media_type) = file_type_of(path) {
                let file = FrameFile {
                    media_type: media_type.to_owned(),
                    bytes: bytes.to_vec(),
                };
                self.resolved.files.insert(place_of(path), file);
            }
        }
        Outcome::Release(Release {
            version: info.version.clone(),
            integrity: info.integrity.clone(),
            files: files.text.clone(),
            broken: None,
        })
    }

    /// One import resolved; once the round's all are, its releases and refusals go to the host, which asks again.
    fn finish(&mut self, need: usize, outcome: Outcome) {
        if self.needs[need].outcome.is_some() {
            return;
        }
        self.needs[need].outcome = Some(outcome);
        if self
            .round
            .iter()
            .any(|&need| self.needs[need].outcome.is_none())
        {
            return;
        }
        let mut releases: Vec<(PackName, Release)> = Vec::new();
        let mut refused = BTreeMap::new();
        for &need in &self.round {
            let Need { needed, outcome } = &self.needs[need];
            let pack = needed.pack.clone();
            match outcome {
                Some(Outcome::Release(release)) => {
                    let known = self.resolved.releases.entry(pack.clone()).or_default();
                    if !known.iter().any(|kept| kept.version == release.version) {
                        known.push(release.clone());
                        releases.push((pack, release.clone()));
                    }
                }
                Some(Outcome::Refused(why)) => {
                    self.resolved.refused.insert(pack.clone(), why.clone());
                    refused.insert(pack, why.clone());
                }
                Some(Outcome::Problem(message)) => self.resolved.problems.push(Problem {
                    pack,
                    message: message.clone(),
                }),
                Some(Outcome::Nothing) | None => {}
            }
        }
        self.round.clear();
        self.ask_missing(releases, refused);
    }
}

#[cfg(test)]
mod tests;
