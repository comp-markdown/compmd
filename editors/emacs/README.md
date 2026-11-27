# compmd-ts-mode

A major mode for [CompMD](https://compmd.dev), Markdown with components, in Emacs 30 and later, on Emacs's built-in
tree-sitter and Eglot ([editor support](https://github.com/comp-markdown/compmd/blob/main/docs/editor-support.md),
decisions 6 and 9).

It is developed in [`comp-markdown/compmd`](https://github.com/comp-markdown/compmd), in its `editors/emacs/` folder,
where changes are made; [`comp-markdown/compmd-ts-mode`](https://github.com/comp-markdown/compmd-ts-mode) is that
folder published as a repository of its own, which MELPA builds the package from.

[`compmd-ts-mode.el`](compmd-ts-mode.el) gives `.comp.md` files:

- **highlighting** from CompMD's three grammars, `compmd`, `compmd-inline` (each block's inlines) and `cel` (each
  expression), from `comp-markdown/compmd-grammars`;
- **Imenu and outlines** of the headings and the containers;
- **the language server** through Eglot (`M-x eglot`): diagnostics, completion, hover and code actions;
- **the preview**: `compmd-ts-preview` (`C-c C-p`), the document in the browser, as the server serves it;
- a project for each `compmd.config.json`'s folder, which the server reads.

## Install

From MELPA, with MELPA in `package-archives`:

```
M-x package-install RET compmd-ts-mode RET
```

or with `use-package`:

```elisp
(use-package compmd-ts-mode
  :ensure t)
```

Until it is listed on MELPA, from its repository:

```
M-x package-vc-install RET https://github.com/comp-markdown/compmd-ts-mode RET
```

or with Emacs 30's `use-package`:

```elisp
(use-package compmd-ts-mode
  :vc (:url "https://github.com/comp-markdown/compmd-ts-mode" :rev :newest))
```

### The grammars

`M-x treesit-install-language-grammar` installs each of `compmd`, `compmd-inline` and `cel` from
`comp-markdown/compmd-grammars`, as the mode names them; it needs git and a C compiler.

### The language server

When `compmd-lsp` isn't found, Eglot asks whether to download it: `compmd-ts-lsp-install` fetches the release's binary
for the platform into `compmd-ts-lsp-install-directory` and checks its SHA-256 sum against the release's `SHA256SUMS`.
`npm i -g @compmd/lsp` or `cargo install compmd-lsp` puts it on the path instead.

## Check

[`test/check.el`](test/check.el) runs in a batch Emacs, given the grammars' folder and `compmd-lsp`:

```
emacs --batch -l test/check.el <grammars' folder> <compmd-lsp>
```

It checks the mode and the project, the faces of a container's name, a param, an interpolation and an inline component
(`compmd-inline`), an expression's number (`cel`) and a definition's name; Imenu; the server through Eglot, its
diagnostics, *Open preview*, `compmd-ts-preview` and the page served; and the release binary's name and sum.
[`test/install-grammars.el`](test/install-grammars.el) installs the three grammars into a folder, as a writer does:
`emacs --batch -l test/install-grammars.el <folder>`.

[`test/lint.el`](test/lint.el) runs MELPA's checks, package-lint, checkdoc and byte-compiling with warnings as errors:
`emacs --batch -l test/lint.el <folder>`, which it installs package-lint into from MELPA. The repository's CI
([`.github/workflows/ci.yml`](.github/workflows/ci.yml)) runs both, in Emacs 30 and the latest release, with the
grammars and the released `compmd-lsp`.

In `comp-markdown/compmd`, `cargo make emacs-check` builds `compmd-lsp` (its preview's page fresh) and the three grammars
from the repository's own sources, and runs `test/check.el` against them. It needs `emacs` (30 or later) and the
tree-sitter CLI. It isn't part of `cargo make ci`.

## Try it

In `comp-markdown/compmd`, `cargo make emacs-interactive [file]` builds the same and opens Emacs in the terminal
(`emacs -nw`) with the mode in place of your own config ([`test/interactive.el`](test/interactive.el)), Eglot started,
on the file, or without one on a copy of the Author's Guide's containers example in a temporary folder. Then
`M-x imenu` lists the headings and containers, `C-c C-p` opens the preview, and `M-x eglot-code-actions` lists the code
actions.

## License

[MIT](LICENSE).
