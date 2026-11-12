# CompMD in a Tauri app

The smallest [Tauri](https://tauri.app) app that shows a CompMD document, with [`tauri-plugin-compmd`](../../tauri/README.md): a Mermaid diagram drawn by the `@comp-markdown/mermaid` pack, a code block highlighted by the frames' Shiki, and a JSX component the document defines, each component in a strict frame of its own. It uses CompMD exactly as an app outside this repository does ([the plugin's *Use*](../../docs/tauri-plugin.md#use)), so it is a guide for a host, and CompMD's test in a real webview: CI runs its check before every release ([the design](../../docs/tauri-plugin.md), decisions 9 and 10).

## Run it

From the repository's root, after `pnpm install`:

```sh
pnpm --filter @compmd/tauri-example... build    # the page, and the bundle it writes into src-tauri/compmd/
cd examples/tauri/src-tauri && cargo tauri dev
```

`cargo tauri dev` works from a checkout with no bundle too: Tauri runs `cargo` only once Vite's dev server answers, and the dev server writes the bundle as it starts. `cargo make tauri-check` runs everything the check needs, and the app twice, with `--check` and `--check-broken`; each exits `0` only when it drew what it should. On Linux, run it under `xvfb-run`.

## What a host does, and where it is here

- **The bundle, from the page's own build** (`vite.config.ts`): `compmd({ out: "src-tauri/compmd", source: "compmd://localhost/" })` writes the frames, built from the `@compmd/sandbox` the page imports, the installed Mermaid pack, and `manifest.json`, for the scheme the plugin serves them on. `src-tauri/compmd/` is build output, in `.gitignore`. Every installed pack is there, strict; a `compmd.config.json` beside `package.json` would give one more, as a web app's does.
- **The plugin, in two calls:** `tauri_plugin_compmd::build::bundle("compmd")` in `src-tauri/build.rs` embeds the bundle in the binary, and `tauri_plugin_compmd::init(tauri_plugin_compmd::include_bundle!())` (`src-tauri/src/mode.rs`) serves it on the scheme the manifest names, `compmd`, and gives the page the origin.
- **The page names nothing:** `createSandbox({ ...sandbox, page: "host" })` (`src/main.ts`), with `sandbox` from `virtual:compmd`, takes the bundle's frames and modules, and the origin the plugin gave the page.
- **The document, processed in Rust** with the `compmd` crate (`src-tauri/src/document.rs`) over `app.compmd().packs()` (the plugin's `processor` feature): the bundle's packs, then the store's, the same files the frames load. Its tree, its host components and its diagnostics go to the page through a command, and the Mermaid pack's built module loads in its frame by its URL.
- **The page's content security policy** (`src-tauri/tauri.conf.json`): what `cspFor` from `@compmd/vite/config` gives, for both forms of the origin (`compmd://localhost`, and `http://compmd.localhost` on Windows and Android): `frame-src` the origin, `script-src` and `connect-src` its `frames/`, never its `packs/`, and `frame-ancestors 'none'`. An app without a policy needs none of it.
- **The app's commands, declared** (`src-tauri/build.rs`) and granted in its capability (`src-tauri/capabilities/main.json`). Tauri counts the frames' scheme as one of the app's own pages, so what keeps a frame's code from the commands is Tauri's invoke key, which only the app's page has. On WebKit a frame does have Tauri's `ipc` message handler: WebKit gives it to every frame, and Tauri drops a message without the key ([the research](../../docs/tauri-plugin.md#appendix-webkits-ipc-handler-in-a-frame)). Tauri 2.11.1 or later is needed.

## What the check looks at

The strict frames are another origin, which the page can't look into, so the check reads the sandbox's observer and what the component reports:

- the `mermaid` and `probe` components mounted, each in a frame;
- the Rust block is highlighted;
- the probe's frame has an opaque origin, and no Tauri in it;
- where WebKit gives the frame Tauri's `ipc` handler, the probe posts a call through it, without the key, to the app's `report`, with a message saying so; if it ever ran, the app would end with that message, and the check fails;
- no `error`, no fallback, no diagnostic;
- then the hosting check every host runs (`@compmd/sandbox/check`), with Tauri's globals as the host API: the frames from the origin, opaque, with no network and no host API; a highlighted code block; the Mermaid pack's module by its URL; the contract's headers on the frames' files, and a `404` outside it.

`--check-broken` serves a copy of the bundle without the frames' runtime (`scripts/broken.mts`, read with `Bundle::read`), and expects the sandbox's error naming the frame document.
