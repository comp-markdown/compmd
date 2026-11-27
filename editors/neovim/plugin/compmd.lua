-- CompMD for Neovim until its entries are upstream: the three parsers of comp-markdown/compmd-grammars registered
-- with nvim-treesitter, which installs their queries with them (`:TSInstall compmd`), and the language server
-- (`lsp/compmd.lua`) enabled, for `compmd` and `markdown` buffers. The file type is `ftdetect/compmd.lua`'s, the
-- buffer's highlighting `ftplugin/compmd.lua`'s, the drawing `lua/compmd/drawing.lua`'s.

if vim.g.loaded_compmd then
  return
end
vim.g.loaded_compmd = true

local grammars = { compmd = 'compmd', compmd_inline = 'compmd-inline', cel = 'cel' }

vim.api.nvim_create_autocmd('User', {
  pattern = 'TSUpdate',
  callback = function()
    local parsers = require('nvim-treesitter.parsers')
    for grammar, folder in pairs(grammars) do
      parsers[grammar] = {
        install_info = {
          url = 'https://github.com/comp-markdown/compmd-grammars',
          location = 'tree-sitter/' .. folder,
          queries = 'tree-sitter/' .. folder .. '/queries',
        },
        requires = grammar == 'compmd' and { 'compmd_inline', 'cel' } or nil,
      }
    end
  end,
})

vim.lsp.enable('compmd')

-- The drawing's highlight groups, linked to the Markdown ones; again after a colour scheme clears them.
require('compmd.drawing').define_groups()
vim.api.nvim_create_autocmd('ColorScheme', {
  callback = function()
    require('compmd.drawing').define_groups()
  end,
})
