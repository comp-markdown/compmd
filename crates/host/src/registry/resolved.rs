//! What a resolution gives ([the registry rules], decision 6): the releases for the request, the packs the size cap
//! refuses, the built modules and the files a frame may load, by place, and what couldn't be got.
//!
//! [the registry rules]: https://github.com/comp-markdown/compmd/blob/main/docs/registry-rules.md

use std::collections::BTreeMap;

use compmd_pack::PackName;
use semver::Version;
use serde::{Deserialize, Serialize};

/// A release for the request's packs, as the processor takes one (`PackRelease`).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Release {
    /// Its version.
    pub version: Version,
    /// The integrity its registry publishes, `null` for none.
    pub integrity: Option<String>,
    /// Its text files, by path; none for a release the processor refuses before it reads a pack.
    pub files: BTreeMap<String, String>,
    /// Why its files can't be used: its tarball doesn't match its integrity (F035).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub broken: Option<String>,
}

/// A file a frame may load, an image or a font's file, with its media type.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FrameFile {
    /// `image/svg+xml`, `font/woff2`.
    pub media_type: String,
    /// Its bytes.
    pub bytes: Vec<u8>,
}

/// Something that couldn't be got, for the host's logs; the document is told by the processor's diagnostics.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Problem {
    /// The pack.
    pub pack: PackName,
    /// What happened.
    pub message: String,
}

/// What a resolution gave.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Resolved {
    /// Every release it added, by pack.
    pub releases: BTreeMap<PackName, Vec<Release>>,
    /// The packs the size cap refuses, each with why.
    pub refused: BTreeMap<PackName, String>,
    /// Each built module's checked text, by its place, `@kit/mermaid@1.2.0/dist/mermaid.js`.
    pub modules: BTreeMap<String, String>,
    /// Each image and font's file of the packs it fetched, by its place.
    pub files: BTreeMap<String, FrameFile>,
    /// What couldn't be got.
    pub problems: Vec<Problem>,
}
