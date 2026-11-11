# The Tauri plugin

**Status: accepted (2026-11-09); revised 2026-11-10, proposed:** the plugin is the Tauri adapter of [hosting CompMD](hosting.md): it compiles the app's bundle into the app, serves it and the pack store on a scheme, and gives the page the origin (decisions 2 to 7 and 11; [what changes from 0.6.0](#what-changes-from-060)). How a [Tauri](https://tauri.app) app shows CompMD documents: `tauri-plugin-compmd`, and an example app in this repository that uses it and checks itself in a real webview, as a guide for hosts and as a test CI runs before every release.

## Why a plugin

A Tauri app's pages come from its own scheme (`tauri://localhost` on macOS and Linux, `http://tauri.localhost` on Windows and Android), which Tauri serves from the app's bundle. The sandbox's frames need an origin of their own, apart from the app's, so that a frame's code can't reach the app's page or its commands (decision 1 of the sandbox's design). An app gets one by registering a URI scheme for the frames. What the handler of that scheme must do is the same in every app, and each detail of it matters for safety, so CompMD writes it once:

- WebKit (macOS, iOS and Linux) reads `'self'` in a sandboxed frame on a custom scheme as matching nothing, so the frames' policies must name where they are served (`buildFrames`'s `source`), or no frame loads its runtime;
- WebKit caches a custom scheme's responses, so a frame document replaced at the same path can keep running its old version after the app is upgraded, and the frames and the page then speak different protocol versions;
- the frames must be served with the headers that sandbox them, and nothing else on that scheme may be served.

Each of these, and whatever a platform or Tauri changes later, is then found and fixed in one place, for every app that uses the plugin.

## Decisions

1. **A crate, `tauri-plugin-compmd`, in `tauri/` of this repository**, released with the implementation's version, by the same release workflow as `compmd` ([releases](releases.md), decision 5). It depends on `tauri` and `compmd-host` ([hosting](hosting.md), decision 1), not on the processor: an app that processes documents in its page, with `@compmd/processor`, uses the plugin as well as one that processes them in Rust. It is outside the main Cargo workspace (its own `Cargo.toml` and lock file), so `cargo make ci` doesn't build Tauri; its tests run in the Tauri check (decision 9).

