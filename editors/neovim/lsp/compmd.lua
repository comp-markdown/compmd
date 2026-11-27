---@brief
---
--- https://compmd.dev/
---
--- The CompMD language server: diagnostics, completion, hover, signature help, go to definition, symbols, folding,
--- semantic tokens and code actions for CompMD documents (`.comp.md`), their preview in the browser
--- (`:LspCompmdPreview`, or the code action *Open preview*), and Markdown files too: a `.md` file is CompMD when its
--- frontmatter has an `md` key, and plain Markdown otherwise. Each buffer is drawn in place by the server's plan
--- (`compmd.drawing`).
---
--- Install it with `npm i -g @compmd/lsp`, `cargo install compmd-lsp`, mason.nvim (`:MasonInstall compmd-lsp`), or the
--- binary on each GitHub release.

local drawing = require('compmd.drawing')

---@type vim.lsp.Config
return {
  cmd = { 'compmd-lsp', '--stdio' },
  filetypes = { 'compmd', 'markdown' },
  root_markers = { 'compmd.config.json', 'package.json', '.git' },
  -- The settings, under `compmd`: `packs`, `preview`, `trace`, `render` (the drawing, and `render.modes`, the modes it
  -- draws in), `markdown`, `tables`.
  settings = { compmd = {} },
  capabilities = {
    experimental = { compmd = { drawing = { primitives = drawing.primitives, refreshSupport = true } } },
  },
  handlers = {
    ['compmd/drawing/refresh'] = function(_, _, context)
      local client = vim.lsp.get_client_by_id(context.client_id)
      if client then
        drawing.refresh(client)
      end
      return vim.NIL
    end,
  },
  on_attach = function(client, bufnr)
    vim.api.nvim_buf_create_user_command(bufnr, 'LspCompmdPreview', function()
      client:exec_cmd({
        title = 'Open preview',
        command = 'compmd.preview',
        arguments = { vim.uri_from_bufnr(bufnr) },
      }, { bufnr = bufnr })
    end, { desc = "Open the CompMD document's preview in the browser" })
    -- Tables aligned as they are typed (`|`, a new line), which Neovim does only for a client that turns it on; the
    -- server's `tables.alignOnType` still decides.
    vim.lsp.on_type_formatting.enable(true, { client_id = client.id })
    local experimental = client.server_capabilities.experimental
    if type(experimental) == 'table' and experimental.compmd and experimental.compmd.drawing then
      drawing.attach(client, bufnr)
    end
  end,
}
