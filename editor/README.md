# The CompMD Playground

A text area on the left, the rendered page on the right, updated as you type, with tabs on what the processor and the sandbox make of it. It opens the documents in its menu, from a tutorial to the spec's complete example, with everything they need bundled: their files, the packs of its configuration, remote and host data. Once loaded it makes no requests, but for a pack a document imports that its configuration doesn't have, which it fetches from npm (below, *Packs*).

**Website:** it is the playground at [compmd.hakanserce.com/playground/](https://compmd.hakanserce.com/playground/), on the [CompMD website](https://compmd.hakanserce.com/) with the spec.

**Status: in development,** as the processor is; not published yet.

## How it works

The playground is a page of the website that embeds the [Developer Tools](../devtools/README.md) (`@compmd/devtools`): the text, the page, the hint line, the diagnostics, the inputs and the tabs are theirs, and their README says how each works. The playground keeps what is the site's (`src/App.svelte`):

- **The menu of documents** (`src/documents.ts`), each opened in the tools with the files it reads, which show as tabs over the text. `?doc=` opens one, and `?select=` a passage of it: the tools' "Copy link to selection" makes such a link.
- **The host's stylesheet:** a page's look is its host's, not the document's, so a select in the header, beside the document's, tries eight: the reference look, GitHub-like, Documentation (a serif reading column), Fancy (an aurora behind frosted glass, headings in a moving sheen), Terminal (green phosphor, scanlines and a blinking cursor, dark in either scheme), Magazine (display headlines, a drop cap, pull quotes), Hand-drawn (a notebook page in marker and highlighter, a chalkboard in the dark) and Unstyled (the browser's defaults). Each sets the renderer's tokens, so boxes follow it, and the charts' ink and surface; their series keep their colors, checked for color vision deficiencies. Moving parts stay still for a reader who asks for less motion. They are rules in one file, `public/frame.css`, which the page's frame links as `frame.css?host=<id>`, each stylesheet's rules under its own link (`src/styles.ts`): a switch changes the link, reloads no page, and components keep their state.
- **The sandbox:** the page renders in [`@compmd/sandbox`](../sandbox/README.md)'s strict frame (frame mode), one per document, with no network (spec §15): the sandbox's frame documents, which [`@compmd/vite`](../vite/README.md) builds and serves under `compmd/`, carry its content security policy, so the playground works on any static host and writes no policy itself. **Math** is typeset as MathML by [Temml](https://temml.org), loaded the first time a page has math, in the sandbox's frame runtime.
- **About:** two short sentences on the first visit say what the playground is for and that the CompMD Inspector tab shows what happens behind the scenes; closed, it comes back from the "i" beside the name. The header has the [CompMD logo](../logo/README.md) beside the name, and the browser's tab its icon, light or dark as the reader's system is; it also links to the website, the spec and the repository.
- **The look** (`src/editor.css`): the desk around the tools, in the tools' paper and ink, light and dark, in the system's fonts.

### Host-language components

The expanded tree names a component but holds no code, so the tools' worker gives each component's code from the processor (`host_components`): a `jsx`, `tsx`, `svelte`, `js` or `ts` fence, an imported file, a pack's file. The tools hand it to the sandbox, which compiles it and mounts it through the spec's bindings:

- **Compiling:** in the sandbox's compile frame, which runs no components: sucrase strips TypeScript's types and turns JSX into calls of React's JSX runtime; Svelte's compiler compiles Svelte, its CSS apart, which the page's frame adopts as a stylesheet. Each compiler loads the first time a component needs it.
- **Linking:** a component imports `react`, `svelte` or `d3` as its author writes it, and the frame's import map resolves it; a frame loads a library only when a component imports it. The sandbox ships React, React DOM and Svelte; the configuration's `libraries` adds d3, for the examples' charts.
- **Built modules:** a pack's component that is one built module (spec §6.4) isn't compiled: `@compmd/vite` serves its file under `compmd/packs/`, and the frame loads it by its URL, checked against its hash.
- **Changing code:** each name's mount goes through the frame's own, so when a component's code changes, its instances on the page are mounted again with the new code and their last params and slots, and the rest of the page keeps its state. Code that doesn't compile or throws shows the component's fallback, and is listed under the text and in the Console.
- **Runtimes:** React 19 for JSX and TSX, Svelte 5. A component for `preact@10` shows its fallback.

### The documents

Each document in the menu shows one side of CompMD. The files it reads (its folder's other files, its packs' files, its remote and host data) are tabs over the text, each shown read-only in its place. Every document renders without diagnostics but the one about them, whose mistakes each have a fix, and fixing them all leaves none; `src/documents.test.ts` checks both, and `src/documents.processing.test.ts` that each one's page after the tools' splices is the page a new process gives.

| `?doc=` | Document | From |
|---|---|---|
| `tour` (the default) | A tutorial in fifteen lessons, each with a change to try: the document, Markdown and math, containers, params, values, expressions, `if` and `each`, definitions (with a d3 map of the tutorial's ideas as a teaser), template slots, data, inputs, layout, an animated JSX component and a d3 bookshelf whose books open on a click, and code renderers, with Mermaid's simplest flowcharts drawn by hand | `examples/tour/`, with a reading list in CSV |
| `definitions` | Definitions and slots: a `compmd` component with params and named slots, used as a container and a leaf, one definition in another, `display=contents`, a template slot | `examples/definitions/` |
| `changelog` | Expressions: a changelog of 12 releases and 128 changes | `../wasm/bench/fixtures/changelog/` |
| `dashboard` | Data: a sales dashboard, with tiles, a column chart bound to an input, a timeline, an area chart, a donut, a sortable table | `examples/dashboard/`, with the charts of `examples/charts/` |
| `inputs` | Inputs: planning a stay, with an input of every type and its component | `examples/inputs/` |
| `kyoto` | Layout: *Kyoto in four days*, with blocks placed by position, grid areas that change at `md` and `lg`, a sticky column, a `display=contents` component, blocks shown and hidden by width | `examples/kyoto/` |
| `complete` | Components in JSX and from packs: the spec's complete example | `../conformance/examples/complete-example-expanded/`, with its packs, remote and host data |
| `diagrams` | Packs: diagrams from a built module, the pack `@preview/minimermaid`'s code renderer, as blocks and as a leaf made from data | `examples/diagrams/`, with the pack of `packs/minimermaid/` |
| `npm` | Packs from npm: the published `@comp-markdown/mermaid`, which the playground fetches from npm, as a flowchart, a sequence diagram and a diagram made from data, and what a reader without it sees | `examples/npm/`; its pack from npm |
| `diagnostics` | Diagnostics and their fixes: an unknown component, param and slot, and values a param doesn't take | `examples/diagnostics/` |
| `notes` | A long document: field notes, a 52 KB body with 20 charts | `../wasm/bench/fixtures/notes/`, with the charts of `examples/charts/` |
| `guides/<book>/<example>` | Each document of the guides' examples, which their **Try it** links open: `guides/developers/quick-start` | `../guides/<book>/examples/` |

### Packs

The playground's packs come from its configuration, [`compmd.config.json`](compmd.config.json), which `@compmd/vite` reads at the site's build, as a web app's would: every document gets them, through `virtual:compmd` (`src/documents.ts`), and the site serves their files.

**Any other pack comes from npm**, which the configuration names as its registry, with `"fetch": "direct"`: the playground has no server, so the reader's browser fetches it ([packs from a registry](../docs/packs-from-a-registry.md), R1), and npm then learns the reader's address and the packs' names and versions, never the document. The playground gives the tools the configuration's registries, and the tools fetch the packs a document lacks once typing pauses, then open it again with them ([how](../devtools/README.md#how-it-works)). What couldn't be fetched is listed under the text and in the Console.

- `examples/charts/` is an ambient pack, `@preview/charts`, under the name the field notes give theirs: React components for columns, areas and a timeline, Svelte components for a donut and a sortable table, in plain SVG, compiled in the sandbox. Their colors are the `--viz-*` custom properties the frame sets for light and dark, a categorical palette checked for color vision deficiencies in both.
- `packs/minimermaid/` is `@preview/minimermaid`, made with [`npm create @compmd/pack`](../create-pack/README.md): Mermaid's simplest flowcharts drawn by hand, as one built module with no library in it, which `pnpm packs` builds before the site.

## Commands

`@compmd/processor`, `@compmd/render`, `@compmd/sandbox`, `@compmd/devtools` and `@compmd/vite` must be built first (`pnpm -r build` from the root does them all).

```sh
pnpm dev     # the minimermaid pack's module, then a dev server with the sandbox under compmd/
pnpm build   # the minimermaid pack's module, then the static site into dist/, the sandbox, the tools' workers and the packs' modules in it
pnpm lint    # eslint, tsc, and svelte-check for the .svelte files
pnpm test    # the documents, every document's page after the tools' splices against processing from scratch, and the document of a pack from npm in Chrome, light and dark, against a local registry (the Mermaid pack built first)
pnpm bench   # the budgets, in Chrome
```

The frame has an opaque origin, so it loads the site's scripts cross-origin: a server for the built site sends `Access-Control-Allow-Origin: *` (the dev server and `vite preview` do). The tools' workers are files of their own, which Vite builds only in a package it doesn't pre-bundle: `vite.config.ts` lists `@compmd/devtools` in `optimizeDeps.exclude`.

## Budgets

`pnpm bench` types into Chrome (the installed one, in a window of its own; `BENCH_HEADLESS=1` for headless) with real key presses at each edit point, a character then Backspace, in turn, 20 to warm up, then 200, one every 80 ms. It times each change from its first key press to the frame painted, with the worker's processing and the frame's part apart, and the text area's input latency by Event Timing. It fails when a budget is exceeded or a document shows a diagnostic or a component problem.

Typing in a paragraph processes that block. Typing in a definition processes the whole document again, and in a host-language one the frame compiles and mounts it again too.

| Edit | Key press to paint, p95 | Processing | The frame | Budget |
|---|---|---|---|---|
| The spec's complete example, in a paragraph | 6.3 ms | 1.9 ms | 1.8 ms | ≤ 16 ms |
| The same, in the sparkline's JSX | 15.2 ms | 4.5 ms | 9.2 ms | ≤ 16 ms |
| The changelog, in its heading | 5.5 ms | 1.8 ms | 2.3 ms | ≤ 50 ms |
| The same, in a definition it uses 384 times | 19.6 ms | 10.5 ms | 8.1 ms | ≤ 50 ms |
| Field notes (52 KB), in a paragraph | 12.2 ms | 1.1 ms | 2.5 ms | ≤ 50 ms |

The text area's input latency is 32 ms (p95) at every edit point, against a budget of 50 ms. Measured on 2026-11-02 on an Apple M4, Chrome 156, in a window, with the page in `@compmd/sandbox`'s frame from `@compmd/vite`'s `compmd/`, the hint at the caret in each answer, and the tools' observer on, as the playground always runs it. Against the run of 2026-10-30, before the observer, field notes went from 7.7 ms to 12.2 ms and the changelog's definition from 16.2 ms to 19.6 ms; the others are within a millisecond. The caret's moves go to the session as they come: waiting for an animation frame to send them, as a first version did, put every paint a frame later.

**Open:**
- Where the field notes' 4.5 ms went isn't measured yet: the observer copies each message on the page frame's port, a splice's text kept by its size, and the frame reports what it mounted every 100 ms.
- The complete example's paragraph edit took 5.8 ms (p95) on 2026-10-29, then 13.6 ms and 15.8 ms in two runs after the page moved to `@compmd/sandbox`, and 5.1 ms and 5.4 ms in two runs on 2026-10-30, with nothing in between that explains either change.
- The input latency was under 16 ms (p95) at every edit point on 2026-10-29, and 32 ms and 40 ms in the two runs on 2026-10-30. Event Timing rounds durations to 8 ms, so these are a few steps of it; the cause isn't measured yet.

**Where the time goes.** In a paragraph, the worker processes the block and gives its splice in about 1 ms, and the renderer's own work in the frame, parsing the splice and rendering the block, is 0.2 ms (p50) in the 4 KB example and in the 52 KB body alike. The rest of the frame's part is the browser's: laying out and painting the frame's page, about 2 ms for the 52 KB body. Between them, the edit and the splice cross from the page to the worker, back, and to the frame. That trip grows with the text area's text: the worker's answer reaches the page about 2 ms after the worker is done in the 52 KB body, against 0.4 ms in the 4 KB example, and reading the text area and finding the edit take 0.7 ms against 0.3 ms. An edit to a definition gives a splice of the whole tree, since the whole document is processed again.

## License

[MIT](../LICENSE-CODE). Copyright © 2026 Hakan Serce.
