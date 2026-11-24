//! A library the frames offer component code.

use semver::Version;
use serde::{Deserialize, Serialize};

/// A library the frames offer: an npm package the app installs, which needn't be scoped (`d3`).
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct Library {
    /// Its npm name.
    pub name: String,
    /// The version the bundle holds.
    pub version: Version,
}
