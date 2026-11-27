-- CompMD's file type: a `.comp.md` file is `compmd`, not Markdown. Neovim's own detection
-- (`runtime/lua/vim/filetype.lua`) gets the same pattern upstream.
vim.filetype.add({
  pattern = {
    ['.*%.comp%.md'] = 'compmd',
  },
})
