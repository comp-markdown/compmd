# tauri-plugin-compmd

Shows CompMD documents in a [Tauri](https://tauri.app) app. It is the Tauri adapter of CompMD's hosting: what every host does (the configuration, the bundle, the pack store, the serving contract) is shared, in `@compmd/vite` and `compmd-host`, and the plugin does what only Tauri can. It embeds the bundle in the app, serves it on a scheme of its own, gives the page the origin, and keeps the packs the app fetches in its data folder. The design, and why each rule is there, is [docs/tauri-plugin.md](../docs/tauri-plugin.md), on [docs/hosting.md](../docs/hosting.md).

It shares its version with the `@compmd` packages: use `@compmd/vite` and `@compmd/sandbox` of the same version.

**Versions:**
- **0.6.0:** the first version: `init()`, which served frames the crate embedded on `compmd://` and gave the page their base; `Builder` for another scheme or an app's own build of the frames; the `base` command. (`0.0.0` is a placeholder with no code.)
- **Next (0.7.0):** the hosting adapter. The app embeds the bundle its own page's build writes (`build::bundle`, `include_bundle!`, `init(bundle)` in place of `init()` and `Builder`); the scheme is the one the bundle was built for; the page gets the origin, `__COMPMD__ = { origin }`; the scheme serves `compmd-host`'s contract, frames under their hash and packs at `packs/<place>`. The crate's own frames, `frames_dir`, the `base` command and the permissions are gone.

## Use

The page's build writes the bundle into the Rust project, with the scheme it is served on:

```js
// vite.config.js
import { compmd } from "@compmd/vite";
export default { plugins: [compmd({ out: "src-tauri/compmd", source: "compmd://localhost/" })] };
```

The app's `build.rs` embeds it, and its `main` gives it to the plugin:

```toml
# src-tauri/Cargo.toml
[dependencies]
tauri-plugin-compmd = "0.7.0"

[build-dependencies]
tauri-plugin-compmd = { version = "0.7.0", features = ["build"] }
```

```rust
// src-tauri/build.rs
fn main() {
    tauri_plugin_compmd::build::bundle("compmd");
    tauri_build::build();
}

// src-tauri/src/main.rs
tauri::Builder::default()
    .plugin(tauri_plugin_compmd::init(tauri_plugin_compmd::include_bundle!()))
```

The page takes everything from its own bundle, and the origin from the plugin:

```js
import { createSandbox } from "@compmd/sandbox";
import { sandbox } from "virtual:compmd";

const view = createSandbox({ ...sandbox, page: "host" }).open(container, { components: processed.hostComponents });
view.render(processed.expanded);
```

`src-tauri/compmd/` is build output: list it in the app's `.gitignore`. A build that finds no bundle there warns, and the app serves none, which its page reports. Run the page's build before the app's: `vite build`, or the dev server, which writes the bundle when it starts.

**An app that processes documents in Rust** turns on the `processor` feature, and gives the processor the same packs the frames load:

```rust
use tauri_plugin_compmd::CompmdExt;

let packs = app.compmd().packs().expect("a bundle"); // the bundle's packs, then the store's
let processor = Processor::new(path, text).parse(&ParseOptions::default()).resolve((), packs, &ambient);
```

**If the app sets a content security policy** (`app.security.csp` in `tauri.conf.json`; Tauri's template sets none), it adds what `cspFor` from `@compmd/vite` gives, for both forms of the origin:

```js
import { cspFor } from "@compmd/vite/config";
cspFor("compmd://localhost"); cspFor("http://compmd.localhost"); // Windows and Android
```

**Keeping frames from the app's commands:** a frame's code can't call a command, since Tauri gives its invoke key to the app's own page only. A capability doesn't do it: Tauri counts a page on a scheme the app registers as one of its own. Declare the app's commands (`tauri_build::AppManifest`) and grant them in the capability, as the example app does. On WebKit (macOS, Linux) a frame has Tauri's `ipc` message handler, which WebKit gives every frame and no content security policy governs: Tauri drops a message without the key, which the example's check shows by posting one ([the research](../docs/tauri-plugin.md#appendix-webkits-ipc-handler-in-a-frame)). The plugin needs Tauri 2.11.1 or later, which has the fix for [GHSA-7gmj-67g7-phm9](https://github.com/tauri-apps/tauri/security/advisories/GHSA-7gmj-67g7-phm9).

## Building it from this repository

`cargo make tauri-plugin` lints and tests the crate, with its features and without, and measures its coverage. Its tests make their bundles with `compmd-host`'s `Bundle::from_files`, so they need no JavaScript build.

## Status

In development, released with CompMD's packages at the same version. Tested on Linux (WebKitGTK) in CI by the example app, `examples/tauri/`, and by hand on macOS; iOS and Android aren't tested yet.
