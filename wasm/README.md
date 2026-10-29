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

An editor keeps the document open instead, and sends each edit ([incremental processing](../processor/docs/incremental.md)):

```ts
import { init, OpenDocument } from "@compmd/processor";
import { createPage } from "@compmd/render";

await init();
const document = new OpenDocument({ ...request, files: { ...request.files, "notes.comp.md": textarea.value } });
const page = createPage(preview);
page.render(document.expanded);

// On each input: the range the selection had before it, and what was typed or pasted.
document.edit(start, end, inserted);
page.render(document.expanded, { text: textarea.value, edit: { start, end } });

// Only a host that fetches ahead: after an edit to the frontmatter, what it hasn't fetched yet.
const { remote, host } = document.sources();
document.reload({ remote: await fetchAll(remote), host: hostData(host) });
```

- **Positions are the editor's**, in UTF-16 units, as a JavaScript string counts them (`selectionStart`, CodeMirror's `from` and `to`). The module turns them into the code points the processor counts. Give it the editor's text (`textarea.value`, `state.doc.toString()`), so both count the same text.
- **Each change gives a splice**, `{splice, inputs, diagnostics}`. The top-level nodes `from..to` became `nodes`, and the nodes after them moved by `lines` and `offset`. `OpenDocument` applies it to the tree it keeps, moving those nodes' origins, so `expanded` is always the whole tree. Only its top-level array is new after a change; the nodes in it are updated in place.
- For an edit inside one block, neither the processing nor the splice grows with the document. Moving the origins after the edit does: it is a pass over those nodes.
- `free()` releases the module's memory for the document.

Time zone offsets come from the page's `Intl`, so the module carries no zone data. Formatting has all of ICU4X's locales.

## How it is built

- The Rust crate `compmd-wasm` in this folder is the binding: `process`, `sources` and the class `Document` take and give JSON text, through `wasm-bindgen`. The request (`src/request.rs`), the open document (`src/document.rs`) and the UTF-16 positions (`src/text.rs`) are tested natively, within the workspace's coverage gate; `src/lib.rs` only passes them through.
- The text kept with an open document has each line's start in bytes, UTF-16 units and code points, so a position is found in its line. An edit that removes a leading byte order mark, or writes text before it, changes where the processor starts counting, so it is sent as an edit of the whole text.
- It is built size-optimized: the workspace's `wasm` profile, with `opt-level = "z"`, LTO and one codegen unit.
- `wasm-bindgen --target web` writes the JavaScript glue to `pkg/`.
- `src/index.ts` parses both ways, gives the types, and keeps an open document's expanded tree (`OpenDocument`).
  - The expanded tree's types are `@compmd/render`'s.
  - The parse tree's types are in `src/parse-tree.ts`, following [the tree schema](../conformance/compmd-tree.schema.json).

## Commands

These need the Rust toolchain of the workspace (with the `wasm32-unknown-unknown` target) and the `wasm-bindgen` CLI at the version `Cargo.toml` pins (`cargo install wasm-bindgen-cli --version 0.2.129`). `@compmd/render` must be built first (`pnpm -r build` from the root does both).

```sh
pnpm lint    # eslint and tsc
pnpm build   # the module, then dist/
pnpm test    # the binding: the spec's complete example as its conformance case expects, errors, Intl's zones, edits of open documents against processing from scratch
pnpm bench   # size and speed against the budgets
```

## Budgets

`pnpm bench` measures the module's size and times it in a worker in Chrome (the installed one) and WebKit (Playwright's; `pnpm exec playwright-core install webkit` once), with origins on and JSON in and out included. It fails when a budget is exceeded.

- **`process`:** each run is a keystroke. The text changes, and the whole request is processed again.
- **A keystroke in an open document:** a character typed into a paragraph in the middle of the body, then deleted, in turn. It is timed in the module alone (the processing), and through `OpenDocument`, which also parses the splice and moves the origins after it. The 500 KB body is the 52 KB one ten times.

The module is 758 KB gzipped, against a budget of 1 MB. Times, p95:

| Document | Budget | Chrome 156 | WebKit (Safari 27.2) |
|---|---|---|---|
| `process`: the spec's complete example (4 KB) | ≤ 14 ms | 2.5 ms | 2.4 ms |
| `process`: a changelog of 12 releases and 128 changes | ≤ 48 ms | 3.9 ms | 3.3 ms |
| `process`: a 52 KB body | ≤ 48 ms | 10.0 ms | 8.4 ms |
| A keystroke in 52 KB, processing | ≤ 1 ms | 0.10 ms | 0.08 ms |
| A keystroke in 52 KB, with the splice applied | | 0.14 ms | 0.14 ms |
| A keystroke in 500 KB, processing | ≤ 1 ms | 0.29 ms | 0.28 ms |
| A keystroke in 500 KB, with the splice applied | | 0.84 ms | 0.78 ms |

The budgets for `process` are one frame (16 ms) from key press to paint for a short document and 50 ms for a long one, less the renderer's 2 ms. A keystroke's budget is the target of incremental processing. Measured on 2026-10-29 on an Apple M4; loading the module takes 6 to 9 ms.

## License

[MIT](../LICENSE-CODE). Copyright © 2026 Hakan Serce.
