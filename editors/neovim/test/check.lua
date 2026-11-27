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
  return texts(mark) == 'H1' and mark.details.virt_text[1][2] == 'CompmdHeadingMarker'
end), "a heading's # is replaced by its level, H1, set back")
-- The status column of line `lnum` in the current window, as Neovim draws it on each of the line's rows.
local function status_column(lnum)
  return vim.api.nvim_eval_statusline(vim.wo.statuscolumn, { winid = 0, use_statuscol_lnum = lnum }).str
end
check(
  status_column(8):find('│', 1, true) ~= nil and not status_column(7):find('│', 1, true),
  "a container's lines have its gutter in the status column, its opener none: " .. status_column(8)
)
check(not on_row(buffer, 7, function(mark)
  return mark.details.virt_text ~= nil
end), 'and its text is as written, so a wrapped row keeps the gutter')
check(on_row(buffer, 6, function(mark)
  return mark.details.hl_group == 'CompmdComponent'
end) and not on_row(buffer, 6, function(mark)
  return mark.details.conceal_lines ~= nil or mark.details.virt_text ~= nil
end), 'its opener is as written, its name in the component style')
check(vim.fn.hlexists('CompmdHeading1') == 1, 'each style has its highlight group')
for level = 1, 6 do
  check(
    vim.api.nvim_get_hl(0, { name = 'CompmdHeading' .. level, link = false }).fg ~= nil,
    'heading ' .. level .. ' has a colour, in the default colour scheme too'
  )
end
vim.api.nvim_win_set_cursor(0, { 5, 0 })
drawing.redraw(buffer)
check(not on_row(buffer, 4, function(mark)
  return texts(mark) == 'H1'
end), "the heading's line gives way while the cursor is on it")

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
  return texts(mark) == 'H1'
end), 'a Markdown buffer is drawn too')
check(not on_row(plain, 2, function(mark)
  return mark.details.conceal_lines ~= nil
end), 'and its ::: is text, not a box')

-- The lines of `of` as drawn, rebuilt from its text and the extmarks: concealed text left out, inline text put in,
-- hidden lines dropped, and lines drawn above or below a line added. A headless Neovim has no screen to read.
local function drawn_lines(of)
  local all = marks(of)
  local out = {}
  for row, text in ipairs(vim.api.nvim_buf_get_lines(of, 0, -1, false)) do
    local hidden, inserted, concealed, above, below, overlay = false, {}, {}, {}, {}, nil
    for _, mark in ipairs(all) do
      local details = mark.details
      if mark.row == row - 1 then
        hidden = hidden or details.conceal_lines ~= nil
        if details.virt_text_pos == 'overlay' then
          overlay = vim.trim(texts(mark))
        end
        if details.conceal == '' then
          table.insert(concealed, { mark.col, details.end_col })
        end
        if details.virt_text_pos == 'inline' then
          table.insert(inserted, { mark.col, texts(mark) })
        end
        if details.virt_lines then
          table.insert(details.virt_lines_above and above or below, texts(mark))
        end
      end
    end
    vim.list_extend(out, above)
    if overlay then
      table.insert(out, overlay)
    elseif not hidden then
      local line = {}
      for col = 0, #text do
        for _, insert in ipairs(inserted) do
          if insert[1] == col then
            table.insert(line, insert[2])
          end
        end
        local inside = vim.iter(concealed):any(function(span)
          return span[1] <= col and col < span[2]
        end)
        if col < #text and not inside then
          table.insert(line, text:sub(col + 1, col + 1))
        end
      end
      table.insert(out, table.concat(line))
    end
    vim.list_extend(out, below)
  end
  return out
end

-- Tables: an aligned one is drawn with its separators in line; typing `|` aligns an unaligned one, which is then
-- drawn so too (on-type formatting, which the plugin turns on).
local tables = folder .. '/tables.md'
vim.fn.writefile({
  'Tides.',
  '',
  '| Tide | Time  |',
  '| ---- | ----: |',
  '| High | 06:12 |',
  '',
  '| Tide | Time |',
  '| --- | ---: |',
  '| Low | 12:30 |',
}, tables)
vim.cmd.edit(tables)
local table_buffer = vim.api.nvim_get_current_buf()
vim.api.nvim_win_set_cursor(0, { 1, 0 })
local function box(first, last)
  local lines = drawn_lines(table_buffer)
  local widths = {}
  for index = first, last do
    widths[vim.fn.strdisplaywidth(lines[index] or '')] = true
  end
  return vim.tbl_count(widths) == 1 and lines[first]:find('┌', 1, true) ~= nil
