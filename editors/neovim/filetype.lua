-- CompMD's file type, for Neovim's own detection (`runtime/lua/vim/filetype.lua`, upstream): a `.comp.md` file is
-- `compmd`, not Markdown. Until Neovim has it, a writer's config adds it with this.
vim.filetype.add({
  pattern = {
    ['.*%.comp%.md'] = 'compmd',
  },
})
