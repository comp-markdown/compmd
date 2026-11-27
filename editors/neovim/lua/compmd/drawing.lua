-- The drawing ([a document drawn in the editor], decisions 2 to 6): the server's plan for the lines a window shows
-- (`compmd/drawing`), drawn by extmarks. A mark gives way while the cursor is on one of its `reveal` lines, and the
-- buffer is drawn only in the modes `compmd.render.modes` names (normal, command and terminal by default). Each style
-- is a highlight group linked to the Markdown one, so a colour scheme or the writer restyles it.
--
-- [a document drawn in the editor]: https://github.com/comp-markdown/compmd/blob/main/docs/editor-rendering.md

local M = {}

local namespace = vim.api.nvim_create_namespace('compmd.drawing')

--- The primitives this renderer draws, all six.
M.primitives = { 'hide', 'replace', 'insert', 'line', 'prefix', 'style' }

--- How many lines around a window's are asked for, so a scroll of a few lines needs no new plan.
local MARGIN = 100

--- The highlight group of each style, and the group it is linked to by default.
local links = {
  ['compmd.heading.1'] = { 'CompmdHeading1', '@markup.heading.1.markdown' },
  ['compmd.heading.2'] = { 'CompmdHeading2', '@markup.heading.2.markdown' },
  ['compmd.heading.3'] = { 'CompmdHeading3', '@markup.heading.3.markdown' },
  ['compmd.heading.4'] = { 'CompmdHeading4', '@markup.heading.4.markdown' },
  ['compmd.heading.5'] = { 'CompmdHeading5', '@markup.heading.5.markdown' },
  ['compmd.heading.6'] = { 'CompmdHeading6', '@markup.heading.6.markdown' },
  ['compmd.code'] = { 'CompmdCode', 'ColorColumn' },
  ['compmd.code.language'] = { 'CompmdCodeLanguage', '@label' },
  ['compmd.code.inline'] = { 'CompmdCodeInline', '@markup.raw.markdown_inline' },
  ['compmd.rule'] = { 'CompmdRule', '@punctuation.special' },
  ['compmd.bullet.1'] = { 'CompmdBullet1', '@markup.list' },
  ['compmd.bullet.2'] = { 'CompmdBullet2', '@markup.list' },
  ['compmd.bullet.3'] = { 'CompmdBullet3', '@markup.list' },
  ['compmd.bullet.4'] = { 'CompmdBullet4', '@markup.list' },
  ['compmd.checkbox.checked'] = { 'CompmdCheckboxChecked', '@markup.list.checked' },
  ['compmd.checkbox.unchecked'] = { 'CompmdCheckboxUnchecked', '@markup.list.unchecked' },
  ['compmd.callout.note'] = { 'CompmdCalloutNote', 'DiagnosticInfo' },
  ['compmd.callout.tip'] = { 'CompmdCalloutTip', 'DiagnosticOk' },
  ['compmd.callout.important'] = { 'CompmdCalloutImportant', 'DiagnosticHint' },
  ['compmd.callout.warning'] = { 'CompmdCalloutWarning', 'DiagnosticWarn' },
  ['compmd.callout.caution'] = { 'CompmdCalloutCaution', 'DiagnosticError' },
  ['compmd.table.border'] = { 'CompmdTableBorder', '@punctuation.special' },
  ['compmd.container.border'] = { 'CompmdContainerBorder', '@punctuation.bracket' },
  ['compmd.container.top'] = { 'CompmdContainerTop', 'CompmdContainerBorder' },
  ['compmd.container.divider'] = { 'CompmdContainerDivider', 'CompmdContainerBorder' },
  ['compmd.container.bottom'] = { 'CompmdContainerBottom', 'CompmdContainerBorder' },
  ['compmd.component'] = { 'CompmdComponent', '@tag' },
  ['compmd.slot'] = { 'CompmdSlot', '@label' },
  ['compmd.param'] = { 'CompmdParam', 'Comment' },
  ['compmd.frontmatter.md'] = { 'CompmdFrontmatterMd', 'Comment' },
}