end
check(vim.wait(20000, function()
  return box(3, 7)
end, 50), "an aligned table's box lines up: " .. table.concat(drawn_lines(table_buffer), ' / '))
-- The last row's closing `|` deleted and typed again.
vim.api.nvim_win_set_cursor(0, { 9, 0 })
vim.api.nvim_feedkeys(vim.keycode('$xa|<Esc>'), 'xt', false)
local aligned = vim.wait(20000, function()
  local text = vim.api.nvim_buf_get_lines(table_buffer, 6, 9, false)
  return vim.fn.strdisplaywidth(text[1]) == vim.fn.strdisplaywidth(text[2])
    and vim.fn.strdisplaywidth(text[1]) == vim.fn.strdisplaywidth(text[3])
end, 50)
check(aligned, 'typing | aligns the table: ' .. table.concat(vim.api.nvim_buf_get_lines(table_buffer, 6, 9, false), ' / '))

-- The frontmatter: its fences drawn as rules and its lines set back, both giving way while the cursor is in it.
local front = folder .. '/front.md'
vim.fn.writefile({ '---', 'title: Harbour notes', '---', '', 'Text.' }, front)
vim.cmd.edit(front)
local front_buffer = vim.api.nvim_get_current_buf()
vim.api.nvim_win_set_cursor(0, { 5, 0 })
local ruled = vim.wait(20000, function()
  local lines = drawn_lines(front_buffer)
  return lines[1] ~= nil and lines[1]:find('─', 1, true) == 1 and lines[2] == 'title: Harbour notes'
end, 50)
check(ruled, 'the frontmatter\'s fences are rules: ' .. table.concat(drawn_lines(front_buffer), ' / '))
check(on_row(front_buffer, 1, function(mark)
  return mark.details.hl_group == 'CompmdFrontmatter' and mark.details.priority > 125
end), 'and its lines are set back, over tree-sitter and semantic tokens')
vim.api.nvim_win_set_cursor(0, { 1, 0 })
drawing.redraw(front_buffer)
check(
  drawn_lines(front_buffer)[1] == '---' and drawn_lines(front_buffer)[3]:find('─', 1, true) == 1,
  'on the first line, as a file opens, the other fence is still a rule'
)
vim.api.nvim_win_set_cursor(0, { 2, 0 })
drawing.redraw(front_buffer)
check(not on_row(front_buffer, 1, function(mark)
  return mark.details.hl_group == 'CompmdFrontmatter'
end), 'its lines show as written while the cursor is in it')

-- A box whose opener goes on over a continuation line (`::: +`): its opener and closer as written, and the gutter
-- before the lines between them only.
local boxed = folder .. '/boxed.comp.md'
vim.fn.writefile({ '::: box {padding=s}', '::: + gap=m', 'Inside.', '::: /box', '', 'After.' }, boxed)
vim.cmd.edit(boxed)
local boxed_buffer = vim.api.nvim_get_current_buf()
vim.api.nvim_win_set_cursor(0, { 6, 0 })
local function boxed_lines()
  return table.concat(drawn_lines(boxed_buffer), ' / ')
end
check(vim.wait(20000, function()
  return status_column(3):find('│', 1, true) ~= nil
end, 50), 'a box has its gutter on the line inside: ' .. status_column(3))
check(
  not status_column(2):find('│', 1, true) and not status_column(4):find('│', 1, true),
  'and none on its continuation line or its closer'
)
check(
  vim.deep_equal(drawn_lines(boxed_buffer), { '::: box {padding=s}', '::: + gap=m', 'Inside.', '::: /box', '', 'After.' }),
  'its lines as written, with no borders: ' .. boxed_lines()
)

-- A code block where it is written: its fence marks set back, its language coloured, its code left to Neovim.
local fenced = folder .. '/fenced.md'
vim.fn.writefile({ 'Code:', '', '```lua', 'local tide = 3', '```' }, fenced)
vim.cmd.edit(fenced)
local fenced_buffer = vim.api.nvim_get_current_buf()
vim.api.nvim_win_set_cursor(0, { 1, 0 })
local styled = function(row, group)
  return on_row(fenced_buffer, row, function(mark)
    return mark.details.hl_group == group
  end)
end
check(vim.wait(20000, function()
  return styled(2, 'CompmdCodeFence')
end, 50) and styled(2, 'CompmdCodeLanguage') and styled(4, 'CompmdCodeFence'), "a code block's fences are set back and its language coloured")
check(#vim.tbl_filter(function(mark)
  return mark.row == 3
end, marks(fenced_buffer)) == 0 and drawn_lines(fenced_buffer)[3] == '```lua', 'its code and fences are as written')

-- Insert mode is drawn too, by default, the cursor's line giving way as in normal mode.
local front_client = vim.lsp.get_clients({ bufnr = front_buffer, name = 'compmd' })[1]
check(
  front_client ~= nil and vim.deep_equal(drawing.modes(front_client), { 'n', 'i', 'c', 't' }),
  'normal, insert, command and terminal modes are drawn by default'
)

for _, running in ipairs(vim.lsp.get_clients({ name = 'compmd' })) do
  running:stop(true)
end
if #failures > 0 then
  print(#failures .. ' failed')
  os.exit(1)
end
print('all hold')
os.exit(0)
