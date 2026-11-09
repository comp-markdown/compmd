# CompMD in a Tauri app

The smallest [Tauri](https://tauri.app) app that shows a CompMD document, with [`tauri-plugin-compmd`](../../tauri/README.md): a Mermaid diagram drawn by the `@comp-markdown/mermaid` pack, a code block highlighted by the frames' Shiki, and a JSX component the document defines, each component in a strict frame of its own. It is a guide for a host, and CompMD's test in a real webview: CI runs its check before every release ([the design](../../docs/tauri-plugin.md), decisions 9 and 10).

## Run it

From the repository's root, after `pnpm install`:

```sh
cargo make tauri-frames                 # the frames the plugin embeds
pnpm --filter @compmd/tauri-example... build
cd examples/tauri/src-tauri && cargo tauri dev
```

`cargo make tauri-check` runs everything the check needs, and the app twice, with `--check` and `--check-broken`; each exits `0` only when it drew what it should. On Linux, run it under `xvfb-run`.

## What a host does, and where it is here

- **The plugin, in one call:** `tauri_plugin_compmd::init()` (`src-tauri/src/mode.rs`). The frames come with it, on the scheme `compmd`.
- **The page names no `base`:** `createSandbox({ page: "host" })` (`src/main.ts`) finds the frames' `base` the plugin gave the page.
- **The document, processed in Rust** with the `compmd` crate (`src-tauri/src/document.rs`): its tree, its host components and its diagnostics, as JSON, through a command. A pack the document imports comes from the app (`Packs`), here the Mermaid pack of this repository; its built module goes to the sandbox as code (`modules`).
- **The page's content security policy** (`src-tauri/tauri.conf.json`): `frame-src` the scheme, `script-src` and `connect-src` the frames' folder, `frame-ancestors 'none'`. An app without a policy needs none of it.
- **The app's commands, declared** (`src-tauri/build.rs`) and granted in its capability (`src-tauri/capabilities/main.json`): Tauri counts the frames' scheme as one of the app's own pages, so what keeps a frame's code from the commands is Tauri's invoke key, which only the app's page has. The tests show a call without it refused, and the check that the strict component's frame has no Tauri in it.
- **An app's own build of the frames** (`--check-broken`): `Builder::new().frames_dir(…)`, built by `buildHashedFrames` (`scripts/broken.mts`), with the page asking for `base` with `plugin:compmd|base`.

## What the check looks at

The strict frames are another origin, which the page can't look into, so the check reads the sandbox's observer and what the component reports:

- the `mermaid` and `probe` components mounted, each in a frame;
- the Rust block is highlighted;
- the probe's frame has an opaque origin, and no Tauri in it;
- no `error`, no fallback, no diagnostic.

`--check-broken` serves frames whose runtime is missing, and expects the sandbox's error naming the frame document.
