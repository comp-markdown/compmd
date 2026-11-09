# The Tauri plugin

**Status: accepted (2026-11-09).** How a [Tauri](https://tauri.app) app shows CompMD documents: `tauri-plugin-compmd`, a plugin that serves the sandbox's frame documents from a scheme of the app's own, and an example app in this repository that uses it and checks itself in a real webview, as a guide for hosts and as a test CI runs before every release. It builds what [running components in the sandbox](running-components-in-the-sandbox.md) asks of a host ("the host serves the frame documents") for a desktop or mobile app, where there is no web server.

## Why a plugin

A Tauri app's pages come from its own scheme (`tauri://localhost` on macOS and Linux, `http://tauri.localhost` on Windows and Android), which Tauri serves from the app's bundle. The sandbox's frames need an origin of their own, apart from the app's, so that a frame's code can't reach the app's page or its commands (decision 1 of the sandbox's design). An app gets one by registering a URI scheme for the frames. What the handler of that scheme must do is the same in every app, and each detail of it matters for safety, so CompMD writes it once:

- WebKit (macOS, iOS and Linux) reads `'self'` in a sandboxed frame on a custom scheme as matching nothing, so the frames' policies must name where they are served (`buildFrames`'s `source`), or no frame loads its runtime;
- WebKit caches a custom scheme's responses, so a frame document replaced at the same path can keep running its old version after the app is upgraded, and the frames and the page then speak different protocol versions;
- the frames must be served with the headers that sandbox them, and nothing else on that scheme may be served.

Each of these, and whatever a platform or Tauri changes later, is then found and fixed in one place, for every app that uses the plugin.

## Decisions

1. **A crate, `tauri-plugin-compmd`, in `tauri/` of this repository**, released with the implementation's version, by the same release workflow as `compmd` ([releases](releases.md), decision 5). It depends on `tauri` only, not on the processor: an app that processes documents in its page, with `@compmd/processor`, uses the plugin as well as one that processes them in Rust. It is outside the main Cargo workspace (its own `Cargo.toml` and lock file), so `cargo make ci` doesn't build Tauri; its tests run in the Tauri check (decision 9).

2. **One call, and the crate brings its frames** (revised in review, 2026-11-09): an app writes `.plugin(tauri_plugin_compmd::init())` and nothing else. The crate embeds the frames, built when CompMD is released, so an app has no Node step, no resource to bundle and no file to keep in step. They are built with content-hashed paths by `@compmd/sandbox/build`'s new `buildHashedFrames({ out, source, … })`, which runs `buildFrames` into a folder named by a hash of everything it wrote (the first 16 hex digits of a SHA-256 over each file's path and bytes), writes `frames.json`, `{ "hash": "<hash>" }`, beside it, and removes older builds. A new build is a new path, so no webview's cache serves an old frame, and every response can be cached for good. The crate's `build.rs` embeds the build it finds in `tauri/frames/` (written by `cargo make tauri-frames` and packaged with the crate), with its hash; a crate built without them, from a checkout that didn't run it, warns at build time and serves nothing, which the page then reports. The frames are about 5 MB, 1.5 MB compressed. The crate's version is the packages' (decision 1), so its frames speak the frame protocol `@compmd/sandbox` of that version does; a page of another version fails with the sandbox's `error` naming both.

3. **Further calls customize it**: `tauri_plugin_compmd::Builder::new()`, then `.scheme("frames")` for another scheme's name (`compmd` by default) and `.frames_dir("compmd")` for the app's own build of the frames (a resource folder holding `frames.json`, or an absolute one, read once when the app starts), then `.build()`. An app needs its own build to offer libraries beyond the runtimes (`buildLibraries`), or to use another scheme, since the embedded frames name `compmd://` as their source (decision 6); a scheme other than `compmd` without `frames_dir` is a mistake the builder refuses.

