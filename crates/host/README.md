# compmd-host

What every host of [CompMD](https://compmd.dev/) documents shares, whatever its platform: the bundle a host builds before anyone reads (the sandbox's frames under a content hash, the packs it installs, and its `manifest.json`), the store of packs it gets later, and `respond`, the serving contract an adapter puts behind its origin. The design, and why each rule is there, is [hosting CompMD](https://github.com/comp-markdown/compmd/blob/main/docs/hosting.md).

```rust
use compmd_host::{respond, Bundle, FolderStore};

let bundle = Bundle::read("compmd".as_ref())?;     // what @compmd/vite wrote
let store = FolderStore::new(app_data.join("compmd/packs"));
let response = respond(&bundle, &store, &request); // for a request on the adapter's origin
```

- **`Bundle`** reads a bundle embedded in a binary, in a folder, or made in memory for a test (`Bundle::from_files`); its `Manifest` is what `@compmd/vite` wrote, and a manifest of a format this crate doesn't know is refused, saying what to upgrade.
- **`Store`** is where a host keeps packs beyond the bundle; `FolderStore` keeps them in a folder, by their place, with their integrity. Each release is written whole beside the one kept, then renamed into place, so a keep that fails halfway leaves the release that was there.
- **`respond`** answers `GET /frames/<hash>/<path>` and `GET /packs/<name>@<version>/<path>`, with the headers that sandbox them, and refuses anything else, saying why.
- **`HostPacks`** (the `processor` feature) gives the processor, `compmd`, the same packs the frames load.
- **`registry::Resolution`** (the `registry` feature) resolves a request's packs from registries: the version each import takes, the policy before any download, the tarball checked against its integrity and read. It does no I/O: it says what it needs next (the processor's missing packs, a cache's files, a registry's metadata or tarball), and the host does it its own way and answers ([the registry rules](https://github.com/comp-markdown/compmd/blob/main/docs/registry-rules.md)). `registry::FolderCache` answers its cache's steps from a folder: the releases, as `FolderStore` keeps them, and each pack's versions for as long as its registry's `Cache-Control` says.
- **`registry::resolve`** (the `client` feature) drives a resolution to its end on a thread of the host's own: its cache's steps from a `FolderCache`, its registries' with `registry::Client` (`ureq`, from each registry's own origin), or none when the host is off the network.
- **`registry::Proxy`** (the `client` feature) is a host's `registry/` proxy, as `@compmd/registry`'s `registryProxy` is in JavaScript: it passes a pack's metadata, its tarballs' URLs its own, and its tarballs on from the registries a bundle's manifest names (`registries`, `fetch`, `maxPackSize`).
- **`registry::LocalRegistry`** (the `local-registry` feature) serves packs over npm's protocol from `http://127.0.0.1`, for a host's tests.
- **`tracing`**: `respond` is a span (`compmd_host::respond`, with the path) with an event of the status it answered, and `FolderStore::keep` one (`compmd_host::keep`, with the release) with a warning when the release isn't kept. The crate installs no subscriber: a host that wants them installs its own.

Adapters: [`tauri-plugin-compmd`](https://github.com/comp-markdown/compmd/blob/main/crates/tauri/README.md) for Tauri.

## Status

In development, released with CompMD's packages at the same version.

**Versions:**
- **Unreleased:** the `registry` feature, the registry rules as a state machine (`registry::Resolution`), with `registry::FolderCache`; the `client` feature, `registry::resolve` and `registry::Client`; the `local-registry` feature, `registry::LocalRegistry`. `FolderStore::files_of` gives a kept release's files. `Manifest` reads `registries`, `fetch` (`FetchedBy`) and `maxPackSize`; `registry::Proxy` and `Client::metadata`.
- **0.11.0:** a request's path is percent-decoded segment by segment before it is read, and a decoded `..` is still refused; `refused` and `Bundle::pack_file_or` are public, so Rust and JavaScript hosts serve a bundle the same way. `FolderStore::keep` writes a release whole, then moves it into place, so a store never holds half a release. With the `processor` feature, `HostPacks::policy()` and `ambient()` give the bundle's policy and ambient packs for `resolve_with`, and a policy the processor can't read is `ManifestError::Policy`. `tracing` spans `compmd_host::respond` and `compmd_host::keep`. Breaking, for Rust code: `compmd_host::Files` is no longer public; `ManifestError` has a new variant.
- **0.10.0:** the same as 0.9.0.
- **0.9.0:** the same as 0.8.0.
- **0.8.0:** the serving contract's decision apart from its answer: `route`, a path's `Route`, a `Refusal` saying why, and `served_headers` and `refusal_headers`, which `@compmd/host` shares as WebAssembly; `respond` answers as before.
- **0.7.1:** the same as 0.7.0; its homepage and links name the website's new address, [compmd.dev](https://compmd.dev/).
- **0.7.0:** the first version.
