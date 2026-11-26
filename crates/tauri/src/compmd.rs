//! What the plugin keeps for the app: the bundle, its origin and the pack store.

use std::sync::Arc;
use std::time::Duration;

use compmd_host::registry::{Client, DEFAULT_MAX_PACK_SIZE, Options, Proxy};
use compmd_host::{Bundle, FetchedBy, Manifest, Refusal, Store, refused, respond};
use tauri::http::{HeaderValue, Request, Response, StatusCode, header};

/// The bundle the app serves, where, and the store of the packs it fetches, as the app's state (`app.compmd()`). The
/// bundle and the store are shared, so the processor can keep them as its packs (`packs`).
pub struct Compmd {
    bundle: Option<Arc<Bundle>>,
    origin: Option<String>,
    store: Arc<dyn Store>,
    /// The `registry/` proxy, when the bundle's configuration fetches registry packs through the host.
    proxy: Option<Proxy>,
}

/// Why the scheme serves nothing without a bundle.
const NO_BUNDLE: &str = "this app has no bundle: build it with @compmd/vite before the app";

/// Where the proxy is served on the scheme.
pub(crate) const REGISTRY: &str = "/registry/";

/// How long one request to a registry may take, its body read whole.
const TIMEOUT: Duration = Duration::from_secs(30);

/// The proxy of `manifest`'s registries, when it fetches through the host (docs/tauri-plugin.md, decision 11).
fn proxy_of(manifest: &Manifest) -> Option<Proxy> {
    if manifest.fetch != FetchedBy::Host || manifest.registries.is_empty() {
        return None;
    }
    let options = Options {
        max_pack_size: manifest.max_pack_size.unwrap_or(DEFAULT_MAX_PACK_SIZE),
        ..Options::new(manifest.registries.clone())
    };
    Some(Proxy::new(options, Client::new(TIMEOUT)))
}

impl Compmd {
    /// The state of an app serving `bundle` at `origin`, with `store`.
    pub(crate) fn new(
        bundle: Option<Bundle>,
        origin: Option<String>,
        store: Arc<dyn Store>,
    ) -> Self {
        Self {
            proxy: bundle
                .as_ref()
                .and_then(|bundle| proxy_of(bundle.manifest())),
            bundle: bundle.map(Arc::new),
            origin,
            store,
        }
    }

    /// The bundle, when the app's build embedded one.
    #[must_use]
    pub fn bundle(&self) -> Option<&Bundle> {
        self.bundle.as_deref()
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

    /// The answer to a webview's `request` on the scheme: under `registry/`, the proxy's, when the bundle fetches
    /// through the host (it blocks on the registry, so the scheme answers it on a thread of its own); otherwise the
    /// serving contract (`compmd-host`'s `respond`), or a `404` saying why when there is no bundle, with the
    /// contract's refusal headers, so the page can read it.
    #[must_use]
    pub fn respond<B>(&self, request: &Request<B>) -> Response<Vec<u8>> {
        let path = request.uri().path();
        if let (Some(proxy), Some(origin), Some(rest)) =
            (&self.proxy, &self.origin, path.strip_prefix(REGISTRY))
        {
            let mut answer = proxy.answer(rest, &format!("{origin}{REGISTRY}"));
            // The app's page is on an origin of its own, and a pack's metadata and tarballs are public.
            answer.headers_mut().insert(
                header::ACCESS_CONTROL_ALLOW_ORIGIN,
                HeaderValue::from_static("*"),
            );
            return answer;
        }
        self.bundle.as_ref().map_or_else(
            || {
                refused(Refusal {
                    status: StatusCode::NOT_FOUND,
                    why: NO_BUNDLE,
                })
            },
            |bundle| respond(bundle, self.store(), request),
        )
    }

    /// The processor's packs: the bundle's, then the store's, the same files the frames load, with the policy and
    /// ambient packs the bundle's configuration made, which the processor is given too; none without a bundle.
    ///
    /// ```ignore
    /// let packs = app.compmd().packs().expect("a bundle");
    /// let (policy, ambient) = (packs.policy()?, packs.ambient());
    /// let processor = Processor::new(path, text).parse(&ParseOptions::default()).resolve_with((), packs, &ambient, &policy);
    /// ```
    #[cfg(feature = "processor")]
    #[must_use]
    pub fn packs(&self) -> Option<compmd_host::HostPacks> {
        let bundle = Arc::clone(self.bundle.as_ref()?);
        Some(compmd_host::HostPacks::new(bundle, Arc::clone(&self.store)))
    }
}
