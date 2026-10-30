<picture>
  <source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/comp-markdown/compmd/main/logo/compmd-dark.svg">
  <img src="https://raw.githubusercontent.com/comp-markdown/compmd/main/logo/compmd-light.svg" alt="CompMD" width="200">
</picture>

# @compmd/render

Renders a CompMD expanded tree ([spec §12.3](../compmd-spec-v1.md#123-the-expanded-tree)) as a live page: Markdown, layout, inputs, and host-language components through the host contract (§7.4), with the React, Svelte 5 and plain bindings (§7.6). A new tree updates the page in place, so components keep their instances and their state.

It renders trees; it doesn't process documents. A processor (stages 1 to 4b, spec §11) gives it the tree: [`@compmd/processor`](https://www.npmjs.com/package/@compmd/processor) in the browser.

```sh
npm install @compmd/render @compmd/processor
```

It implements the draft of the [CompMD specification](https://compmd.hakanserce.com/spec/), version 1. Its versions are `0.x` until the spec's version 1 is published, and it shares its version with `@compmd/processor` ([how releases work](https://github.com/comp-markdown/compmd/blob/main/docs/releases.md)).

**Versions:**
- **0.1.1:** the same as 0.1.0; the first version the release workflow publishes.
- **0.1.0:** the first version.

## Commands

```sh
pnpm install         # in the repository root
pnpm test            # unit tests in a DOM (happy-dom)
pnpm lint            # eslint and the type check, svelte-check for the Svelte test component
pnpm build           # ESM and types into dist/
```

## The three layers

| Layer | Entry point | What it renders |
|---|---|---|
| **Page** | `createPage(root, options)` | The whole tree, block by block, from whole trees or splices, plus what only the whole tree has (the container breakpoints query, the theme tokens, footnote numbering and the list at the end, link definitions, the inputs) |
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
page.splice([splice], { text, diagnostics });               // a keystroke, with a document kept open
```

**Blocks.** The page keeps the tree's top-level nodes as blocks, each rendered once. A change is a splice of them, as a processor that keeps the document open gives it (`Splice`: the nodes `from..to` became `nodes`, and the nodes after them moved by `lines` and `offset`). `page.splice` takes splices in order, each of the tree the one before left, and renders only the blocks they replaced; `page.render` is a splice of every block. What only the whole tree has comes from a summary of each block, so a block elsewhere is updated only when it uses what changed: footnote numbers, a link definition, the inputs. Blocks that raw HTML pairs across (`<details>`, Markdown, `</details>`) render together.

**Keys.** Every node with an `origin` (spec §12.3, the `origins` feature) gets a key: an id per origin range, followed through each edit as an editor maps positions, with the node's occurrence. A node whose range lands where an old one went keeps its view. A splice has its edit, in code points. For `render`, `edit` is the old text's range the editor replaced, in UTF-16 offsets, as a text area's selection or a CodeMirror transaction gives it; without it, the texts are compared. Nodes without origins are matched by place.

**Invalid text.** Given a change with diagnostics, the page keeps what it showed where the text is invalid mid-typing, on top of lenient recovery (spec §10.3):

1. Frontmatter that doesn't parse (F001, F033): the last good page stays. The blocks follow the splices meanwhile, and show once it parses again.
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

**Code blocks and math.** `code: { mermaid: (source, target) => … }` renders fenced code of a language, and `math: (tex, { display }, target) => …` renders math. Without them, code shows as a code block and math as its TeX source, as spec §2.3 says. A code block a document's code renderer renders (spec §7.7; the tree gives it `component` and `alt`) mounts that component from `components` in its box, as a leaf: the block's text is its `code` param, `alt` its `fallback.text`, and the code block its fallback. It takes the place of `code` for its language, since a document's import wins over the host's.

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
frame.splice(splices, change);                    // each a Splice, or a processor's JSON text of one

// inside the frame
addEventListener("message", ({ ports: [port] }) => serveFrame(port, { container: document.body, components }));
```

Host to frame: `render {tree, change?}`, `splices {splices, change?, replace?}`, `theme {tokens}`. A splice can be the JSON text a processor gives (`@compmd/processor`'s `OpenDocumentJson`), which a host passes on unread: posted text is copied in one piece, and the frame parses it once. `replace` says the first splice is of an empty tree, as after opening a document again. The frame ignores any other message, so a host can send its own on the same port, as an editor does with the components to compile. Frame to host: `ready {version}`, `set {input, value}`, `error {reason, component?, message}`, `rendered {ms}` (after the render is painted). The host's messages wait for `ready` with this protocol's version, since a host page and its frame may be deployed apart. Component code stays in the frame: modules can't be posted.

**A component that renders its slots itself.** `rendersSlots(mount)` makes a component for `components` whose slots the renderer doesn't render: `mount` gets the use's node, with its slots as expanded nodes, and the document's inputs, and its `update` gets them again. A sandbox uses it to render a use and its slots inside the use's own frame ([`@compmd/sandbox`](../sandbox/README.md)); a code block's renderer gets its leaf, with the block's text as `code`.

## The reference rendering of the built-ins

The HTML and CSS this package writes for the built-ins is the reference rendering spec §13.1 calls informative. Every layout param means the CSS property it names (§9.4); the stylesheet is one per document (a constructed stylesheet, which a sandbox's CSP allows), and each styled block gets a class.

| Built-in | HTML | CSS |
|---|---|---|
| `box` (and `stack`, `row`, `grid`, which expand to it) | `<div class="compmd-box compmd-bN">` holding its children | its layout and common params on `.compmd-bN`; tokens as `var(--compmd-space-m, 1rem)` and so on (§9.4); breakpoint params in `@container (min-width: 768px) { … }`; `show` and `hide` as `display: none` outside their range; its first child's block-start margin and its last child's block-end margin trimmed, as `margin-trim: block` would, so its padding is all the space at its edges (also in a component's box and a slot item) |
| A box whose blocks use breakpoints | adds `compmd-container` | `container-type: inline-size` |
| A box whose block fills it (`fill`) | adds `compmd-fills` | `display: flex; flex-direction: column`, the block `flex-grow: 1` |
| A block with a position (`inset-*` not `auto`) | — | `position: absolute` with its insets; every box is `position: relative` |
| A host-language component | `<div class="compmd-component compmd-bN">` (its `target`), each slot item in `<div class="compmd-item">` | its common params on its box |
| An input component | `<label class="compmd-box compmd-input">` with `<span class="compmd-input-label">`, the fallback before its first colon (`From: today` gives `From`, since what follows is the default), then `<input type="date">`, `number`, `text`, `checkbox`, or `<select>` with the enum's members | the label and the control side by side |
| An alert | `<div class="compmd-alert compmd-alert-note" role="note">` with `<p class="compmd-alert-title">Note</p>` first | a rule on the start edge |
| A quote | `<blockquote class="compmd-quote">` | a lighter rule on the start edge |
| A tight list (`spread: false`) | `<ul class="compmd-tight">`, `<ol class="compmd-tight">` | its items' paragraphs without margins, as HTML renders a tight list without `<p>` |
| A task list item | `<li class="compmd-task">` with `<input type="checkbox" disabled>` first | no bullet; the box on its first paragraph's line |
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

- **[`src/vocabulary.test.ts`](src/vocabulary.test.ts):** the params it styles are exactly the processor's (`conformance/params.json`), and every token of theirs gets CSS; a token CSS takes as it is is listed there.
- **Unit tests** next to the code: the styles, raw HTML, keys, the component layer, the page and its splices (blocks kept, several in order, footnotes, links, headings, raw HTML across blocks, inputs, broken frontmatter), a region's container, the frame protocol.
- **[`test/identity.test.ts`](test/identity.test.ts):** every edit of a document of stateful components (typing above, inside and in params, inside an `each`, adding, removing and pasting blocks, undo, a record added, a block moved), checking which components keep their instance and their state. Each runs with whole trees and with splices, which give the same page.
- **[`test/invalid.test.ts`](test/invalid.test.ts):** an unclosed container, a half-written expression, a half-typed component line and broken frontmatter, typed and finished, with whole trees and with splices.
- **[`test/conformance.test.ts`](test/conformance.test.ts):** every expanded tree in the conformance set renders, as a page and as a region per block, with the same text.
- **[`test/bindings.test.ts`](test/bindings.test.ts):** the React and Svelte bindings.

The fixtures in [`test/fixtures/`](test/fixtures/) are expanded trees with origins, each with its text, diagnostics and edit. They were made with a prototype processor, and are to be made again with the `compmd` processor.
