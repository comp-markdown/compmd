//! What a resolution asks its host to do, and what the host answers ([the registry rules], decision 2): the I/O of
//! resolving, and nothing else. Both are plain data, so a host in JavaScript passes them as JSON.
//!
//! [the registry rules]: https://github.com/comp-markdown/compmd/blob/main/docs/registry-rules.md

use std::collections::BTreeMap;

use compmd_pack::{Integrity, PackName};
use semver::Version;
use serde::{Deserialize, Serialize};

use super::metadata::VersionInfo;
use super::resolved::Release;
use super::tar::split;

/// A step, by the number the resolution gave it: the host answers it with the same number.
pub type StepId = u64;

/// A pack an import, or a pack's `peerDependencies`, names that the request has no release of: what the processor's
/// `sources` lists (`NeededPack`).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Needed {
    /// The pack.
    pub pack: PackName,
    /// The version or range, as written; `*` for none.
    pub range: String,
    /// The integrity an import pins with an exact version.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub integrity: Option<Integrity>,
    /// Whether it is a library pack, whose version every range asked for it matches.
    #[serde(default)]
    pub library: bool,
}

/// A release as a host's cache keeps it: its pack, version and tarball's integrity, so one pack's files are never
/// given as another's, whatever integrity an import pins (audit S4).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeptRelease {
    /// The pack.
    pub pack: PackName,
    /// The exact version.
    pub version: Version,
    /// The tarball's integrity, `sha512-…`.
    pub integrity: String,
}

impl KeptRelease {
    /// The release as one key: two are the same only when their pack, version and integrity are.
    pub(crate) fn key(&self) -> String {
        format!("{}@{}#{}", self.pack, self.version, self.integrity)
    }
}

/// A release's checked files: its text, which the processor reads, and the rest as bytes, its images among them.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Files {
    /// The UTF-8 files, by path in the package.
    pub text: BTreeMap<String, String>,
    /// The other files, by path in the package.
    #[serde(default)]
    pub binary: BTreeMap<String, Vec<u8>>,
}

/// A release's files as bytes, split: the UTF-8 ones are its text, the others stay bytes.
impl From<BTreeMap<String, Vec<u8>>> for Files {
    fn from(files: BTreeMap<String, Vec<u8>>) -> Self {
        let (text, binary) = split(files);
        Self { text, binary }
    }
}

impl Files {
    /// Their size in bytes, as a registry's `unpackedSize` counts it: each file's bytes.
    pub(crate) fn size(&self) -> u64 {
        let text: usize = self.text.values().map(String::len).sum();
        let binary: usize = self.binary.values().map(Vec::len).sum();
        u64::try_from(text + binary).unwrap_or(u64::MAX)
    }
}

/// A pack's versions as a host's cache keeps them, and whether the list is fresh enough to use without asking the
/// registry: the host's clock says.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeptVersions {
    /// The versions.
    pub versions: Vec<VersionInfo>,
    /// Whether they may be used as they are.
    pub fresh: bool,
}

/// What the resolution needs the host to do.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum Step {
    /// Add `releases` to the request's packs and `refused` to its policy's, then ask the processor which packs the
    /// request still imports that it has no release of (`sources`): [`Answer::Missing`].
    Missing {
        /// The step's number.
        id: StepId,
        /// The releases resolved since the last time, by pack.
        releases: Vec<(PackName, Release)>,
        /// The packs the host's size cap refuses, each with why, for `policy.refused`.
        refused: BTreeMap<PackName, String>,
    },
    /// Read the cache's list of `pack`'s versions at `registry`: [`Answer::KeptVersions`].
    KeptVersions {
        /// The step's number.
        id: StepId,
        /// The registry's URL.
        registry: String,
        /// The pack.
        pack: PackName,
    },
    /// Ask the registry at `registry` for `pack`'s versions: [`Answer::Versions`], none when it doesn't have the pack,
    /// or [`Answer::Failed`]. A registry that speaks npm's protocol GETs [`super::metadata_url`] and reads the answer
    /// with [`super::answered_versions`]; a host's own registry gives its versions as it has them.
    Versions {
        /// The step's number.
        id: StepId,
        /// The registry's URL.
        registry: String,
        /// The pack.
        pack: PackName,
    },
    /// Keep `pack`'s `versions` at `registry` in the cache, for `max_age` seconds: [`Answer::Done`].
    KeepVersions {
        /// The step's number.
        id: StepId,
        /// The registry's URL.
        registry: String,
        /// The pack.
        pack: PackName,
        /// The versions.
        versions: Vec<VersionInfo>,
        /// How long they may be kept, from the registry's `Cache-Control`.
        max_age: u64,
    },
    /// Read the cache's files of `release`: [`Answer::Kept`].
    Kept {
        /// The step's number.
        id: StepId,
        /// The release.
        release: KeptRelease,
    },
    /// Get `version`'s tarball from the registry at `registry`, its bytes as they come, stopping past `max_bytes`:
    /// [`Answer::Tarball`], or [`Answer::Failed`]. A registry that speaks npm's protocol fetches it from its own origin
    /// only ([`super::off_origin`]); the resolution checks the bytes, whoever gives them.
    Tarball {
        /// The step's number.
        id: StepId,
        /// The registry's URL.
        registry: String,
        /// The version, with where its tarball is.
        version: VersionInfo,
        /// The most bytes it may have.
        max_bytes: u64,
    },
    /// Keep `release`'s checked `files` in the cache: [`Answer::Done`].
    Keep {
        /// The step's number.
        id: StepId,
        /// The release.
        release: KeptRelease,
        /// Its files.
        files: Files,
    },
}

impl Step {
    /// Its number, which the answer goes with.
    #[must_use]
    pub fn id(&self) -> StepId {
        match self {
            Self::Missing { id, .. }
            | Self::KeptVersions { id, .. }
            | Self::Versions { id, .. }
            | Self::KeepVersions { id, .. }
            | Self::Kept { id, .. }
            | Self::Tarball { id, .. }
            | Self::Keep { id, .. } => *id,
        }
    }
}

/// What the host did.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum Answer {
    /// The packs the request still imports that it has no release of.
    Missing {
        /// The packs, as the processor lists them.
        packs: Vec<Needed>,
    },
    /// The cache's list, if it has one.
    KeptVersions {
        /// The list.
        kept: Option<KeptVersions>,
    },
    /// The pack's versions at the registry, none when it doesn't have the pack, and how long the list may be kept: for
    /// npm's protocol, the `max-age` of the answer's `Cache-Control` (0 without one).
    Versions {
        /// The versions.
        versions: Vec<VersionInfo>,
        /// The seconds it may be kept.
        max_age: u64,
    },
    /// The cache's files of a release, if it has them all (its binary files too, when it keeps them).
    Kept {
        /// The files.
        files: Option<Files>,
    },
    /// The tarball's bytes, as they came.
    Tarball {
        /// The bytes.
        bytes: Vec<u8>,
    },
    /// A request that got no answer, a registry that failed, or a tarball larger than allowed, with why.
    Failed {
        /// Why.
        message: String,
    },
    /// A step that gives nothing back was done.
    Done,
}