4. **The handler serves the build's files and nothing else:**
   - only `GET`; any other method is `405`, with why in plain text;
   - only paths `/frames/<hash>/<path>`, where `<hash>` is the one `frames.json` names, `<path>` a file in that folder, and no part of it empty, `.` or `..`; anything else, `frames.json` itself included, is `404`, with why;
   - each file with `Content-Security-Policy: sandbox allow-scripts` (the documents are sandboxed even when opened outside a frame), `Access-Control-Allow-Origin: *` (a frame's module scripts come from an opaque origin), `X-Content-Type-Options: nosniff`, its type by extension (`text/html`, `text/javascript`, `text/css`, `application/json`, `application/wasm`, fonts, images), and `Cache-Control: max-age=31536000, immutable`, since its path names its build;
   - an app built without its frames (no `frames.json`, or one that doesn't name a folder) serves nothing, and says so.

5. **The frames' `base` comes from the plugin, never built by the page, and needs no call** (revised in review): with the embedded frames, the plugin's initialization script (the app's own pages, main frame only) sets `globalThis.__COMPMD__ = { base }`, and `createSandbox` takes `base` from there when it isn't given, so the page writes `createSandbox({ page: "host" })`. `base` is `<scheme>://localhost/frames/<hash>/`, or `http://<scheme>.localhost/frames/<hash>/` on Windows and Android. For an app's own build (`frames_dir`), whose hash is known only when the app starts, the page asks for it with the command `plugin:compmd|base` (`null` without a build), which the plugin's default permission set grants: the app lists `compmd:default` in the capability of the windows that render documents. Rust code gets it from `app.compmd().base()`.

6. **The frames' `source` is the custom-scheme form, `<scheme>://localhost/frames/`**: the embedded frames are built with `compmd://localhost/frames/`, and an app's own build passes its scheme's to `buildHashedFrames` (the README's snippet). It names the folder, not the hash, which isn't known until the build is written. On Windows and Android the frames are served over `http://<scheme>.localhost`, where `'self'` already matches; the source names nothing there and is harmless, so one build serves every platform.

7. **An app that sets a content security policy for its page adds four things to it**, which the README gives as a snippet and the plugin can't write (an app's policy is compiled from its `tauri.conf.json`); an app without one, as Tauri's template is, adds nothing:
   - `frame-src` the scheme, in both forms, for the frames;
   - `script-src` and `connect-src` the frames' folder (`<scheme>://localhost/frames/` and `http://<scheme>.localhost/frames/`), for a host-mode page, which loads the frames' Shiki and Temml from there ([libraries in the sandbox](libraries-in-the-sandbox.md));
   - `frame-ancestors 'none'`.

8. **No page on the scheme reaches a Tauri command.** The scheme is never in a capability, so Tauri refuses every invoke from it: a frame's code, whatever it is, can't call the app. A test calls commands from a page at each form of the scheme's URL and expects each refused.

9. **The example app is the plugin's test, in a real webview** (`examples/tauri/`): the smallest app that shows one `comp.md` with a Mermaid diagram (the [`@comp-markdown/mermaid`](mermaid-pack.md) pack), a highlighted code block and a small JSX component in a strict frame, processed in Rust with the `compmd` crate and rendered by `@compmd/sandbox` in host mode. Its README is the guide for a host. With `--check` it renders, waits for `view.settled()`, checks that the diagram drew an SVG, the code block is highlighted, the component mounted and no `error` came, and exits `0` or `1` with what failed; with `--check-broken` it serves frames whose runtime is missing and expects the sandbox's error for it. `cargo make tauri-check` runs both.

10. **CI runs the check on Linux**, with WebKitGTK under `xvfb-run`, in a workflow of its own that runs when `sandbox/`, `render/`, `processor/`, `tauri/` or `examples/tauri/` change, and on release tags: a release waits for it. WebKitGTK has WebKit's `'self'` and its cache, so it catches what matters here; macOS (WKWebView) is checked by hand before a release with the same command, since a macOS runner costs about ten times the minutes.

## Alternatives

- **Each app writes its handler.** What apps did until now. Each one has to find WebKit's `'self'`, its cache and the headers on its own, and gets them subtly different.
- **The frames served from the app's own scheme** (`tauri://`, next to the app's page). No plugin, but the frames would share the app's origin, which the sandbox's design rules out: a frame's code could reach the app's page and its commands.
- **The frames at fixed paths, uncached**, as a web server can serve them. No hashing, but WebKit's custom-scheme cache can't be relied on to honour `no-cache` and validators, and a stale frame after an upgrade speaks another protocol version.
- **A JavaScript package for the page** (`@compmd/tauri`), wrapping the `base` command. With the initialization script, the page needs no call at all.
- **The app builds the frames itself**, as the first version of this design had it: a Node step before each build, a resource to bundle, and a `frames.json` to keep in step, which the review found too much to ask (2026-11-09). It stays as `frames_dir`, for an app that needs its own build.

## Risks

- **Tauri's own changes to custom schemes**: Tauri may add headers of its own, or change how a scheme is served on a platform. The example's check shows the headers a frame arrives with, so a change shows in CI, and it is fixed once, in the plugin, for every app that uses it, rather than in each app's own handler.
- **Mobile** (iOS, Android) isn't tested in CI; the plugin targets them, and its README says so.
