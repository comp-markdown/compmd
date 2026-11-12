//! `manifest.json`, the bundle's one description ([hosting], decision 3), which `@compmd/vite` writes and every host
//! reads, in Rust here and in JavaScript.
//!
//! [hosting]: https://github.com/comp-markdown/compmd/blob/main/docs/hosting.md

use std::collections::BTreeMap;

use compmd_pack::{FilePath, Integrity, PackName, ReleaseId};
use semver::Version;
use serde::{Deserialize, Serialize};

use crate::frames_hash::FramesHash;
use crate::manifest_error::ManifestError;

/// The manifest's format this crate reads; a bundle of another is refused, with what to upgrade.
pub const FORMAT: u32 = 1;

/// A bundle's manifest.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Manifest {
    /// The format it is written in, [`FORMAT`].
    pub format: u32,
    /// The frames' build.
    pub frames: Frames,
    /// Where the frames' policies say they are served, when that is a custom scheme (`compmd://localhost/`), whose
    /// `'self'` `WebKit` reads as nothing in a sandboxed frame; none for http and https origins.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    /// The packs the bundle holds, under `packs/<name>@<version>/`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub packs: Vec<BundledPack>,
    /// The libraries the frames offer component code, besides the runtimes.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub libraries: Vec<Library>,
    /// The processor's policy the configuration made (`policyOf`), as JSON: what a host that processes in Rust gives
    /// the processor.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub policy: Option<serde_json::Value>,
    /// The packs every document gets without importing them.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ambient: Vec<PackName>,
}

/// The frames' build: its folder, `frames/<hash>/`.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct Frames {
    /// The build's hash.
    pub hash: FramesHash,
}

/// A pack the bundle holds. In JSON its release is flat: `{"pack", "version", "integrity", "trust", "modules"}`.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct BundledPack {
    /// The release.
    #[serde(flatten)]
    pub release: ReleaseId,
    /// The integrity the registry publishes for it (`sha512-…`), from the app's lock file; none when the lock file
    /// gave none.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub integrity: Option<Integrity>,
    /// The trust the configuration gives it.
    #[serde(default)]
    pub trust: Trust,
    /// Its built modules, by their path in the pack, with their integrity (`sha384-…`), which a frame checks.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub modules: BTreeMap<FilePath, Integrity>,
}

/// The trust a configuration gives a pack (spec §15).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Trust {
    /// In a strict sandbox: the default.
    #[default]
    Strict,
    /// In a permissive sandbox, with the network to its listed origins.
    Permissive,
    /// In the host's page: a folder pack built with the app only.
    Trusted,
}

/// A library the frames offer: an npm package the app installs, which needn't be scoped (`d3`).
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct Library {
    /// Its npm name.
    pub name: String,
    /// The version the bundle holds.
    pub version: Version,
}

impl Manifest {
    /// The manifest in `text`.
    ///
    /// # Errors
    ///
    /// [`ManifestError`] when it isn't a manifest, is of another format, or names a frames' hash, a pack, a path or
    /// an integrity that isn't one.
    pub fn parse(text: &str) -> Result<Self, ManifestError> {
        #[derive(Deserialize)]
        struct Format {
            format: u32,
        }
        let json = |error: serde_json::Error| ManifestError::Json(error.to_string());
        let format: Format = serde_json::from_str(text).map_err(json)?;
        if format.format != FORMAT {
            return Err(ManifestError::UnknownFormat {
                found: format.format,
                known: FORMAT,
            });
        }
        serde_json::from_str(text).map_err(json)
    }

    /// The custom scheme the bundle was built for, from its `source` (`compmd://localhost/` is `compmd`); none when
    /// it was built for an http or https origin.
    #[must_use]
    pub fn scheme(&self) -> Option<&str> {
        let scheme = self.source.as_deref()?.split_once("://")?.0;
        (scheme != "http" && scheme != "https").then_some(scheme)
    }

    /// The pack at `release`, when the bundle holds it.
    #[must_use]
    pub fn pack(&self, release: &ReleaseId) -> Option<&BundledPack> {
        self.packs.iter().find(|pack| pack.release == *release)
    }
}

#[cfg(test)]
mod tests;
