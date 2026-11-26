-- What a writer's config does until the upstream entries are in, against local builds: the file type, the parsers and
-- their queries, the server's config on the runtime path, highlighting and folds in compmd buffers. The check and the
-- interactive session both start from it:
--
--   dofile('editors/neovim/test/setup.lua')(grammars, parsers, server, settings)
--
-- `grammars` is the grammars' `tree-sitter/` folder, `parsers` the built parsers' folder, `server` the compmd-lsp
-- binary, `settings` the server's `compmd` settings.

local here = vim.fs.dirname(vim.fs.dirname(debug.getinfo(1, 'S').source:sub(2)))

return function(grammars, parsers, server, settings)
  -- A script run with `-l` starts with file type detection off.
  vim.cmd('filetype on')
  dofile(here .. '/filetype.lua')
  vim.opt.runtimepath:prepend(here)
  for grammar, folder in pairs({ compmd = 'compmd', compmd_inline = 'compmd-inline', cel = 'cel' }) do
    vim.treesitter.language.add(grammar, { path = parsers .. '/' .. grammar .. '.so' })
    for _, query in ipairs({ 'highlights', 'injections', 'folds' }) do
      local path = grammars .. '/' .. folder .. '/queries/' .. query .. '.scm'
      if vim.uv.fs_stat(path) then
        vim.treesitter.query.set(grammar, query, table.concat(vim.fn.readfile(path), '\n'))
      end
    end
  end
  vim.api.nvim_create_autocmd('FileType', {
    pattern = 'compmd',
    callback = function(event)
      vim.treesitter.start(event.buf, 'compmd')
      vim.wo.foldmethod = 'expr'
      vim.wo.foldexpr = 'v:lua.vim.treesitter.foldexpr()'
      vim.wo.foldlevel = 99
    end,
  })
  vim.lsp.config('compmd', {
    cmd = { server, '--stdio' },
    settings = { compmd = settings },
  })
  vim.lsp.enable('compmd')
end
