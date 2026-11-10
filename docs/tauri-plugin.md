# The Tauri plugin

**Status: accepted (2026-11-09); revised 2026-11-10, proposed:** the app builds its frames, with its libraries and packs, from the `@compmd/sandbox` its page imports, and the plugin compiles that build into the app and serves it (decisions 2, 3, 5, 6 and 11; [what changes from 0.6.0](#what-changes-from-060)). How a [Tauri](https://tauri.app) app shows CompMD documents: `tauri-plugin-compmd`, a plugin that serves the sandbox's frame documents from a scheme of the app's own, and an example app in this repository that uses it and checks itself in a real webview, as a guide for hosts and as a test CI runs before every release. It builds what [running components in the sandbox](running-components-in-the-sandbox.md) asks of a host ("the host serves the frame documents") for a desktop or mobile app, where there is no web server.

## Why a plugin

A Tauri app's pages come from its own scheme (`tauri://localhost` on macOS and Linux, `http://tauri.localhost` on Windows and Android), which Tauri serves from the app's bundle. The sandbox's frames need an origin of their own, apart from the app's, so that a frame's code can't reach the app's page or its commands (decision 1 of the sandbox's design). An app gets one by registering a URI scheme for the frames. What the handler of that scheme must do is the same in every app, and each detail of it matters for safety, so CompMD writes it once:

- WebKit (macOS, iOS and Linux) reads `'self'` in a sandboxed frame on a custom scheme as matching nothing, so the frames' policies must name where they are served (`buildFrames`'s `source`), or no frame loads its runtime;
- WebKit caches a custom scheme's responses, so a frame document replaced at the same path can keep running its old version after the app is upgraded, and the frames and the page then speak different protocol versions;
- the frames must be served with the headers that sandbox them, and nothing else on that scheme may be served.

Each of these, and whatever a platform or Tauri changes later, is then found and fixed in one place, for every app that uses the plugin.

## Decisions

1. **A crate, `tauri-plugin-compmd`, in `tauri/` of this repository**, released with the implementation's version, by the same release workflow as `compmd` ([releases](releases.md), decision 5). It depends on `tauri` only, not on the processor: an app that processes documents in its page, with `@compmd/processor`, uses the plugin as well as one that processes them in Rust. It is outside the main Cargo workspace (its own `Cargo.toml` and lock file), so `cargo make ci` doesn't build Tauri; its tests run in the Tauri check (decision 9).

2. **The app builds its frames from the `@compmd/sandbox` its page imports** (revised 2026-11-10). The page already imports `@compmd/sandbox` from npm, so the frames are built from that same package, and the page and its frames speak one frame protocol by construction: there is one version to pin, in the app's `package.json`, and no second copy of anything. The build runs in the app's JavaScript build, before its dev server or bundler starts, through `@compmd/sandbox/build`'s `buildHashedFrames({ out, scheme, libraries, packs })`:
   - it runs `buildFrames` with the scheme's `source` (decision 6) and the libraries the app offers (`buildLibraries`, [libraries in the sandbox](libraries-in-the-sandbox.md)) into a folder named by a hash of everything it wrote (the first 16 hex digits of a SHA-256 over each file's path and bytes), and removes older builds. A new build is a new path, so no webview's cache serves an old frame, and every response can be cached for good;
   - it copies the packs the app offers into the same build (decision 11);
   - it writes `frames.json` beside it: `{ "format": 1, "hash": "<hash>", "scheme": "<scheme>", "packs": { … } }`.

   One build holds everything the frames and the page load from the scheme: the frame documents, the runtimes, Shiki and Temml, the app's libraries and its packs. The runtimes and Shiki's grammars are about 5 MB, 1.5 MB compressed; a page loads only what a document uses.

3. **The plugin compiles that build into the app and serves it; one call in `build.rs`, one in `main`.** The app's `build.rs` calls `tauri_plugin_compmd::build::frames("compmd")` (the crate as a build dependency, with its `build` feature), which reads `compmd/frames.json`, writes the Rust that embeds the build's files with `include_bytes!` into `OUT_DIR`, and tells Cargo to build again when `frames.json` changes. The app's `main` passes it to the plugin: `.plugin(tauri_plugin_compmd::init(tauri_plugin_compmd::include_frames!()))`. The binary carries its frames: no resource to bundle, no file read at startup, and the hash is known when the app is compiled (decision 5). The scheme is the one `frames.json` names, so the frames' `source` and the scheme they are served on can't disagree. `build::frames` refuses a `frames.json` of a format it doesn't know, with what to upgrade; a checkout that didn't build the frames compiles with a warning and serves nothing, which the page reports (`error`). The crate brings no frames of its own: it is a server, a few hundred lines, and builds no JavaScript.

