//! What the plugin keeps for the app: the bundle, its origin and the pack store.

use compmd_host::{Bundle, Store, respond};
use tauri::http::{Request, Response, StatusCode};

/// The bundle the app serves, where, and the store of the packs it fetches, as the app's state (`app.compmd()`).
pub struct Compmd {
    bundle: Option<Bundle>,
    origin: Option<String>,
    store: Box<dyn Store>,
}

/// Why the scheme serves nothing without a bundle.
const NO_BUNDLE: &str = "this app has no bundle: build it with @compmd/vite before the app";

impl Compmd {
    /// The state of an app serving `bundle` at `origin`, with `store`.
    pub(crate) fn new(
        bundle: Option<Bundle>,
        origin: Option<String>,
        store: Box<dyn Store>,
    ) -> Self {
        Self {
            bundle,
            origin,
            store,
        }
    }

    /// The bundle, when the app's build embedded one.
    #[must_use]
    pub const fn bundle(&self) -> Option<&Bundle> {
        self.bundle.as_ref()
    }

    /// Where the bundle is served, as the page has it in `__COMPMD__`: `compmd://localhost`, or
    /// `http://compmd.localhost` on Windows and Android.
    #[must_use]
    pub fn origin(&self) -> Option<&str> {
        self.origin.as_deref()
    }

    /// The store of the packs the app fetches, in its data folder.
    #[must_use]
    pub fn store(&self) -> &dyn Store {
        self.store.as_ref()
    }

    /// The answer to a webview's `request` on the scheme: the serving contract (`compmd-host`'s `respond`), or a
    /// `404` saying why when there is no bundle.
    #[must_use]
    pub fn respond<B>(&self, request: &Request<B>) -> Response<Vec<u8>> {
        self.bundle.as_ref().map_or_else(
            || {
                let mut response = Response::new(NO_BUNDLE.as_bytes().to_vec());
                *response.status_mut() = StatusCode::NOT_FOUND;
                response
            },
            |bundle| respond(bundle, self.store(), request),
        )
    }

    /// The processor's packs: the bundle's, then the store's, the same files the frames load; none without a bundle.
    #[cfg(feature = "processor")]
    #[must_use]
    pub fn packs(&self) -> Option<compmd_host::HostPacks<'_>> {
        Some(compmd_host::HostPacks::new(
            self.bundle.as_ref()?,
            self.store(),
        ))
    }
}
