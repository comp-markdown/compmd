# Wireframes in CompMD

**Status: accepted (2026-11-17), option B with F2's language.** B's code renderer drawn as HTML and CSS, its language
Wireloom's syntax written down as a grammar of our own (Appendix C), in ```` ```wireframe ```` blocks: [the
wireframe language](wireframe-language.md).

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

1. **The language, written down** (designed, before F2): Wireloom's syntax as a grammar in ABNF (an outline: one part
   per line, its children indented), the parts the pack draws first and their attributes, its errors, with real
   example documents; reviewed first.
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

## Decision

**B, a code renderer for a wireframe language, drawn as HTML and CSS** (decided 2026-11-17), **with F2's language**
(decided the same day, after Appendix C): Wireloom's syntax as its examples and documentation show it on 2026-11-17,
written down as a grammar of our own in ABNF, with our own parser and drawing, and no dependency on Wireloom. Writers
get the short text they know from Mermaid, and the syntax those who know Wireloom already write; renderers get a
grammar they can comply with; the browser does the layout. The fence is ```` ```wireframe ````. It is built in the five
steps of *What B takes*, each a task; the first writes the language down instead of designing it.

**Credit.** The pack's README says, briefly, that its language is based on Wireloom's syntax as its examples showed it
on 2026-11-17 (MIT, Stardock), and that the grammar, the parser and the HTML and CSS drawing are the pack's own; it
doesn't use the Wireloom name for itself.

## What B is

- `@comp-markdown/wireframe`, in `packs/wireframe/`, written with `npm create @compmd/pack -- --renders wireframe`, as
  [the Mermaid pack](mermaid-pack.md) is, released on its own tag: a code renderer of ```` ```wireframe ```` blocks
  (spec §7.7), with `alt` for its text alternative.
- Its parts, as Wireloom's syntax names them (`window`, `row`, `col`, `grid`, `panel`, `text`, `button`, `input`,
  `combo`, `checkbox`, `radio`, `tabs`, `tree`, `menubar`, `image`, `icon`, `annotation` and the rest): the
  language's writing-down (step 1) settles which of them the pack draws first, and how each is drawn.
- The look: borders a little uneven, greys from the host's tokens, light and dark by the page's scheme, a hand-written
  font only if the host offers one.
- Tests: the grammar's examples parsed, broken input with its errors' places, each part drawn on the pack's test page
  in Chrome and WebKit, light and dark, printed, and the block as a plain renderer shows it.

## Not in this change

- A Salt-compatible subset (C): later, if writers ask for it, with its limit said plainly.
- `{{md.fallback}}`, a `compmd` component reading its leaf's fallback text: a spec change of its own, for any
  `compmd` component with a label, which B doesn't need.

## Appendix A: how much Salt is used

Researched on 2026-11-17. No usage figures for Salt itself were found: no download counts, and no surveys.

