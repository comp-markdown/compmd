# compmd.nvim

CompMD in Neovim: `.comp.md` files as the file type `compmd`, highlighted by CompMD's tree-sitter grammars with the
languages inside them injected, and the language server, `compmd-lsp`, with `:LspCompmdPreview`, the document in the
browser. Both `.comp.md` and `.md` files are drawn in place: headings, code blocks, lists, quotes, tables and CompMD's
containers as boxes, the raw text back on the cursor's line ([Markdown files](#markdown-files)). It works with
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

The server plans what is drawn and the plugin draws it with extmarks: a heading's `#`s as its glyph, a code block's
fences hidden with its language above, bullets and checkboxes as glyphs, quotes and alerts with their bars and icons,
a table's borders, a container as a box (`┌─ box {padding=m} ───`, `│ `, `└───`), a leaf's params set back. A mark
gives way to the raw text while the cursor is on its line, or on its element; it is drawn in normal, command and
terminal mode, and the raw text shows in insert and visual mode. Everything is a server setting, under `render`:

```lua
vim.lsp.config('compmd', {
  settings = {
    compmd = {
      render = {
        enabled = true, -- false: nothing drawn
        glyphs = 'unicode', -- or 'nerd', 'ascii'
        reveal = 'line', -- or 'element', 'off'
        modes = { 'n', 'c', 't' }, -- the modes drawn in, by their first letter
        code = { language = 'above' }, -- or 'fence', 'off'; each element takes false to leave it as written
      },
    },
  },
})
```

The server's README lists them all. Each style is a highlight group, `CompmdHeading1` to `CompmdHeading6`,
`CompmdCode`, `CompmdContainerBorder`, `CompmdComponent`, `CompmdParam` and the others, linked to the Markdown and
tree-sitter groups by default, so a colour scheme restyles them; set one yourself to change it:

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
- the drawing: a heading's glyph, a container's opener hidden with its border above, the highlight groups, the
  opener's line giving way to the cursor, and a `.md` file drawn as Markdown, its `:::` left as text.

It needs `nvim` 0.12, `git` and the tree-sitter CLI. It isn't part of `cargo make ci`.

## Try it

`cargo make nvim-interactive [file]` builds the same and opens Neovim with the plugin in place of your own config
([`test/interactive.lua`](test/interactive.lua)), on the file, or without one on a copy of the Author's Guide's
containers example in a temporary folder. Then `:Inspect` shows a span's captures, `zc` folds a container, `K` hovers,
`gra` lists the code actions and `:LspCompmdPreview` opens the preview.
