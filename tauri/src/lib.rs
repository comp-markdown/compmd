//! `tauri-plugin-compmd`: CompMD's sandbox frames for a Tauri app, served from a scheme of the app's own
//! (docs/tauri-plugin.md).
//!
//! The sandbox (`@compmd/sandbox`) runs a document's components in frames, which need an origin apart from the app's
//! page. The plugin brings the frames, serves them on a scheme of their own, and tells the page where they are. An app
//! adds it in one call:
//!
//! ```
//! let app = tauri::Builder::<tauri::Wry>::default().plugin(tauri_plugin_compmd::init());
//! // then, as every Tauri app: app.run(tauri::generate_context!())
//! # drop(app);
//! ```
//!
//! and its page opens a sandbox with no `base`, which the plugin gave it:
//!
//! ```js
//! const view = createSandbox({ page: "host" }).open(container, { components: processed.hostComponents });
//! ```
//!
//! [`Builder`] customizes it: another scheme's name, or the app's own build of the frames.

mod command;
mod content_type;
mod embedded;
mod files;
mod frames;
mod respond;
mod scheme;
mod served;

use std::path::{Path, PathBuf};

pub use files::Files;
pub use frames::{Build, FRAMES_JSON, Frames};
pub use served::Served;
use tauri::plugin::{Builder as PluginBuilder, TauriPlugin};
use tauri::{Manager, Runtime};

/// The scheme's name when the app gives none, the one the embedded frames are built for.
pub const SCHEME: &str = "compmd";

/// The plugin as it comes: the embedded frames on the scheme `compmd`, their `base` given to the app's pages.
///
/// # Example
///
/// ```
/// let app = tauri::Builder::<tauri::Wry>::default().plugin(tauri_plugin_compmd::init());
/// # drop(app);
/// ```
#[must_use]
pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new().build()
}

/// The plugin, set up by further calls.
///
/// # Example
///
/// ```
/// // Frames the app built itself, with libraries it offers, on a scheme of its own name.
/// let plugin = tauri_plugin_compmd::Builder::new()
///     .scheme("myframes")
///     .frames_dir("sandbox")
///     .build::<tauri::Wry>();
/// # drop(plugin);
/// ```
#[derive(Clone, Debug)]
pub struct Builder {
    scheme: String,
    frames_dir: Option<PathBuf>,
}

impl Default for Builder {
    fn default() -> Self {
        Self::new()
    }
}

impl Builder {
    /// The plugin with the embedded frames on the scheme `compmd`, as [`init`] gives it.
    #[must_use]
    pub fn new() -> Self {
        Self {
            scheme: SCHEME.to_owned(),
            frames_dir: None,
        }
    }

    /// The scheme the frames are served on: a name of the app's choosing, lowercase letters, digits, `+`, `-` and
    /// `.`, starting with a letter. A scheme other than `compmd` needs the app's own build (`frames_dir`), built with
    /// its `source`, since the embedded frames name `compmd://`.
    #[must_use]
    pub fn scheme(mut self, scheme: impl Into<String>) -> Self {
        self.scheme = scheme.into();
        self
    }

    /// The app's own build of the frames (`buildHashedFrames`) in place of the embedded one: the folder holding its
    /// `frames.json`, relative to the app's resources, or absolute. Its page then asks for `base` with the command
    /// `plugin:compmd|base`, which `compmd:default` grants, since its hash is known only when the app starts.
    #[must_use]
    pub fn frames_dir(mut self, folder: impl Into<PathBuf>) -> Self {
        self.frames_dir = Some(folder.into());
        self
    }

    /// The plugin, named `compmd`: its scheme, the `base` command, the frames, and, for the embedded frames, the
    /// script that gives the app's pages their `base`.
    ///
    /// # Panics
    ///
    /// When the scheme's name isn't one, or isn't `compmd` with the embedded frames: mistakes in the app's code.
    #[must_use]
    pub fn build<R: Runtime>(self) -> TauriPlugin<R> {
        let Self { scheme, frames_dir } = self;
        assert!(is_scheme(&scheme), "`{scheme}` isn't a scheme's name");
        assert!(
            frames_dir.is_some() || scheme == SCHEME,
            "the embedded frames are built for the scheme `{SCHEME}`: the scheme `{scheme}` needs the app's own build (`frames_dir`)"
        );
        let embedded = frames_dir.is_none().then(Frames::embedded);
        let mut plugin =
            PluginBuilder::new("compmd").invoke_handler(tauri::generate_handler![command::base]);
        if let Some(base) = embedded.as_ref().and_then(|frames| frames.base(&scheme)) {
            plugin = plugin.js_init_script(init_script(&base));
        }
        let served_scheme = scheme.clone();
        plugin
            .register_uri_scheme_protocol(served_scheme, scheme::serve)
            .setup(move |app, _api| {
                let frames = embedded.clone().unwrap_or_else(|| {
                    let folder = frames_dir
                        .as_deref()
                        .and_then(|dir| folder_of(dir, app.path().resource_dir().ok()));
                    folder.map_or_else(Frames::none, |folder| Frames::read(&folder))
                });
                app.manage(Served {
                    frames,
                    scheme: scheme.clone(),
                });
                Ok(())
            })
            .build()
    }
}

/// The script that gives the app's pages the frames' `base`, as `createSandbox` reads it.
fn init_script(base: &str) -> String {
    let base = serde_json::to_string(base).expect("a string serializes");
    format!("globalThis.__COMPMD__ = Object.freeze({{ base: {base} }});")
}

/// Where the app's own frames are: `frames` itself when it is absolute, else under the app's `resources`; none when
/// those can't be found, so the app starts with no frames, and gives no `base`.
fn folder_of(frames: &Path, resources: Option<PathBuf>) -> Option<PathBuf> {
    if frames.is_absolute() {
        Some(frames.to_path_buf())
    } else {
        resources.map(|resources| resources.join(frames))
    }
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
