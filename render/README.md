# @compmd/render

Renders a CompMD expanded tree ([spec §12.3](../compmd-spec-v1.md#123-the-expanded-tree)) as a live page: Markdown, layout, inputs, and host-language components through the host contract (§7.4), with the React, Svelte 5 and plain bindings (§7.6). A new tree updates the page in place, so components keep their instances and their state.

It renders trees; it doesn't process documents. A processor (stages 1 to 4b, spec §11) gives it the tree.

## Commands

```sh
pnpm install         # in the repository root
pnpm test            # unit tests in a DOM (happy-dom)
pnpm lint            # eslint and the type check
pnpm build           # ESM and types into dist/
```

## The three layers

| Layer | Entry point | What it renders |
|---|---|---|
| **Page** | `createPage(root, options)` | The whole tree: the region at the root, plus what only the whole tree has (the container breakpoints query, the theme tokens, footnote numbering and the list at the end, the inputs) |
| **Region** | `createRegion(target, options)` | Any list of nodes, updated in place by key; components through the component layer |
| **Component** | `mountComponent(target, node, options)` | One host-language component in its outer box; the caller renders its slots and its fallback |

```ts
import { createPage } from "@compmd/render";
import { react } from "@compmd/render/react";

const page = createPage(document.body, {
  components: { chart: drawChart, card: react(Card) },     // plain components are their own mount
  onSet: (input, value) => processAgain({ [input]: value }),
  tokens: { "--compmd-accent": "#0b7285" },
});

page.render(tree);                                          // an input or data changed: the text didn't
page.render(tree, { text, diagnostics, edit });             // a keystroke in an editor
```

**Keys.** Every node with an `origin` (spec §12.3, the `origins` feature) gets a key: an id per origin range, followed through each edit as an editor maps positions, with the node's occurrence. A node whose range lands where an old one went keeps its view. `edit` is the old text's range the editor replaced, in UTF-16 offsets, as a text area's selection or a CodeMirror transaction gives it; without it, the texts are compared. Nodes without origins are matched by place.

**Invalid text.** Given a change with diagnostics, the page keeps what it showed where the text is invalid mid-typing, on top of lenient recovery (spec §10.3):

1. Frontmatter that doesn't parse (F001, F033): the last good page stays.
2. A top-level container that isn't closed (F004): its opener shows as a paragraph, and what follows stays where it was.
3. Any other error in a top-level block: that block keeps its last good render, and the rest updates.

**Breakpoints** are container queries: each block's params follow the width of the box around it (spec §9.3), so a box is a container when blocks placed in it use breakpoints, and the page always is. The page takes its width from where its caller puts it, so a page root in a flex row needs a width, or it shrinks to nothing.

**Regions** follow the nearest container around them, which the caller provides: inside a page, the page or a box in it; elsewhere, the region's target or an element around it with the class `compmd-container` (or `container-type: inline-size`). A region whose blocks use breakpoints with no container around it reports `no-container` once through `onError`, when its target is in the document. Regions side by side give each pane its own container, so each follows its own width:

```ts
for (const [pane, nodes] of [[left, before], [right, after]]) {
  pane.classList.add("compmd-container");
  createRegion(pane, { components, stylesheet }).render(nodes);
}
```

**Code blocks and math.** `code: { mermaid: (source, target) => … }` renders fenced code of a language, and `math: (tex, { display }, target) => …` renders math. Without them, code shows as a code block and math as its TeX source, as spec §2.3 says.

## In a frame

The page runs inside a frame its caller supplies, such as a sandboxed iframe, and a port carries trees in and `set` calls out. Both ends take a port the caller made, so a host whose sandbox already has a channel to the frame hands one over. A `MessagePort` is a port; any bridge with `postMessage` and a message listener fits behind a small adapter.

```ts
// in the host page
const { port1, port2 } = new MessageChannel();
iframe.contentWindow.postMessage("compmd", "*", [port2]);
const frame = connectFrame(port1);                 // from "@compmd/render/frame"
frame.on("set", ({ input, value }) => processAgain({ [input]: value }));
frame.on("error", ({ reason, component }) => report(reason, component));   // missing, or threw (W118)
frame.render(tree, change);

// inside the frame
addEventListener("message", ({ ports: [port] }) => serveFrame(port, { container: document.body, components }));
```

Host to frame: `render {tree, change?}`, `theme {tokens}`. Frame to host: `ready {version}`, `set {input, value}`, `error {reason, component?, message}`, `rendered {ms}` (after the render is painted). The host's messages wait for `ready` with this protocol's version, since a host page and its frame may be deployed apart. Component code stays in the frame: modules can't be posted.

## The reference rendering of the built-ins

The HTML and CSS this package writes for the built-ins is the reference rendering spec §13.1 calls informative. Every layout param means the CSS property it names (§9.4); the stylesheet is one per document (a constructed stylesheet, which a sandbox's CSP allows), and each styled block gets a class.

| Built-in | HTML | CSS |
|---|---|---|
| `box` (and `stack`, `row`, `grid`, which expand to it) | `<div class="compmd-box compmd-bN">` holding its children | its layout and common params on `.compmd-bN`; tokens as `var(--compmd-space-m, 1rem)` and so on (§9.4); breakpoint params in `@container (min-width: 768px) { … }`; `show` and `hide` as `display: none` outside their range |
| A box whose blocks use breakpoints | adds `compmd-container` | `container-type: inline-size` |
| A box whose block fills it (`fill`) | adds `compmd-fills` | `display: flex; flex-direction: column`, the block `flex-grow: 1` |
| A block with a position (`inset-*` not `auto`) | — | `position: absolute` with its insets; every box is `position: relative` |
| A host-language component | `<div class="compmd-component compmd-bN">` (its `target`), each slot item in `<div class="compmd-item">` | its common params on its box |
| An input component | `<div class="compmd-box compmd-input">` with `<input type="date">`, `number`, `text`, `checkbox`, or `<select>` with the enum's members; `aria-label` from the fallback | — |
| An alert | `<div class="compmd-alert compmd-alert-note" role="note">` with `<p class="compmd-alert-title">Note</p>` first | a rule on the start edge |
| A task list item | `<li class="compmd-task">` with `<input type="checkbox" disabled>` first | no bullet |
| Footnotes | `<sup><a href="#compmd-fn-1" id="compmd-fnref-1">1</a></sup>`, and at the end `<section class="compmd-footnotes"><hr><ol><li id="compmd-fn-1">…</li></ol></section>` | smaller text |
| Math without a renderer | `<code class="language-math math-inline">`, `<pre><code class="language-math math-display">` | — |
| A table | `<thead>` for the first row, alignment as `compmd-align-left`, `-center`, `-right` | `text-align` |
| Raw HTML | `details`, `summary`, `sub`, `sup`, `kbd`, `br`, `img` (`src`, `alt`) and `a` (`href`) with safe URLs; other tags show as text | — |

```md
::: grid {min=14rem gap=m}
```

```html
<div class="compmd-box compmd-b3">…</div>
<!-- .compmd-b3 { display: grid; grid-template-columns: repeat(auto-fill, minmax(14rem, 1fr)); gap: var(--compmd-space-m, 1rem); } -->
```

## Tests

- **Unit tests** next to the code: the styles, raw HTML, keys, the component layer, the page, a region's container, the frame protocol.
- **[`test/identity.test.ts`](test/identity.test.ts):** every edit of a document of stateful components (typing above, inside and in params, inside an `each`, adding, removing and pasting blocks, undo, a record added, a block moved), checking which components keep their instance and their state.
- **[`test/invalid.test.ts`](test/invalid.test.ts):** an unclosed container, a half-written expression, a half-typed component line and broken frontmatter, typed and finished.
- **[`test/conformance.test.ts`](test/conformance.test.ts):** every expanded tree in the conformance set renders, as a page and as a region per block, with the same text.
- **[`test/bindings.test.ts`](test/bindings.test.ts):** the React and Svelte bindings.

The fixtures in [`test/fixtures/`](test/fixtures/) are expanded trees with origins, each with its text, diagnostics and edit. They were made with a prototype processor, and are to be made again with the `compmd` processor.
