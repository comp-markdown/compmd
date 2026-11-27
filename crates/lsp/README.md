# compmd-lsp

The [CompMD](https://compmd.dev/) language server ([its design](https://github.com/comp-markdown/compmd/blob/main/docs/language-server.md)): one binary, `compmd-lsp`, which an editor starts and speaks LSP with over its standard input and output (`compmd-lsp --stdio`).

- Each open document has a worker of its own, which keeps the document's processor and applies the writer's edits to it incrementally; the diagnostics are published after each, with their code, severity, related places and a link to the spec's catalog.
- A document's root, configuration and packs are read as its hosts read them (`compmd-project`), so the editor shows what the site renders; its components are analysed too (W104, `compmd-components`).
- The settings are the editor's, under `compmd` (`workspace/configuration`, or `initializationOptions`): `packs.suggest`, `packs.network`, `preview.browser`, `trace`, the drawing's `render` (below), values as inlay hints' `render.values`, `markdown.enabled`, and the tables' `tables.alignOnType` and `tables.alignOnFormat` (both `true`).
- A change to the configuration, a package, a lock file or an imported file reloads the open documents.
- Completion, hover, signature help and go to definition, from the processor's hints (`hint_at`, `signature_at`, `definition_at`): components, params and their values, slots, frontmatter keys, types, `md.` paths, and functions with their documentation from the processor's catalog; a component of the project's folder or installed packs completes with the import that takes it, in the frontmatter's own syntax, and so does a code block's language one of their code renderers renders; on a code block whose language none the document names renders, the quick fix *Import `<pack>` to render this block*.
- Links and headings ([its design](https://github.com/comp-markdown/compmd/blob/main/docs/language-server.md#links-and-headings), decision 15), in CompMD and plain Markdown files alike: go to definition on a link (its file, the heading its `#anchor` names, a reference's definition), references (on a heading, every link to its anchor across the project; on a reference or a definition, its label's uses), a hover on a link (the target's section, its first 12 lines), and workspace symbols (`workspace/symbol`: every file's headings, the query's characters in order, any case). A heading's anchor is the expanded tree's (spec §12.3). They come from a **project index** per root: its Markdown files' headings and links, read in the background when its first document opens, skipping `node_modules`, hidden folders and what `.gitignore` names, at most 10,000 files of at most 1 MB; an open document's entry follows its edits, and a file changed on the disk is read again. `compmd.links.enabled` (`true`) set to `false` switches them off.
- Document symbols (headings, containers and their slots, leaves, definitions, nested as the document nests them), folding (the frontmatter, containers, definitions, code blocks), and `compmd/blocks`, a request of CompMD's own: each code block a code renderer renders, with its language, its renderer and the range of its text, for an editor package to hand to the writer's server for that language.
- Semantic tokens (`textDocument/semanticTokens/full` and `/range`) by the token names every CompMD grammar shares (`compmd-grammars`' `syntax/tokens.json`), from the processor's `tokens()`; a name that stands for no component has the modifier `unknown`.
- Values as inlay hints (`textDocument/inlayHint`): beside each interpolation and expression param, the value the last render gave it, from the processor's `evaluations()` ([its design](https://github.com/comp-markdown/compmd/blob/main/docs/editor-rendering.md), decision 9); below.
- The drawing plan (`compmd/drawing`, a request of CompMD's own): what an editor draws in place of, or beside, a document's text, for a renderer that knows no Markdown or CompMD ([its design](https://github.com/comp-markdown/compmd/blob/main/docs/editor-rendering.md)); below.
- Tables aligned in the text ([its design](https://github.com/comp-markdown/compmd/blob/main/docs/editor-rendering.md), decision 7), below: by formatting (`textDocument/formatting`, and `rangeFormatting` for the tables a range meets), and as they are typed (`textDocument/onTypeFormatting`, on `|` and a newline); and changed by the code actions on a table (`refactor.rewrite`).
- Markdown files ([its design](https://github.com/comp-markdown/compmd/blob/main/docs/editor-rendering.md), decision 13): a `.md` file is read as the spec reads it (section 2). With an `md` key in its frontmatter it is CompMD, served as a `.comp.md` file is. Without one it is CommonMark and GFM: `:::`, `{…}` and `{{…}}` stay text, and it has Markdown's features only: the drawing, tables aligned and their code actions, folding, symbols, and the preview, which renders it as CommonMark and GFM; no CompMD diagnostic, completion, hover or inlay hint. Its one other code action, *Read as CompMD* (`refactor.rewrite`), adds `md.version: 1` to its frontmatter, or a frontmatter with it. `compmd.markdown.enabled` (`true`) set to `false` switches `.md` files off: no answers and no diagnostics, live; `compmd.render.markdownFiles` (`true`) set to `false` switches off only their drawing.
- Positions are counted in UTF-32 or UTF-8 when the editor offers them, else UTF-16.
- The preview: the command `compmd.preview` (`workspace/executeCommand`, the document's URI its argument) and the code action *Open preview* open the document's page in the browser, the system's or `preview.browser`. The server serves it on `127.0.0.1` under a random token, the frames on a port of their own, and sends it each change; its page is `@compmd/devtools`'s preview, built into `preview/` (`pnpm --dir npm-packages/devtools/preview build`) and embedded by `build.rs`. A build without it says so when the preview is asked for.

## The drawing plan

What an editor's renderer implements to draw a document in place ([a document drawn in the editor](https://github.com/comp-markdown/compmd/blob/main/docs/editor-rendering.md), decisions 1 to 6 and 11). The server makes the plan from the processor's tree; the renderer applies its marks with the editor's own API, and reveals them.

### Capabilities

The client declares the drawing in `initialize`, in its capabilities' `experimental`: the primitives it draws, and whether it takes refreshes. A name that is no primitive is let go.

```json
{ "capabilities": { "experimental": { "compmd": { "drawing": { "primitives": ["hide", "replace", "insert", "line", "prefix", "style"], "refreshSupport": true } } } } }
```

The server then answers `"experimental": { "compmd": { "drawing": {} } }` in its capabilities. A client that declares nothing gets neither, is sent no drawing message, and a plan it asks for anyway has no marks.

### `compmd/drawing`

A request from the client: the document and the range it shows (with some lines around it), asked on scrolling and on editing.

```json
{ "textDocument": { "uri": "file:///notes/harbour.comp.md", "version": 4 }, "range": { "start": { "line": 0, "character": 0 }, "end": { "line": 60, "character": 0 } } }
```

The answer is the plan of the lines of the range, its marks sorted by where they start, every range and position in the encoding agreed at `initialize`:

```json
{ "version": 4, "marks": [ … ] }
```

The plan is always of the text as the server holds it, which has every change the client sent before the request; `version` says which. A renderer drops a plan whose version is older than its buffer's; the request's own version isn't checked. A document that isn't open is answered with `null`.

### `compmd/drawing/refresh`

A request from the server, with no params and no result, as `workspace/inlayHint/refresh` is: the client asks again for the plans of the documents it draws. It is sent only to a client that declared `refreshSupport`, when the drawing settings change, and after a change that moved marks beyond the lines it edited: the lines of an element that spans lines (a container from its opener to its closer, a fenced code block, a table, a block quote, a setext heading, the frontmatter) aren't where they were moved by the lines the change added or removed, or a table's header's pipes moved (a fence opened, a closer typed, a column widened). It may be sent more often than needed.

### Marks

A mark is one primitive, named by its `kind`. Lines count from 0. `reveal` is the lines the mark gives way on: while the cursor is on one of them, the renderer shows the raw text instead (decision 5). A mark that only decorates (a `style`, a `prefix`, an `insert` of a box's side) has no `reveal` and never gives way.

```json
{ "kind": "hide", "range": { "start": { "line": 4, "character": 0 }, "end": { "line": 4, "character": 7 } }, "reveal": { "first": 4, "last": 4 } }
{ "kind": "replace", "range": { "start": { "line": 0, "character": 0 }, "end": { "line": 0, "character": 2 } }, "text": "◎", "style": "compmd.heading.2", "reveal": { "first": 0, "last": 0 } }
{ "kind": "insert", "position": { "line": 3, "character": 2 }, "text": "│ ", "style": "compmd.container.border" }
{ "kind": "line", "line": 4, "above": true, "segments": [{ "text": "rust", "style": "compmd.code.language" }], "fill": { "text": " ", "style": "compmd.code" }, "reveal": { "first": 4, "last": 4 } }
{ "kind": "prefix", "lines": { "first": 8, "last": 12 }, "text": "│ ", "style": "compmd.container.border" }
{ "kind": "style", "range": { "start": { "line": 6, "character": 0 }, "end": { "line": 8, "character": 3 } }, "style": "compmd.code", "wholeLine": true }
```

- `hide`: the range isn't shown. A range from a line's start to the next line's start hides the whole line.
- `replace`: the range is shown as `text`.
- `insert`: `text` at `position`, shifting what follows.
- `line`: a line of its own above (or below) `line`: its `segments` in turn, then, when there is a `fill`, its text repeated to the window's edge. A `line` above a line that a `hide` hides whole stands in that line's place: a container's borders, a rule.
- `prefix`: `text` at the start of each of `lines`, wrapped lines too. Where a `line` is on one of those lines, the server puts the prefix's text first in its segments.
- `style`: the range in `style`, to the window's edge when `wholeLine`.

Each mark's text is plain characters from the glyph set; a style is a name, never a colour: the renderer maps it to its highlight group, face or theme colour. The styles: `compmd.heading.1` to `compmd.heading.6`, `compmd.code`, `compmd.code.language`, `compmd.code.inline`, `compmd.rule`, `compmd.bullet.1` to `compmd.bullet.4`, `compmd.checkbox.checked`, `compmd.checkbox.unchecked`, `compmd.quote.1` and deeper, `compmd.callout.note` (`tip`, `important`, `warning`, `caution`), `compmd.table.border`, `compmd.container.border`, `compmd.container.top`, `compmd.container.divider`, `compmd.container.bottom` (a container's opener, slot line and closer drawn as borders on their own lines, for a client without `line`), `compmd.component`, `compmd.slot`, `compmd.param` and `compmd.frontmatter.md` (set back).

### What is drawn

| Element | With every primitive | Without one |
|---|---|---|
| ATX heading | the `#`s `replace`d by `heading.N`; the line `style`d to the edge (`heading.fill`) | no `replace`: the `#`s stay |
| setext heading | `heading.N` `insert`ed before the text; the underline hidden; the lines styled | |
| fenced code block | the fences hidden; the language on a `line` above (`code.language: above`), its icon first with `nerd`; the lines in `compmd.code` to the edge | no `line`, or `code.language: fence`: the opening fence `replace`d by the language; no `replace` either: the fence's marks hidden, the language left |
| inline code | the span styled | |
| thematic break | hidden whole, a `line` in its place filled with `rule` | no `line`: its text `replace`d by as many `rule` glyphs, the line styled |
| list item | a bullet `replace`d by `bullet.N` by depth (four, then again); a number stays | no `replace`: the bullet styled |
| task item | `[ ]` and `[x]` `replace`d by `checkbox.unchecked` and `checkbox.checked` | no `replace`: styled |
| block quote | each line's `>` `replace`d by `quote`, in `compmd.quote.N` by depth | no `replace`: styled |
| GitHub alert | `[!NOTE]` `replace`d by its icon and title (`ℹ Note`); its `>`s in its callout style | no `replace`: styled |
| table | each `\|` `replace`d by `│`, the delimiter row by `├─┼─┤`, a border `line` above and below (`table.borders`); every mark reveals the whole table | no `line`: no borders; no `replace`: the pipes and the delimiter row styled |
| container | the opener hidden whole, a top border `line` in its place titled with its name and params (`┌─ box {padding=m} ───`); each slot a divider (`├─ footer ───`); the closer a bottom border (`└────`); the lines between with `│ ` (`prefix` at a line's start, `insert` at the box's column further in) | no `line`: the marker `replace`d by the corner, the name and params styled, the line in `compmd.container.top`, `divider` or `bottom`; no `prefix`: the side `insert`ed on each line |
| leaf | its name in `compmd.component`, its params (with their continuation lines) set back | |
| frontmatter | its `md` key set back | |

Images, math, HTML, component definitions and link icons are left as written.

### Settings

Under `compmd.render`, applied at the next request, a change of them followed by a refresh:

| Setting | Default | |
|---|---|---|
| `enabled` | `true` | documents drawn at all |
| `heading`, `code`, `table`, `list`, `checkbox`, `quote`, `callout`, `rule`, `container`, `component`, `param` | `true` | each element's switch: `true`, `false` or its options, which turn it on |
| `heading.fill` | `true` | a heading's line in its style to the edge |
| `code.language` | `above` | `above`, `fence` or `off` |
| `table.borders` | `true` | a border above and below a table |
| `glyphs` | `unicode` | `unicode` (`─ │ ┌ ● ☐ ☑`), `nerd` (Nerd Font icons for headings, languages and callouts) or `ascii` |
| `glyph` | | the writer's own text for a glyph, by its name |
| `reveal` | `line` | `line` (a mark's part: a heading's line, a fence's, a table's lines), `element` (the whole element) or `off` (no mark gives way) |
| `markdownFiles` | `true` | `.md` files drawn; off, their plan has no marks, for a writer who keeps another plugin's drawing (*Markdown files*, above) |
| `values` | `true` | values as inlay hints ([below](#values-as-inlay-hints)); a change of it is followed by `workspace/inlayHint/refresh` |

`render.modes` (the modes a renderer draws in) is the renderer's own, read from the same namespace; the server ignores it.

The glyphs' names: `heading.1` to `heading.6`, `bullet.1` to `bullet.4`, `checkbox.unchecked`, `checkbox.checked`, `quote`, `callout.note` (`tip`, `important`, `warning`, `caution`), `rule`, `box.horizontal`, `box.vertical`, `box.top-left`, `box.top`, `box.top-right`, `box.left`, `box.cross`, `box.right`, `box.bottom-left`, `box.bottom`, `box.bottom-right`, and `language.<name>` (a code block's language, by its info string: `language.rust`), which only `nerd` has.

## Values as inlay hints

Beside each interpolation and each expression param of a document, the value the processor's last render gave it ([a document drawn in the editor](https://github.com/comp-markdown/compmd/blob/main/docs/editor-rendering.md), decision 9): `# {{md.doc.title}} = Harbour notes`, `{tone=(done ? "success" : "neutral")} = success`. They are on by default; `compmd.render.values` turns them off, and the server then answers with none.

- **Where:** after the interpolation's `}}` or the expression's `)`, with `paddingLeft`, and no `kind` (LSP's kinds are a type and a parameter; without one, the client falls back to its default). Only those whose place is in the range asked for.
- **The label:** `= ` and the value as interpolation writes it into text (spec §8.1): a string as itself, a number as JavaScript's `String(n)` (`12.5`, `1e+21`), `true` or `false`, a record with a string `name` as its name, and `null` as nothing, `=` alone. A list is its size, `= [3 items]`, and another map its keys', `= {2 keys}`. A value longer than 40 characters is cut to 40, ending in `…`; a line break in it is a space.
- **The tooltip:** what the label leaves out: the whole text of a value cut short, a list's or a map's JSON, `null`; at most 1000 characters.
- **Evaluated more than once** (in an `each`, a template slot, a component's definition, or at a responsive body's breakpoints): the first value and how many times, `= Ana, … (3)`.
- **None** for an evaluation that failed (its diagnostic says why: one failed iteration is enough), for what the render didn't reach (the branch an `if` didn't take), for what another document or an example evaluates, and for the interpolations of link and image destinations.
- **Refreshes:** `workspace/inlayHint/refresh`, only to a client whose `workspace.inlayHint.refreshSupport` is `true`, when `render.values` changes and, while they are shown, when an edit or a reload changes the values the document's hints show (a frontmatter key, a data file), not when it only moves them.

## Tables

A GFM table is read from the processor's tree, so its cells are where GFM puts them: an escaped pipe (`\|`) is a cell's text, and a pipe in a code span splits cells, as GitHub shows them.

- **Aligned:** each cell is padded with spaces to its column's width, the widest cell's on screen (an East Asian wide character takes two columns) and at least three; a column aligned right has its cells' spaces on their left, a centred one on both sides; the delimiter row's dashes are as wide, its colons kept; the pipes line up. Only spaces change, and the delimiter row's dashes. A row keeps what is before it on its line (a list's indentation, a quote's `>`), its outer pipes as it writes them (a table without them gets none), and the cells it has: a missing cell, which GFM shows empty, isn't added. The edits are only of the lines that change, a run of spaces at a time, so a cursor keeps its place.
- **On type:** a `|` typed in a table aligns it; a newline, the table on the line it ended. `tables.alignOnType` switches it off, `tables.alignOnFormat` the formatting's.
- **Code actions**, at the cursor's cell, each leaving the table aligned: *Insert a row above* or *below* (not above the header), *Delete the row* (a body row), *Insert a column before* or *after*, *Delete the column* (when there is another), *Move the column left* or *right*, *Align the column left*, *center* or *right*, *Remove the column's alignment*. None of a row's on the delimiter row. A column's change gives every row a cell in each column first, and an empty cell at a row's end gets a pipe there.

## Status

In development, released with CompMD's packages at the same version. It isn't published on crates.io.

**Versions:**
- **0.11.0:** the first version: documents, their diagnostics, settings, watched files, semantic tokens, completion, hover, signature help and go to definition.
