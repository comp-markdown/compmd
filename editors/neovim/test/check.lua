-- The plugin checked in a headless Neovim, as a writer installs it, against local builds: the file type, the three
-- parsers installed through nvim-treesitter with their queries, and the language server through `lsp/compmd.lua`.
-- `cargo make neovim-check` builds what it needs and runs it:
--
--   nvim --headless -l editors/neovim/test/check.lua <nvim-treesitter> <compmd-grammars> <install folder> <compmd-lsp>
--
-- It exits 0 when everything holds, else 1, saying what didn't.

local treesitter, grammars, site, server = arg[1], arg[2], arg[3], arg[4]
local here = vim.fs.dirname(vim.fs.dirname(vim.fs.abspath(arg[0])))
local failures = {}

local function check(holds, what)
  print((holds and 'ok   ' or 'FAIL ') .. what)
  if not holds then
    table.insert(failures, what)
  end
end

-- A browser that opens nothing, so the preview's command can run here.
dofile(here .. '/test/setup.lua')(treesitter, grammars, site, server, { preview = { browser = 'true' } })
check(
  vim.iter({ 'compmd', 'compmd_inline', 'cel' }):all(function(grammar)
    return vim.list_contains(require('nvim-treesitter').get_installed(), grammar)
  end),
  ':TSInstall compmd installs the three parsers with their queries'
)

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
check(vim.treesitter.highlighter.active[buffer] ~= nil, 'a compmd buffer is highlighted by its parser')

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

local attached = vim.wait(20000, function()
  return #vim.lsp.get_clients({ bufnr = buffer, name = 'compmd' }) > 0
end, 50)
check(attached, 'compmd-lsp attaches through lsp/compmd.lua, enabled by the plugin')
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

-- The drawing: the server's plan, drawn by extmarks, giving way on the cursor's line.
local drawing = require('compmd.drawing')
local function marks(of)
  return vim.iter(vim.api.nvim_buf_get_extmarks(of, drawing.namespace, 0, -1, { details = true }))
    :map(function(mark)
      return { row = mark[2], col = mark[3], details = mark[4] }
    end)
    :totable()
end
local function texts(mark)
  local chunks = mark.details.virt_text or (mark.details.virt_lines or {})[1] or {}
  return table.concat(vim.iter(chunks):map(function(chunk)
    return chunk[1]
  end):totable())
end
local function on_row(of, row, holds)
  return vim.iter(marks(of)):any(function(mark)
    return mark.row == row and holds(mark)
  end)
end
vim.api.nvim_win_set_cursor(0, { 8, 0 })
local drawn = vim.wait(20000, function()
  return #marks(buffer) > 0
end, 50)
check(drawn, 'the buffer is drawn by the server\'s plan')
check(on_row(buffer, 4, function(mark)
  return texts(mark) == '◉'
end), "a heading's # is replaced by its glyph")
check(on_row(buffer, 6, function(mark)
  return mark.details.conceal_lines == ''
end), "a container's opener is hidden")
check(on_row(buffer, 6, function(mark)
  return texts(mark):find('┌─ nowhere', 1, true) ~= nil and mark.details.virt_lines_above
end), 'and a top border drawn above it, titled with its name')
check(vim.fn.hlexists('CompmdHeading1') == 1, 'each style has its highlight group')
vim.api.nvim_win_set_cursor(0, { 7, 0 })
drawing.redraw(buffer)
check(not on_row(buffer, 6, function(mark)
  return mark.details.conceal_lines ~= nil
end), "the opener's line gives way while the cursor is on it")

local markdown = folder .. '/notes.md'
vim.fn.writefile({ '# Notes', '', '::: box', 'Text.', '::: /box' }, markdown)
vim.cmd.edit(markdown)
local plain = vim.api.nvim_get_current_buf()
check(vim.bo[plain].filetype == 'markdown', 'a .md file is markdown')
vim.api.nvim_win_set_cursor(0, { 4, 0 })
local plain_drawn = vim.wait(20000, function()
  return #marks(plain) > 0
end, 50)
check(plain_drawn and on_row(plain, 0, function(mark)
  return texts(mark) == '◉'
end), 'a Markdown buffer is drawn too')
check(not on_row(plain, 2, function(mark)
  return mark.details.conceal_lines ~= nil
end), 'and its ::: is text, not a box')

for _, running in ipairs(vim.lsp.get_clients({ name = 'compmd' })) do
  running:stop(true)
end
if #failures > 0 then
  print(#failures .. ' failed')
  os.exit(1)
end
print('all hold')
os.exit(0)
