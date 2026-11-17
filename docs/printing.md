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
2. **A component prints as it is drawn.** Browsers print a frame's content where it is on the page (to be confirmed for sandboxed cross-origin frames in both browsers by the tests below), and the sandbox sizes each frame to what it shows, so nothing scrolls inside it and nothing is cut. A component draws itself for paper only if it wants to, from `scheme` and its own print styles; the processor asks nothing more of it. An input component prints as it shows, its control drawn as on screen: what is visible is what prints, and a component that wants its value as text on paper draws it so.
3. **A host that prints waits for the page to settle.** `view.print()` in `@compmd/sandbox`, and `page.print()` in `@compmd/render`: the light scheme set, the page settled (every frame redrawn and sized, every `busy` drawing done), then the browser's print. A reader's own ⌘P can't wait: the browser prints at once, so a frame still redrawing for the light scheme prints as it was. Hosts that offer printing offer their own button for it.
4. **Breaks fall between blocks, not through them.** On paper: a component's box, a table, a code block, an image and a figure are kept whole (`break-inside: avoid`) when they fit on a page; a heading stays with what follows it (`break-after: avoid`). Breakpoints follow the box's width on paper, as on screen, since they are container queries.
5. **Links print as they show.** A link prints its text, as on screen; the renderer adds no URLs after links. A host that wants them adds them in its own print styles (`a[href^="http"]::after`).
6. **A processor without a browser prints the page it can make.** One that renders to HTML on a server, or exports a static page, has no sandbox to run host-language components: it prints their fallbacks, as any renderer that can't run them does (spec §14), and `compmd` components, which run no code, as they render. A headless browser is a browser: a converter (a Pandoc writer, a PDF export) that runs `@compmd/render` and `@compmd/sandbox` in headless Chrome prints every component as drawn, frames included, as this site's PDFs are printed. A DOM without a browser (happy-dom, jsdom) has no frames or policies, so it is the case above. A Pandoc reader needs no renderer: it maps the expanded tree (spec §12.3), with fallbacks for host-language components.
7. **A component can print its fallback instead.** `print: fallback` in its signature (spec §7.3), for a component that means nothing on paper, an interactive map or a video: on paper its use shows its fallback, as a renderer that can't run it shows it. The default, `print: drawn`, prints it as drawn (decision 2). It is in the signature, not in a host's styles, so the pack's author says it once for every host. A spec change: a new signature key, its two values, and a diagnostic for any other value, with conformance cases for the key's validation (rendering itself isn't covered, spec §13.2).
8. **The spec says the rest in two sentences** (§14, *Printing*, informative): a processor that prints shows the document as it renders it, in a light scheme on paper unless the host says otherwise, after its components have drawn; one that can't run a component prints its fallback. No conformance cases: printing is rendering, which the conformance set doesn't cover (spec §13.2).

Reviewed on 2026-11-17: light paper (decision 1), `print()` on the view and the page (decision 3), inputs printed as they show (decision 2) and `print: fallback` (decision 7) were the open questions, decided as written here.

## Tests

- Print to PDF in Chrome (Playwright's `page.pdf`) a page with a component in a frame, a dark page, a long table and a heading at a page's end: the frame's content is in the PDF, light; the table and the code block aren't split; the heading isn't left at a page's foot.
- WebKit can't print to PDF through Playwright, so it is checked by the print media: `page.emulateMedia({ media: "print" })`, then the scheme, the frames and the breaks' styles as WebKit computes them.
- `view.print()` sets the light scheme and calls `print` only after the page has settled.
- A use of a component with `print: fallback` prints its fallback, in both modes; the processor checks the key's value.
