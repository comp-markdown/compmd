# CompMD in Helix

Helix needs no package of its own ([editor support](../../docs/editor-support.md), decision 6): CompMD comes through an
entry in Helix's `languages.toml` and its queries, which these files draft. They are opened as a pull request to Helix
after the first release of `compmd-lsp` and `tree-sitter-compmd`, with the maintainer's go (CMD-433).

| File | Upstream | What it adds |
|---|---|---|
| [`languages.toml`](languages.toml) | `languages.toml` | the languages `compmd` (`.comp.md` files) and `compmd-inline`, the language server `compmd-lsp`, and the two grammars from `comp-markdown/compmd-grammars` |
| [`queries/compmd/`](queries/compmd) | `runtime/queries/compmd/` | highlights (on Helix's Markdown ones), injections and text objects |
| [`queries/compmd-inline/`](queries/compmd-inline) | `runtime/queries/compmd-inline/` | the same for the inline grammar |

- The queries are the grammars' own (`queries/` in each grammar's folder) with Helix's capture names and properties.
  The inline ones spell out Helix's Markdown inline captures: Helix's `; inherits:` can't name `markdown.inline`.
- An expression's CEL is Helix's own `cel` language: a second one can't have its name.
- Helix has no folding, so there are no fold queries. A component is a type for text objects (`mat`, `]t`), a param a
  parameter (`maa`, `]a`).
- The preview opens from the code action *Open preview* (`space a`): Helix has no plugins to add a command.

## Until then

Copy the entry into `~/.config/helix/languages.toml` (the `[[grammar]]` entries' `rev` a commit of
`comp-markdown/compmd-grammars`), the queries into `~/.config/helix/runtime/queries/`, then `hx --grammar fetch` and
`hx --grammar build`. `compmd-lsp` comes from npm (`npm i -g @compmd/lsp`), cargo (`cargo install compmd-lsp`) or the
release's binary.

## Check

`cargo make helix-check` builds `compmd-lsp` (its preview's page fresh) and the two parsers, and runs
[`test/check.ts`](test/check.ts): a real Helix in a tmux session of its own, with the drafts as its config and runtime
in a temporary folder, read from its screen. It checks:

- the languages, their parsers and queries, with every query compiled (Helix's log says which it couldn't);
- the highlights of a container's name and a param, an interpolation and an inline component (`compmd-inline`,
  injected), an expression's number (`cel`, injected), a definition's name;
- the server's diagnostics, *Open preview* among the code actions, the browser opened at the page, and the page served.

It needs `hx`, `tmux` and the tree-sitter CLI. It isn't part of `cargo make ci`.

## Try it

`cargo make helix-interactive [file]` builds the same and opens Helix with the drafts in place of your own config
([`test/interactive.ts`](test/interactive.ts)), on the file, or without one on a copy of the Author's Guide's
containers example in a temporary folder. Then `:tree-sitter-highlight-name` shows a span's highlights, `mat` selects a
component, `space k` hovers and `space a` lists the code actions, *Open preview* among them.
