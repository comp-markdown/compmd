# A document drawn in the editor

- **Status:** Approved (2026-11-27, CR-171). Its tasks: CMD-438 to CMD-448 (T1 to T11, below).

This document designs how CompMD's language server and editor packages draw a document in place, in Markdown and
CompMD files alike, in every editor as far as the editor allows. Headings, code blocks, tables, lists, callouts, and
CompMD's containers and components are drawn while the text stays as written. The raw text comes back where the
cursor is. It decides the messages between the server and the editors, what each editor draws, the settings, the
first version's elements, and the features that come with it: tables aligned as they are typed, and links and
headings. It ends with the tasks that build it.

## Context

render-markdown.nvim draws Markdown this way in Neovim, and the aim was the same for CompMD. It can't be layered on
CompMD: it runs only on trees whose language is named `markdown` or `markdown_inline` (its handlers, its nesting check
and its queries name them), and CompMD's grammars are `compmd` and `compmd_inline`. Drawing it ourselves, from the
language server, lets Emacs and VS Code draw the same, draws what only CompMD has, and serves plain Markdown files
too, which no tool today draws the same in two editors.

## Terms

- **Drawing**: what an editor shows in place of, or beside, a document's text, without changing the text. The text on
  disk and in the buffer stays as written.
- **Drawing plan**: the language server's answer for a range of a document: a list of **marks**, each one primitive
  (below) at a range, with a style and the lines it gives way on.
- **Primitive**: one thing an editor can draw. There are six: `hide`, `replace`, `insert`, `line`, `prefix`, `style`
  (decision 2).
- **Style**: a name for what a mark is (`compmd.heading.2`, `compmd.container.border`), which each editor maps to its
  own highlight group or face. Never a colour.
- **Renderer**: the part of an editor's package that asks for the plan and draws its marks with the editor's own API.
- **Revealing**: showing a mark's raw text again while the cursor is on its lines.

## Decisions

1. **The language server makes the plan; each editor only draws it.** The server has the processor's tree, so it
   knows each heading, table, list, callout, container and param, and where each part of its syntax is. It computes
   the plan in Rust, once, tested there. A renderer has no Markdown or CompMD knowledge: it applies marks. The logic
   repeated per editor is the six primitives, the style table and revealing.

2. **Six primitives, which every editor that draws can map onto its own API:**

   | Primitive | What it draws | Neovim | Emacs | VS Code |
   |---|---|---|---|---|
   | `hide {range}` | the range not shown | extmark `conceal = ""` (`conceal_lines` for whole lines) | `invisible` property | CSS in `textDecoration` (decision 8) |
   | `replace {range, text, style}` | the range shown as `text` | `conceal` + `virt_text_pos = "inline"`, or `overlay` when the widths match | `display` string | hidden range + `before` attachment |
   | `insert {position, text, style}` | `text` at a position, shifting what follows | `virt_text_pos = "inline"` | `before-string` / `after-string` | `before` / `after` attachment |
   | `line {line, above, segments, fill}` | a line of its own, above or below, optionally filled to the window's width with a character | `virt_lines` (`virt_lines_above`) | `before-string` / `after-string` ending in a newline | none in the stable API: falls back (decision 3) |
   | `prefix {lines, text, style}` | `text` at the start of each line, wrapped ones too | inline `virt_text` at column 0 (`virt_text_repeat_linebreak`) | `line-prefix` / `wrap-prefix` | `before` attachment on each line |
   | `style {range, style, whole_line}` | the range in a style; to the window's edge when `whole_line` | `hl_group` (`hl_eol`, `line_hl_group`) | face (`:extend`) | decoration type (`isWholeLine`) |

   A mark's text is plain characters; the server picks them from a glyph set (decision 6). A width the server can't
   know (the window's) is a `fill`, which the renderer draws to the edge.

3. **The editor says which primitives it draws, and the server plans for those.** Both declare the feature in
   `initialize`, in the capabilities' `experimental` (LSP 3.18: it "can be used to pass experimental capabilities under
   development"): the client its primitives and whether it takes refreshes
   (`capabilities.experimental.compmd.drawing: { primitives, refreshSupport }`), the server that it draws
   (`ServerCapabilities.experimental.compmd.drawing`). A client that declares nothing is never sent a drawing message.
   Where a primitive is missing, the server draws the
   element another way or leaves that part out: without `line`, a code block's language goes on its opening fence as
   a `replace`, and a container's top border becomes a `style` with a border on its opening line. A renderer never
   works around a missing primitive itself.

4. **The plan is asked for, by range: `compmd/drawing`.** A request of the server's own, as `compmd/blocks` is (the
   protocol allows custom methods; `$/` is kept for the protocol's own), with the document as a
   `VersionedTextDocumentIdentifier` and the visible range (a renderer asks for what is on screen, with some lines
   around it); the answer is the plan, its marks sorted by position, every range and position in the encoding agreed
   at `initialize`, as the server's other answers are. A plan for an older version than the buffer's is dropped. The
   server asks the editor to ask again as the protocol's own refreshes do (`workspace/semanticTokens/refresh`,
   `workspace/inlayHint/refresh`): a request from the server, `compmd/drawing/refresh`, with no params and no result,
   sent only to a client that declared `refreshSupport`, after a change that moves marks beyond the edited lines (a
   table's column widths, a container's closer). A request the editor sends on scrolling or editing (debounced) carries
   the rest.

