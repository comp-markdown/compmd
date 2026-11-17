# Printing a document

- **Status:** Proposed, for review (2026-11-17). Nothing is built until it is approved.
- **Decides:** what a CompMD document looks like on paper, and in a PDF printed from a page: its components, its colours, its layout, and what a processor without a browser prints.
- **Related:** spec §7.4 (`scheme`, `busy`), §14 (degradation), [writing a pack](writing-a-pack.md) (S8), [running components in the sandbox](running-components-in-the-sandbox.md) (decision 8, a view that has settled).

## Where it stands

- `@compmd/render` and `@compmd/sandbox` read the colour scheme again on `beforeprint` and `afterprint`, so components redraw for paper and back ([writing a pack](writing-a-pack.md), S8). What the scheme then is, nothing says: a dark page prints dark unless the host's print styles change it.
- A view says when its page has settled (`view.settled()`, `data-compmd-settled`), which a host that prints waits for.
- Nothing else is printing-specific: no print styles in the renderer, nothing about frames on paper, nothing in the spec. The site's PDFs (the guides) are printed from prerendered pages, with no live components.

## Decisions

1. **Paper is light.** When printing, the renderer shows the page, and its components, in the light scheme, whatever the screen shows: `@media print { .compmd-page { color-scheme: light } }` in its own stylesheet, which a host's print styles can override. Components follow through `scheme` (spec §7.4), as they do for any change.
2. **A component prints as it is drawn.** Browsers print a frame's content where it is on the page (to be confirmed for sandboxed cross-origin frames in both browsers by the tests below), and the sandbox sizes each frame to what it shows, so nothing scrolls inside it and nothing is cut. A component draws itself for paper only if it wants to, from `scheme` and its own print styles; the processor asks nothing more of it.
3. **A host that prints waits for the page to settle.** `view.print()` in `@compmd/sandbox`, and `page.print()` in `@compmd/render`: the light scheme set, the page settled (every frame redrawn and sized, every `busy` drawing done), then the browser's print. A reader's own ⌘P can't wait: the browser prints at once, so a frame still redrawing for the light scheme prints as it was. Hosts that offer printing offer their own button for it.
4. **Breaks fall between blocks, not through them.** On paper: a component's box, a table, a code block, an image and a figure are kept whole (`break-inside: avoid`) when they fit on a page; a heading stays with what follows it (`break-after: avoid`). Breakpoints follow the box's width on paper, as on screen, since they are container queries.
5. **Links print as they show.** A link prints its text, as on screen; the renderer adds no URLs after links. A host that wants them adds them in its own print styles (`a[href^="http"]::after`).
6. **A processor without a browser prints the page it can make.** One that renders to HTML on a server, or exports a static page, has no sandbox to run host-language components: it prints their fallbacks, as any renderer that can't run them does (spec §14), and `compmd` components, which run no code, as they render.
7. **The spec says it in two sentences** (§14, *Printing*, informative): a processor that prints shows the document as it renders it, in a light scheme on paper unless the host says otherwise, after its components have drawn; one that can't run a component prints its fallback. No conformance cases: printing is rendering, which the conformance set doesn't cover (spec §13.2).

## Open questions

| # | Question | Notes |
|---|---|---|
| Q1 | Should a component be able to **print its fallback** instead, an interactive map or a video that means nothing on paper? | For example `print: fallback` in its signature (spec §7.3), or a class on its box the host's styles use. Without it, such a component prints as drawn. |
| Q2 | Is **light paper** right for every host, or should the default follow the page? | Light is what readers expect on paper and saves ink; a host printing to a dark PDF overrides it. |
| Q3 | Do we offer **`print()` on the view and the page**, or only document the wait (`await view.settled()`, then `print()`)? | The method keeps the order right (scheme, then settle, then print); the documentation alone is less to keep. |
| Q4 | Should an **input component** print its current value as text rather than as a control? | A slider on paper shows its knob, not its number. |

## Tests

- Print to PDF in Chrome (Playwright's `page.pdf`) a page with a component in a frame, a dark page, a long table and a heading at a page's end: the frame's content is in the PDF, light; the table and the code block aren't split; the heading isn't left at a page's foot.
- WebKit can't print to PDF through Playwright, so it is checked by the print media: `page.emulateMedia({ media: "print" })`, then the scheme, the frames and the breaks' styles as WebKit computes them.
- `view.print()` sets the light scheme and calls `print` only after the page has settled.
