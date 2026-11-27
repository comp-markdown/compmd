# compmd.nvim

CompMD in Neovim: `.comp.md` files as the file type `compmd`, highlighted by CompMD's tree-sitter grammars with the
languages inside them injected, and the language server, `compmd-lsp`, with `:LspCompmdPreview`, the document in the
browser. Both `.comp.md` and `.md` files are drawn in place: headings, code blocks, lists, quotes, tables and CompMD's
containers with a gutter, the raw text back on the cursor's line ([Markdown files](#markdown-files)). It works with
any plugin manager, or none, and with or without a distribution such as LazyVim. Published as
[`comp-markdown/compmd.nvim`](https://github.com/comp-markdown/compmd.nvim) until its entries are upstream
([editor support](../../docs/editor-support.md), decision 6).

## What it needs

- **Neovim 0.12** or later.
- **[nvim-treesitter](https://github.com/nvim-treesitter/nvim-treesitter)'s `main` branch**, with `git` and the
  tree-sitter CLI, which build the grammars.
- **`compmd-lsp`** on the `PATH`: from npm (`npm i -g @compmd/lsp`), cargo (`cargo install compmd-lsp`) or the
  release's binary.

## Install

With [lazy.nvim](https://github.com/folke/lazy.nvim):

```lua
{ 'comp-markdown/compmd.nvim', dependencies = { 'nvim-treesitter/nvim-treesitter' } }
```

With Neovim's own `vim.pack`:

```lua
vim.pack.add({ 'https://github.com/nvim-treesitter/nvim-treesitter', 'https://github.com/comp-markdown/compmd.nvim' })
```

Any other manager the same way: the plugin has no `setup()` to call. Then, once, install the grammars:

```vim
:TSInstall compmd
```

which installs `compmd` with the two it needs, `compmd_inline` and `cel`, and their queries.

## What it does

| File | What it does |
|---|---|
| [`ftdetect/compmd.lua`](ftdetect/compmd.lua) | `.comp.md` files are `compmd`, not Markdown |
| [`plugin/compmd.lua`](plugin/compmd.lua) | adds the three grammars to nvim-treesitter (from `comp-markdown/compmd-grammars`), and enables the language server |
| [`ftplugin/compmd.lua`](ftplugin/compmd.lua) | highlights a `compmd` buffer by its grammar, once it is installed |
| [`lsp/compmd.lua`](lsp/compmd.lua) | `compmd-lsp --stdio` for `compmd` and `markdown` buffers, with `:LspCompmdPreview`, and the drawing |
| [`lua/compmd/drawing.lua`](lua/compmd/drawing.lua) | the drawing: the server's plan (`compmd/drawing`) as extmarks, and a highlight group per style |

The server's settings go where any server's do:

```lua
vim.lsp.config('compmd', { settings = { compmd = { --[[ the server's settings ]] } } })
```

and `vim.lsp.enable('compmd', false)` turns it off.

## Markdown files

The server and the drawing serve `markdown` buffers too, by default. A `.md` file is CompMD when its frontmatter has an
`md` key, as the spec says; otherwise it is plain Markdown and gets Markdown's part only: the drawing, tables aligned
as they are typed and their code actions, folding and the preview, with the code action *Read as CompMD*. Neovim's
`markdown` parser still highlights it.

To turn Markdown files off, or only their drawing (for a writer who keeps
[render-markdown.nvim](https://github.com/MeanderingProgrammer/render-markdown.nvim) or
[markview.nvim](https://github.com/OXY2DEV/markview.nvim), which draw Markdown buffers too; with both, the buffer is
drawn twice, so turn one off):

```lua
vim.lsp.config('compmd', { settings = { compmd = { markdown = { enabled = false } } } }) -- no Markdown files at all
vim.lsp.config('compmd', { settings = { compmd = { render = { markdownFiles = false } } } }) -- served, not drawn
```

## The drawing

The server plans what is drawn and the plugin draws it with extmarks: a heading's `#`s as its level (`H1`), a code
block's fences set back and its language coloured, its code left to Neovim's own highlighting of the language, bullets and checkboxes as glyphs, quotes and alerts with their bars and icons,
a table's borders, a container's lines with a gutter (`│`) in the status column, beside the line numbers, which a
wrapped line keeps on all its rows, and its opener and closer as written, a leaf's params set back, the
frontmatter's fences as rules and its lines greyed out. A mark gives way to the raw text while the cursor is on its
line, or on its element (the frontmatter's grey while the cursor is anywhere in it); the drawing's colours are drawn
over tree-sitter's and the server's semantic tokens; it is drawn in normal, insert, command and terminal mode, and the
raw text shows in visual mode. The gutter is added to the window's `'statuscolumn'` after your own, or after
Neovim's folds, signs and numbers when you have none. A line that stands for one it hides (a rule, a frontmatter fence) is drawn over that line, as Neovim hides a hidden line's own virtual lines with it. A table is aligned as you type `|` (on-type formatting, which the plugin turns on), or
by formatting (`vim.lsp.buf.format()`); one that isn't aligned yet is drawn over its text as it is. Everything is a
server setting, under `render`:

```lua
vim.lsp.config('compmd', {
  settings = {
    compmd = {
      render = {
        enabled = true, -- false: nothing drawn
        glyphs = 'unicode', -- or 'nerd', 'ascii'
        reveal = 'line', -- or 'element', 'off'
        modes = { 'n', 'i', 'c', 't' }, -- the modes drawn in, by their first letter
        -- Each element takes false to leave it as written, or its options:
        heading = { marker = true, fill = true }, -- marker = false keeps the #s, set back
        code = { language = 'inline', fence = true }, -- or 'above', 'fence', 'off'; fence = false: the ``` as written
        container = { gutter = true },
        frontmatter = { rules = true, dim = true },
        table = { borders = true },
      },
    },
  },
})
```

The server's README lists them all. Each style is a highlight group, `CompmdHeading1` to `CompmdHeading6`,
`CompmdHeadingMarker`, `CompmdCodeFence`, `CompmdCodeLanguage`, `CompmdContainerBorder`, `CompmdComponent`,
`CompmdParam` and the others, linked to the Markdown and tree-sitter groups by default, so a colour scheme restyles
them. A heading level takes the scheme's Markdown heading colour when the scheme has one; otherwise (Neovim's default
scheme draws headings as bold text) it is bold in a colour every scheme has: the diagnostics' warning, info, ok and
hint colours, then `Special` and `Comment`. Set one yourself to change it:

```lua
vim.api.nvim_set_hl(0, 'CompmdHeading1', { fg = '#e0af68', bold = true })
```

## Upstream

The entries that will replace the plugin, drafted in [`upstream/`](upstream): `parsers.lua` for nvim-treesitter and
`mason/package.yaml` for mason-registry; the file type for Neovim's `runtime/lua/vim/filetype.lua` is
`ftdetect/compmd.lua`'s pattern, and `lsp/compmd.lua` is nvim-lspconfig's. They are opened as pull requests after the
first release of `compmd-lsp` and the grammars, each with the maintainer's go. Text objects (`textobjects.scm`, a
component and its params) go to nvim-treesitter-textobjects the same way.

## Check

`cargo make neovim-check` builds `compmd-lsp`, clones nvim-treesitter's `main` into `target/neovim/`, and runs
[`test/check.lua`](test/check.lua) in a headless Neovim with the plugin on its runtime path and the grammars from this
checkout. It checks:

- the file type;
- `:TSInstall compmd` installing the three parsers with their queries, and a buffer highlighted by its parser;
- each grammar's highlights, injected ones included;
- a container's fold;
- the server attaching, enabled by the plugin; its diagnostics, *Open preview*, `:LspCompmdPreview`;
- `compmd.preview` serving the page, with a browser that opens nothing;
- the drawing: a heading's glyph and its line giving way to the cursor, a container's gutter in the status column
  with its text as written, the highlight groups, and a `.md` file drawn as Markdown, its `:::` left as text;
- tables, each line rebuilt as drawn from the text and the extmarks (a headless Neovim has no screen): an aligned
  table's box lined up, and typing `|` aligning an unaligned one;
- the frontmatter's fences drawn as rules, one still drawn while the cursor is on the other, its lines set back over
  tree-sitter's colours and shown as written while the cursor is in it; the modes drawn in by default;
- a container whose opener goes on over a `::: +` line: the gutter on the lines inside only, its lines as written.

It needs `nvim` 0.12, `git` and the tree-sitter CLI. It isn't part of `cargo make ci`.

## Try it

`cargo make nvim-interactive [file]` builds the same and opens Neovim with the plugin in place of your own config
([`test/interactive.lua`](test/interactive.lua)), on the file, or without one on a copy of the Author's Guide's
containers example in a temporary folder. Then `:Inspect` shows a span's captures, `zc` folds a container, `K` hovers,
`gra` lists the code actions and `:LspCompmdPreview` opens the preview.