5. **Revealing is the renderer's, on the lines the server names.** Each mark carries `reveal`, the lines it gives way
   on (a table's mark: the table's lines, so a cell is edited raw; a heading's: its line). The renderer hides the marks
   whose `reveal` holds the cursor's line (and, in Neovim, the visual selection), and draws them again when it leaves,
   as render-markdown does with its anti-conceal, as org-appear does in Emacs. In Neovim, insert mode shows the raw text
   by default, as render-markdown does (`render.modes`). *Settled with the user (2026-11-26):* the raw text follows the
   cursor, by its line by default; `compmd.render.reveal` reveals the whole element instead, or never.

6. **Every choice in this design is a setting, and the settings are the server's, in `compmd`, so one setting reaches
   every editor** (the language server's decision 12). Each editor sets them where it sets a server's (Neovim's
   `vim.lsp.config('compmd', { settings = … })`, Eglot's `eglot-workspace-configuration`, VS Code's `settings.json`,
   Zed's `lsp.compmd-lsp.settings`, Helix's `config` in `languages.toml`), and a change applies without a restart
   (`workspace/didChangeConfiguration`). The defaults are this design's decisions; what the architecture fixes (the
   server plans, the renderer draws) is the only part that isn't a setting.
   - **Which files:** `markdown.enabled` (default `true`): `.md` files served at all (decision 13);
     `render.markdownFiles` (default `true`): `.md` files drawn, so a writer who keeps another plugin's drawing keeps
     the rest of ours.
   - **What is drawn:** `render.enabled` (default `true`) and one switch per element (`heading`, `code`, `table`,
     `list`, `checkbox`, `quote`, `callout`, `rule`, `container`, `component`, `param`), each `true`, `false` or its own
     options: `code.language` (`above`, the default, `fence` or `off`), `heading.fill` (the line in its style to the
     edge, default `true`), `table.borders` (default `true`).
   - **How it looks:** `render.glyphs`: `unicode` (default: box drawing and plain symbols, `─ │ ┌ ● ☐ ☑`), `nerd` (Nerd
     Font icons for headings, languages and callouts) or `ascii` (for terminals and fonts without either); and
     `render.glyph`, any glyph by its name (`heading.2`, `bullet.1`, `checkbox.checked`, `callout.warning`) as the
     writer's own text. Each style (Terms) is the package's highlight group, face or theme colour, linked to the
     editor's Markdown ones by default, so a colour scheme or the writer restyles it: `CompmdHeading2` in Neovim, the
     face `compmd-heading-2` in Emacs, the colour `compmd.heading2` in VS Code.
   - **When it gives way:** `render.reveal`: `line` (default), `element` (the whole element, a table, while the cursor
     is in it) or `off`; the renderer's own, read from the same namespace: `render.modes` (Neovim's modes that draw;
     default normal, command, terminal).
   - **The rest:** `render.vscodeHiding` (decision 8), `render.values` (decision 9), `tables.alignOnType` and
     `tables.alignOnFormat` (decision 7, both default `true`), and the links' (decision 14).

