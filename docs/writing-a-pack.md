# Writing a pack

**Status: proposed.** A pack author should write only what is theirs: the component, its signature, a document to try it in, and its tests. Today `npm create @compmd/pack` copies everything else into the pack too, about 150 lines: a Vite configuration, a publish check, a test page, a TypeScript configuration. Those copies are never updated in packs made earlier. A component that draws asynchronously also repeats about 100 lines of its own: the latest params winning, the reader's colour scheme, a failure shown late. This design moves the copies into a package packs depend on (A), the component routine into helpers (B), and the two things only the host can do well, the colour scheme and a late failure, into the host contract (C).

## Decisions

1. **A pack depends on `@compmd/pack`, and holds no copied code.** A pack has its `package.json`, its document (`index.comp.md`), its components (`src/`), their tests, `demo.comp.md`, its README and a one-line `tsconfig.json`. No `vite.config.ts`, no `scripts/`, no `demo/` folder.
2. **`compmd-pack` runs everything:** `build` (one module per component, then the publish check), `dev` (the test page), `test` (the tests in happy-dom). The pack's scripts call it. Upgrading `@compmd/pack` updates the build, the check and the test page of every pack.
3. **The test page is served, never written.** `compmd-pack dev` serves its HTML, script and configuration from memory, with the pack as a folder pack in host mode, and a light/dark switch. It renders `demo.comp.md`, the only part of it in the pack.
4. **The publish check is part of `build`:** `export.version` is `package.json`'s, every file the pack's documents import is there, the module is one file, and its size is under the registry's default cap. `prepublishOnly` runs `build`, so a pack can't publish without it.
5. **The host contract gains `context.scheme` and `context.fail`** (spec §7.4, C):
   - **`context.scheme`**, `"light"` or `"dark"`: the colour scheme the processor shows the component in. When it changes, the processor sets `context.scheme` and calls `update` with the same params. `@compmd/render` reads it from the container it renders in (its `color-scheme`, and the reader's preference where it allows both): at each render, when the preference changes, when printing starts and ends (S8), and when the host calls `page.scheme()` after switching its own theme (S2). In a frame, the container is the frame's document, which takes its page's scheme from the sandbox.
   - **`context.fail(message)`**, for a component that can't draw what it was given, found after `mount` or `update` returned (an asynchronous parse, a module loaded late). The processor reports W118 with `message`, and shows the fallback in `target`. Until the next `update`, `target` is the processor's. Before calling `update`, the processor empties `target` again, so a failure lasts until the params, the slots or the scheme change (S3). It is for input the component can't draw, not for a resource that may come back (S5).
   - **A throw in `update`** is the same as `fail`: the component is still mounted, so it is updated again at the next change (S4). A throw in `mount` stays as it is.
6. **Helpers for what remains** (`@compmd/pack/component`, B), bundled into the module (a few KB): `draw(render, options)` runs an asynchronous `render(params, { scheme, signal })` that returns an element or SVG/HTML text. It drops a draw that a newer one overtakes, optionally runs one draw at a time per frame (for libraries with a global configuration, as Mermaid has), sets the text alternative from `fallback.text`, empties `target` on `destroy`, and calls `context.fail` when `render` throws. A component that draws synchronously writes plain `mount`, as today.
7. **A testing kit** (`@compmd/pack/testing`): `mountForTest(component, { params, fallback, scheme })` runs a component through the contract in happy-dom, and returns `update`, `setScheme`, `settled()` (waits for its draws), what it `set` and what it `fail`ed. Tests need no stubs for `matchMedia` or the frame's document.

## Scenarios

Each tests C against something a real pack or host does. Each scenario gives its outcome, and what it changed in the decisions.

**S1. A Mermaid diagram on a page the reader switches to dark.** The page is `color-scheme: light dark`; the reader's system goes dark. The sandbox sees the preference change, sends the frames `scheme {dark}`, and the frame's renderer sets `context.scheme` and calls `update`. Mermaid redraws with its dark theme. The component reads `context.scheme`; it never reads `prefers-color-scheme`, which is the reader's, not the page's. *Holds.*

**S2. A page with its own theme switch.** A site toggles a class on `<html>` that sets `color-scheme: dark`. No media query changes. The renderer reads its container's scheme at every render, but a class switch isn't a render. *Changes decision 5:* the renderer also gets `page.scheme()`, which a host calls after it switches, and `@compmd/sandbox`'s view passes it on. A host that switches with a media query needs nothing.

**S3. A typo in a Mermaid block, in an editor.** The author types `A -->` and stops. The diagram's draw fails after `update` returned, so the component calls `fail("Parse error on line 3: …")`. The editor shows W118 with Mermaid's message on the block, and the page shows the block's code (its fallback). The author types `B`, an `update` follows, the processor empties `target`, and the diagram draws. Today, the same typo leaves the fallback in place for good, since a component that threw is never updated again; `fail` is what makes the fix show. *Holds; the reader sees the code, not Mermaid's message, which goes to the diagnostics.*

**S4. A KaTeX-like renderer that throws synchronously.** `katex.render` throws on `\frac{`. A throw in `mount` stays as the spec has it: W118, fallback, never mounted. A throw in `update` is the same as `fail`: the processor shows the fallback and updates the component again on the next change, rather than never. *Changes the spec's rule for a throw in `update`*: it recovers at the next update, as `fail` does, since the component is still mounted.

**S5. A map whose tiles stop loading.** A permissive map pack draws, then the network drops and tiles fail to load. That isn't a failure to draw the params: the map is drawn, with missing tiles. Calling `fail` would replace a usable map with its fallback until the next change, which may never come. *Holds, with guidance:* `fail` is for "can't draw what it was given", not for a resource that may come back; the map shows its own "offline" state and retries. The spec's text says so.

**S6. A React chart.** The React binding passes `compmd.scheme` and `compmd.fail` as props (`compmd` already carries `set` and `fallback`), and re-renders when `scheme` changes. The chart picks its palette from `compmd.scheme` in render. Svelte's binding does the same. *Holds.*

**S7. A trusted component inline in the host's page.** No frame is involved; the host page's renderer reads the scheme of the page's container, as the sandbox does for frames. The same component works inline and framed. *Holds; the scheme reading moves from `@compmd/sandbox` into `@compmd/render`, which both use.*

**S8. Printing.** A dark page printed on paper: browsers print the page as it is unless its print styles change it. A site whose print styles set `color-scheme: light` wants light diagrams. The renderer reads the scheme again on `beforeprint` and `afterprint`, so components redraw for paper and back. *Changes decision 5:* `@compmd/render` listens for printing.

**S9. Two diagrams in one frame (frame mode), one with a typo.** `fail` is per component: the other diagram stays drawn. One draw at a time per frame (decision 6, for Mermaid's global configuration) keeps the two draws from mixing themes. *Holds.*

**S10. A component that never draws asynchronously (stars, a counter).** It ignores `scheme`, or uses CSS for colours, and never calls `fail`. `update` on a scheme change costs it nothing it doesn't already do on any update. *Holds; nothing new to write.*

**S11. A host that doesn't render HTML (a PDF processor that renders fallbacks).** Components don't run, so neither field matters. *Holds.*

**Alternatives challenged:**
- **Components read `prefers-color-scheme`** (what the Mermaid pack did first): it is the reader's preference, not the page's; a light page for a reader who prefers dark gets dark diagrams in light boxes (S1, S2).
- **A token, `--compmd-scheme`, read with CSS:** CSS colours can follow it, but a library that takes its theme at render (Mermaid, a canvas chart) needs a value and a call when it changes.
- **Rejections reported as W118 automatically:** a frame can't tell which component an unhandled rejection came from when several share it (S9), and a rejection isn't always a failure to draw (S5).

## What the Mermaid pack becomes

```ts
// src/mermaid.ts
import mermaid from "mermaid";
import { draw } from "@compmd/pack/component";

let drawn = 0;
export default draw<{ code: string; theme: "auto" | "default" | "dark" | "neutral" | "forest" }>(
  async ({ code, theme }, { scheme }) => {
    mermaid.initialize({ startOnLoad: false, securityLevel: "strict", theme: theme === "auto" ? (scheme === "dark" ? "dark" : "default") : theme });
    return (await mermaid.render(`mermaid-${String((drawn += 1))}`, code)).svg;
  },
  { role: "img", oneAtATime: true },
);
```

Its unit test mocks Mermaid, and checks the theme for each scheme and that a parse error fails with Mermaid's message, with `mountForTest`. The pack keeps its browser tests (`test/`) only because they are this repository's checks of the sandbox and the registry; a pack author's are the unit tests.

## What changes

- **The spec**, §7.4: `scheme` and `fail` in the contract; a throw in `update` recovers at the next update; when to `fail` (S5). §7.6: the bindings pass both. §10.2: W118 is "threw while mounting or updating, or failed".
- **`@compmd/render`**: the contract's two fields; reading the container's scheme at render, on a preference change, when printing and on `page.scheme()`; failure until the next update; the React and Svelte bindings.
- **`@compmd/sandbox`**: frames pass the scheme to their renderer (the `scheme` message stays, and the frame's document keeps it, so a frame stays transparent on its page); the view's `scheme()`.
- **`@compmd/pack`**, new: the `compmd-pack` command, the test page, the publish check, the TypeScript base, the component helpers and the testing kit. Released with the other packages.
- **`@compmd/create-pack`**: writes packs on `@compmd/pack`, leaf, renderer and library alike.
- **The Mermaid pack**: rewritten on it.

## Not in this change

- A Vite plugin of a pack's own. `compmd-pack` adds React's or Svelte's by the component's language; a pack that needs another is rare enough to wait for a real one.
- Recovering from a throw in `mount`: the component never mounted, so there is nothing to update.