- Salt is still part of PlantUML, which lists it among its diagram types ([plantuml.com/salt](https://plantuml.com/salt)).
- PlantUML's forum has a `salt` tag with steady, small traffic, the latest seen in late 2024, mostly bugs and feature
  requests (spacing, scroll bars, icons, a window's title bar) ([forum.plantuml.net/tag/salt](https://forum.plantuml.net/tag/salt)).
- A 2025 study of UML's popularity reports renewed interest in text-based diagram formats, PlantUML and Mermaid
  among them, without numbers for Salt ([Rome et al., USI](https://www.inf.usi.ch/faculty/lanza/PUBS/P/Rome2025a.pdf)).
- 2026 brought a new generation of wireframe languages written for Markdown and for AI agents to write: Wireloom,
  wiremd, TypMo, WireScript, Mockdown and Wiretext ([gihyo.jp](https://gihyo.jp/article/2026/02/text-based-wireframe-tools)).
  Salt is the established reference; these are where text wireframes are going.

## Appendix B: Salt and our draft language, compared

Salt as its documentation and community examples show it ([plantuml.com/salt](https://plantuml.com/salt),
[hello.nplant](https://mkoertgen.github.io/hello.nplant/2-salt-ui)); "no" means its documentation shows none.

| Capability | Salt | Our draft |
|---|---|---|
| Buttons, text inputs, checkboxes | yes | yes |
| Radio buttons | yes `( )` | missing |
| Dropdowns | yes `^…^` | yes (`select`) |
| A grid of cells, with line styles (`{#`, `{!`, `{-`, `{+`) | yes | a `grid`, without line styles |
| Tables (empty cells, spans) | yes | missing |
| Trees (`{T`) | yes | missing |
| Tabs | yes, also vertical | yes, horizontal |
| Menu bars and menus (`{*`) | yes, basic | missing |
| Scroll areas (`{S`) | yes | missing |
| Titled groups, separators | yes | missing |
| Text formatting | yes (Creole) | missing (could be Markdown's) |
| Named icons | yes (OpenIconic) | a generic `icon` |
| Device and window frames | no; a window's title bar is asked for on its forum | yes (`screen`) |
| Notes pointing at a part | no | yes, numbered |
| Image placeholders, placeholder text | no | yes |
| Button variants, password and placeholder inputs | partly | yes |
| A layout that adapts to the width | no: a fixed grid | yes, from CSS |
| Light and dark, text that is read and selected, printed | no: an image | yes: HTML |
| A grammar a renderer can comply with | no: defined by PlantUML's code | yes, in ABNF |

Salt is stronger on desktop applications' widgets (tables, trees, menus, scroll areas, group boxes); the draft on web
and phone screens and on saying what a screen does (frames, notes, placeholders, an adaptive layout, HTML). Neither
holds the other.

## Appendix C: adopting an existing language (option F)

Researched on 2026-11-17, from each project's repository and its npm record. The criteria, in order: maturity,
stability and the continuity of its development; and, where continuity is in doubt, what a fork of our own would cost
to keep.

| | Wireloom | wiremd (teezeit) | wiremd (akonan, the original) |
|---|---|---|---|
| Who | Stardock ([StardockCorp/Wireloom](https://github.com/StardockCorp/Wireloom)), one main committer, the company's founder; built for Stardock's own AI workspace | one person ([teezeit/wiremd](https://github.com/teezeit/wiremd)), a fork the original's README now calls the main project | one person ([akonan/wiremd](https://github.com/akonan/wiremd)) |
| First and last release on npm | 0.3.0 on 2026-04-19, 0.7.0 on 2026-05-31: seven in six weeks | `@eclectic-ai/wiremd` 0.3.0 on 2026-05-01, 0.6.1 on 2026-05-12 | 0.1.5 (README) |
| Last commit seen | 2026-05-30 | 2026-05-15 | 2026-05-03 |
| Months without a commit, as of 2026-11-17 | about 5½ | about 6 | about 6½ |
| Stars, forks | 75, 7 | 6, 1 | 110, 14 |
| Open issues, pull requests | 1, 1 | 15, 1 | 40, 24 |
| Stability | pre-1.0; minor releases may break its AST and its theme interface; it says to pin a version | pre-1.0; a breaking 0.4 with a migration script | pre-1.0 |
| Licence | MIT; "Wireloom" is Stardock's name | MIT | MIT |
| A formal grammar | no: rules and examples, errors with line and column | no | no |
| Output | an SVG string: its own layout engine and fixed metrics; light and dark themes; accessibility not addressed | HTML, React, Tailwind; several looks | HTML |
| Runs in a strict frame | likely: no runtime dependencies, a pure function, no scripts or external references in its SVG; eval and network not stated | no: a Node tool (`chalk`, `chokidar`, the unified and remark stack, Tabler's icons) | as the fork |
| Size | no dependencies, 10 files; about 120 KB raw, under 40 KB gzipped | 10 dependencies, 353 files, 5 MB unpacked | |
| Syntax | an indented outline, close to our draft: `window "Sign in":`, `button "Sign in" primary`; about 50 parts, among them tree, menubar, radio, chart, progress, and annotations pointing at parts | Markdown with marks: `[Sign in]*`, `[___]{type:password}`, `::: tabs` | as the fork |

**What it shows.** None of them is mature: all three started in April or May 2026, were built quickly (bursts of
dozens of commits a day, much of it with an AI assistant), are pre-1.0, and have had no commit for five to seven
months. Each rests on one person. wiremd's ownership is split between the original and the fork that took over, and
it is a Node tool that would not run in a strict frame. Wireloom is the strongest on every measure here: no
dependencies, a small pure function, an MIT licence, the broadest set of parts, and a syntax nearly the same as the one
we drafted; but it is a company's project built for its own product, idle since May, without a grammar, and drawing
SVG with its own layout engine, which is the part B avoided by drawing HTML.

**What a fork would cost.** Wireloom is the only one worth forking. Its fork would be its parser and its SVG layout
engine, in TypeScript with no dependencies: small enough to keep, but we would own a layout engine, its text metrics
and its look for good, and depend on its pre-1.0 AST.

**The ways it could go:**

- **F1, the pack wraps Wireloom**, pinned to one version, as the Mermaid pack wraps Mermaid, forked if it stays idle:
  the fewest tasks now (the pack and the guide), but SVG rather than HTML (no adaptive layout, text not selectable,
  accessibility added by us with `alt`), and a dependency on a project already idle.
- **F2, Wireloom's language, our renderer**: B's plan, with FIG-513 writing down Wireloom's language (the parts we
  need, in ABNF) instead of designing one, our own parser, and our HTML drawing. No runtime dependency on any of them,
  so their continuity doesn't matter; writers and AI agents that know Wireloom's syntax write ours; the gaps of
  Appendix B are mostly closed by Wireloom's parts. We would say "Wireloom's syntax" for compatibility, not use the
  name for the pack.
- **B as decided**: our own language, the gaps of Appendix B closed in its design.

**Recommended: F2.** By the criteria, depending on any of them at run time is the risk to avoid: none is mature, and
all three have stopped. F2 takes what is most valuable in the best of them, a language already thought through and
close to ours, without its continuity risk, at about B's cost.
