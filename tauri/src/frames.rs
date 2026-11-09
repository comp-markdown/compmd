//! The build of the frames the app bundles (docs/tauri-plugin.md, decision 2): the folder `frames.json` names, read
//! once when the app starts.

use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

/// The file beside the builds that names the current one, as `buildHashedFrames` writes it.
pub const FRAMES_JSON: &str = "frames.json";

/// The prefix of every path the frames are served under.
pub const PREFIX: &str = "/frames/";

/// What `frames.json` holds.
#[derive(Deserialize)]
struct Named {
    hash: String,
}

/// The build the app serves: its folder and its hash, or none when the app was built without its frames.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Frames(Option<Build>);

/// A build of the frames: the folder named by its hash.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Build {
    /// The folder holding the frame documents and their files.
    pub folder: PathBuf,
    /// The hash naming it, 16 lowercase hex digits.
    pub hash: String,
}

impl Frames {
    /// The build `frames.json` in `folder` names; none when it is missing, can't be read, or names something that
    /// isn't a build's hash.
    #[must_use]
    pub fn read(folder: &Path) -> Self {
        let named = fs::read_to_string(folder.join(FRAMES_JSON))
            .ok()
            .and_then(|text| serde_json::from_str::<Named>(&text).ok());
        Self(
            named
                .filter(|named| is_hash(&named.hash))
                .map(|named| Build {
                    folder: folder.join(&named.hash),
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
