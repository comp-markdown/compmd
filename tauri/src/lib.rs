//! `tauri-plugin-compmd`: the Tauri adapter of CompMD's hosting (docs/tauri-plugin.md, docs/hosting.md).
//!
//! What every host does is shared: the configuration, the bundle `@compmd/vite` builds from the page's own
//! `@compmd/sandbox`, the pack store and the serving contract (`compmd-host`). The plugin does the four things only
//! Tauri can: it embeds the bundle in the app, serves it on the scheme the bundle was built for, gives the page the
//! origin, and gives the store a folder.
//!
//! The bundle is written by `@compmd/vite` (`compmd({ out: "src-tauri/compmd", source: "compmd://localhost/" })`);
//! the app's `build.rs` embeds it, with the `build` feature:
//!
//! ```no_run
//! # #[cfg(feature = "build")]
//! tauri_plugin_compmd::build::bundle("compmd");
//! ```
//!
//! and its `main` passes it to the plugin:
//!
//! ```ignore
//! tauri::Builder::default().plugin(tauri_plugin_compmd::init(tauri_plugin_compmd::include_bundle!()))
//! ```
//!
//! The page then takes everything from its own bundle and the origin from the plugin, and passes nothing:
//!
//! ```js
//! import { sandbox } from "virtual:compmd";
//! const view = createSandbox({ ...sandbox, page: "host" }).open(container, { components });
//! ```

#[cfg(feature = "build")]
pub mod build;
#[cfg(feature = "build")]
mod build_error;
mod compmd;
mod origin;
mod scheme;

use std::path::PathBuf;

pub use compmd::Compmd;
/// The shared layers of hosting: the bundle, the store and the serving contract.
pub use compmd_host;
use compmd_host::{Bundle, FolderStore, NoStore, Store};
use tauri::plugin::{Builder, TauriPlugin};
use tauri::{Manager, Runtime};

#[cfg(feature = "build")]
pub use build_error::BuildError;

/// The bundle `build::bundle` embedded in the app, or `None` when the build found none (with a warning then): the
/// plugin then serves nothing, and the page reports that its frames didn't load.
///
/// ```ignore
/// tauri::Builder::default().plugin(tauri_plugin_compmd::init(tauri_plugin_compmd::include_bundle!()))
/// ```
#[macro_export]
macro_rules! include_bundle {
    () => {
        $crate::embedded(include!(concat!(env!("OUT_DIR"), "/compmd_bundle.rs")))
    };
}

/// The bundle of the files `build::bundle` embedded, for [`include_bundle!`]; none when there are none.
#[doc(hidden)]
#[must_use]
pub fn embedded(files: Option<&'static [(&'static str, &'static [u8])]>) -> Option<Bundle> {
    files.and_then(|files| Bundle::embedded(files).ok())
}

/// The plugin, serving `bundle` on the scheme its manifest names (`compmd`), with the origin given to the app's pages
/// and the pack store in the app's data folder.
///
/// # Panics
///
/// When the bundle's scheme isn't a scheme's name, or it was built without one (`source`): mistakes in the app's
/// build, which `build::bundle` refuses already.
///
/// # Example
///
/// ```
/// # use compmd_host::Bundle;
/// let bundle = Bundle::from_files([(
///     "manifest.json",
///     br#"{"format": 1, "frames": {"hash": "1df336e75685de8c"}, "source": "compmd://localhost/"}"#.to_vec(),
/// )])?;
/// let app = tauri::Builder::<tauri::Wry>::default().plugin(tauri_plugin_compmd::init(bundle));
/// # drop(app);
/// # Ok::<(), compmd_host::BundleError>(())
/// ```
#[must_use]
pub fn init<R: Runtime>(bundle: impl Into<Option<Bundle>>) -> TauriPlugin<R> {
    let bundle = bundle.into();
    let scheme = bundle.as_ref().map(|bundle| {
        let scheme = bundle.manifest().scheme();
        let scheme = scheme.unwrap_or_else(|| panic!("{}", BUILT_WITHOUT_SOURCE));
        assert!(
            origin::is_scheme(scheme),
            "`{scheme}` isn't a scheme's name"
        );
        scheme.to_owned()
    });
    let origin = scheme.as_deref().map(origin::origin_of);
    let mut plugin = Builder::new("compmd");
    if let (Some(scheme), Some(origin)) = (&scheme, &origin) {
        plugin = plugin
            .js_init_script(origin::init_script(origin))
            .register_uri_scheme_protocol(scheme.clone(), scheme::serve);
    }
    plugin
        .setup(move |app, _api| {
            let store = store_in(app.path().app_data_dir().ok());
            app.manage(Compmd::new(bundle, origin, store));
            Ok(())
        })
        .build()
}

/// The pack store in the app's data folder, `compmd/packs/` there (docs/tauri-plugin.md, decision 11); none when the
/// platform gives the app no data folder.
fn store_in(data: Option<PathBuf>) -> Box<dyn Store> {
    data.map_or_else(
        || Box::new(NoStore) as Box<dyn Store>,
        |data| Box::new(FolderStore::new(data.join("compmd").join("packs"))),
    )
}

/// Why a bundle built for an http origin can't be served on a scheme.
pub(crate) const BUILT_WITHOUT_SOURCE: &str = "the bundle was built without a `source`, so it names no scheme: build it with the scheme it is served on, `compmd({ out, source: \"compmd://localhost/\" })`";

/// What the plugin gives Rust code of the app.
pub trait CompmdExt<R: Runtime> {
    /// The bundle the app serves, its origin and the pack store.
    fn compmd(&self) -> &Compmd;
}

impl<R: Runtime, T: Manager<R>> CompmdExt<R> for T {
    fn compmd(&self) -> &Compmd {
        self.state::<Compmd>().inner()
    }
}

#[cfg(test)]
mod tests;
