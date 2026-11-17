# Reading CompMD with pandoc (proposed)

Pandoc turns Markdown into anything it writes: PDF through LaTeX or Typst, Word, EPUB, slides, other markups. A CompMD
document is more than Markdown it can read: imports, definitions, `if` and `each`, interpolation and layout mean
something only once the processor has expanded them (spec §12.3), and its components draw what pandoc can't. This
decides how pandoc reads a CompMD document: through the processor and the renderer, never a second parser of the
syntax, so pandoc gets what the spec says, and each component as it prints.

## What it is for

- **A PDF of a report or a book**, through LaTeX or Typst: book typography, a contents with page numbers, a template of
  the writer's or their organisation's.
- **Word, to share with people who don't use CompMD**, or to hand a draft to an editor who works in Word.
- **EPUB**, for e-readers: the guides as books, a long document read offline.
- **Slides** (Beamer, reveal.js), from a document whose sections are slides.
- **Other markups** pandoc writes (AsciiDoc, reStructuredText, MediaWiki, Org): moving a document into another wiki or
  docs system.
- **A docs pipeline** that already runs pandoc (with its filters, citations through `--citeproc`, templates): CompMD
  documents become one more input.
- **Plain text and accessible output**: what the document says, without the page, for a screen reader or an archive.

## Decisions

1. **Components are taken as they print.** A component draws itself on paper (docs/printing.md), and that is what
   pandoc gets: the renderer renders the document in a headless browser, as for printing (light, settled), and each
   leaf or inline host-language component becomes a picture of its box, an SVG when it draws one (charts stay vector
   in a PDF), a PNG otherwise, with its fallback as the picture's alt text. A component whose signature says
   `print: fallback` gives its fallback, as on paper. Without a browser (`--no-draw`), every host-language component
   gives its fallback (spec §4.5), as any processor that can't run it does.

2. **The bridge is the expanded tree, as pandoc's JSON.** The processor (`@compmd/processor`, its WebAssembly) expands
   the document as for any render, and the bridge writes the expanded tree as pandoc's JSON AST (pandoc-types' API
   1.23), with the pictures of decision 1 in place of the components. It reads no CompMD itself.

3. **How the tree maps.** Most of Markdown is one to one; what CompMD adds becomes what pandoc's writers and filters
   already understand:

   | In the expanded tree | In pandoc's AST |
   |---|---|
   | headings, paragraphs, lists, tables, code, math, footnotes, links, images | the same nodes |
   | alerts (`> [!NOTE]`) | a `Div` with the class `note`, its title a first paragraph, as pandoc's GFM reader makes them |
   | `box` and the layout built-ins | a `Div` with the class `compmd-box` and its params as attributes (`gap=m`), so a filter or a template styles it |
   | a `compmd` component | what it expands to; its box a `Div` with the component's name as a class |
   | a host-language leaf or inline component | its picture as it prints (decision 1), or its fallback |
   | a host-language container | a `Div` with its name as a class, holding its slots as Markdown: their text stays text, and the components in them are taken one by one |
   | an input component | its current value, as it prints |
   | a code block a renderer draws (spec §7.7) | the renderer's picture, with its `alt`; the code block with `--no-draw` |
   | wiki links | links, by the host's `wikiLink` rule, or their text |
   | the frontmatter | pandoc's metadata (`title`, `author`, `date` and the rest), without `md` |

4. **Two ways in, one bridge.** A command, `compmd-pandoc doc.comp.md | pandoc -f json -o doc.pdf`, and a pandoc reader,
   `pandoc -f compmd.lua doc.comp.md -o doc.pdf`, a custom reader in Lua that runs the command and reads its JSON.

5. **What the request carries: what any processor's does.** The document's folder gives its files and data; packs come
   from npm through `@compmd/registry` (its cache on disk), or a folder (`--packs dir`); remote data only from origins
   given with `--allow-origin`, since there is no reader to ask; inputs with `--input name=value`. Diagnostics go to
   stderr with their line and column; `--strict` fails on any F (spec §10.1).

6. **Where it lives: a repository of its own, `comp-markdown/compmd-pandoc`.** It is a tool built on CompMD's released
   packages, not part of them: an npm package (`@compmd/pandoc`, with the `compmd-pandoc` command) depending on
   `@compmd/processor`, `@compmd/render`, `@compmd/sandbox` and `@compmd/registry` by version, Playwright for the
   headless browser, and the Lua reader beside it. Its own releases follow pandoc's and CompMD's as they need, and its
   tests run against the released packages, which keeps the bridge honest about what they offer.

7. **Testing:** every expanded tree of CompMD's conformance set through the bridge, checked against pandoc's JSON
   schema; and a few documents, the complete example among them, to PDF (with Typst, small to install in CI) and DOCX,
   read back by pandoc and compared as text, with a component drawn and one with `print: fallback`.

## What it changes in this repository

Nothing but this design and a link from the README: the bridge is built in its own repository, on the released packages.

## Open

- Whether layout should map to pandoc's own constructs where it can (a two-column `row` as columns through a filter
  shipped with the reader), or stay `Div`s for the template to style (proposed: `Div`s, and a filter later).
- The size at which a component's picture is taken: the page's width for the format (proposed: the text width of a
  7 × 10 in page, as the site's books, with `--width` to change it).
