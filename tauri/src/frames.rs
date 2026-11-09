//! The build of the frames the app serves (docs/tauri-plugin.md, decisions 2 and 3): the one the crate embeds, or the
//! app's own, the folder its `frames.json` names, read once when the app starts.

use std::fs;
use std::path::Path;

use serde::Deserialize;

use crate::embedded;
use crate::files::Files;

/// The file beside the builds that names the current one, as `buildHashedFrames` writes it.
pub const FRAMES_JSON: &str = "frames.json";

/// The prefix of every path the frames are served under.
pub const PREFIX: &str = "/frames/";

/// What `frames.json` holds.
#[derive(Deserialize)]
struct Named {
    hash: String,
}

/// The build the app serves, or none when there is none.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Frames(Option<Build>);

/// A build of the frames: its hash, which its paths name, and its files.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Build {
    /// The hash naming it, 16 lowercase hex digits.
    pub hash: String,
    /// Its frame documents and their files.
    pub files: Files,
}

impl Frames {
    /// The build the crate embeds; none when it was built without one (`cargo make tauri-frames` writes it).
    #[must_use]
    pub fn embedded() -> Self {
        Self::embedded_as(embedded::HASH, embedded::FILES)
    }

    /// The build embedded as `hash` and `files`, if there is one.
    fn embedded_as(hash: Option<&str>, files: &'static [(&'static str, &'static [u8])]) -> Self {
        Self(hash.map(|hash| Build {
            hash: hash.to_owned(),
            files: Files::Embedded(files),
        }))
    }

    /// The build `frames.json` in `folder` names, an app's own; none when it is missing, can't be read, or names
    /// something that isn't a build's hash.
    #[must_use]
    pub fn read(folder: &Path) -> Self {
        let named = fs::read_to_string(folder.join(FRAMES_JSON))
            .ok()
            .and_then(|text| serde_json::from_str::<Named>(&text).ok());
        Self(
            named
                .filter(|named| is_hash(&named.hash))
                .map(|named| Build {
                    files: Files::Folder(folder.join(&named.hash)),
                    hash: named.hash,
                }),
        )
    }

    /// No build: the app serves nothing and gives no `base`.
    #[must_use]
    pub const fn none() -> Self {
        Self(None)
    }

    /// The build, if there is one.
    #[must_use]
    pub const fn build(&self) -> Option<&Build> {
        self.0.as_ref()
    }

    /// The frames' `base` on `scheme`, for `@compmd/sandbox`: `<scheme>://localhost/frames/<hash>/`, or the
    /// `http://<scheme>.localhost/frames/<hash>/` form where Tauri serves custom schemes over http (Windows and
    /// Android); none without a build.
    #[must_use]
    pub fn base(&self, scheme: &str) -> Option<String> {
        self.base_over(scheme, OVER_HTTP)
    }

    /// `base`, where custom schemes are served over http (`over_http`) or not.
    fn base_over(&self, scheme: &str, over_http: bool) -> Option<String> {
        let build = self.build()?;
        let origin = if over_http {
            format!("http://{scheme}.localhost")
        } else {
            format!("{scheme}://localhost")
        };
        Some(format!("{origin}{PREFIX}{}/", build.hash))
    }
}

/// Whether Tauri serves custom schemes over http on this platform: on Windows and Android, as
/// `http://<scheme>.localhost`.
const OVER_HTTP: bool = cfg!(any(windows, target_os = "android"));

/// Whether `name` is a build's hash: 16 lowercase hex digits, so it can't name another folder.
fn is_hash(name: &str) -> bool {
    name.len() == 16
        && name
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

#[cfg(test)]
pub(crate) mod tests;
