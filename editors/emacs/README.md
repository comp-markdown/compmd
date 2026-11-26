# compmd-ts-mode

A major mode for CompMD in Emacs 30 and later ([editor support](../../docs/editor-support.md), decisions 6 and 9), on
Emacs's built-in tree-sitter and Eglot. It is written here until it moves to its own repository,
`comp-markdown/compmd-ts-mode`, and to MELPA, each with the maintainer's go.

[`compmd-ts-mode.el`](compmd-ts-mode.el) gives `.comp.md` files:

- **highlighting** from CompMD's three grammars, `compmd`, `compmd-inline` (each block's inlines) and `cel` (each
  expression), from `comp-markdown/compmd-grammars`: `M-x treesit-install-language-grammar` installs each;
- **Imenu and outlines** of the headings and the containers;
- **the language server** through Eglot (`M-x eglot`): diagnostics, completion, hover and code actions. When
  `compmd-lsp` isn't found, Eglot asks whether to download it: `compmd-lsp-install` fetches the release's binary for
  the platform into `compmd-lsp-install-directory` and checks its SHA-256 sum against the release's `SHA256SUMS`;
- **the preview**: `compmd-preview` (`C-c C-p`), the document in the browser, as the server serves it;
- a project for each `compmd.config.json`'s folder, which the server reads.

## Check

`cargo make emacs-check` builds `compmd-lsp` (its preview's page fresh) and the three grammars, and runs
[`test/check.el`](test/check.el) in a batch Emacs. It checks the mode and the project, the faces of a container's name,
a param, an interpolation and an inline component (`compmd-inline`), an expression's number (`cel`) and a definition's
name; Imenu; the server through Eglot, its diagnostics, *Open preview*, `compmd-preview` and the page served; and the
release binary's name and sum. It needs `emacs` (30 or later) and the tree-sitter CLI. It isn't part of `cargo make ci`.

## Try it

`cargo make emacs-interactive [file]` builds the same and opens Emacs in the terminal (`emacs -nw`) with the mode in
place of your own config ([`test/interactive.el`](test/interactive.el)), Eglot started, on the file, or without one on
a copy of the Author's Guide's containers example in a temporary folder. Then `M-x imenu` lists the headings and
containers, `C-c C-p` opens the preview, and `M-x eglot-code-actions` lists the code actions.
