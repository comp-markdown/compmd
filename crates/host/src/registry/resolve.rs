//! A resolution driven to its end on one thread ([the language server], decision 6): the cache's steps answered by a
//! [`FolderCache`], the network's by a [`Client`], and what is missing by the host.
//!
//! [the language server]: https://github.com/comp-markdown/compmd/blob/main/docs/language-server.md

use std::collections::BTreeMap;

use compmd_pack::PackName;

use super::client::Client;
use super::folder_cache::FolderCache;
use super::options::Options;
use super::resolution::{Next, Resolution};
use super::resolved::{Problem, Release, Resolved};
use super::step::{Answer, Needed, Step};

/// What the network's steps answer when the host is off it.
const OFFLINE: &str = "the network is off";

/// Resolves the packs `missing` names, with the host's `options`, from `cache` and, unless it is `None`, the
/// network through `client`. `missing` is the host's [`Step::Missing`]: given the releases and refusals resolved since
/// it was last asked, it says which packs the request still has no release of.
///
/// A cache that can't keep what was fetched is a problem in what it gives, not a failure: the release is used all
/// the same.
pub fn resolve(
    options: Options,
    client: Option<&Client>,
    cache: &FolderCache,
    mut missing: impl FnMut(Vec<(PackName, Release)>, BTreeMap<PackName, String>) -> Vec<Needed>,
) -> Resolved {
    let mut resolution = Resolution::new(options);
    let mut unkept = Vec::new();
    let mut resolved = loop {
        let steps = match resolution.next_steps() {
            Next::Steps(steps) => steps,
            Next::Done(resolved) => break resolved,
        };
        for step in steps {
            let id = step.id();
            let answer = match step {
                Step::Missing {
                    releases, refused, ..
                } => Answer::Missing {
                    packs: missing(releases, refused),
                },
                Step::KeptVersions { registry, pack, .. } => Answer::KeptVersions {
                    kept: cache.kept_versions(&registry, &pack),
                },
                Step::Versions { registry, pack, .. } => {
                    client.map_or_else(offline, |client| client.versions(&registry, &pack))
                }
                Step::KeepVersions {
                    registry,
                    pack,
                    versions,
                    max_age,
                    ..
                } => {
                    if let Err(error) = cache.keep_versions(&registry, &pack, versions, max_age) {
                        unkept.push(not_kept(pack, &error));
                    }
                    Answer::Done
                }
                Step::Kept { release, .. } => Answer::Kept {
                    files: cache.kept(&release),
                },
                Step::Tarball {
                    registry,
                    version,
                    max_bytes,
                    ..
                } => client.map_or_else(offline, |client| {
                    client.tarball(&registry, &version, max_bytes)
                }),
                Step::Keep { release, files, .. } => {
                    if let Err(error) = cache.keep(&release, files) {
                        unkept.push(not_kept(release.pack, &error));
                    }
                    Answer::Done
                }
            };
            resolution.give(id, answer);
        }
    };
    resolved.problems.extend(unkept);
    resolved
}

/// A network step's answer off the network.
fn offline() -> Answer {
    Answer::Failed {
        message: OFFLINE.to_owned(),
    }
}

/// What the cache couldn't keep of `pack`.
fn not_kept(pack: PackName, error: &std::io::Error) -> Problem {
    Problem {
        pack,
        message: format!("the cache couldn't keep it: {error}"),
    }
}

#[cfg(test)]
mod tests;
