-- An interactive Neovim with the drafts against local builds, in place of the writer's own config.
-- `cargo make nvim-interactive [file]` builds what it needs and starts it:
--
--   nvim -u editors/neovim/test/interactive.lua [file]
--
-- Without a file, it opens the Author's Guide's containers example.

local neovim = vim.fs.dirname(vim.fs.dirname(debug.getinfo(1, 'S').source:sub(2)))
local repository = vim.fs.dirname(vim.fs.dirname(neovim))
dofile(neovim .. '/test/setup.lua')(
  repository .. '/grammars/compmd-grammars/tree-sitter',
  repository .. '/target/neovim',
  repository .. '/target/debug/compmd-lsp',
  {}
)
if vim.fn.argc() == 0 then
  vim.cmd.edit(repository .. '/guides/authors/examples/containers/garden.comp.md')
end
