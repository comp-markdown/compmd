-- The plugin as a writer installs it, against local builds: nvim-treesitter and this folder on the runtime path, the
-- three parsers and their queries installed by nvim-treesitter from the local grammars rather than the repository,
-- and the server's config pointed at the local binary. The check and the interactive session both start from it:
--
--   dofile('editors/neovim/test/setup.lua')(treesitter, grammars, site, server, settings)
--
-- `treesitter` is an nvim-treesitter checkout (its `main` branch), `grammars` the compmd-grammars checkout, `site` the
-- folder nvim-treesitter installs into, `server` the compmd-lsp binary, `settings` the server's `compmd` settings.

local here = vim.fs.dirname(vim.fs.dirname(debug.getinfo(1, 'S').source:sub(2)))

return function(treesitter, grammars, site, server, settings)
  treesitter, grammars, site = vim.fs.abspath(treesitter), vim.fs.abspath(grammars), vim.fs.abspath(site)
  vim.opt.runtimepath:prepend(treesitter)
  vim.opt.runtimepath:prepend(here)
  -- A script run with `-l` loads no plugins and starts with file type detection off.
  vim.cmd('filetype plugin on')
  vim.cmd.runtime('plugin/compmd.lua')
  vim.api.nvim_create_autocmd('User', {
    pattern = 'TSUpdate',
    callback = function()
      local parsers = require('nvim-treesitter.parsers')
      for _, grammar in ipairs({ 'compmd', 'compmd_inline', 'cel' }) do
        parsers[grammar].install_info.url = nil
        parsers[grammar].install_info.path = grammars
      end
    end,
  })
  -- Neovim caches the runtime path's folders: a `parser/` that the first install creates isn't seen until a restart.
  vim.fn.mkdir(site .. '/parser', 'p')
  vim.fn.mkdir(site .. '/queries', 'p')
  require('nvim-treesitter').setup({ install_dir = site })
  require('nvim-treesitter').install({ 'compmd' }, { force = true }):wait(300000)
  vim.lsp.config('compmd', {
    cmd = { server, '--stdio' },
    settings = { compmd = settings },
  })
end
