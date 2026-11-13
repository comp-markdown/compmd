# 19. A Tauri app

A desktop app shows documents the way a web app does, with one difference: its page has no server of its own to serve the bundle from. [`tauri-plugin-compmd`](../../tauri/README.md) is that server for a [Tauri](https://tauri.app) app. It embeds the bundle in the app's binary, serves it on a scheme of its own, gives the page the bundle's origin, and keeps the packs the app fetches later in its data folder. The page's side is what you know from Part II: `@compmd/vite`, `virtual:compmd` and `createSandbox`.

This chapter walks through the repository's Tauri example, [`examples/tauri/`](../../examples/tauri). It is the smallest Tauri app that shows a CompMD document: a Mermaid diagram drawn by the `@comp-markdown/mermaid` pack, a code block highlighted by the frames' Shiki, and a JSX component the document defines, each component in a strict frame of its own. The code blocks here are its files, and its own check runs it in a real webview before every release.

## What goes where

| Job | Where |
|---|---|
| builds the bundle, for the plugin's scheme | the page's build: `@compmd/vite` with `out` and `source` |
| embeds the bundle in the binary | the app's `build.rs`: `tauri_plugin_compmd::build::bundle` |
| serves it on a scheme, and gives the page the origin | the app's `main`: `tauri_plugin_compmd::init(include_bundle!())` |
| renders documents | the page: `createSandbox({ ...sandbox })`, as in a web app |
| processes documents | the page (`@compmd/processor`), or Rust (`compmd`, with the plugin's `packs()`) |
| checks the host | the page: `runCheck`, in the app's own webview |

## The page's build

The page's Vite configuration writes the bundle into the Rust project, for the scheme the plugin serves it on:

```ts snippet=../../../examples/tauri/vite.config.ts
// The page's build, for Tauri: its dev server on the port `tauri.conf.json` names, a build into `dist/`, and the
// bundle (the frames, the installed Mermaid pack, the manifest) into `src-tauri/compmd/`, built for the scheme the
// plugin serves it on, which `src-tauri/build.rs` embeds in the app (docs/tauri-plugin.md, decision 3).
import { compmd } from "@compmd/vite";
import { defineConfig } from "vite";

export default defineConfig({
  clearScreen: false,
  server: { port: 1420, strictPort: true },
  build: { target: "es2024", outDir: "dist", emptyOutDir: true },
  plugins: [compmd({ out: "src-tauri/compmd", source: "compmd://localhost/" })],
});
```

- **`out`** is where the bundle goes, in place of the web output, with its `manifest.json`. The dev server then serves no bundle, since the plugin does, and writes it when it starts. `src-tauri/compmd/` is build output: list it in `.gitignore`.
- **`source`** is the scheme's form, `compmd://localhost/`. WebKit (macOS, Linux) reads `'self'` in a sandboxed frame as nothing on a scheme other than http and https, so the frames' policies name the scheme instead, and the manifest records it. The plugin serves the scheme the bundle was built for.

Every installed pack is in the bundle, strict. A `compmd.config.json` beside `package.json` lists packs that get more, as a web app's does ([chapter 21](21-the-configurations-reference.md)).

## The app's side

Two dependencies, the same crate with different features:

```toml
# src-tauri/Cargo.toml
[dependencies]
tauri-plugin-compmd = { version = "0.7.0", features = ["processor"] }

[build-dependencies]
tauri-plugin-compmd = { version = "0.7.0", features = ["build"] }
```

The `build` feature embeds the bundle; `processor` is for an app that processes documents in Rust (below). The plugin needs Tauri 2.11.1 or later, which has the fix for [GHSA-7gmj-67g7-phm9](https://github.com/tauri-apps/tauri/security/advisories/GHSA-7gmj-67g7-phm9).

The app's `build.rs` embeds the bundle, before Tauri's own build step:

```rust snippet=../../../examples/tauri/src-tauri/build.rs
//! Tauri's build step: the bundle the page's build wrote into `compmd/`, embedded (docs/tauri-plugin.md, decision 3);
//! the app's context from `tauri.conf.json`; and its commands, each with an allow permission its capability grants.
//! An app must declare its commands: without them Tauri lets any page call every one, a page on the frames' scheme
//! too (docs/tauri-plugin.md, decision 8).

fn main() {
    tauri_plugin_compmd::build::bundle("compmd");
    let manifest = tauri_build::AppManifest::new().commands(&["mode", "document", "report"]);
    tauri_build::try_build(tauri_build::Attributes::new().app_manifest(manifest))
        .expect("the Tauri build runs");
}
```

`build::bundle("compmd")` reads the folder the page's build wrote, relative to the Rust project, checks its manifest, and writes the code that embeds it, so the binary carries the bundle and needs no files beside it. A build that finds no bundle warns, and the app serves none, which its page reports. Run the page's build before the app's: `vite build`, or the dev server, which `cargo tauri dev` starts first.

The app's `main` gives the embedded bundle to the plugin:

```rust excerpt=../../../examples/tauri/src-tauri/src/mode.rs
                tauri_plugin_compmd::init(tauri_plugin_compmd::include_bundle!())
```

`init` takes the bundle (`include_bundle!()` gives `None` when the build found none), and from it:

- serves the scheme its manifest names, `compmd`, with the serving contract ([chapter 18](18-other-hosts.md)): the frames of the bundle's hash and the packs' files, each with the contract's headers, and a `404` for anything else, with `compmd-host`'s `respond`;
- gives the app's page the origin, in an initialization script: `globalThis.__COMPMD__ = { origin }`, `compmd://localhost` on macOS, iOS and Linux, and `http://compmd.localhost` on Windows and Android, where Tauri serves a scheme in that form;
- keeps packs the app stores later in the app's data folder, served beside the bundle's.

## The page

The page names nothing about where the frames are. `virtual:compmd`'s `sandbox` has the bundle's frames and modules, and `createSandbox` takes the origin the plugin gave the page:

```ts snippet=../../../examples/tauri/src/main.ts
// The page: the document the app processed, rendered by `@compmd/sandbox` in host mode, with what the page's own build
// gives (`virtual:compmd`'s `sandbox`: the bundle's frames and modules) under the origin the plugin gives the page. With
// `--check` or `--check-broken`, it checks what it drew and tells the app (`check.ts`).

import { invoke } from "@tauri-apps/api/core";
import { createSandbox, type Sandbox, type View, type ViewEvents } from "@compmd/sandbox";
import { sandbox } from "virtual:compmd";
import { check, type Mode } from "./check.ts";

/** What the app's `document` command gives: the processed document, in the JSON the sandbox reads. */
interface Processed {
  tree: Parameters<View["render"]>[0];
  hostComponents: Parameters<Sandbox["open"]>[1]["components"];
  diagnostics: unknown[];
}

const mode = await invoke<Mode>("mode");
const processed = await invoke<Processed>("document");
const container = document.querySelector<HTMLElement>("#document");
const hosting = document.querySelector<HTMLElement>("#hosting");
if (!container || !hosting) throw new Error("the page has no #document or #hosting");

const view = createSandbox({ ...sandbox, page: "host" }).open(container, { components: processed.hostComponents });
const observation = view.observe(() => undefined);
const errors: ViewEvents["error"][] = [];
const sets: ViewEvents["set"][] = [];
view.on("error", (error) => {
  errors.push(error);
  console.error(`CompMD: ${error.component}: ${error.message}`);
});
view.on("set", (set) => sets.push(set));
view.render(processed.tree);

if (mode !== "show") {
  const drawn = { view, container, hosting, observation, errors, sets, diagnostics: processed.diagnostics, components: processed.hostComponents };
  const report = await check(mode, drawn);
  const status = document.querySelector("#status");
  if (status) status.textContent = report.message;
  await invoke("report", { ok: report.ok, message: report.message });
}
```

Host mode puts the page in the app's own page, with each component in a frame of its level, so the page looks and scrolls as the app's own. A page that processes its documents itself does it as in [chapter 4](04-processing-documents.md), with `virtual:compmd`'s `request`; this one gets the processed document from the app, through a command.

## Processing in Rust

An app whose documents live on the Rust side processes them there, with the `compmd` crate ([chapter 20](20-the-rust-crates.md)). The plugin's `processor` feature gives the processor the same packs the frames load, the bundle's and then the store's, so what the processor checks is what the frames run:

```rust excerpt=../../../examples/tauri/src-tauri/src/commands.rs
    match app.compmd().packs() {
        Some(packs) => crate::document::processed(now, packs),
        None => crate::document::processed(now, ()),
    }
```

```rust excerpt=../../../examples/tauri/src-tauri/src/document.rs
    let processor = Processor::new("document.comp.md", DOCUMENT)
        .parse(&ParseOptions::default())
        .resolve((), packs, &Ambient::default())
        .validate(&[])
        .load(context, (), ())
        .expand(&Options::default());
```

`app.compmd()` comes with `tauri_plugin_compmd::CompmdExt`, and `packs()` is `None` when the app has no bundle. The command gives the page the expanded tree, the host components and the diagnostics, as JSON: the same JSON `@compmd/processor` gives, which the sandbox reads.

## Keeping frames from the app's commands

A frame's code must never reach the app: a strict component sees its params and its box, nothing else. In Tauri, what keeps it from the app's commands is Tauri's **invoke key**, which Tauri gives the app's own page only. A capability doesn't do it, since Tauri counts a page on a scheme the app registers as one of the app's own. So:

- **Declare the app's commands** (`tauri_build::AppManifest`, in `build.rs` above) and grant them in the capability, to the app's window only. Without the declaration, Tauri lets any page call every command.

```json snippet=../../../examples/tauri/src-tauri/capabilities/main.json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "main",
  "description": "The main window's own pages: the app's commands. The plugin has none, and the frames' scheme is never listed: no frame reaches a command.",
  "windows": ["main"],
  "permissions": ["core:default", "allow-mode", "allow-document", "allow-report"]
}
```

- **WebKit's `ipc` handler.** On macOS and Linux, WebKit gives every frame Tauri's `ipc` script message handler, and no content security policy governs it. Tauri drops a message without the key, so a frame's call through it runs nothing; the example's check posts one with a guessed key to prove it. [The plugin's design](../../docs/tauri-plugin.md#appendix-webkits-ipc-handler-in-a-frame) has the research.

**If the app sets a content security policy** (`app.security.csp`; Tauri's template sets none), it adds what `cspFor` from `@compmd/vite/config` gives, for both forms of the origin: `frame-src` the origin, `script-src` and `connect-src` its `frames/` (a host-mode page loads Shiki and Temml from there), never its `packs/`, and `frame-ancestors 'none'`:

```json snippet=../../../examples/tauri/src-tauri/tauri.conf.json
{
  "$schema": "https://schema.tauri.app/config/2",
  "productName": "CompMD in Tauri",
  "version": "0.0.0",
  "identifier": "com.compmd.tauri-example",
  "build": {
    "devUrl": "http://localhost:1420",
    "frontendDist": "../dist",
    "beforeDevCommand": "pnpm dev",
    "beforeBuildCommand": "pnpm build"
  },
  "app": {
    "windows": [{ "label": "main", "title": "CompMD in Tauri", "width": 900, "height": 760 }],
    "security": {
      "csp": "default-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data: blob:; frame-src compmd://localhost/ http://compmd.localhost/; script-src 'self' compmd://localhost/frames/ http://compmd.localhost/frames/; connect-src 'self' ipc: http://ipc.localhost compmd://localhost/frames/ http://compmd.localhost/frames/; frame-ancestors 'none'; object-src 'none'; base-uri 'self'"
    }
  },
  "bundle": { "active": false }
}
```

## The app checks itself

A desktop app's webview is the browser that matters, so the app checks itself in it. With `--check`, the example's page checks what it drew (each component mounted in its frame, the code highlighted, the probe's frame opaque, with no Tauri in it), then runs the hosting check every host runs, with Tauri's globals as the host API a frame must not see:

```ts excerpt=../../../examples/tauri/src/check.ts
  const hosting = await runCheck({ container: drawn.hosting, frames: sandbox.frames, modules: sandbox.modules, hostApi: TAURI, webkitIpc: "guarded", packs });
```

`webkitIpc: "guarded"` says WebKit's handler is there and the host guards it, as above. The page sends the report to the app through a command, and the app prints it and exits `0` when it passed and `1` when it didn't, so CI runs the app as a test: `cargo make tauri-check` in the repository runs it twice, with `--check` and with `--check-broken`, which serves a bundle whose frames' runtime is missing and expects the sandbox to say so. On Linux, run it under a virtual display (`xvfb-run`).

## Run it

From the repository's root, after `pnpm install`:

```sh
pnpm --filter @compmd/tauri-example... build    # the page, and the bundle it writes into src-tauri/compmd/
cd examples/tauri/src-tauri && cargo tauri dev
```

The window shows the document: the heading, the diagram, the highlighted code, and the probe, which says it was drawn in a strict frame of the origin `null`, without Tauri. `cargo tauri dev` works from a checkout with no bundle too: Tauri runs `cargo` only once Vite's dev server answers, and the dev server writes the bundle as it starts. An app of your own uses the published `@compmd/vite`, `@compmd/sandbox` and `tauri-plugin-compmd`, all of the same version, as [the example's README](../../examples/tauri/README.md) says.

Next: [the Rust crates](20-the-rust-crates.md), processing documents outside a browser.
