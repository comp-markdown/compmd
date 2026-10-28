# @compmd/processor

The [CompMD processor](../processor/README.md) as WebAssembly, for the browser: one call takes a document with everything it needs and gives its parse tree, its expanded tree and its diagnostics, typed. The expanded tree is what [`@compmd/render`](../render/README.md) draws.

**Status: in development,** as the crate is; not published yet.

```ts
import { init, process, sources } from "@compmd/processor";

await init(); // loads the module, once

const request = {
  path: "notes.comp.md",
  files: { "notes.comp.md": text, "team.yaml": team },
  now: new Date().toISOString(),
  tz: Intl.DateTimeFormat().resolvedOptions().timeZone,
  origins: true,
};
const { remote, host } = sources(request); // the URLs and host: sources to fetch first, if any
const { parse, expanded, diagnostics } = process(request);
```

A request holds the files of the document's root folder by path, the registry's packs with their files, the ambient packs, the origins remote data may come from, remote data fetched beforehand by URL (`{text, contentType}`), host data by source, the render time (RFC 3339), the time zone, the inputs' values, and whether expanded nodes carry origins. `process` throws when the request isn't valid or its document isn't one of its files; everything wrong with the document itself is a diagnostic.

Time zone offsets come from the page's `Intl`, so the module carries no zone data. Formatting has all of ICU4X's locales.

## How it is built

- The Rust crate `compmd-wasm` in this folder is the binding: `process` and `sources` take and give JSON text, through `wasm-bindgen`. Its request (`src/request.rs`) maps to the processor's stages and is tested natively, within the workspace's coverage gate.
- It is built size-optimized: the workspace's `wasm` profile, with `opt-level = "z"`, LTO and one codegen unit.
- `wasm-bindgen --target web` writes the JavaScript glue to `pkg/`.
- `src/index.ts` parses both ways and gives the types.
  - The expanded tree's types are `@compmd/render`'s.
  - The parse tree's types are in `src/parse-tree.ts`, following [the tree schema](../conformance/compmd-tree.schema.json).

## Commands

These need the Rust toolchain of the workspace (with the `wasm32-unknown-unknown` target) and the `wasm-bindgen` CLI at the version `Cargo.toml` pins (`cargo install wasm-bindgen-cli --version 0.2.129`). `@compmd/render` must be built first (`pnpm -r build` from the root does both).

```sh
pnpm lint    # eslint and tsc
pnpm build   # the module, then dist/
pnpm test    # the binding: the spec's complete example as its conformance case expects, errors, Intl's zones
pnpm bench   # size and speed against the budgets
```

## Budgets

`pnpm bench` measures the module's size and times `process` in a worker in Chrome (the installed one) and WebKit (Playwright's; `pnpm exec playwright-core install webkit` once). Each run is a keystroke: the text changes, and the whole request is processed again, with origins on and JSON in and out included. It fails when a budget is exceeded.

The module is 720 KB gzipped, against a budget of 1 MB. Times, p95:

| Document | Budget | Chrome 156 | WebKit (Safari 27.2) |
|---|---|---|---|
| The spec's complete example (4 KB) | ≤ 14 ms | 2.7 ms | 2.6 ms |
| A changelog of 12 releases and 128 changes | ≤ 48 ms | 4.0 ms | 3.5 ms |
| A 52 KB body | ≤ 48 ms | 9.8 ms | 8.3 ms |

The time budgets are one frame (16 ms) from key press to paint for a short document and 50 ms for a long one, less the renderer's 2 ms. Measured on 2026-10-28 on an Apple M4; loading the module takes 9 to 10 ms.

## License

[MIT](../LICENSE-CODE). Copyright © 2026 Hakan Serce.
