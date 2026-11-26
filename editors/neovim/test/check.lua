-- The Neovim drafts checked in a headless Neovim, against local builds: the file type, the three grammars with their
-- queries, and the language server through `lsp/compmd.lua`. `cargo make neovim-check` builds what it needs and runs
-- it:
--
--   nvim --headless -l editors/neovim/test/check.lua <grammars' tree-sitter/> <built parsers' folder> <compmd-lsp>
--
-- It exits 0 when everything holds, else 1, saying what didn't.

local grammars, parsers, server = arg[1], arg[2], arg[3]
local here = vim.fs.dirname(vim.fs.dirname(vim.fs.abspath(arg[0])))
local failures = {}

local function check(holds, what)
  print((holds and 'ok   ' or 'FAIL ') .. what)
  if not holds then
    table.insert(failures, what)
  end
end

local function read(path)
  local file = assert(io.open(path, 'r'))
  local text = file:read('*a')
  file:close()
  return text
end

-- What a writer's config does until the upstream entries are in: the file type, the parsers and their queries, the
-- server's config on the runtime path. A script run with `-l` starts with file type detection off.
vim.cmd('filetype on')
dofile(here .. '/filetype.lua')
vim.opt.runtimepath:prepend(here)
for grammar, folder in pairs({ compmd = 'compmd', compmd_inline = 'compmd-inline', cel = 'cel' }) do
  vim.treesitter.language.add(grammar, { path = parsers .. '/' .. grammar .. '.so' })
  for _, query in ipairs({ 'highlights', 'injections', 'folds' }) do
    local path = grammars .. '/' .. folder .. '/queries/' .. query .. '.scm'
    if vim.uv.fs_stat(path) then
      vim.treesitter.query.set(grammar, query, read(path))
    end
  end
end

-- A document with a component its project doesn't have (F002), an interpolation and an expression.
local folder = vim.fn.tempname()
vim.fn.mkdir(folder, 'p')
local document = folder .. '/notes.comp.md'
vim.fn.writefile({
  '---',
  'title: Harbour notes',
  '---',
  '',
  '# {{md.doc.title}}',
  '',
  '::: nowhere {wide=(1 + 2)}',
  'The tide is in.',
  '::: /nowhere',
}, document)
vim.cmd.edit(document)
local buffer = vim.api.nvim_get_current_buf()
check(vim.bo[buffer].filetype == 'compmd', 'a .comp.md file is compmd')

vim.treesitter.start(buffer, 'compmd')
-- Headless, nothing is drawn, so nothing parses the buffer: it is parsed whole, its injections too.
vim.treesitter.get_parser(buffer, 'compmd'):parse(true)
local function captures(row, col)
  return vim.iter(vim.treesitter.get_captures_at_pos(buffer, row, col))
    :map(function(capture)
      return capture.capture
    end)
    :totable()
end
check(vim.list_contains(captures(6, 4), 'tag'), "a container's name is @tag")
check(vim.list_contains(captures(4, 5), 'variable'), "an interpolation's path is @variable (compmd_inline, injected)")
check(vim.list_contains(captures(6, 19), 'number'), "an expression's number is @number (cel, injected)")
check(vim.treesitter.foldexpr(7) ~= '0', 'a container folds')

vim.lsp.config('compmd', {
  cmd = { server, '--stdio' },
  -- A browser that opens nothing, so the preview's command can run here.
  settings = { compmd = { preview = { browser = 'true' } } },
})
vim.lsp.enable('compmd')
local attached = vim.wait(20000, function()
  return #vim.lsp.get_clients({ bufnr = buffer, name = 'compmd' }) > 0
end, 50)
check(attached, 'compmd-lsp attaches through lsp/compmd.lua')
local diagnosed = vim.wait(20000, function()
  return vim.iter(vim.diagnostic.get(buffer)):any(function(diagnostic)
    return diagnostic.code == 'F002'
  end)
end, 50)
check(diagnosed, 'the unknown component is F002')

local client = vim.lsp.get_clients({ bufnr = buffer, name = 'compmd' })[1]
if client then
  local position = { line = 0, character = 0 }
  local actions = client:request_sync('textDocument/codeAction', {
    textDocument = { uri = vim.uri_from_bufnr(buffer) },
    range = { start = position, ['end'] = position },
    context = { diagnostics = {} },
  }, 20000, buffer)
  local titles = vim.iter((actions and actions.result) or {})
    :map(function(action)
      return action.title
    end)
    :totable()
  check(vim.list_contains(titles, 'Open preview'), 'Open preview is a code action')
  check(vim.api.nvim_buf_get_commands(buffer, {}).LspCompmdPreview ~= nil, ':LspCompmdPreview is the buffer\'s')
  local preview = client:request_sync('workspace/executeCommand', {
    command = 'compmd.preview',
    arguments = { vim.uri_from_bufnr(buffer) },
  }, 20000, buffer)
  local url = preview and preview.result
  check(type(url) == 'string' and url:match('^http://127%.0%.0%.1:%d+/%x+/$') ~= nil, 'compmd.preview serves the page')
end

for _, running in ipairs(vim.lsp.get_clients({ name = 'compmd' })) do
  running:stop(true)
end
if #failures > 0 then
  print(#failures .. ' failed')
  os.exit(1)
end
print('all hold')
os.exit(0)
