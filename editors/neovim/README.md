# compmd.nvim

CompMD in Neovim: `.comp.md` files as the file type `compmd`, highlighted by CompMD's tree-sitter grammars with the
languages inside them injected, and the language server, `compmd-lsp`, with `:LspCompmdPreview`, the document in the browser. It works with
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
| [`lsp/compmd.lua`](lsp/compmd.lua) | `compmd-lsp --stdio` for `compmd` buffers, with `:LspCompmdPreview` |

The server's settings go where any server's do:

```lua
vim.lsp.config('compmd', { settings = { compmd = { --[[ the server's settings ]] } } })
```

and `vim.lsp.enable('compmd', false)` turns it off.

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
- `compmd.preview` serving the page, with a browser that opens nothing.

It needs `nvim` 0.12, `git` and the tree-sitter CLI. It isn't part of `cargo make ci`.

## Try it

`cargo make nvim-interactive [file]` builds the same and opens Neovim with the plugin in place of your own config
([`test/interactive.lua`](test/interactive.lua)), on the file, or without one on a copy of the Author's Guide's
containers example in a temporary folder. Then `:Inspect` shows a span's captures, `zc` folds a container, `K` hovers,
`gra` lists the code actions and `:LspCompmdPreview` opens the preview.
