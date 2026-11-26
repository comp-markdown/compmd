//! `manifest.json`, the bundle's one description ([hosting], decision 3), which `@compmd/vite` writes and every host
//! reads, in Rust here and in JavaScript.
//!
//! [hosting]: https://github.com/comp-markdown/compmd/blob/main/docs/hosting.md

use std::collections::BTreeMap;

use compmd_pack::{PackName, ReleaseId};
use serde::{Deserialize, Serialize};

use crate::bundled_pack::BundledPack;
use crate::fetched_by::FetchedBy;
use crate::frames::Frames;
use crate::library::Library;
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
    /// The registries packs come from, by scope (`@kit`, `*` for every other), each a URL ending in `/`; none when
    /// the configuration names none.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub registries: BTreeMap<String, String>,
    /// Who fetches from them: with [`FetchedBy::Host`], the host serves `registry/` as their proxy.
    #[serde(default)]
    pub fetch: FetchedBy,
    /// In bytes: the largest pack the host takes from a registry; the registry rules' default when absent.
    #[serde(
        default,
        rename = "maxPackSize",
        skip_serializing_if = "Option::is_none"
    )]
    pub max_pack_size: Option<u64>,
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