2. **The plugin is the Tauri adapter, and nothing more** (revised 2026-11-10). What every host does is shared ([hosting](hosting.md)): the configuration (`compmd.config.json`), the bundle (`@compmd/vite`, from the `@compmd/sandbox` the page imports), the pack store, the serving contract (`compmd-host`'s `respond`) and the check. The plugin does the four things only Tauri can: it compiles the bundle into the app, registers the scheme and passes its requests to `respond`, gives the page the origin, and gives the store a folder. It brings no frames of its own: a page's frames always come from the page's own package.

3. **Two calls, in `build.rs` and in `main`.** The app's `build.rs` calls `tauri_plugin_compmd::build::bundle("compmd")` (the crate as a build dependency, with its `build` feature), which reads the bundle's `manifest.json` in that folder, writes the Rust that embeds its files with `include_bytes!` into `OUT_DIR`, and tells Cargo to build again when the manifest changes. The app's `main` passes it to the plugin: `.plugin(tauri_plugin_compmd::init(tauri_plugin_compmd::include_bundle!()))`. The binary carries its bundle: no resource to bundle, no file read at startup. `build::bundle` refuses a manifest of a format it doesn't know, with what to upgrade; a checkout that didn't build the bundle compiles with a warning and serves nothing, which the page reports (`error`). Tests make a bundle of their own, `Bundle::from_files`, so `cargo test` needs no JavaScript build.

4. **The scheme serves the contract, through `compmd-host`'s `respond`** ([hosting](hosting.md), decision 8): `frames/<hash>/` from the bundle, `packs/<place>` from the bundle or the store, with the contract's methods, paths, refusals and headers. The plugin adds nothing to it and changes nothing in it: a header the check needs, or a rule WebKit wants, is fixed in `respond`, for every host.

5. **The page gets the origin from an initialization script, and the rest from its own bundle** ([hosting](hosting.md), decision 9): the script (the app's own pages, main frame only) sets `globalThis.__COMPMD__ = { origin }`, `<scheme>://localhost`, or `http://<scheme>.localhost` on Windows and Android, where Tauri serves custom schemes over http. `createSandbox({ ...sandbox, page: "host" })`, with `sandbox` from `virtual:compmd`, reads it. The page makes no call: the plugin has no command and no permission to grant. Rust code gets the origin and the manifest from `app.compmd()`.

6. **The scheme is the one the bundle was built for.** WebKit needs the frames' policies to name where they are served (above), so the bundle is built with the custom-scheme form as its `source` (`compmd({ source: "compmd://localhost/" })` in the app's Vite configuration), which the manifest records; the plugin registers the scheme the manifest names, so the two can't disagree. The scheme is the app's choice (`compmd` if it has no reason for another). On Windows and Android, where `'self'` already matches, the source names nothing there and is harmless, so one bundle serves every platform.

7. **The page's content security policy is `cspFor` of the origin** ([hosting](hosting.md), decision 10), pasted into `app.security.csp` in `tauri.conf.json`, which the plugin can't write (Tauri compiles it from the file). An app without one, as Tauri's template is, adds nothing. Both forms of the origin go in: `frame-src` the scheme, `script-src` and `connect-src` `<origin>/frames/`, never `<origin>/packs/`, and `frame-ancestors 'none'`.

8. **No frame reaches a Tauri command, since no frame has Tauri's invoke key** (revised 2026-11-09, when the example's test found that a capability doesn't do it). Tauri counts a page on any scheme the app registers as one of the app's own (`is_local_url`, on macOS and Linux), so the capability of the frames' window covers the frames' scheme too: listing it or not changes nothing. What keeps a frame's code from the app's commands is Tauri's invoke key: every call must carry it, and Tauri puts it only into the main frame's IPC script (`for_main_frame_only`), which a frame, an opaque origin in another document, can't read. The tests show both halves: a call from the scheme without the key is refused, and in the real webview the strict component in the example finds no Tauri in its frame (no `__TAURI_INTERNALS__`). An app must also declare its own commands (`tauri_build::AppManifest`) and grant them in its capability, so they are allowed only as the capability says. Keeping custom schemes out of Tauri's `local` would be a second barrier, asked of Tauri upstream.

9. **The example app is the plugin's test, in a real webview** (`examples/tauri/`): the smallest app that shows one `comp.md` with a Mermaid diagram (the [`@comp-markdown/mermaid`](mermaid-pack.md) pack, installed as a bundled pack), a highlighted code block and a small JSX component in a strict frame, processed in Rust with the `compmd` crate and rendered by `@compmd/sandbox` in host mode. It uses CompMD exactly as an app outside this repository does ([use](#use)), at the versions in this repository, so a release checks what hosts will get. With `--check` it runs [hosting](hosting.md)'s check (decision 11), with `__TAURI_INTERNALS__` as the host API a frame must not see, and exits `0` or `1` with what failed; with `--check-broken` it serves a bundle whose runtime is missing and expects the sandbox's error for it. `cargo make tauri-check` runs both. Its README is the guide for a host.

10. **CI runs the check on Linux**, with WebKitGTK under `xvfb-run`, in a workflow of its own that runs when `sandbox/`, `render/`, `processor/`, `vite/`, `registry/`, `host/`, `tauri/` or `examples/tauri/` change, and on release tags: a release waits for it. WebKitGTK has WebKit's `'self'` and its cache, so it catches what matters here; macOS (WKWebView) is checked by hand before a release with the same command, since a macOS runner costs about ten times the minutes.

11. **The store's folder is the app's data folder** ([hosting](hosting.md), decisions 4 and 5): registry packs a document imports, fetched when it needs them, are kept under `compmd/packs/` in `app.path().app_data_dir()`, by integrity and place, and served from there at `packs/<place>` like a bundled pack; compiled code is kept beside them (`compmd/compiled/`, [hosting](hosting.md), decision 6). The app fetches them, not its page: an app that processes in Rust resolves and fetches through `compmd-host` (H1 there); one that processes in its page has the plugin serve `registry/` as the proxy on the scheme, with `fetch: "host"`, so the plugin keeps what passes through. Either way a version is fetched once per device, and every document and view after loads it from the scheme.

## Use

An app that shows CompMD documents, with the Mermaid pack installed and the frames on the scheme `compmd`:

```json
// package.json
"dependencies": { "@compmd/sandbox": "0.7.0", "@compmd/vite": "0.7.0", "@comp-markdown/mermaid": "0.1.0" }
```

```js
// vite.config.js: the bundle, into the Rust project
import { compmd } from "@compmd/vite";
export default { plugins: [compmd({ out: "src-tauri/compmd", source: "compmd://localhost/" })] };
```

```json
// compmd.config.json: what documents get; every installed pack is there, strict
{ "registries": { "*": "https://registry.npmjs.org/" } }
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
    tauri_plugin_compmd::build::bundle("compmd");
    tauri_build::build();
}

// src-tauri/src/main.rs
tauri::Builder::default()
    .plugin(tauri_plugin_compmd::init(tauri_plugin_compmd::include_bundle!()))
```

```js
// the page
import { sandbox } from "virtual:compmd";
const view = createSandbox({ ...sandbox, page: "host" }).open(container, { components: processed.hostComponents });
view.render(processed.expanded);
```

And, when it processes documents in Rust, the packs come from the store, which `compmd-host` gives as the processor's `Packs`:

```rust
let packs = app.compmd().packs(); // the bundle's packs, then the store's
let processor = Processor::new(path, text)
    .parse(&ParseOptions::default())
    .resolve((), packs, &ambient);
```

`src-tauri/compmd/` is the bundle, the build's output: the app's `.gitignore` lists it.

## What changes from 0.6.0

The plugin and the packages change together, in 0.7.0, with [hosting](hosting.md)'s changes:

- **`build::bundle`, `include_bundle!` and `init(bundle)`** in place of `init()` and `Builder`; the scheme from the manifest.
- **No frames of its own:** `build.rs`'s embedding, `tauri/frames/`, `cargo make tauri-frames` and their packaging go, and so does publishing the crate with `--allow-dirty` ([releases](releases.md)).
- **No `frames_dir`, no `base` command, no permissions:** the page gets the origin from the initialization script.
- **The handler is `compmd-host`'s `respond`:** the headers of 0.6.0's decision 4 with today's fixes (`Access-Control-Expose-Headers: *` on files, `Access-Control-Allow-Origin: *` on refusals), and `packs/<place>`.
- **The store in the app's data folder**, and `registry/` for apps that process in the page.
- **The example app** uses CompMD as [use](#use) shows: `@compmd/vite` with the Mermaid pack installed, the plugin's two calls, and `app.compmd().packs()` in place of `packs/mermaid/` read from disk and the `?raw` import.
- **The docs:** the plugin's README, the example's README, and the guides' chapter on hosting.

## Appendix: cases that challenge each decision

| Decision | Case | What happens |
|---|---|---|
| 2 | An app wants its own handler for one extra path on the scheme | It registers a scheme of its own for it; the frames' scheme serves the contract and nothing else, or a frame could load what the app put there. |
| 3 | `cargo test` in a fresh clone, with no JavaScript build | The build warns, the binary has no bundle, and the tests make theirs with `Bundle::from_files`. |
| 3 | A manifest written by a newer `@compmd/vite` | `build::bundle` stops the build with the format it found and the plugin version that reads it. |
| 3 | The UI rebuilds the bundle in `tauri dev` | The manifest changed, so Cargo builds again and the app restarts with it. |
| 4 | WebKit needs one more header | Added to `respond`, released; every adapter gets it. |
| 5 | The page renders before the initialization script ran | It can't: Tauri runs initialization scripts before the page's own. |
| 5 | A frame reads `__COMPMD__` | The script runs in the main frame only; a frame has no such global, and nothing in it is secret anyway. |
| 6 | The app registers `compmd` but the bundle was built for `myframes` | Impossible: the scheme comes from the manifest. |
| 6 | Two apps on one machine both use the scheme `compmd` | Each app's webview has its own schemes; they don't meet. |
| 7 | The app's CSP lacks `<origin>/frames/` | Code blocks and math show as text in host mode, and the check fails on highlighting. |
| 8 | A strict frame's code calls `fetch("ipc://localhost/…")` with a guessed key | Blocked twice: the frame's policy allows no fetch, and Tauri checks the key. |
| 8 | A strict frame's code posts to `window.webkit.messageHandlers.ipc` | What the check's probe finds (Risks, layer 3); if the handler is there, only the key refuses it, and the check fails the release until that's closed. |
| 9 | The example app passes on Linux but a header differs on macOS | macOS is checked by hand before each release with the same command (decision 10). |
| 11 | The app is offline when a document imports a registry pack it never fetched | Pending, then F026 when the app gives up (spec §6.4); a pack fetched before loads from the store. |
| 11 | The user clears the app's data folder | The store is empty; packs are fetched again when documents need them. The bundle, in the binary, is untouched. |

## Alternatives

- **The crate brings its frames** (0.6.0's decision 2): one call and no build step. But a page imports `@compmd/sandbox` from npm anyway, so its frames came from a second place, at a version that had to match the page's, checked only when a page loaded them; an app with its own libraries or packs needed its own build too, and then carried both; and the crate was published with built files that git doesn't keep.
- **The app's build read from its resources when the app starts** (0.6.0's `frames_dir`): no build step in Rust, but a resource to bundle, and a hash known only at startup, so the page asked for `base` with a command and the app granted a permission.
- **Each app writes its handler.** What apps did until now. Each one has to find WebKit's `'self'`, its cache and the headers on its own, and gets them subtly different.
- **The frames served from the app's own scheme** (`tauri://`, next to the app's page). No plugin, but the frames would share the app's origin, which the sandbox's design rules out: a frame's code could reach the app's page and its commands.
- **The frames at fixed paths, uncached**, as a web server can serve them. No hashing, but WebKit's custom-scheme cache can't be relied on to honour `no-cache` and validators, and a stale frame after an upgrade speaks another protocol version.
- **A JavaScript package for the page** (`@compmd/tauri`), wrapping a command. With the initialization script, the page needs no call at all.

## Risks

- **What keeps a frame from the app's commands, layer by layer** (decision 8). Tauri's capabilities don't, since they count the frames' scheme as the app's own. What does:
  1. **No invoke key in a frame:** Tauri gives it to the main frame only, and a call without it is refused (the tests show both).
  2. **No network in a strict frame:** its policy loads and fetches nothing, so it can't reach Tauri's IPC endpoint (`ipc://localhost`, `http://ipc.localhost`) with or without a key. A permissive frame may reach only its pack's listed `https:` origins.
  3. **To check: the WebKit message handler.** Tauri's IPC also has a path through a WebKit script message handler (`window.webkit.messageHandlers`, used when the fetch path isn't available), which a content security policy doesn't govern. If WebKit exposes it to sub-frames, layer 2 doesn't cover it, and the key is all that stands there. The example's check gains a probe for it: a strict component reports whether `window.webkit?.messageHandlers?.ipc` exists in its frame, and the check fails if it does, before a release.
  4. **Asked of Tauri:** a capability that can leave out a scheme the app registers, so a frame's call is refused by the ACL whatever it carries. Until then an app can't add this check itself: a command's `InvokeMessage` doesn't give the URL of the frame that called it (Tauri's API reference).

  A change in how Tauri hands out the key (into every frame, say) would remove layer 1; the check, which looks for Tauri in a strict frame, would catch it.
- **Tauri's own changes to custom schemes**: Tauri may add headers of its own, or change how a scheme is served on a platform. The check shows the headers a frame arrives with, so a change shows in CI, and it is fixed once, in `respond`, for every host.
- **Mobile** (iOS, Android) isn't tested in CI; the plugin targets them, and its README says so.
- **The order of the two builds in development.** `tauri dev` starts the app's `beforeDevCommand` without waiting for it (Tauri's configuration reference: `wait` defaults to `false`), so the Rust may compile before Vite has written the bundle. Then the binary has no bundle, or the last one, and the page says so; `build::bundle` makes Cargo build again when the manifest changes. To check when it's built (the example's `--check` under `tauri dev`): whether `tauri dev` compiles only once the dev server answers, which comes after the plugin's `buildStart`; if not, the README says to run `vite build` once before `tauri dev`.
