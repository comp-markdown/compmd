-- CompMD's entries in nvim-treesitter's `lua/nvim-treesitter/parsers.lua` (upstream): the three grammars of
-- comp-markdown/compmd-grammars, each from its folder, at the revision of a release. nvim-treesitter keeps their
-- queries in `runtime/queries/<grammar>/`: the grammar repository's `queries/` folders, whose captures are
-- nvim-treesitter's own names already. `revision` and `maintainers` are filled in when the pull request is opened.
return {
  compmd = {
    install_info = {
      url = 'https://github.com/comp-markdown/compmd-grammars',
      location = 'tree-sitter/compmd',
      revision = 'the release commit',
    },
    maintainers = { 'the maintainer' },
    requires = { 'compmd_inline', 'cel' },
    tier = 2,
  },
  compmd_inline = {
    install_info = {
      url = 'https://github.com/comp-markdown/compmd-grammars',
      location = 'tree-sitter/compmd-inline',
      revision = 'the release commit',
    },
    maintainers = { 'the maintainer' },
    readme_note = 'used by compmd',
    tier = 2,
  },
  cel = {
    install_info = {
      url = 'https://github.com/comp-markdown/compmd-grammars',
      location = 'tree-sitter/cel',
      revision = 'the release commit',
    },
    maintainers = { 'the maintainer' },
    readme_note = "CompMD's expressions",
    tier = 2,
  },
}
