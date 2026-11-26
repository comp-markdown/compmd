---@brief
---
--- https://compmd.dev/
---
--- The CompMD language server: diagnostics, completion, hover, signature help, go to definition, symbols, folding,
--- semantic tokens and code actions for CompMD documents (`.comp.md`), and their preview in the browser
--- (`:LspCompmdPreview`, or the code action *Open preview*).
---
--- Install it with `npm i -g @compmd/lsp`, `cargo install compmd-lsp`, mason.nvim (`:MasonInstall compmd-lsp`), or the
--- binary on each GitHub release.

---@type vim.lsp.Config
return {
  cmd = { 'compmd-lsp', '--stdio' },
  filetypes = { 'compmd' },
  root_markers = { 'compmd.config.json', 'package.json', '.git' },
  -- The settings, under `compmd`: `packs.suggest`, `packs.network`, `preview.browser` and `trace`.
  settings = { compmd = {} },
  on_attach = function(client, bufnr)
    vim.api.nvim_buf_create_user_command(bufnr, 'LspCompmdPreview', function()
      client:exec_cmd({
        title = 'Open preview',
        command = 'compmd.preview',
        arguments = { vim.uri_from_bufnr(bufnr) },
      }, { bufnr = bufnr })
    end, { desc = "Open the CompMD document's preview in the browser" })
  end,
}
