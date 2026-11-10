# tauri-plugin-compmd

Shows CompMD documents in a [Tauri](https://tauri.app) app: the plugin brings `@compmd/sandbox`'s frames, serves them from a scheme of their own, and tells the page where they are. The design, and why each rule is there, is [docs/tauri-plugin.md](../docs/tauri-plugin.md).

It shares its version with the `@compmd` packages, and embeds the frames of `@compmd/sandbox` at that version, so a page uses the sandbox of the same version.

**Versions:**
- **0.6.0:** the first version: `init()`, which serves the embedded frames on `compmd://` and gives the page their base; `Builder` for another scheme or an app's own build of the frames; the `base` command. (`0.0.0` is a placeholder with no code.)
- **Next:** a refusal is readable from any origin, so a page reads why; files come with `Access-Control-Expose-Headers: *`, so a page can check their headers.

## Use

```rust
tauri::Builder::default()
    .plugin(tauri_plugin_compmd::init())
```

```js
import { createSandbox } from "@compmd/sandbox";

const view = createSandbox({ page: "host" }).open(container, { components: processed.hostComponents });
view.render(processed.expanded);
```

That's all: no build step, nothing to bundle, no permission to grant. Use the `@compmd/sandbox` of the crate's version: they are released together, and a page of another version gets an `error` naming both.

**If the app sets a content security policy** (`app.security.csp` in `tauri.conf.json`; Tauri's template sets none), it adds:

```
frame-src compmd: http://compmd.localhost;
script-src 'self' compmd://localhost/frames/ http://compmd.localhost/frames/;
connect-src 'self' ipc: http://ipc.localhost compmd://localhost/frames/ http://compmd.localhost/frames/;
frame-ancestors 'none'
```

## Customizing

```rust
tauri_plugin_compmd::Builder::new()
    .scheme("myframes")      // another scheme's name
    .frames_dir("sandbox")   // the app's own build of the frames
    .build()
```

An app builds its own frames to offer libraries beyond the runtimes (`buildLibraries`), or to use another scheme. It builds them before `tauri build` and `tauri dev`, into a folder it bundles as a resource, with its scheme as their source:

```js
import { buildHashedFrames } from "@compmd/sandbox/build";
await buildHashedFrames({ out: "src-tauri/sandbox", source: "myframes://localhost/frames/" });
```

and its page asks for `base`, which the plugin's `compmd:default` permission grants:

```js
const base = await invoke("plugin:compmd|base");
const view = createSandbox({ base, page: "host" }).open(container, { components });
```

**Keeping frames from the app's commands:** a frame's code can't call a command, since Tauri gives its invoke key to the app's own page only. A capability doesn't do it: Tauri counts a page on a scheme the app registers as one of its own. Declare the app's commands (`tauri_build::AppManifest`) and grant them in the capability, as the example app does.

## Building it from this repository

The crate embeds the frames, which `cargo make tauri-frames` builds into `tauri/frames/` from `@compmd/sandbox`; `cargo make tauri-plugin` builds them, lints and tests the crate. A crate built without them warns, and serves none.

## Status

In development, released with CompMD's packages at the same version. Tested on Linux (WebKitGTK) in CI by the example app, `examples/tauri/`, and by hand on macOS; iOS and Android aren't tested yet.
