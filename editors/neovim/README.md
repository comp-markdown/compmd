# CompMD in Neovim

Neovim needs no package of its own ([editor support](../../docs/editor-support.md), decision 6): CompMD comes through
entries upstream, which these files draft. They are opened as pull requests after the first release of `compmd-lsp`
and `tree-sitter-compmd`, each with the maintainer's go.

| File | Upstream | What it adds |
|---|---|---|
| [`filetype.lua`](filetype.lua) | Neovim, `runtime/lua/vim/filetype.lua` | `.comp.md` files are `compmd`, not Markdown |
| [`parsers.lua`](parsers.lua) | nvim-treesitter, `lua/nvim-treesitter/parsers.lua` | the grammars `compmd`, `compmd_inline` and `cel` from `comp-markdown/compmd-grammars`; their queries go in `runtime/queries/`, from each grammar's `queries/` |
| [`lsp/compmd.lua`](lsp/compmd.lua) | nvim-lspconfig, `lsp/compmd.lua` | `compmd-lsp --stdio` for `compmd` buffers, and `:LspCompmdPreview`, the preview in the browser |
| [`mason/package.yaml`](mason/package.yaml) | mason-registry, `packages/compmd-lsp/package.yaml` | `:MasonInstall compmd-lsp`, from `@compmd/lsp` on npm |

Text objects (`textobjects.scm`, a component and its params) go to nvim-treesitter-textobjects the same way.

## Until then

A writer's config does what the entries will:

```lua
vim.filetype.add({ pattern = { ['.*%.comp%.md'] = 'compmd' } })

-- The parsers, built from comp-markdown/compmd-grammars (`tree-sitter build`), and their queries.
vim.treesitter.language.add('compmd', { path = '/path/to/compmd.so' })

-- lsp/compmd.lua from this folder on the runtime path, then:
vim.lsp.enable('compmd')
```

## Check

`cargo make neovim-check` builds `compmd-lsp` and the three parsers, and runs [`test/check.lua`](test/check.lua) in a
headless Neovim. It checks:

- the file type;
- each grammar's highlights, injected ones included;
- a container's fold;
- the server attaching through `lsp/compmd.lua`, its diagnostics, *Open preview*, `:LspCompmdPreview`;
- `compmd.preview` serving the page, with a browser that opens nothing.

It needs `nvim` (0.11 or later) and the tree-sitter CLI. It isn't part of `cargo make ci`.

## Try it

`cargo make nvim-interactive [file]` builds the same and opens Neovim with the drafts in place of your own config
([`test/interactive.lua`](test/interactive.lua)), on the file, or the Author's Guide's containers example without one.
Then `:Inspect` shows a span's captures, `zc` folds a container, `K` hovers, `gra` lists the code actions and
`:LspCompmdPreview` opens the preview.