7. **Tables are aligned in the text, and drawn on top.** The formatter pads a table's cells to its columns' widths
   (`textDocument/formatting`, the formatter's rule for tables in the language server's design), and on-type
   formatting does it when a `|` or a newline is typed in a table, as markdown-mode, markdown-ts-mode and org-mode
   align theirs by editing the text. Aligned text reads the same in every editor, on GitHub and in a diff, and Helix
   and Zed, which can't draw, get it too. The drawing then only draws over it: the pipes as `│`, the delimiter row as
   `├─┼─┤`, a border above and below. render-markdown pads visually instead, which needs hiding and inline text in every
   cell; that is kept out.

8. **VS Code hides text by CSS in a decoration's `textDecoration`, the one way it can.** Its API has no way to hide or
   replace text; extensions that draw Markdown in place (Markless, Markdown Inline Editor) put `none; display: none`
   or `font-size: 0` in `textDecoration`, which VS Code copies into its CSS unchecked. It is undocumented and may break
   in a release, so the VS Code renderer declares `hide` and `replace` only when `compmd.render.vscodeHiding` is on
   (default on), and its check runs in every VS Code the extension supports. Without them, the server's plan for VS
   Code styles and inserts only (decision 3). *Settled with the user (2026-11-26).*

9. **Every editor gets a level from plain LSP, drawing or not.** What a server gives through the protocol, which Helix
   and Zed draw from too:
   - **semantic tokens**: a heading's level, a component's name, a param (the shared token names); Neovim, Emacs 31
     (Eglot 1.20), VS Code, Zed (off by default, `semantic_tokens`); not Helix, which has none;
   - **inlay hints**: an interpolation's value beside it (`{{md.doc.title}}` → `Harbour notes`), an expression's
     result, with no `kind` (the protocol's kinds are a type and a parameter; without one, "the client should fall
     back to a reasonable default"); every editor (off by default in Helix and Zed, by their own setting); on by
     default from the server, and `compmd.render.values` turns them off (*settled with the user, 2026-11-26*);
   - **formatting and on-type formatting**: tables aligned (decision 7); on-type in Neovim, Emacs, VS Code and Zed,
     not Helix;
   - **code actions** on a table: insert or delete a row or a column, move a column, change its alignment;
   - **folding ranges**: already served; Neovim, VS Code and Zed (opt-in) use them, Emacs and Helix fold by tree-sitter
     or not at all.

10. **Who draws what:**

    | Editor | Draws | Level |
    |---|---|---|
    | Neovim 0.12, terminal or GUI (Neovide) | all six primitives, by extmarks | full |
    | Emacs 30 and 31, terminal | all six, by overlays and text properties | full |
    | Emacs GUI | the same, and its faces may be larger (a heading's `:height`), as markdown-mode's header scaling | full, richer |
    | VS Code | `insert`, `prefix`, `style`; `hide` and `replace` by decision 8; no `line` | most |
    | Zed | nothing: its extensions have no API to draw in a buffer, and it ignores a server's own notifications | plain LSP |
    | Helix 25.07 | nothing: no plugin system in a release (Steel is an open draft) | plain LSP, without semantic tokens |

    A terminal draws one cell per character, in one size, with no images; the GUIs' extras are the renderer's (a face's
    height), never the plan's. Images, which only Emacs's GUI and VS Code could draw in place, are kept out of the first
    version.

11. **The first version draws:**
    - **Markdown**: ATX and setext headings (the `#`s replaced by the level's glyph, the line in its style to the edge);
      fenced code blocks (the fences hidden, the language on a line above, the block's lines in the code style to the
      edge); inline code; thematic breaks (a `fill`); list bullets by depth; task list checkboxes; block quotes (`▋` by
      depth); GitHub alerts (`[!NOTE]` replaced by its icon and title); tables (decision 7).
    - **CompMD**: a container as a box: its opening line `replace`d by a top border titled with its name (and its
      params, set back), its lines `prefix`ed with `│`, its closer `replace`d by the bottom border; slots as a divider
      in the box, titled with the slot's name; a leaf component's line with its name in the component style and its
      params set back; the frontmatter's `md` key set back.
    - **Left out**: images, math and HTML (no drawing at all), link icons, a heading's size in terminals, and
      everything that would need the window's width beyond a `fill`.

12. **Where it is built:**
    - the plan in `compmd-lsp`, a module of its own on the processor's tree, its tests a set of documents and the plan
      each gives, by primitive set (as the grammars share one test set, editor support's decision 2);
    - a renderer in each package that draws: `compmd.nvim` (a Lua module), `compmd-ts-mode` (overlays, with
      `cursor-sensor-functions` for revealing), and VS Code's extension when it is built;
    - each package's check draws a document and checks its marks (extmarks, overlays, decorations), in the terminal
      where the editor has one.

13. **Markdown files too, read as the spec reads them.** The server and the packages attach to `.md` files as well as
    `.comp.md` ones, and the spec decides what each file is (section 2, *What makes a file CompMD*): a `.md` file whose
    frontmatter has an `md` key is CompMD; one without is CommonMark and GFM, and CompMD processing never applies to
    it, so `:::`, `{…}` and `{{…}}` stay text. For such a file the server offers only what Markdown has:
    - the drawing of decision 11's Markdown elements, tables aligned as they are typed and the table code actions
      (decisions 7 and 9), folding, and the preview, which renders it as CommonMark and GFM with its code highlighted;
    - no component, import, value or CompMD diagnostic; one code action, *Read as CompMD*, adds the `md` key, as the
      spec lets a processor offer.

    A `.md` file keeps its editor's own Markdown mode and highlighting (Neovim's `markdown` parser, markdown-mode or
    markdown-ts-mode, VS Code's, Zed's and Helix's grammars): the packages add the server and the renderer to it and
    replace nothing. *Settled with the user (2026-11-26):* the packages serve `.md` and `.comp.md` files by default,
    drawing both; each package's README says so first and shows how to turn Markdown off (`compmd.markdown.enabled =
    false`), or only its drawing (`compmd.render.markdownFiles = false`) for a writer who keeps render-markdown's or
    markview's. A writer who keeps both draws twice; the README says to turn one off. What else is there stays: VS
    Code's own Markdown features, a Markdown language server (Marksman); the READMEs say which of their features ours
    repeat (decision 14), so a writer turns one off.

    **What each group of writers gets, by editor:**

    | Editor | Markdown writers today | Markdown writers, with CompMD's package | CompMD writers |
    |---|---|---|---|
    | Neovim 0.12 | tree-sitter highlighting; drawing by a plugin (render-markdown, markview); links and headings by Marksman; LazyVim's Markdown extra bundles these with a browser preview | the drawing, full; tables aligned as typed; the table code actions; links and headings (decision 14); the preview; with any plugin manager, LazyVim or not | all of that, and containers and components drawn, values as inlay hints, completion, hover, diagnostics, imports, the preview with components |
    | Emacs 30 and 31 | markdown-mode: highlighting, headings scaled in the GUI, markup hidden on demand (`markdown-hide-markup`), tables aligned by command, a preview by an external command; markdown-ts-mode (31): highlighting | the drawing, full (headings larger in the GUI); tables aligned as typed; the table code actions; links and headings; the preview; in markdown-mode or markdown-ts-mode | the same as Neovim's, in `compmd-ts-mode` |
    | VS Code | highlighting, a preview beside the text, the outline, folding, completion of paths and headings, rename, link checks; Markdown All in One for tables, a table of contents and lists; nothing drawn in place | the drawing, most (no lines of its own; hiding by decision 8); tables aligned as typed; the table code actions; links and headings, as VS Code's own (one of the two turned off); the preview, beside VS Code's own | the same as Neovim's, without lines of its own |
    | Zed | tree-sitter highlighting; a preview of its own; a Markdown language server by an extension | no drawing; tables aligned as typed; the table code actions; links and headings; the preview; folding (opt-in) | plain LSP: semantic tokens (opt-in), values as inlay hints (opt-in), completion, hover, diagnostics, imports, the preview with components |
    | Helix 25.07 | tree-sitter highlighting; Marksman and markdown-oxide as its Markdown servers; no preview | no drawing; tables aligned by formatting (no on-type formatting); the table code actions; links and headings; the preview, its first | plain LSP without semantic tokens: values as inlay hints (opt-in), completion, hover, diagnostics, imports, the preview with components |

    What is the same in every editor, for both groups, comes from the one server: the tables, the code actions, links
    and headings, the preview, and the drawing wherever the editor draws.

14. **Links and headings, in both kinds of file**, as Marksman and VS Code's Markdown language service have them, so
    the server can be a Markdown writer's only one (*settled with the user, 2026-11-26*). On the project's files (the
    document's root, the language server's decision 2), from the processor's tree:
    - **completion**: a link's path (the project's files), a heading after `#` (the file's or the linked file's), a
      reference link's label;
    - **go to definition**: a link to its file, its heading or its reference definition; **references**: a heading's
      and a file's links across the project;
    - **rename**: a heading's text renames its anchor in every link to it; a file renamed in the editor
      (`workspace/willRenameFiles`) has its links updated;
    - **diagnostics**: a link to a file, a heading or a reference label that isn't there, as a warning;
    - **document symbols** (the outline, by heading) and **workspace symbols** (every file's headings);
    - **hover** on a link: the start of its target's section.
    - **wiki links** (`[[Page]]`, `[[Page|label]]`, `[[Page#Heading]]`), in a document that opts in
      (`md.syntax: [wiki-links]`, spec section 2.3): the spec leaves a target to the processor, so the server's rule is
      the one Marksman and Obsidian follow, a project file by its name without `.comp.md` or `.md`, any case, and a
      heading after `#`; completion, go to definition, references, rename (a file or a heading renamed updates its wiki
      links) and the broken-link warning, as for links;
    - **links to a passage** (`#:~:text=…`, spec section 2.3): go to definition selects the passage in the source,
      its first match after the prefix and before the suffix, as the spec lets a tool; a fragment that matches nothing
      is the broken-link warning.

    Its settings: `links.enabled` (default `true`), and `links.diagnostics`, `links.completion`, `links.rename` (each
    default `true`), so a writer who keeps Marksman or VS Code's own turns ours off or theirs; `links.wiki` (default
    `true`: in documents that opt in), and `markdown.wikiLinks` (default `false`): wiki links in `.md` files too, which
    CommonMark doesn't have but Obsidian's and Marksman's writers do. A heading's anchor is the expanded tree's
    `anchor` in a CompMD file (the spec's gap, below), and the same rule on the plain tree in a `.md` file, so both kinds
    of file link alike. These features go into the language server's design with their task.

## Settled with the user (2026-11-26)

- VS Code hides text by CSS, behind `compmd.render.vscodeHiding`, on by default (decision 8).
- Values as inlay hints are on by default, `compmd.render.values` turns them off (decision 9).
- The raw text follows the cursor, by line by default (decision 5).
- `.md` files are served and drawn by default, and each README shows how to turn that off (decision 13).
- Links and headings are the server's, in both kinds of file (decision 14).
- Every choice is a setting (decision 6).

## The spec's gap

**A CompMD heading's anchor (decision 14).** Section 8.1 says a processor that makes heading anchors uses the expanded
text, but not how that text becomes an anchor, so a link's `#heading` can't be checked or renamed in a CompMD file by
the spec. No processor or renderer makes anchors yet (`@compmd/render` writes headings without an `id`), so a
`#heading` link in a rendered CompMD page has nothing to land on. Two ways were weighed:

- **The rule as text in section 8.1**, for processors that make anchors. The smallest change, but nothing in a tree
  shows it, so no conformance case can check it (section 13.1 tests the expanded tree), and every tool that makes
  anchors writes it again (the language server in Rust, `@compmd/render` in TypeScript, other renderers); GitHub's
  rule has edge cases (letters beyond ASCII, emoji, punctuation, repeats), so they would drift.
- **Each heading of the expanded tree carries its `anchor`** (section 12.3, beside a heading's computed `depth`):
  `{"type": "heading", "depth": 2, "anchor": "harbour-notes", …}`. The processor computes it once; the language
  server reads it, `@compmd/render` writes it as the heading's `id`, and conformance cases check it.

The proposal is the second (T1). The rule is GitHub's: the expanded text in lower case, punctuation but `-` and `_`
dropped, spaces as `-`, a repeat numbered `-1`, `-2` in document order, headings from `compmd` bodies included. An
example's headings (spec section 7.8) are numbered within the example's root, a document of its own, so an example
never moves the page's anchors. It costs the field in `compmd-tree.schema.json`, `anchor` in the four expanded trees
that have a heading (`each-join`, `relative-headings`, `fonts-own-and-a-packs`, `complete-example-expanded`) and cases
of its own (the rule's edge cases, repeats, an example's headings). No grammar changes: the anchor is computed from
the expanded text, not parsed (markdown-rs, tree-sitter, TextMate and Lezer stay as they are).

## What the editors allow, from the research

- **The protocol, LSP 3.18** (its specification): a custom method is allowed, a `$/` one is the protocol
  implementation's own; `experimental` in both sides' capabilities carries what is under development; every refresh
  (semantic tokens, inlay hints, folding ranges, code lenses, diagnostics) is a request from the server, gated by the
  client's `refreshSupport`; on-type formatting's trigger characters are plain strings (`firstTriggerCharacter`,
  `moreTriggerCharacter`).

- **Neovim 0.12** (its 0.12.6 runtime docs, `api.txt`, `lsp.txt`, `news.txt`): extmarks with `conceal`,
  `conceal_lines` (0.11), `virt_text` (`eol`, `eol_right_align`, `overlay`, `right_align`, `inline`), `virt_lines`
  (above or below), `hl_group`, `hl_eol`, `line_hl_group`, `sign_text`, `url`. Concealed text shows on the cursor's
  line unless `concealcursor` says otherwise; virtual text doesn't, so a plugin removes it itself on `CursorMoved`. No
  images (`vim.ui.img` is 0.13's, experimental) and no size per line, terminal or GUI. Its client: semantic tokens,
  inlay hints, formatting, on-type formatting (new in 0.12), code actions, folding ranges, document colors and links,
  code lens, a server's own notifications through `handlers`.
- **render-markdown.nvim** (its README and source): headings, paragraphs, code blocks and inline code, thematic
  breaks, bullets, checkboxes, quotes, callouts, pipe tables, links, LaTeX, HTML, `==highlights==`, the sign column,
  indent and the YAML frontmatter; by overlays, conceal with inline text, backgrounds to the edge and virtual lines.
  It hides its marks on the cursor's line (`anti_conceal`), draws in normal, command and terminal modes, and aligns
  tables only on screen.
- **Emacs 30 and 31** (the Emacs Lisp Reference Manual, 31.1's `eglot.el` and NEWS): `display` strings and specified
  spaces (`:width` and `:align-to` work in a terminal, heights don't), `invisible`, `before-string` and `after-string`,
  `line-prefix` and `wrap-prefix`, faces with `:height`, `:box`, `:overline`, `:extend`; images and fringes only in a
  GUI. Revealing by `cursor-sensor-functions` or `post-command-hook` (org-appear). markdown-mode, markdown-ts-mode (31)
  and org-mode align tables by editing the text. Eglot: semantic tokens from 1.20 (in Emacs 31; on 30 from GNU ELPA),
  inlay hints, formatting, on-type formatting, code actions, a server's own notifications by
  `eglot-handle-notification`; no folding ranges, document links or colors.
- **Helix 25.07.1** (its docs, CHANGELOG and `helix-lsp` source): inlay hints (off by default), formatting and range
  formatting, code actions, workspace commands, document colors as swatches (new in 25.07); no semantic tokens (an
  open draft), no folding, no on-type formatting; a server's own notifications are dropped. No plugin system released.
- **VS Code 1.148** (`vscode.d.ts`): decorations with `before` and `after` text or icons, borders, backgrounds,
  `opacity`, `fontWeight`, `isWholeLine`, gutter icons; no `fontSize` or `display` (only through CSS in
  `textDecoration`); no lines of its own in the stable API (`editorInsets` is a proposed API, which Marketplace
  extensions can't use); images in `before` and `after`. `vscode-languageclient` 11: semantic tokens, inlay hints, the
  three formattings, code actions, folding, document colors and links, a server's own notifications by
  `onNotification`.
- **Zed 1.30** (its docs, `zed_extension_api` 0.7, its settings): extensions give languages (tree-sitter queries),
  language servers, slash commands, themes; no API to draw in a buffer and no way to receive a server's own
  notifications. Semantic tokens since 0.224 (off by default; colour, background, weight and style, no size), inlay
  hints (off by default), the three formattings, code actions, LSP folding (opt-in), document colors and links.
- **Markdown today** (each editor's docs and packages): Neovim highlights by its `markdown` parsers, and plugins draw
  (render-markdown, markview) and preview; LazyVim's Markdown extra adds render-markdown, Marksman and a preview.
  Emacs's markdown-mode scales headings in the GUI, hides markup on demand and aligns tables by command;
  markdown-ts-mode (31) highlights. VS Code's own Markdown features (its Markdown language service): preview, outline,
  folding, path and heading completion, rename, link validation; Markdown All in One adds tables, a table of contents
  and list editing. Zed highlights and previews. Helix's `hx --health markdown` lists Marksman and markdown-oxide.
  None draws the same in two editors.

## Proposed tasks and sprints

Created on approval as CMD-438 (T1) to CMD-448 (T11). The server's tasks and Neovim's, Emacs's and Helix's go to the current sprint,
*Editors 1: Neovim, Helix and Emacs*; VS Code's and Zed's to *Editors 2: VS Code and Zed*, after their extensions
(CMD-25, CMD-28).

**Editors 1: Neovim, Helix and Emacs**

| Task | What it builds | Waits for |
|---|---|---|
| T1. The spec: a heading's anchor | `anchor` on the expanded tree's headings, by GitHub's rule (the spec's gap, above): sections 8.1 and 12.3, the schema, the changelog entry, the conformance cases; the processor computes it, `@compmd/render` writes it as the heading's `id` | |
| T2. The language server: the drawing plan | `compmd/drawing` and `compmd/drawing/refresh`, the capabilities, the six primitives and their fallbacks, decision 11's elements, the `render` settings, live settings (decisions 1 to 6, 11); its test set of documents and plans by primitive set | |
| T3. The language server: Markdown files | `.md` files read as the spec reads them, only Markdown's features, *Read as CompMD*, `markdown.enabled` (decision 13) | T2 |
| T4. The language server: tables | tables aligned by formatting and on-type formatting, the table code actions, the `tables` settings (decisions 7, 9); the formatter (CMD-34) builds on it | |
| T5. The language server: values as inlay hints | an interpolation's value and an expression's result, `render.values` (decision 9) | |
| T6. The language server: links and headings | decision 14 (links, headings, wiki links, links to a passage), with its settings, written into the language server's design | T1 for CompMD files |
| T7. compmd.nvim: the renderer | extmarks for the six primitives, revealing, modes, highlight groups, `.md` buffers too; its check draws a document and checks its marks; the README's Markdown section (decision 13) | T2, T3, CMD-431 |
| T8. compmd-ts-mode: the renderer | overlays and text properties, revealing by `cursor-sensor-functions`, faces, a minor mode for markdown-mode and markdown-ts-mode buffers; its check in `--batch` and `-nw`; the README's Markdown section | T2, T3 |
| T9. Helix: Markdown files | the server for `markdown` in `languages.toml`, beside Marksman, and its settings, in the README | T3 |

**Editors 2: VS Code and Zed**

| Task | What it builds | Waits for |
|---|---|---|
| T10. VS Code: the renderer | decorations for `insert`, `prefix` and `style`, hiding by CSS behind `render.vscodeHiding` (decision 8), theme colours, `.md` files beside VS Code's own Markdown features; its check in VS Code | T2, T3, CMD-25 |
| T11. Zed: Markdown files | the server for Markdown in the extension, its settings, in the README | T3, CMD-28 |

The current sprint gains nine tasks with four days left (due 2026-11-30), more than it has left now: T7 to T9 could
move to *Editors 2* if it runs late.
