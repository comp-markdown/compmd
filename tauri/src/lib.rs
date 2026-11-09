//! `tauri-plugin-compmd`: CompMD's sandbox frames for a Tauri app, served from a scheme of the app's own
//! (docs/tauri-plugin.md).
//!
//! The sandbox (`@compmd/sandbox`) runs a document's components in frames, which need an origin apart from the app's
//! page. The plugin registers a scheme for them, serves the build `buildHashedFrames` wrote into the app's resources,
//! and gives the page the frames' `base`:
//!
//! ```
//! let app = tauri::Builder::<tauri::Wry>::default().plugin(tauri_plugin_compmd::Builder::new().build());
//! // then, as every Tauri app: app.run(tauri::generate_context!())
//! # drop(app);
//! ```
//!
//! The page then asks for `base` (the capability of its window lists `compmd:default`):
//!
//! ```js
//! const base = await invoke("plugin:compmd|base");
//! const sandbox = createSandbox({ base, page: "host" });
//! ```

mod command;
mod content_type;
mod frames;
mod respond;
mod served;

use std::path::{Path, PathBuf};

pub use frames::{Build, FRAMES_JSON, Frames};
pub use served::Served;
use tauri::plugin::{Builder as PluginBuilder, TauriPlugin};
use tauri::{Manager, Runtime};

/// The scheme's name when the app gives none.
pub const SCHEME: &str = "compmd";

/// The resource folder holding `frames.json` and the build, when the app gives none.
pub const FRAMES: &str = "compmd";

/// The plugin, as an app sets it up.
///
/// # Example
///
/// ```no_run
/// let plugin = tauri_plugin_compmd::Builder::new().scheme("myframes").frames("sandbox").build::<tauri::Wry>();
/// ```
#[derive(Clone, Debug)]
pub struct Builder {
    scheme: String,
    frames: PathBuf,
}

impl Default for Builder {
    fn default() -> Self {
        Self::new()
    }
}

impl Builder {
    /// The plugin with the scheme `compmd` and the frames in the resource folder `compmd`.
    #[must_use]
    pub fn new() -> Self {
        Self {
            scheme: SCHEME.to_owned(),
            frames: PathBuf::from(FRAMES),
        }
    }

    /// The scheme the frames are served on: a name of the app's choosing, lowercase letters, digits, `+`, `-` and
    /// `.`, starting with a letter.
    #[must_use]
    pub fn scheme(mut self, scheme: impl Into<String>) -> Self {
        self.scheme = scheme.into();
        self
    }

    /// The folder holding `frames.json` and the build: relative to the app's resources, or absolute.
    #[must_use]
    pub fn frames(mut self, frames: impl Into<PathBuf>) -> Self {
        self.frames = frames.into();
        self
    }

    /// The plugin, named `compmd`: its scheme, the `base` command, and the frames read when the app starts.
    ///
    /// # Panics
    ///
    /// When the scheme's name isn't one (`Builder::scheme`): a mistake in the app's code.
    #[must_use]
    pub fn build<R: Runtime>(self) -> TauriPlugin<R> {
        assert!(
            is_scheme(&self.scheme),
            "`{}` isn't a scheme's name",
            self.scheme
        );
        let Self { scheme, frames } = self;
        let served_scheme = scheme.clone();
        PluginBuilder::new("compmd")
            .invoke_handler(tauri::generate_handler![command::base])
            .register_uri_scheme_protocol(served_scheme, |context, request| {
                answer(context.app_handle(), &request)
            })
            .setup(move |app, _api| {
                let folder = folder_of(&frames, app.path().resource_dir().ok());
                app.manage(Served {
                    frames: folder.map_or_else(Frames::none, |folder| Frames::read(&folder)),
                    scheme: scheme.clone(),
                });
                Ok(())
            })
            .build()
    }
}

/// Where the frames are: `frames` itself when it is absolute, else under the app's `resources`; none when those can't
/// be found, so the app starts with no frames, and gives no `base`.
fn folder_of(frames: &Path, resources: Option<PathBuf>) -> Option<PathBuf> {
    if frames.is_absolute() {
        Some(frames.to_path_buf())
    } else {
        resources.map(|resources| resources.join(frames))
    }
}

/// What the scheme answers `request` with: the frames the app serves (`Frames::respond`).
fn answer<R: Runtime>(
    app: &tauri::AppHandle<R>,
    request: &tauri::http::Request<Vec<u8>>,
) -> tauri::http::Response<Vec<u8>> {
    app.compmd().frames.respond(request)
}

/// Whether `name` is a scheme's name (RFC 3986): a letter, then letters, digits, `+`, `-` or `.`; lowercase, as
/// webviews compare them.
fn is_scheme(name: &str) -> bool {
    let mut bytes = name.bytes();
    bytes.next().is_some_and(|first| first.is_ascii_lowercase())
        && bytes.all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'+' | b'-' | b'.')
        })
}

/// What the plugin gives Rust code of the app.
pub trait CompmdExt<R: Runtime> {
    /// The frames the app serves, and their scheme.
    fn compmd(&self) -> &Served;
}

impl<R: Runtime, T: Manager<R>> CompmdExt<R> for T {
    fn compmd(&self) -> &Served {
        self.state::<Served>().inner()
    }
}

#[cfg(test)]
mod tests;