--- The highlight group of `style`: its own, or for a quote of any depth `CompmdQuoteN`, linked to `@markup.quote`.
function M.group(style)
  local known = links[style]
  if known then
    return known[1]
  end
  local depth = style:match('^compmd%.quote%.(%d+)$')
  if depth then
    local group = 'CompmdQuote' .. depth
    vim.api.nvim_set_hl(0, group, { link = '@markup.quote', default = true })
    return group
  end
  return 'Normal'
end

--- Links every style's group to its Markdown one, unless the colour scheme or the writer set it.
function M.define_groups()
  for _, link in pairs(links) do
    vim.api.nvim_set_hl(0, link[1], { link = link[2], default = true })
  end
end

--- The drawing of each buffer: its client, its last plan and the mode it was drawn in.
local buffers = {}

--- The byte column of `position` in `buffer`, which counts in `encoding`.
local function column(buffer, position, encoding)
  local line = vim.api.nvim_buf_get_lines(buffer, position.line, position.line + 1, false)[1] or ''
  if encoding == 'utf-8' then
    return math.min(position.character, #line)
  end
  local ok, byte = pcall(vim.str_byteindex, line, encoding, position.character, false)
  return ok and byte or #line
end

--- Whether the cursor of `window` is on one of `mark`'s reveal lines, so it gives way.
local function revealed(mark, cursor)
  local reveal = mark.reveal
  return reveal ~= nil and reveal.first <= cursor and cursor <= reveal.last
end

--- The modes the buffer is drawn in, by their first letter: the client's `compmd.render.modes`.
local function modes(client)
  local settings = client.settings and client.settings.compmd or {}
  return (settings.render and settings.render.modes) or { 'n', 'c', 't' }
end

--- The chunks of a `line` mark: its segments, then its fill repeated to `width` display cells.
local function line_chunks(mark, width)
  local chunks, used = {}, 0
  for _, segment in ipairs(mark.segments or {}) do
    table.insert(chunks, { segment.text, M.group(segment.style) })
    used = used + vim.fn.strdisplaywidth(segment.text)
  end
  local fill = mark.fill
  if fill and fill.text ~= '' then
    local cell = vim.fn.strdisplaywidth(fill.text)
    local count = math.max(0, math.floor((width - used) / math.max(cell, 1)))
    table.insert(chunks, { fill.text:rep(count), M.group(fill.style) })
  end
  return chunks
end

--- Draws `mark` in `buffer`, whose window is `width` cells wide.
local function draw(buffer, mark, encoding, width)
  local set = function(line, col, options)
    pcall(vim.api.nvim_buf_set_extmark, buffer, namespace, line, col, options)
  end
  local kind = mark.kind
  if kind == 'hide' or kind == 'replace' then
    local range = mark.range
    local whole = range.start.character == 0 and range['end'].character == 0 and range['end'].line > range.start.line
    if kind == 'hide' and whole then
      set(range.start.line, 0, { end_row = range['end'].line - 1, conceal_lines = '' })
      return
    end
    local start = column(buffer, range.start, encoding)
    set(range.start.line, start, {
      end_row = range['end'].line,
      end_col = column(buffer, range['end'], encoding),
      conceal = '',
    })
    if kind == 'replace' then
      set(range.start.line, start, { virt_text = { { mark.text, M.group(mark.style) } }, virt_text_pos = 'inline' })
    end
  elseif kind == 'insert' then
    local position = mark.position
    set(position.line, column(buffer, position, encoding), {
      virt_text = { { mark.text, M.group(mark.style) } },
      virt_text_pos = 'inline',
    })
  elseif kind == 'line' then
    set(mark.line, 0, { virt_lines = { line_chunks(mark, width) }, virt_lines_above = mark.above })
  elseif kind == 'prefix' then
    for line = mark.lines.first, mark.lines.last do
      set(line, 0, {
        virt_text = { { mark.text, M.group(mark.style) } },
        virt_text_pos = 'inline',
        virt_text_repeat_linebreak = true,
      })
    end
  elseif kind == 'style' then
    local range = mark.range
    set(range.start.line, column(buffer, range.start, encoding), {
      end_row = range['end'].line,
      end_col = column(buffer, range['end'], encoding),
      hl_group = M.group(mark.style),
      hl_eol = mark.wholeLine,
      priority = 90,
    })
  end
end

--- Draws `buffer`'s last plan again, as the cursor and the mode are now: nothing outside the drawing's modes, and a
--- mark on the cursor's reveal lines left out.
function M.redraw(buffer)
  local state = buffers[buffer]
  if not state or not vim.api.nvim_buf_is_valid(buffer) then
    return
  end
  vim.api.nvim_buf_clear_namespace(buffer, namespace, 0, -1)
  local mode = vim.api.nvim_get_mode().mode:sub(1, 1)
  if not state.plan or not vim.list_contains(modes(state.client), mode) then
    return
  end
  local window = vim.fn.bufwinid(buffer)
  local cursor = window ~= -1 and vim.api.nvim_win_get_cursor(window)[1] - 1 or -1
  local width = window ~= -1 and vim.api.nvim_win_get_width(window) or 80
  for _, mark in ipairs(state.plan.marks) do
    if not revealed(mark, cursor) then
      draw(buffer, mark, state.client.offset_encoding, width)
    end
  end
end

--- Asks for the plan of the lines `buffer`'s window shows, and draws it unless the buffer changed since.
function M.request(buffer)
  local state = buffers[buffer]
  if not state or not vim.api.nvim_buf_is_valid(buffer) then
    return
  end
  local window = vim.fn.bufwinid(buffer)
  local first, last = 0, vim.api.nvim_buf_line_count(buffer)
  if window ~= -1 then
    first = math.max(0, vim.fn.line('w0', window) - 1 - MARGIN)
    last = vim.fn.line('w$', window) + MARGIN
  end
  local params = {
    textDocument = { uri = vim.uri_from_bufnr(buffer), version = vim.lsp.util.buf_versions[buffer] },
    range = { start = { line = first, character = 0 }, ['end'] = { line = last, character = 0 } },
  }
  state.client:request('compmd/drawing', params, function(err, plan)
    if err or not plan or plan == vim.NIL or not buffers[buffer] then
      return
    end
    if plan.version < (vim.lsp.util.buf_versions[buffer] or 0) then
      return
    end
    buffers[buffer].plan = plan
    M.redraw(buffer)
  end, buffer)
end

--- Draws `buffer` with `client`'s plans from now on: asked for when it opens, after each change the client sends, on
--- a scroll, and when the server asks (`compmd/drawing/refresh`); drawn again when the cursor or the mode moves.
function M.attach(client, buffer)
  if buffers[buffer] then
    return
  end
  buffers[buffer] = { client = client }
  local window = vim.fn.bufwinid(buffer)
  if window ~= -1 then
    vim.wo[window][0].conceallevel = 2
    vim.wo[window][0].concealcursor = 'nvic'
  end
  local group = vim.api.nvim_create_augroup('compmd.drawing.' .. buffer, { clear = true })
  local on = function(events, callback, options)
    vim.api.nvim_create_autocmd(events, vim.tbl_extend('force', { group = group, buffer = buffer, callback = callback }, options or {}))
  end
  on('LspNotify', function(event)
    if event.data.method == 'textDocument/didChange' and event.data.client_id == client.id then
      M.request(buffer)
    end
  end)
  on('WinScrolled', function()
    M.request(buffer)
  end)
  on({ 'CursorMoved', 'ModeChanged' }, function()
    M.redraw(buffer)
  end)
  on('BufWinEnter', function()
    local shown = vim.fn.bufwinid(buffer)
    vim.wo[shown][0].conceallevel = 2
    vim.wo[shown][0].concealcursor = 'nvic'
  end)
  on('LspDetach', function(event)
    if event.data.client_id == client.id then
      M.detach(buffer)
    end
  end)
  M.request(buffer)
end

--- Stops drawing `buffer`, and clears what was drawn.
function M.detach(buffer)
  buffers[buffer] = nil
  pcall(vim.api.nvim_del_augroup_by_name, 'compmd.drawing.' .. buffer)
  if vim.api.nvim_buf_is_valid(buffer) then
    vim.api.nvim_buf_clear_namespace(buffer, namespace, 0, -1)
  end
end

--- The server asks for every plan again: each buffer `client` draws asks.
function M.refresh(client)
  for buffer, state in pairs(buffers) do
    if state.client.id == client.id then
      M.request(buffer)
    end
  end
end

--- The namespace the marks are drawn in, for a check to read.
M.namespace = namespace

return M
