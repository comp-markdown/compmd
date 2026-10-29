# The CompMD editor

A text area on the left, the rendered page on the right, updated as you type. It opens the spec's complete example and the other documents in its menu, with everything they need bundled: their files, packs, remote and host data. Once loaded it makes no requests.

**Status: in development,** as the processor is; not published yet.

## How it works

- **Processing runs in a worker,** never in the text area's input handler, so each key press is painted first and the preview follows it. The worker keeps the document open with [`@compmd/processor`](../wasm/README.md)'s `OpenDocument`, so an edit inside a block processes that block, not the document.
- **Each input is an edit.** The selection before the input is the range it replaced, so a cut or a paste is placed where the user made it. Backspace, Delete and undo change text outside the selection, so the range then also covers what changed (`src/edit.ts`). Edits made while the worker is busy go to it together, in order (`src/session.ts`).
- **The page renders in a sandboxed iframe** (`sandbox="allow-scripts"`, an opaque origin), through [`@compmd/render`](../render/README.md)'s frame protocol. Each change goes with its text, its diagnostics and the edit's range, which the renderer's keys follow, so components keep their state through edits around them. Edits merged into one change have no range, and the keys follow a diff.
- **Inputs:** an input component's `set` (a select, a chart's column) opens the document again with the new value.

### Host-language components

The expanded tree names a component but holds no code, so the worker sends the frame each component's code from the processor (`host_components`): a `jsx`, `tsx`, `svelte`, `js` or `ts` fence, an imported file, a pack's file. The frame compiles it and mounts it through the spec's bindings (`src/frame/`):

- **Compiling:** sucrase strips TypeScript's types and turns JSX into calls of React's JSX runtime; Svelte's compiler compiles Svelte, its CSS apart, which the frame adopts as a stylesheet. Each compiler loads the first time a component needs it.
- **Linking:** a component imports `react` or `svelte` as its author writes it, and the page's import map resolves it. `pnpm libs` builds the shared libraries into `public/libs/`, one build per npm package, its import names as entries, so modules that share state share one copy; `react-dom` imports the one React through the map. The editor's own code imports them through the map too, so each page has one React and one Svelte runtime, and the Svelte runtime is the compiler's version.
- **Changing code:** each name's mount goes through the frame's own, so when a component's code changes, its instances on the page are mounted again with the new code and their last params and slots, and the rest of the page keeps its state. Code that doesn't compile or throws shows the component's fallback and is listed under the text.
- **Runtimes:** React 19 for JSX and TSX, Svelte 5. A component for `preact@10` shows its fallback.

### The documents

| Document | From |
|---|---|
| The spec's complete example (the default) | `../conformance/examples/complete-example-expanded/`, with its packs, remote and host data |
| A sales dashboard: tiles, a column chart bound to an input, a timeline, an area chart, a donut, a sortable table | `examples/dashboard/`, with the charts of `examples/charts/` |
| A layout: *Kyoto in four days*, with blocks placed by position, grid areas that change at `md` and `lg`, a sticky column, a `display=contents` component, blocks shown and hidden by width | `examples/kyoto/` |
| A changelog of 12 releases and 128 changes | `../wasm/bench/fixtures/changelog/` |
| The release notes of spec §16, which import a pack and a file their folder doesn't hold, so they show their fallbacks | `../conformance/examples/spec-16-release-notes/` |
| Field notes, a 52 KB body with 20 charts | `../wasm/bench/fixtures/notes/`, with the charts of `examples/charts/` |

`examples/charts/` is an ambient pack, `@preview/charts`, under the name the field notes give theirs: React components for columns, areas and a timeline, Svelte components for a donut and a sortable table, in plain SVG. Their colors are the `--viz-*` custom properties the frame sets for light and dark, a categorical palette checked for color vision deficiencies in both. `?doc=<n>` opens the menu's *n*th document.

## Commands

`@compmd/processor` and `@compmd/render` must be built first (`pnpm -r build` from the root does all three).

```sh
pnpm dev     # the shared libraries, then a dev server
pnpm build   # the shared libraries, then the static site into dist/
pnpm lint    # eslint and tsc
pnpm test    # edits, the session and the editor's loop, the worker against processing from scratch, compiling and swapping components, every document
pnpm bench   # the budgets, in Chrome
```

The frame has an opaque origin, so it loads the site's scripts cross-origin: a server for the built site sends `Access-Control-Allow-Origin: *` (the dev server and `vite preview` do).

## Budgets

`pnpm bench` types into Chrome (the installed one, in a window of its own; `BENCH_HEADLESS=1` for headless) with real key presses: into a paragraph in the middle of the document, a character then Backspace, in turn, 20 to warm up, then 200, one every 80 ms. It times each change from its first key press to the frame painted, and the text area's input latency by Event Timing, and fails when a budget is exceeded or a document shows a diagnostic or a component problem.

| Document | Key press to paint, p95 | Budget | Input latency, p95 | Budget |
|---|---|---|---|---|
| The spec's complete example | 12.5 ms | ≤ 16 ms | 32 ms | ≤ 50 ms |
| The changelog | 20.0 ms | ≤ 50 ms | 32 ms | ≤ 50 ms |
| Field notes (52 KB) | 16.2 ms | ≤ 50 ms | 32 ms | ≤ 50 ms |

Measured on 2026-10-29 on an Apple M4, Chrome 156.

## License

[MIT](../LICENSE-CODE). Copyright © 2026 Hakan Serce.
