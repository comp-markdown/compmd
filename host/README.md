# compmd-host

What every host of CompMD documents shares, whatever its platform: the bundle a host builds before anyone reads (the sandbox's frames under a content hash, the packs it installs, and its `manifest.json`), the store of packs it gets later, and `respond`, the serving contract an adapter puts behind its origin. The design, and why each rule is there, is [hosting CompMD](../docs/hosting.md).

```rust
use compmd_host::{respond, Bundle, FolderStore};

let bundle = Bundle::read("compmd".as_ref())?;     // what @compmd/vite wrote
let store = FolderStore::new(app_data.join("compmd/packs"));
let response = respond(&bundle, &store, &request); // for a request on the adapter's origin
```

- **`Bundle`** reads a bundle embedded in a binary, in a folder, or made in memory for a test (`Bundle::from_files`); its `Manifest` is what `@compmd/vite` wrote, and a manifest of a format this crate doesn't know is refused, saying what to upgrade.
- **`Store`** is where a host keeps packs beyond the bundle; `FolderStore` keeps them in a folder, by their place, with their integrity.
- **`respond`** answers `GET /frames/<hash>/<path>` and `GET /packs/<name>@<version>/<path>`, with the headers that sandbox them, and refuses anything else, saying why.
- **`HostPacks`** (the `processor` feature) gives the processor, `compmd`, the same packs the frames load.

Adapters: [`tauri-plugin-compmd`](../tauri/README.md) for Tauri.

## Status

In development, released with CompMD's packages at the same version.
