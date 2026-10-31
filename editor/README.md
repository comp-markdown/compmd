# The CompMD editor

A text area on the left, the rendered page on the right, updated as you type. It opens the documents in its menu, from a tutorial to the spec's complete example, with everything they need bundled: their files, packs, remote and host data. Once loaded it makes no requests.

**Status: in development,** as the processor is; not published yet.

## How it works

- **Processing runs in a worker,** never in the text area's input handler, so each key press is painted first and the preview follows it. The worker keeps the document open with [`@compmd/processor`](../wasm/README.md)'s `OpenDocumentJson`, so an edit inside a block processes that block, not the document. Its nodes carry origins, which the renderer's keys come from.
- **Each input is an edit.** The selection before the input is the range it replaced, so a cut or a paste is placed where the user made it. Backspace, Delete and undo change text outside the selection, so the range then also covers what changed (`src/edit.ts`). Edits made while the worker is busy go to it together, in order (`src/session.ts`).
- **Splices, not trees, cross to the frame.** The worker gives a splice per edit, as the processor's JSON text, and the editor passes it on unread with the text and its diagnostics. Only the frame holds the expanded tree, as the renderer's blocks.
- **The page renders in [`@compmd/sandbox`](../sandbox/README.md)'s strict frame** (frame mode), one per document, with no network (spec §15): the sandbox's frame documents, which `pnpm frames` builds into `public/sandbox/`, carry its content security policy, so the playground works on any static host and writes no policy itself. The page renders through [`@compmd/render`](../render/README.md)'s frame protocol, inside the sandbox. The renderer applies each splice and renders only the blocks it replaced; its keys follow each splice's edit, so components keep their state through edits around them, merged edits included.
- **The look** (`src/editor.css`): the text and the page as two sheets on a desk, in cool paper and blue-black ink with one blue accent, light and dark, in the system's fonts, so it loads nothing to show itself. The handle between them drags, or moves with the arrow keys; Home and End fold a side away, Enter or a double click opens it, and the browser remembers where it was (`src/split.ts`). On a narrow screen the sheets stack.
- **Math** is typeset as MathML, which the browser lays out, through the renderer's math hook: [Temml](https://temml.org) turns the TeX into it, loaded the first time a page has math (`@compmd/sandbox`'s frame runtime). Until then, and for TeX it can't typeset, the math shows as its source.
- **The host's stylesheet:** a page's look is its host's, not the document's, so a select in the header, beside the document's, tries eight: the reference look, GitHub-like, Documentation (a serif reading column), Fancy (an aurora behind frosted glass, headings in a moving sheen), Terminal (green phosphor, scanlines and a blinking cursor, dark in either scheme), Magazine (display headlines, a drop cap, pull quotes), Hand-drawn (a notebook page in marker and highlighter, a chalkboard in the dark) and Unstyled (the browser's defaults). Each sets the renderer's tokens, so boxes follow it, and the charts' ink and surface; their series keep their colors, checked for color vision deficiencies. Moving parts stay still for a reader who asks for less motion. They are rules in one file, `public/frame.css`, which the page's frame links as `frame.css?host=<id>`, each stylesheet's rules under its own link: a switch changes the link, reloads no page, and components keep their state.
- **About:** a short paragraph on the first visit says what the editor, *CompMD Playground* on its page, is for; closed, it comes back from the "i" beside the name. The header has the [CompMD logo](../logo/README.md) beside the name, and the browser's tab its icon, light or dark as the reader's system is; it also links to the spec and the repository.
- **The hint line**, under the text and above the diagnostics, says what is at the caret and what it takes, from the processor's [positional hints](../processor/docs/positional-hints.md): a component's name and where it comes from, a param and what it takes, the params a list takes, a slot, a step of a path with its type, a function, a frontmatter key or a type name, with the diagnostic at the caret first and its fix. A param's value and a type name offer their values as buttons, which replace what is written, as a fix does. Each message to the worker carries the caret, so each answer brings the hint at it for the text it processed; a caret that moves without an edit asks for the hint alone, once the worker is free (`src/session.ts`, `src/hint.ts`). It is one line, cut short, all of it on hover.
- **Diagnostics** are listed under the text, each with its code, place and message; clicking its place selects its range in the text (to the end of the line when it has no end).
- **Views:** tabs beside the page (the CompMD tab) show what a plain CommonMark and GFM renderer shows of the text (spec §14: markdown-rs with CompMD's syntax off, through `@compmd/processor`'s `plainHtml`, in a frame without scripts); what the frontmatter gives the render (`frontmatterOf`): its own fields, each data import with its source, schema and what was loaded, the inputs with their values, and the settings; the components (`componentsOf`) in two lists, those the document defines, with their kinds, params and slots, each definition's line selecting it in the text, and those it uses, wherever they come from, unknown names among them, each use's line selecting it in the text, and the imported and built-in ones it doesn't use folded under them; and the parse tree and the expanded tree (spec §12), the expanded one without origins. The trees and the data are JSON that folds (`src/JsonTree.svelte`): two levels open, a folded record showing its first values. The editor's worker keeps no tree, so a second worker makes these from the current text while one of their tabs is open, the latest text only when it is busy (`src/views.ts`). The page stays mounted under them, with its state.
- **Inputs:** a bar under the host's stylesheet lists the document's inputs, each with a control by its type (a select for an enum, a checkbox, a number, a date, text). Changing one there, or through the page (an input component's `set`: a select, a chart's column), opens the document again with the new value.
- **Links to the text:** a link in the page to a text fragment (`[price: 12](#:~:text=price:%2012)`, spec §2.3) selects that passage in the text and scrolls to it, or says it isn't there any more; the sandbox passes the reader's click to the editor (`src/fragment.ts`). With text selected, "Copy link to selection" copies an address that opens the document with that passage selected (`?doc=tour&select=…`, the fragment's parts as written, since browsers hide `#:~:text=` from scripts). The tutorial's "Try it" steps link what they name. A passage scrolls to where the text area draws it, its wrapped lines counted (`src/caret.ts`). A link to another site opens in a new tab, which the editor opens for the frame: the sandboxed frame may not leave the playground, and most sites refuse to be shown in a frame. Only the reader's own clicks open one, since a component may not navigate (spec §15).
- **Fixes:** a diagnostic that suggests a name or a token (`did you mean `subtle`?`) has a button under the text that makes its fix, as an edit.

### Host-language components

The expanded tree names a component but holds no code, so the worker gives the editor each component's code from the processor (`host_components`): a `jsx`, `tsx`, `svelte`, `js` or `ts` fence, an imported file, a pack's file. The editor hands it to the sandbox, which compiles it and mounts it through the spec's bindings:

- **Compiling:** in the sandbox's compile frame, which runs no components: sucrase strips TypeScript's types and turns JSX into calls of React's JSX runtime; Svelte's compiler compiles Svelte, its CSS apart, which the page's frame adopts as a stylesheet. Each compiler loads the first time a component needs it.
- **Linking:** a component imports `react`, `svelte` or `d3` as its author writes it, and the frame's import map resolves it; a frame loads a library only when a component imports it. The sandbox ships React, React DOM and Svelte; `pnpm frames` adds d3, for the examples' charts (`scripts/frames.ts`).
- **Changing code:** each name's mount goes through the frame's own, so when a component's code changes, its instances on the page are mounted again with the new code and their last params and slots, and the rest of the page keeps its state. Code that doesn't compile or throws shows the component's fallback and is listed under the text.
- **Runtimes:** React 19 for JSX and TSX, Svelte 5. A component for `preact@10` shows its fallback.

### The documents

Each document in the menu shows one side of CompMD. The files it reads (its folder's other files, its packs' files, its remote and host data) are tabs over the text, each shown read-only in its place. Every document renders without diagnostics but the one about them, whose mistakes each have a fix, and fixing them all leaves none; `src/documents.test.ts` checks both.

| `?doc=` | Document | From |
|---|---|---|
| `tour` (the default) | A tutorial in fourteen lessons, each with a change to try: the document, Markdown and math, containers, params, values, expressions, `if` and `each`, definitions (with a d3 map of the tutorial's ideas as a teaser), template slots, data, inputs, layout, an animated JSX component, and a d3 bookshelf whose books open on a click | `examples/tour/`, with a reading list in CSV |
| `definitions` | Definitions and slots: a `compmd` component with params and named slots, used as a container and a leaf, one definition in another, `display=contents`, a template slot | `examples/definitions/` |
| `changelog` | Expressions: a changelog of 12 releases and 128 changes | `../wasm/bench/fixtures/changelog/` |
| `dashboard` | Data: a sales dashboard, with tiles, a column chart bound to an input, a timeline, an area chart, a donut, a sortable table | `examples/dashboard/`, with the charts of `examples/charts/` |
| `inputs` | Inputs: planning a stay, with an input of every type and its component | `examples/inputs/` |
| `kyoto` | Layout: *Kyoto in four days*, with blocks placed by position, grid areas that change at `md` and `lg`, a sticky column, a `display=contents` component, blocks shown and hidden by width | `examples/kyoto/` |
| `complete` | Components in JSX and from packs: the spec's complete example | `../conformance/examples/complete-example-expanded/`, with its packs, remote and host data |
| `diagnostics` | Diagnostics and their fixes: an unknown component, param and slot, and values a param doesn't take | `examples/diagnostics/` |
| `notes` | A long document: field notes, a 52 KB body with 20 charts | `../wasm/bench/fixtures/notes/`, with the charts of `examples/charts/` |

`examples/charts/` is an ambient pack, `@preview/charts`, under the name the field notes give theirs: React components for columns, areas and a timeline, Svelte components for a donut and a sortable table, in plain SVG. Their colors are the `--viz-*` custom properties the frame sets for light and dark, a categorical palette checked for color vision deficiencies in both.

## Commands

`@compmd/processor`, `@compmd/render` and `@compmd/sandbox` must be built first (`pnpm -r build` from the root does them all).

```sh
pnpm dev     # the sandbox's frame documents, then a dev server
pnpm build   # the sandbox's frame documents, then the static site into dist/
pnpm lint    # eslint, tsc, and svelte-check for the .svelte files
pnpm test    # edits, the session and the editor's loop, every document's page after the worker's splices against processing from scratch
pnpm bench   # the budgets, in Chrome
```

The frame has an opaque origin, so it loads the site's scripts cross-origin: a server for the built site sends `Access-Control-Allow-Origin: *` (the dev server and `vite preview` do).

## Budgets

`pnpm bench` types into Chrome (the installed one, in a window of its own; `BENCH_HEADLESS=1` for headless) with real key presses at each edit point, a character then Backspace, in turn, 20 to warm up, then 200, one every 80 ms. It times each change from its first key press to the frame painted, with the worker's processing and the frame's part apart, and the text area's input latency by Event Timing. It fails when a budget is exceeded or a document shows a diagnostic or a component problem.

Typing in a paragraph processes that block. Typing in a definition processes the whole document again, and in a host-language one the frame compiles and mounts it again too.

| Edit | Key press to paint, p95 | Processing | The frame | Budget |
|---|---|---|---|---|
| The spec's complete example, in a paragraph | 13.6 ms | 1.7 ms | 1.8 ms | ≤ 16 ms |
| The same, in the sparkline's JSX | 13.0 ms | 5.5 ms | 6.5 ms | ≤ 16 ms |
| The changelog, in its heading | 5.2 ms | 1.9 ms | 2.2 ms | ≤ 50 ms |
| The same, in a definition it uses 384 times | 21.3 ms | 11.0 ms | 8.8 ms | ≤ 50 ms |
| Field notes (52 KB), in a paragraph | 8.5 ms | 1.0 ms | 2.7 ms | ≤ 50 ms |

The text area's input latency is under 16 ms (p95) at every edit point but the complete example's paragraph, 16 ms, against a budget of 50 ms. Measured on 2026-10-30 on an Apple M4, Chrome 156, headless, with the page in `@compmd/sandbox`'s frame and the hint at the caret in each answer. The caret's moves go to the session as they come: waiting for an animation frame to send them, as a first version did, put every paint a frame later.

**Open:** the complete example's paragraph edit took 5.8 ms (p95) on 2026-10-29, before the page moved to `@compmd/sandbox`, and 13.6 ms and 15.8 ms in two runs since, while its processing and the frame's part are the same as before: the time it gained is between them, in the page's messages, and not yet explained.

**Where the time goes.** In a paragraph, the worker processes the block and gives its splice in about 1 ms, and the renderer's own work in the frame, parsing the splice and rendering the block, is 0.2 ms (p50) in the 4 KB example and in the 52 KB body alike. The rest of the frame's part is the browser's: laying out and painting the frame's page, about 2 ms for the 52 KB body. Between them, the edit and the splice cross from the page to the worker, back, and to the frame. That trip grows with the text area's text: the worker's answer reaches the page about 2 ms after the worker is done in the 52 KB body, against 0.4 ms in the 4 KB example, and reading the text area and finding the edit take 0.7 ms against 0.3 ms. An edit to a definition gives a splice of the whole tree, since the whole document is processed again.

## License

[MIT](../LICENSE-CODE). Copyright © 2026 Hakan Serce.