4. **The handler serves the build's files and nothing else:**
   - only `GET`; any other method is `405`, with why in plain text;
   - only paths `/frames/<hash>/<path>` and `/packs/<hash>/<path>` (decision 11), where `<hash>` is the one `frames.json` names, `<path>` a file of the build, and no part of it empty, `.` or `..`; anything else, `frames.json` itself included, is `404`, with why;
   - a refusal with `Access-Control-Allow-Origin: *` too, so a page that fetches from the scheme reads why, not only that the fetch failed;
   - each file with `Content-Security-Policy: sandbox allow-scripts` (the documents are sandboxed even when opened outside a frame), `Access-Control-Allow-Origin: *` (a frame's module scripts come from an opaque origin), `Access-Control-Expose-Headers: *` (a page that checks how its frames are served, as an app's self-check does, reads these headers), `X-Content-Type-Options: nosniff`, its type by extension (`text/html`, `text/javascript`, `text/css`, `application/json`, `application/wasm`, fonts, images), and `Cache-Control: max-age=31536000, immutable`, since its path names its build;
   - an app built without its frames (no `frames.json`, or one that doesn't name a folder) serves nothing, and says so.

5. **The page gets the frames' `base` and packs from the plugin, never builds them, and makes no call** (revised 2026-11-10): the plugin's initialization script (the app's own pages, main frame only) sets `globalThis.__COMPMD__ = { base, packs }`, and `createSandbox` takes both from there when they aren't given, so the page writes `createSandbox({ page: "host" })`. `base` is `<scheme>://localhost/frames/<hash>/`, or `http://<scheme>.localhost/frames/<hash>/` on Windows and Android. Since the build is compiled into the app (decision 3), its hash is known before any page loads, so one way serves every app: the plugin has no command and no permission to grant. Rust code gets `base` from `app.compmd().base()`.

6. **The frames' `source` is the custom-scheme form of the app's scheme, `<scheme>://localhost/frames/`,** which `buildHashedFrames` writes from its `scheme` (and `<scheme>://localhost/packs/` for packs, decision 11). It names the folder, not the hash, which isn't known until the build is written. The scheme is the app's choice (`compmd` if it has no reason for another), written once, in the frames' build. On Windows and Android the frames are served over `http://<scheme>.localhost`, where `'self'` already matches; the source names nothing there and is harmless, so one build serves every platform.

7. **An app that sets a content security policy for its page adds four things to it**, which the README gives as a snippet and the plugin can't write (an app's policy is compiled from its `tauri.conf.json`); an app without one, as Tauri's template is, adds nothing:
   - `frame-src` the scheme, in both forms, for the frames;
   - `script-src` and `connect-src` the frames' folder (`<scheme>://localhost/frames/` and `http://<scheme>.localhost/frames/`), for a host-mode page, which loads the frames' Shiki and Temml from there ([libraries in the sandbox](libraries-in-the-sandbox.md)). Never `/packs/` and never the whole scheme: a pack's code runs in frames only (decision 11);
   - `frame-ancestors 'none'`.

8. **No frame reaches a Tauri command, since no frame has Tauri's invoke key** (revised 2026-11-09, when the example's test found that a capability doesn't do it). Tauri counts a page on any scheme the app registers as one of the app's own (`is_local_url`, on macOS and Linux), so the capability of the frames' window covers the frames' scheme too: listing it or not changes nothing. What keeps a frame's code from the app's commands is Tauri's invoke key: every call must carry it, and Tauri puts it only into the main frame's IPC script (`for_main_frame_only`), which a frame, an opaque origin in another document, can't read. The tests show both halves: a call from the scheme without the key is refused, and in the real webview the strict component in the example finds no Tauri in its frame (no `__TAURI_INTERNALS__`). An app must also declare its own commands (`tauri_build::AppManifest`) and grant them in its capability, so they are allowed only as the capability says. Keeping custom schemes out of Tauri's `local` would be a second barrier, asked of Tauri upstream.

9. **The example app is the plugin's test, in a real webview** (`examples/tauri/`): the smallest app that shows one `comp.md` with a Mermaid diagram (the [`@comp-markdown/mermaid`](mermaid-pack.md) pack, offered as decision 11 says), a highlighted code block and a small JSX component in a strict frame, processed in Rust with the `compmd` crate and rendered by `@compmd/sandbox` in host mode. It builds and uses CompMD exactly as an app outside this repository does ([use](#use)), from the packages' and crates' versions in this repository, so the release checks what hosts will get. Its README is the guide for a host. With `--check` it renders, waits for `view.settled()`, checks that the diagram drew an SVG, the code block is highlighted, the component mounted and no `error` came, and exits `0` or `1` with what failed; with `--check-broken` it serves frames whose runtime is missing and expects the sandbox's error for it. `cargo make tauri-check` runs both.

10. **CI runs the check on Linux**, with WebKitGTK under `xvfb-run`, in a workflow of its own that runs when `sandbox/`, `render/`, `processor/`, `tauri/` or `examples/tauri/` change, and on release tags: a release waits for it. WebKitGTK has WebKit's `'self'` and its cache, so it catches what matters here; macOS (WKWebView) is checked by hand before a release with the same command, since a macOS runner costs about ten times the minutes.

11. **Packs are served from the same build, under `/packs/`, and their code runs in frames only** (2026-11-10). An app offers packs by naming them to `buildHashedFrames` (`packs: ["@comp-markdown/mermaid"]`, installed from npm like any dependency, at the version its `package.json` pins). The build copies each pack's published files into `/packs/<hash>/<name>@<version>/` and writes into `frames.json`'s `packs` each pack's version, the registry's integrity for it (`sha512-…`, from the app's lock file, which an import's `integrity` is checked against; none when the lock file doesn't give one), and each module's own (SHA-384, which a frame checks), which the initialization script hands the page (decision 5). `createSandbox`'s default `modules` then gives a frame a pack's module as `{ url, integrity }` on the scheme, which the frame checks before it runs it:
    - **one copy:** the pack's module is a file of the build, cached for good like a frame, not text inlined into the page's bundle (Mermaid's is 5.2 MB);
    - **frames only:** the frames' policies name `<scheme>://localhost/packs/` (decision 6); the page's policy never does (decision 7), so a pack's code can't run in the app's page;
    - **the same files for the processor:** a document's imports are resolved from a pack's releases and files, which the processor reads through the app's `compmd::host::Packs`. The plugin gives the app the build's packs, `app.compmd().packs()`: the version of each pack in the build, with its integrity, and its files, so the app's `Packs` reads exactly what the frames run. The plugin doesn't depend on the processor (decision 1): the app writes the few lines that join them, as the example does.

## Use

An app that shows CompMD documents, with the Mermaid pack and the frames on the scheme `compmd`:

```json
// package.json
"scripts": {
  "frames": "compmd-frames --out src-tauri/compmd --scheme compmd --pack @comp-markdown/mermaid",
  "dev": "pnpm frames && vite dev",
  "build": "pnpm frames && vite build"
},
"dependencies": { "@compmd/sandbox": "0.7.0", "@comp-markdown/mermaid": "0.1.0" }
```

`compmd-frames` is `buildHashedFrames` as a command (`@compmd/sandbox`'s bin); an app with libraries to offer calls `buildHashedFrames` from a script of its own instead:

```js
import { buildHashedFrames } from "@compmd/sandbox/build";
await buildHashedFrames({ out: "src-tauri/compmd", scheme: "compmd", libraries: ["chart.js"], packs: ["@comp-markdown/mermaid"] });
```

```toml
# src-tauri/Cargo.toml
[dependencies]
tauri-plugin-compmd = "0.7.0"
compmd = "0.7.0"            # when it processes documents in Rust

[build-dependencies]
tauri-plugin-compmd = { version = "0.7.0", features = ["build"] }
```

```rust
// src-tauri/build.rs
fn main() {
    tauri_plugin_compmd::build::frames("compmd");
    tauri_build::build();
}

// src-tauri/src/main.rs
tauri::Builder::default()
    .plugin(tauri_plugin_compmd::init(tauri_plugin_compmd::include_frames!()))
```

```js
// the page
const view = createSandbox({ page: "host" }).open(container, { components: processed.hostComponents });
view.render(processed.expanded);
```

And, when it processes documents in Rust, its `Packs` over the build's packs:

```rust
use compmd::host::{Packs, Release};
use semver::Version;

/// The packs the app offers: the ones its frames' build holds.
struct BuiltPacks<'a>(&'a tauri_plugin_compmd::BuiltPacks);

impl Packs for BuiltPacks<'_> {
    fn releases(&self, pack: &str) -> Vec<Release> {
        self.0
            .release(pack)
            .map(|built| Release { version: built.version.clone(), integrity: built.integrity.clone(), broken: None })
            .into_iter()
            .collect()
    }

    fn pack_file(&self, pack: &str, version: &Version, path: &str) -> Option<String> {
        self.0.file(pack, version, path)
    }
}

// where it processes: `.resolve((), BuiltPacks(app.compmd().packs()), &Ambient::default())`
```

`src-tauri/compmd/` is the build's output: the app's `.gitignore` lists it. Its tests use `Frames::from_files` for a build of their own, so `cargo test` needs no JavaScript build.

## What changes from 0.6.0

The plugin and the packages change together, in 0.7.0, since the API does:

- **`@compmd/sandbox`:** `buildHashedFrames` takes `scheme` (in place of `source`) and `packs`, copies the packs, and writes `frames.json`'s `format`, `scheme` and `packs`; the `compmd-frames` command; `createSandbox`'s `modules` defaults to the packs `__COMPMD__` names; `buildFrames`' policies name the packs' source.
- **`tauri-plugin-compmd`:**
  - `build::frames` (the `build` feature) and `include_frames!`, and `init(frames)` in place of `init()` and `Builder`;
  - no embedded frames of its own: `build.rs`'s embedding, `tauri/frames/`, `cargo make tauri-frames` and its packaging go, and so does publishing the crate with `--allow-dirty` ([releases](releases.md));
  - no `frames_dir`, no `base` command and no permissions;
  - `/packs/<hash>/` served beside `/frames/<hash>/`, `app.compmd().packs()` (`BuiltPacks`: `release`, `file`), and `__COMPMD__.packs`;
  - the headers of decision 4, done: `Access-Control-Expose-Headers: *` on files, `Access-Control-Allow-Origin: *` on refusals;
  - `Frames::from_files`, for apps' tests.
- **The example app** uses CompMD as [use](#use) shows: `compmd-frames` with the Mermaid pack, the plugin's two calls, and a `Packs` over `pack_file` in place of `packs/mermaid/` on disk and the `?raw` import.
- **The Mermaid pack** ([its design](mermaid-pack.md), decision 7): a host loads it by URL from the build; "as text" stays possible for hosts without a scheme.
- **The docs:** the plugin's README, the example's README, [libraries in the sandbox](libraries-in-the-sandbox.md) (the build holds the app's libraries and packs), and the Tauri guide for hosts, from [use](#use).

## Alternatives

- **The crate brings its frames** (0.6.0's decision 2): one call and no Node step. But a page imports `@compmd/sandbox` from npm anyway, so its frames came from a second place, at a version that had to match the page's, checked only when a page loaded them; an app with its own libraries or packs needed its own build too, and then carried both; and the crate was published with built files that git doesn't keep. Kept out: one build, the page's, serves every app.
- **The app's build read from its resources when the app starts** (0.6.0's `frames_dir`): no build step in Rust, but a resource to bundle, a hash known only at startup, so the page asked for `base` with a command and the app granted a permission; two ways to get `base` for one job.
- **Packs given to frames as code** (`{ code: Blob }`, imported `?raw` into the page): no scheme needed, but the pack's whole module is in the page's bundle, uncached, and the processor reads the pack from somewhere else. It stays for hosts that serve no scheme.
- **Each app writes its handler.** What apps did until now. Each one has to find WebKit's `'self'`, its cache and the headers on its own, and gets them subtly different.
- **The frames served from the app's own scheme** (`tauri://`, next to the app's page). No plugin, but the frames would share the app's origin, which the sandbox's design rules out: a frame's code could reach the app's page and its commands.
- **The frames at fixed paths, uncached**, as a web server can serve them. No hashing, but WebKit's custom-scheme cache can't be relied on to honour `no-cache` and validators, and a stale frame after an upgrade speaks another protocol version.
- **A JavaScript package for the page** (`@compmd/tauri`), wrapping a `base` command. With the initialization script, the page needs no call at all.

## Risks

- **The invoke key is the one barrier between a frame and the app's commands** (decision 8). A change in how Tauri injects it (into every frame, say) would let frame code call commands the main window's capability grants; the example's check, which looks for Tauri in a strict frame, would catch it.

- **Tauri's own changes to custom schemes**: Tauri may add headers of its own, or change how a scheme is served on a platform. The example's check shows the headers a frame arrives with, so a change shows in CI, and it is fixed once, in the plugin, for every app that uses it, rather than in each app's own handler.
- **Mobile** (iOS, Android) isn't tested in CI; the plugin targets them, and its README says so.
- **The order of the two builds in development.** `tauri dev` starts the app's `beforeDevCommand` without waiting for it (Tauri's configuration reference: `wait` defaults to `false`), so the Rust may compile before `compmd-frames` has written `frames.json`. Then the binary has no frames, or the last ones, and the page says so; `build::frames` makes Cargo build again when `frames.json` changes. To check when it's built (the example's `--check` under `tauri dev`): whether `tauri dev` compiles only once the dev server answers, which the `frames` step comes before; if not, the README says to run `pnpm frames` once before `tauri dev`.
