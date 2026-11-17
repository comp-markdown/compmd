# A tree-sitter grammar (proposed)

`tree-sitter-compmd`: CompMD for editors that use tree-sitter, to highlight it, fold it and select by its structure
(Neovim, Helix, Zed, and GitHub's code view in time). It parses what the spec says, tested on the spec's examples and
the conformance set.

## Decisions

1. **Two grammars on tree-sitter-markdown's, as it is split.** [tree-sitter-markdown] is a block grammar and an inline
   grammar, the inline one injected into each block's text. `tree-sitter-compmd` is the same pair, each extending its
   Markdown grammar with tree-sitter's grammar inheritance (`grammar(markdown, { … })`), so CommonMark and GFM stay
   theirs and only CompMD's constructs are new. The block grammar's external scanner (C, where Markdown's indentation
   and containers are tracked) is copied and given CompMD's lines; it is the one part that can't be inherited.
2. **The block grammar** adds component lines (spec §3): a container's `::: name {params}` and `::: /name`, a slot's
   `:: #name`, a leaf's `:: name[fallback] {params}`, with their params (§5: bare, quoted, expressions in
   parentheses, flags) and continuation lines; the frontmatter's `md` key, as a node of the YAML or TOML frontmatter,
   which is injected as Markdown's is; and component definitions (§7), whose fences inject their language from the
   info string (`component=chart lang=jsx` takes `jsx`, `compmd` takes this grammar again).
3. **The inline grammar** adds inline components (`:name[fallback]{params}`), interpolation (`{{md.doc.title}}`),
   wiki links (opted in by `md.syntax`, so parsed always and highlighted as such; a grammar can't read the
   frontmatter), and inline math by Pandoc's rule (§2.3).
4. **Expressions are CEL, injected.** An expression in parentheses (`{when=(md.input.n > 2)}`) and an interpolation's
   path are injected as CEL, from a small `tree-sitter-cel` grammar in the same repository (none is published that
   covers CEL's grammar as spec §8.2 uses it), so highlighting knows a function from a field.
5. **Queries for what editors do:** `highlights.scm` (component names as tags, params as attributes, CEL as code),
   `folds.scm` (a container to its closer, a definition fence), `injections.scm`, and `textobjects.scm` for a
   component and its params.
6. **Tests from the spec.** tree-sitter's corpus tests, one file per section, with the spec's examples; and a test that
   parses every `input.comp.md` of the conformance set with no `ERROR` node except where a case expects a diagnostic
   about the syntax.
7. **Where it lives:** `tree-sitter/` in this repository, beside the processor, versioned with the spec (a grammar for
   version 1). Published as tree-sitter grammars are: npm `tree-sitter-compmd` and crates.io `tree-sitter-compmd`,
   with the bindings `tree-sitter generate` writes, and to editors where they list grammars: an nvim-treesitter
   parser entry, a Helix language, and a Zed extension (`zed-compmd`, with `.comp.md` files).

## What it needs

- Dependencies: `tree-sitter-cli` (to generate and test, at build only), `@tree-sitter-grammars/tree-sitter-markdown`
  (the grammars it extends), and the bindings' own (`node-gyp-build`, the `cc` and `tree-sitter-language` crates),
  which every tree-sitter grammar has.
- A CI job: generate, test the corpus and the conformance set, and check the generated parser is committed (tree-sitter
  grammars commit `src/parser.c`).

## Open

- The editors to try it in first: Neovim and Helix (recommended: both load a local grammar without publishing), then
  Zed's extension once it works.
- Whether the copied external scanner is acceptable, or tree-sitter-markdown should be asked first to let a grammar
  add lines to its scanner (a change upstream, slower).

[tree-sitter-markdown]: https://github.com/tree-sitter-grammars/tree-sitter-markdown
