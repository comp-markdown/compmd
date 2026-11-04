<picture>
  <source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/comp-markdown/compmd/main/logo/compmd-dark.svg">
  <img src="https://raw.githubusercontent.com/comp-markdown/compmd/main/logo/compmd-light.svg" alt="CompMD" width="200">
</picture>

# @compmd/render

Renders a CompMD expanded tree ([spec §12.3](../compmd-spec-v1.md#123-the-expanded-tree)) as a live page: Markdown, layout, inputs, and host-language components through the host contract (§7.4), with the React, Svelte 5 and plain bindings (§7.6). A new tree updates the page in place, so components keep their instances and their state.

**Website:** [compmd.hakanserce.com](https://compmd.hakanserce.com/), with the spec and a playground to try CompMD in.

It renders trees; it doesn't process documents. A processor (stages 1 to 4b, spec §11) gives it the tree: [`@compmd/processor`](https://www.npmjs.com/package/@compmd/processor) in the browser.

```sh
npm install @compmd/render @compmd/processor
```

It implements the draft of the [CompMD specification](https://compmd.hakanserce.com/spec/), version 1. Its versions are `0.x` until the spec's version 1 is published, and it shares its version with `@compmd/processor` ([how releases work](https://github.com/comp-markdown/compmd/blob/main/docs/releases.md)).

**Versions:**
- **0.4.0:** the spec's draft as of 2026-11-03 ([its changelog](https://github.com/comp-markdown/compmd/blob/main/CHANGELOG.md)): where the page came from, both ways (`sourceOf`, `elementsAt`, `reveal`, and through a frame `reveal`, `rectsAt` and `point`); `highlight`, for code blocks that show as code; a pending pack's uses drawn as loading (`componentPending`); a document's images from elsewhere (`images`, `imageSources`); each failure naming its use, and `onRecovered` once it draws again; components mounted once their boxes are placed and styled. Breaking: the frame protocol is version 3, so a page and a frame of 0.3.0 refuse each other, and `Footnotes.definitions` gives each definition's node.
- **0.3.0:** the same renderer as 0.2.2, released with `@compmd/sandbox`'s observer, for developer tools, and the first version of `@compmd/devtools`, the Developer Tools as a part any page embeds.
- **0.2.2:** the same as 0.2.1; the first version the release workflow stages and the author approves, and the first from the workflow of the packages first published by hand at 0.2.0, which proves their trusted publishing.
- **0.2.1:** `@compmd/sandbox` sizes a frame the browser doesn't render (below the fold, or too small to see) to its content, which a frame could miss in 0.2.0; the other packages are as in 0.2.0. Published for `@compmd/render` and `@compmd/processor` only: the release stopped at the sandbox, whose trusted publisher allows staging alone.
- **0.2.0:** the spec's draft as of 2026-11-02 ([its changelog](https://github.com/comp-markdown/compmd/blob/main/CHANGELOG.md)): among the changes, code renderers (a code block drawn by a component, with its code block as the fallback), and the host contract's `scheme` and `fail`: components follow the page's colour scheme (`page.scheme()`, `schemeOf`), and a component that fails shows its fallback until its next update. Released with the first versions of `@compmd/sandbox`, `@compmd/registry`, `@compmd/vite`, `@compmd/pack` and `@compmd/create-pack`.
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

**When a component mounts.** A page or a region mounts the components a render made once it has placed their boxes in the document and written their styles, so `mount` gets a box with its common params, the page's theme tokens and its place, in document order: a component before those in its slots, which it places in its own DOM as it mounts. `mountComponent` called directly mounts at once, into the target its caller gives; its `later` option delays the mount for a caller that places the box afterwards.

**Regions** follow the nearest container around them, which the caller provides: inside a page, the page or a box in it; elsewhere, the region's target or an element around it with the class `compmd-container` (or `container-type: inline-size`). A region whose blocks use breakpoints with no container around it reports `no-container` once through `onError`, when its target is in the document. Regions side by side give each pane its own container, so each follows its own width:

```ts
for (const [pane, nodes] of [[left, before], [right, after]]) {
  pane.classList.add("compmd-container");
  createRegion(pane, { components, stylesheet }).render(nodes);
}
```

**Code blocks and math.** `code: { mermaid: (source, target) => … }` renders fenced code of a language, and `math: (tex, { display }, target) => …` renders math. Without them, code shows as a code block and math as its TeX source, as spec §2.3 says. A code block a document's code renderer renders (spec §7.7; the tree gives it `component` and `alt`) mounts that component from `components` in its box, as a leaf: the block's text is its `code` param, `alt` its `fallback.text`, and the code block its fallback. It takes the place of `code` for its language, since a document's import wins over the host's.

**Highlighting.** A code block that shows as code, `<pre><code class="language-…">`, is one no renderer draws, or a code renderer's fallback, when its component is missing or fails. `highlight: (code, language, target) => …` highlights it: `target` is the block's `<code>`, holding its text, which the highlighter replaces with its tokens, at once or once what it needs has loaded; `language` is the fence's, or `""`. It is called again only when the block's text or language changes. Without it, the text is plain. [`@compmd/sandbox`](../sandbox/README.md#code-blocks) gives one by default, Shiki's (`shikiHighlight()`), which a page rendered with `createPage` can take too.

**Images from elsewhere.** A page that can't load images itself, such as one in a frame without the network, gives `images: (src) => …`, which resolves with the URL to show for an image's `src` (a `blob:` URL of a file its host fetched), or `undefined` when it can't be had. Markdown images, images by reference and raw HTML images then show their alt text, in `<span class="compmd-image-alt">`, until it resolves, and keep it when it can't. Without it, an image is an `<img>` loading from its `src`. `imageSources(nodes)` lists the `src` of every image a tree can show, for a host that fetches only those.

**Light and dark.** A component's `context.scheme` is the colour scheme of what the page or region renders into: its `color-scheme`, and the reader's preference where it allows both (`schemeOf(element)`). It is read at each render, when the reader's preference changes, and when printing starts and ends; a host that switches its own theme with a class calls `page.scheme()` (or `region.scheme()`) after it. When the scheme changes, every component on the page gets it and an `update` with the same params. A component that draws with CSS colours needs none of this; one that takes its colours when it draws (a diagram, a canvas chart) reads `context.scheme`.

**A component that fails.** `context.fail(message)` is for a component that finds, after `mount` or `update` returned, that it can't draw what it was given: it reports `threw` through `onError` with `message` (W118), and shows the fallback until its next `update`, when the box is emptied and given back. A throw in `update` does the same; a throw in `mount` shows the fallback for good. Each error names its use, `use`, an id unique in the window, and when the use draws again after such a failure, `onRecovered({component, use})` says so, so a host that lists failures takes that one off; a frame posts it as `recovered`.

**A component still drawing** (proposed). `context.busy(work)` is for a component that draws after `mount` or `update` returned: `work` is a promise of the drawing (spec §7.4). `page.drawn()` (and `region.drawn()`) resolves once the components of the last render and splice have mounted and every promise they gave `busy` has settled, fulfilled or rejected, or after `BUSY_MS` (10 s) for one that never does; a later render waits for its own. Nothing shows differently meanwhile. A frame's end tells its host the same (`drawn`, *In a frame*), which a sandbox builds its `settled` on.

## Where the page came from

An app that embeds the page often needs the link between what the page shows and the text that made it. An editor moves its caret to what the reader clicked, or scrolls the preview to the caret; a review tool marks the passage a comment is on. The page and a region keep it, from the tree's origins (spec §12.3: request them with `origins: true`), and give it both ways:

| Call | Gives |
|---|---|
| `page.sourceOf(node)` | The range of the text that made `node`, an element or a text node of the page (one in a component's shadow root too), or `undefined` |
| `page.elementsAt(offset)` | The elements the text at `offset` drew, or `[]` |
| `reveal(elements)` | Scrolls to them, as little as shows the first, and marks each for `REVEALED_MS` (1.5 s) |

```ts
import { codePointOffset, createPage, reveal, utf16Offset } from "@compmd/render";

const page = createPage(preview, { components });
page.render(tree, { text });                               // a tree processed with `origins: true`

// The reader clicked in the preview: the caret goes where it came from.
preview.addEventListener("click", (event) => {
  const source = page.sourceOf(event.target as Node);
  if (source?.start.offset === undefined) return;
  const caret = utf16Offset(text, source.start.offset);
  textarea.setSelectionRange(caret, caret);
});

// The caret moved: the preview shows what it is in.
textarea.addEventListener("selectionchange", () => reveal(page.elementsAt(codePointOffset(text, textarea.selectionStart))));
```

**A range** (`Source`) is `{start, end}`, each point as an origin has it: `line` and `column` 1-based, `offset` 0-based, columns and offsets in code points (spec §10.5). It is in the current text: after `page.splice`, the origins of the blocks it kept are behind the text, by what changed before them, and the page moves them on. An editor's offsets are UTF-16 (a text area's selection, CodeMirror's positions): `codePointOffset(text, utf16)` and `utf16Offset(text, codePoint)` convert them.

**What maps to what.** Each node of the page's DOM maps to the expanded node it was drawn for, or, when it has none of its own, to the nearest one around it that has one. So the rules for origins (spec §12.3) decide what a click gives:

| What was clicked | Its range |
|---|---|
| Markdown | its own node's: a word in `*em*` gives the emphasis, a link its link |
| Text a `value` or an interpolation produced | the node holding it, its parent |
| What a `compmd` definition's body drew | the invocation in this document, at any depth |
| A host-language component's own DOM | the component's; its slot items, each its own block |
| A footnote's item in the list at the end | its definition's |
| What the renderer adds itself (a task's checkbox, a footnote's ↩, an alert's title) | the node it belongs to |
| A component's fallback, while it shows | its own nodes' |

**Which elements a range drew.** `elementsAt` takes the innermost range that holds the offset, from its start to its end; where one range ends and the next starts, the one that starts there, and at the end of the last, that one. A range can draw several elements: each iteration of an `each` and each instance of a template slot (they share their template's range), all of them. Of elements one range drew inside one another, as a `compmd` component's box and its body, the outermost.

**Its limits.** A tree without origins maps nothing. A block that keeps its last good render while its text is invalid (rule 3 above) maps all of it to the block's range in the current text, since what it shows is an older text's; while the frontmatter doesn't parse (rule 1), what the page still shows of the blocks it let go maps to nothing. Inside a component's own DOM there is nothing finer than the component, and what a component renders in a frame of its own (a sandbox's use frame) maps to the component as a whole. A region maps from the origins of the nodes it rendered last, as given: it has no splices.

**`reveal`'s mark** is the class `compmd-revealed`, an outline in the theme's accent (`--compmd-accent`), which a host may restyle. A second reveal of an element starts its time again.

## Where each line is

A viewer that shows the text's line numbers in a gutter beside the page, or marks a line a comment is on, needs each line of the text where the page shows it. The page and a region give that from the same origins, as boxes, and say when they moved:

| Call | Gives |
|---|---|
| `page.lines(from?)` | a generator of `{line, rects}`, one for each line of the current text that draws something, in line order, from line `from` (1 without it): `rects` are the boxes its text shows in, `{x, y, width, height}`, one per visual line it covers, in CSS pixels from the page root's top left corner. Each line is measured as the generator reaches it, so a gutter that stops at the bottom of what shows measures no more |
| `onLayout()`, an option of `createPage` and `createRegion` | called when what the page shows moved, at most once an animation frame |

```ts
import { createPage } from "@compmd/render";

const page = createPage(preview, { components, onLayout: drawGutter });
page.render(tree, { text });                    // a tree processed with `origins: true`

/** The numbers of the lines that show, beside them: the gutter scrolls with the page. */
function drawGutter() {
  const numbers = [];
  for (const { line, rects: [first] } of page.lines(firstLineShown)) {
    if (first.y > preview.scrollTop + preview.clientHeight) break;   // past what shows: the rest isn't measured
    numbers.push(number(line, first.y));
  }
  gutter.replaceChildren(...numbers);
}
```

**A line's place** is where what is drawn from it shows, from its first character to its last, by these rules:

| What the line holds | Its rects |
|---|---|
| Text, in a paragraph, a heading, a list item, a quote, a table's row, a footnote's definition | the boxes of its text, one per visual line it covers, each as tall as that line box: a line that wraps has several. A paragraph's later lines too, each where its text shows |
| A code block's line | that line in the block's `<pre>`, highlighted or not |
| A use of a component, a code block a component draws, math, an image, a thematic break, an alert's first line: what has no text of its own on the page | the box of what its range drew, on the line where its range starts; the lines after it, inside it, draw nothing of their own |
| Only what draws nothing: blank lines, frontmatter, a definition, a link definition, a container's closing line, a table's delimiter row, a code block's fences | left out, for the gutter to leave blank |

**Finding a later line of a paragraph.** A text node holds its text as the processor gave it, a soft line break as `\n`, so the text of a node's third line follows its second `\n`, whatever escapes, entities or markup the line has. A text node whose `\n`s don't match its lines, such as an interpolation whose value has line breaks, places only the line it starts on. Where a line starts inside inline markup (`**bold** text`), its first character is the first one the markup draws.

**What a gutter sees.**
- **Lines can share a visual line:** a soft line break shows as a space, so a paragraph's lines 3 and 4 are often on one line of the page: 3's last rect and 4's first at the same `y`, 4's starting where 3's ends. A gutter shows the first, or both; a tool that marks a line marks its rects.
- **A place is the first that shows:** of the iterations of an `each`, which draw the same lines, the first; a line whose text is hidden (a closed `<details>`, a breakpoint's `hide`, a component's fallback that doesn't show) has no place, unless another drawing of it shows.
- **Places follow the page, not the text:** a footnote's definition is placed at the end of the page, with its list. Content scrolled inside a box of its own (a code block wider than the page) is placed where it is, which may be outside the box's visible part.

**When the page moved.** `onLayout` is called after a render or a splice is painted, when the page root or any block in it changes size (a component sizing itself, an image that loaded, a frame that reported its height), and when fonts finish loading. It says that places may have moved, not that the page is done moving: a page whose components keep growing calls it again. Waiting until it stops is [the sandbox's `settled`](../sandbox/README.md), for tests and printing.

**Its limits.** Inside a component's own DOM there is nothing finer than the component: its lines after the first draw nothing of their own, and so does what a component renders in a frame of its own. A tree without origins has no lines. While the page keeps a block's last good render (rule 3 of *Invalid text*), that block's lines draw nothing, since its text is an older text's.

**A generator of the page as it is.** Each line is measured when it is reached, so a generator kept across a render or a `layout` gives the new page's places from there on; ask again for each drawing of the gutter. `firstLineShown` above is the host's to know, such as the line of the last number it drew above what shows.

Through a frame, `frame.lines(from?)` is an async generator of the same, in the frame's page, which the frame measures in batches as the host's loop asks for them (`for await`), and `frame.on("layout", …)` is called as `onLayout` would be (*In a frame*, below).

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
frame.reveal(offset);                             // scroll the frame's page to what the text at offset drew
const boxes = await frame.rectsAt(offset);        // its boxes, in the frame's viewport
frame.point(true);                                // report the source of what the reader clicks
frame.on("pointed", ({ source }) => moveCaret(source.start.offset));
frame.on("layout", async () => {
  for await (const { line, rects } of frame.lines()) place(line, rects);   // where each line is, in the frame's page
});

// inside the frame
addEventListener("message", ({ ports: [port] }) => serveFrame(port, { container: document.body, components }));
```

Host to frame: `render {tree, change?}`, `splices {splices, change?, replace?}`, `theme {tokens}`, `reveal {offset}`, `rects {id, offset}`, `point {on}`, `lines {id, from, count}`. A splice can be the JSON text a processor gives (`@compmd/processor`'s `OpenDocumentJson`), which a host passes on unread: posted text is copied in one piece, and the frame parses it once. `replace` says the first splice is of an empty tree, as after opening a document again. The frame ignores any other message, so a host can send its own on the same port, as an editor does with the components to compile. Frame to host: `ready {version}`, `set {input, value}`, `error {reason, component?, use?, message}`, `recovered {component, use}`, `rendered {ms}` (after the render is painted), `rects {id, rects}` (the answer to `rects` of the same `id`, each `{x, y, width, height}` in the frame's viewport), and, after `point {on: true}`, `pointed {source}` for each click in the page; `lines {id, lines, done}` (the answer to `lines`: up to `count` lines from line `from`, each `{line, rects}` in the frame's page, and whether there are more; `frame.lines()` asks for the next batch as its loop goes on, and for none once it stops), `layout {}` (as `onLayout`) and `drawn {seen}` (as `page.drawn()` resolves, for the first `seen` renders and splices the frame took). The host's messages wait for `ready` with this protocol's version (4, with `lines`, `layout` and `drawn`), since a host page and its frame may be deployed apart. Component code stays in the frame: modules can't be posted.

**Where the page came from, through the frame.** The host can't reach the frame's elements, so the frame's end does what `elementsAt` and `sourceOf` would: `reveal(offset)` scrolls the frame's page and marks what the text there drew, `rectsAt(offset)` resolves with its boxes (none once the connection closes), and `point(true)` has the frame report the source of each click, as `pointed`. `lines()` gives each line's place in the frame's page, a batch at a time (ending once the connection closes), and `layout` says when to ask again. A frame's code may be untrusted, so `connectFrame` passes on only a well-formed `source`, the boxes that are numbers, and lines that are whole numbers from 1 with rects that are numbers.

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
| What `reveal` shows | adds `compmd-revealed` for 1.5 s | an outline in `--compmd-accent` |
| A use of a component on its way (`componentPending`, spec §4.5) | its fallback in `<div class="compmd-pending" aria-busy="true">`, a `<span>` in inline content; in a host component's slot, an item whose `component` is `{name, params: {}, pending: true}` | a little faded (`opacity: 0.6`), which a host may restyle |
| An alert | `<div class="compmd-alert compmd-alert-note" role="note">` with `<p class="compmd-alert-title">Note</p>` first | a rule on the start edge |
| A quote | `<blockquote class="compmd-quote">` | a lighter rule on the start edge |
| A tight list (`spread: false`) | `<ul class="compmd-tight">`, `<ol class="compmd-tight">` | its items' paragraphs without margins, as HTML renders a tight list without `<p>` |
| A task list item | `<li class="compmd-task">` with `<input type="checkbox" disabled>` first | no bullet; the box on its first paragraph's line |
| Footnotes | `<sup><a href="#compmd-fn-1" id="compmd-fnref-1">1</a></sup>`, and at the end `<section class="compmd-footnotes"><hr><ol><li id="compmd-fn-1">…</li></ol></section>` | smaller text |
| A code block that shows as code | `<pre><code class="language-js">`, without the class for a block of no language; with `highlight`, its tokens in the `<code>`, as the highlighter writes them | — |
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
- **Unit tests** next to the code: the styles, raw HTML, images from a source, keys, the component layer, the page and its splices (blocks kept, several in order, footnotes, links, headings, raw HTML across blocks, inputs, broken frontmatter), a region's container, the frame protocol.
- **[`test/identity.test.ts`](test/identity.test.ts):** every edit of a document of stateful components (typing above, inside and in params, inside an `each`, adding, removing and pasting blocks, undo, a record added, a block moved), checking which components keep their instance and their state. Each runs with whole trees and with splices, which give the same page.
- **[`test/invalid.test.ts`](test/invalid.test.ts):** an unclosed container, a half-written expression, a half-typed component line and broken frontmatter, typed and finished, with whole trees and with splices.
- **[`src/source.test.ts`](src/source.test.ts), [`test/source.test.ts`](test/source.test.ts):** where the page came from: each rule of what maps to what, splices, a region, `reveal`; and on every edit of the fixtures, a page brought up to date by splices maps every element and every offset as one given each whole tree does.
- **[`test/conformance.test.ts`](test/conformance.test.ts):** every expanded tree in the conformance set renders, as a page and as a region per block, with the same text.
- **[`test/bindings.test.ts`](test/bindings.test.ts):** the React and Svelte bindings.

The fixtures in [`test/fixtures/`](test/fixtures/) are expanded trees with origins, each with its text, diagnostics and edit. They were made with a prototype processor, and are to be made again with the `compmd` processor.
