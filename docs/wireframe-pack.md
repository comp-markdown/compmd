# Wireframes in CompMD (proposed)

A writer sketching a screen in a design note, an issue or a spec wants a **wireframe**: the screen's boxes, its
buttons, inputs and images, and notes on them, drawn roughly enough that nobody mistakes it for the design. Mermaid,
which the [Mermaid pack](mermaid-pack.md) draws, has no wireframe diagram. This compares the ways a CompMD pack could
draw them and recommends one; the choice is the user's.

## What a wireframe pack needs

- **Text a writer can read and write**, which a plain Markdown renderer shows readably (spec §14): the wireframe is
  also its own description.
- **It runs where packs run:** in a strict frame, with no network, no workers and no `eval`
  ([the Mermaid pack](mermaid-pack.md#what-it-runs-into-in-the-strict-sandbox)), as one ES module of a size a page can
  fetch while a reader waits.
- **A licence a pack can ship under** (MIT, or as permissive).
- **The things screens have:** a screen (a phone, a window, a browser), rows, columns and grids, a heading and text,
  buttons, inputs and selects, checkboxes, images and icons as placeholders, a list, tabs, a note pointing at a part.
- **Light and dark**, by the host contract's `scheme`, and a look that says "sketch".

## The options

| | Option | Source in the document | Runs in a strict frame | Size | Licence |
|---|---|---|---|---|---|
| A | **CompMD components**, on `box`, `row`, `stack` and `grid` | components (`::: wf-screen`, `:: wf-button[Sign in]`) | no code: `compmd` definitions | a pack of definitions, a few KB | the pack's own (MIT) |
| B | **A code renderer for a small wireframe language**, written for the pack, in Salt's style | a fenced block (```` ```wireframe ````) | yes, our own code, no `eval` | tens of KB | the pack's own (MIT) |
| C | **PlantUML's Salt** | a fenced block (```` ```salt ````) | no: PlantUML is Java; its browser builds are experimental and megabytes | MB | GPL, with editions under MIT and other licences |
| D | **Excalidraw**, its scenes as JSON | a fenced block of JSON, or a file | yes, with work: a React app; its fonts and its export need care | about 1 MB and more | MIT |
| E | **A rendering service** (Kroki, PlantUML's server) | a fenced block | no: it needs the network, which strict frames don't have, and it sends the document's text out | none | varies |

### A: CompMD components

The wireframe is the document's own structure, laid out by CompMD's layout built-ins:

````md
::: wf-screen {device=phone title="Sign in"}
:: wf-image[Logo] {label="Logo" height=4rem}
:: wf-input[Email] {label="Email" placeholder="you@example.com"}
:: wf-input[Password] {label="Password" type=password}

::: row {gap=s}
:: wf-button[Sign in] {label="Sign in" variant=primary}
:: wf-button[Forgot your password?] {label="Forgot your password?" variant=link}
::: /row

:: wf-note[An error shows under the field that failed] {label="An error shows under the field that failed"}
::: /wf-screen
````

- **Gives:** no code at all, so it renders in every CompMD processor, with nothing to sandbox, and prints as it
  draws; layout is CompMD's (`row`, `grid`, breakpoints), so a wireframe is responsive for free; every part is a
  component, which editors complete and the language server checks; in a plain renderer each line still reads as what
  it is ("wf-input[Email]").
- **Costs:** a wireframe is many lines; the sketchy look is CSS only (dashed borders, a hand-drawn font if the host
  has one), no wobbly lines.
- **A gap it finds in the spec:** a `compmd` definition can't read its leaf's fallback text (`[Sign in]`); only
  host-language components get it (`fallback.text`, §7.4). So each label is written twice, as above. A spec change,
  `{{md.fallback}}` in a `compmd` body for the leaf's or inline component's fallback text, would let
  `:: wf-button[Sign in] {variant=primary}` say it once. It is a change of its own, for any `compmd` component with a
  label, not only these.

### B: a code renderer for a small wireframe language

A language made for wireframes, in the spirit of Salt, drawn by our own parser and an SVG writer, with a rough look
from a small hand-drawn line generator (Rough.js, MIT, about 9 KB, if we take it rather than write our own):

````md
```wireframe {alt="The sign-in screen"}
screen phone "Sign in"
  image "Logo" height=4rem
  input "Email" placeholder="you@example.com"
  input "Password" password
  row
    button "Sign in" primary
    link "Forgot your password?"
  note "An error shows under the field that failed"
```
````

- **Gives:** short, readable source, as Mermaid's is; a sketch look; one block that copies between documents; in a
  plain renderer, an indented outline.
- **Costs:** a language to design, document and keep (its grammar, its errors, W118 with the line), and it is code in
  a frame, so it waits for the frame like any renderer.
- **Drawn as HTML, not SVG.** The renderer turns the parsed outline into HTML elements laid out with CSS (a `row` is a
  flex row, a `grid` a CSS grid, a screen a sized box), so the browser does the layout, wraps text and sizes each part,
  and there is no layout engine of our own to write. The sketch look is CSS too: borders a little uneven, a hand-written
  font when the page has one, greys from the host's tokens; Rough.js (MIT, about 9 KB) can add wobbly lines later, over
  the same boxes. Drawing to SVG instead would need our own layout engine, its largest part by far.

#### What B takes

With the HTML drawing, about five tasks, each reviewed:

1. **The language, designed**: its grammar in ABNF (an outline: one part per line, its children indented), its parts
   and their attributes, its errors, with real example documents; reviewed first.
2. **The parser**: text to an outline, with each error's line and column, tested on the grammar's examples and on
   broken input.
3. **The drawing**: the outline to HTML and CSS, each part a small function, light and dark, printed.
4. **The pack**: `@comp-markdown/wireframe` with `npm create @compmd/pack -- --renders wireframe`, its signature, its
   test page, its tests in Chrome and WebKit; released on its own tag, as the Mermaid pack is.
5. **The guide**: a chapter section in the Author's Guide, and its examples.

A is about two (the spec change, then the pack of `compmd` definitions).

### C to E

- **C, Salt**, is the format people know, but nothing runs it in a page: PlantUML is Java, and its browser builds are
  megabytes and experimental. Salt has no specification or published grammar to follow: PlantUML's documentation
  describes it by examples, and its Java implementation is what defines it. So no renderer can claim to comply with
  Salt in full; it could only match PlantUML on the documented examples, and follow it as it changes. A
  Salt-compatible subset in B's renderer is possible later, if writers ask for Salt, with that limit said plainly; B's
  own grammar, written in ABNF, is one a renderer can comply with.
- **D, Excalidraw**, draws wireframes well, but by hand, not from text: its scenes are JSON no one writes. An Excalidraw
  pack, a drawing the reader draws in the document, is work of its own. Its scenes can't be read in a plain renderer.
- **E, a service**, can't run in a strict frame and sends the document's text to someone else.

## Recommendation

**Either A or B, drawn as HTML; B if the shorter text is worth about three more tasks.** With B drawn as HTML and CSS,
the browser does its layout, so B is a language and a small renderer, not a drawing engine: about five tasks against
A's two. B gives writers the short text they know from Mermaid and a grammar of our own that renderers can comply
with; A needs no frame, renders and prints everywhere, and shows what CompMD's own layout can do. A's one gap, labels
written twice, is a small spec change of its own (`{{md.fallback}}`), worth making either way.

## What A would be

- `@comp-markdown/wireframe`, in `packs/wireframe/`, written with `npm create @compmd/pack`, as
  [the Mermaid pack](mermaid-pack.md) is, released on its own tag.
- Components, each a `compmd` definition: `wf-screen` (`device`: `phone`, `tablet`, `window`, `browser`; `title`),
  `wf-heading`, `wf-text` (lines of placeholder text, `lines=3`), `wf-button` (`variant`: `default`, `primary`,
  `link`), `wf-input` (`type`, `placeholder`), `wf-select`, `wf-checkbox`, `wf-image` (a crossed box, `height`),
  `wf-icon`, `wf-list` (`items`), `wf-tabs` (`items`, `selected`), `wf-note` (a note in the margin, numbered).
- The look: dashed and solid borders, greys from the host's tokens, light and dark by the page's scheme, a
  sketch font only if the host offers one.
- Tests: each component on the pack's test page, light and dark, printed, and in a plain renderer.

## Open

- Whether to make the spec change (`{{md.fallback}}` in a `compmd` body) first, so labels are written once.
- The names: `wf-` as a prefix (proposed), so they don't clash with a document's own `button` or `note`; a document
  that wants shorter ones renames them on import (`{name: wf-button, as: button}`).
