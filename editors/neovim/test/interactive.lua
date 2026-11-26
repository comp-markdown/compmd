-- An interactive Neovim with the drafts against local builds, in place of the writer's own config.
-- `cargo make nvim-interactive [file]` builds what it needs and starts it:
--
--   nvim -u editors/neovim/test/interactive.lua [file]
--
-- Without a file, it opens a copy of the Author's Guide's containers example, in a folder of its own, to edit freely.

local neovim = vim.fs.dirname(vim.fs.dirname(debug.getinfo(1, 'S').source:sub(2)))
local repository = vim.fs.dirname(vim.fs.dirname(neovim))
dofile(neovim .. '/test/setup.lua')(
  repository .. '/grammars/compmd-grammars/tree-sitter',
  repository .. '/target/neovim',
  repository .. '/target/debug/compmd-lsp',
  {}
)
if vim.fn.argc() == 0 then
  local folder = vim.fn.tempname()
  vim.fn.mkdir(folder, 'p')
  local copy = folder .. '/garden.comp.md'
  vim.fn.writefile(vim.fn.readfile(repository .. '/guides/authors/examples/containers/garden.comp.md'), copy)
  vim.cmd.edit(copy)
end
