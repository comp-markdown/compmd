# Reading CompMD with pandoc (proposed)

Pandoc turns Markdown into anything it writes: PDF through LaTeX or Typst, Word, EPUB, HTML. A CompMD document is more
than Markdown it can read: imports, definitions, `if` and `each`, interpolation and layout mean something only once the
processor has expanded them (spec §12.3). This decides how pandoc reads a CompMD document: through the processor, never
a second parser of the syntax, so pandoc gets what the spec says.

## Decisions

1. **The bridge is the expanded tree, as pandoc's JSON.** The processor expands the document as for any render, and a
   new crate, `compmd-pandoc`, writes its expanded tree as pandoc's JSON AST (pandoc-types' API version 1.23). It reads
   no CompMD itself, so every construct pandoc sees is the processor's.

2. **Three ways in, one bridge.**
   - **A command:** `compmd pandoc doc.comp.md | pandoc -f json -o doc.pdf`, a small `compmd` binary (a new crate,
     `compmd-cli`, so the processor crate stays a library).
   - **A pandoc reader:** `pandoc -f compmd.lua doc.comp.md -o doc.pdf`, a custom reader in Lua that runs the command
     and reads its JSON, shipped with the binary.
   - **For JavaScript hosts:** the same bridge in `@compmd/processor` (`toPandoc(expanded)`), later, when a host asks.

3. **How the tree maps.** Most of Markdown is one to one; what CompMD adds becomes what pandoc's writers and filters
   already understand:

   | In the expanded tree | In pandoc's AST |
   |---|---|
   | headings, paragraphs, lists, tables, code, math, footnotes, links, images | the same nodes |
   | alerts (`> [!NOTE]`) | a `Div` with the class `note`, its title a first paragraph, as pandoc's own GFM reader makes them |
   | `box` and the layout built-ins | a `Div` with the class `compmd-box` and its params as attributes (`gap=m`), so a filter or a template styles it; columns stay one after the other |
   | a `compmd` component | what it expands to; its box a `Div` with the component's name as a class |
   | a host-language component | its fallback (spec §4.5): pandoc runs no code; a container's slots as Markdown |
   | an input component | its current value, as a processor that isn't interactive shows it (spec §8.7) |
   | a code block a renderer draws (spec §7.7) | the code block, with its `alt` as a caption |
   | wiki links | links, by the host's `wikiLink` rule, or their text |
   | the frontmatter | pandoc's metadata (`title`, `author`, `date` and the rest), without `md` |

4. **What the request carries: what any processor's does.** The document's folder gives its files and data; packs come
   from a folder (`--packs dir`, a pack store's layout) and, with `--registry`, from npm through `compmd-host`'s store;
   remote data only from origins given with `--allow-origin`, since there is no reader to ask; inputs with
   `--input name=value`.

5. **Diagnostics go to stderr**, as the processor writes them, with the line and column; `--strict` fails on any F
   (spec §10.1), and the JSON is still written without it.

6. **Testing:** every expanded tree of the conformance set through the bridge, checked against pandoc's JSON schema;
   and a few documents, the complete example among them, to PDF (with Typst, small to install in CI) and DOCX, read back
   by pandoc and compared as text.

## What it changes

- New crates `compmd-pandoc` (the bridge) and `compmd-cli` (the `compmd` binary, with `pandoc` its first command), and
  `compmd.lua`.
- No change to the spec or the processor: the bridge reads the expanded tree, which the spec defines.

## Open

- Whether layout should map to pandoc's own constructs where it can (a two-column `row` as LaTeX columns through a
  filter shipped with the reader), or stay `Div`s for the template to style (proposed: `Div`s, and a filter later).
- The binary's name, `compmd`, which the crate's name already is; a binary of that name is what users would type.
