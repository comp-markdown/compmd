# CompMD in editors (proposed)

CompMD for the editors writers use: VS Code, Neovim, Helix, Zed and Emacs. Each gets highlighting, folding and selection
by structure from a grammar, and the processor's own help (its diagnostics, completion, exact highlighting) from a
language server. The text of other languages a document holds, a renderer's blocks, gets the same help as a file of that
language, as [content languages in editors](content-languages.md) decided. This design covers three things and what
they share: a tree-sitter grammar, a TextMate grammar and a language server.

## Who uses what

| Editor | Grammar | Language server through | Its package |
|---|---|---|---|
| VS Code | TextMate | the extension's client | `vscode-compmd`, on the Marketplace and Open VSX |
| Neovim | tree-sitter | its built-in client | entries in nvim-treesitter and nvim-lspconfig |
| Helix | tree-sitter | its built-in client | an entry in Helix's `languages.toml`, with its queries |
| Zed | tree-sitter | the extension | `zed-compmd`, in Zed's extensions |
| Emacs (29 and later) | tree-sitter (built in) | Eglot | `compmd-ts-mode`, on MELPA |

Shiki reads TextMate grammars too, so the site, the guides, the playground and the Developer Tools highlight CompMD
(a ```` ```compmd ```` block) with the same grammar VS Code uses.

## Decisions

1. **Three parts, each doing what it is good at.** A grammar runs in the editor, on every keystroke, without CompMD
   installed: it gives colours, folds and structure from the text alone, as well as a grammar can. The language server
   runs the processor: it knows what the text means (which names are components, which imports resolve, which params a
   signature takes) and gives diagnostics, completion, hover and exact highlighting on top of the grammar's (semantic
   tokens). An editor without the server still has a good grammar, and the server never re-parses CompMD itself.

2. **What the three share, written once:**
   - **The syntax's pieces** (`syntax/`, JavaScript): the regular expressions for a component's name, a param, a
     component line's opener and closer (`:::`, `::`, `::: /name`), an inline component, an interpolation, the
     frontmatter's `md` key and the opt-in syntax (spec §2, §3, §5). The tree-sitter grammar imports them
     (`grammar.js` is JavaScript), and the TextMate grammar is generated from them, so the two read the same text the
     same way.
   - **The token names** (`syntax/tokens.json`): one list of what a span is (a component's name, a param's key, a
     param's value, an expression, an interpolation's braces, a slot's name, a definition's fence), each with its
     tree-sitter capture (`@tag`, `@attribute`), its TextMate scope (`entity.name.tag.compmd`) and its semantic token
     type (`type`, `property`). A component's name is then the same colour in every editor, from a grammar or from the
     server.
   - **The injections** (`syntax/injections.json`): CEL inside an expression and an interpolation; a definition fence's
     language from its info string (`component=chart lang=jsx` is JSX, `compmd` is CompMD again); a renderer's block by
     its fence's language (content languages, decision 7); YAML or TOML for the frontmatter.
   - **The tests**: one test set (`tests/`), drawn from the spec's examples and the conformance set, in which each case
     is a document and the token name each span should have. All three are checked against it: the tree-sitter
     grammar with `tree-sitter query` on its `highlights.scm`, the TextMate grammar by tokenizing with `vscode-textmate`
     (as VS Code does), and the language server by its semantic tokens. A case is written once and checks all three.

3. **The tree-sitter grammar** (`tree-sitter-compmd`): two grammars on [tree-sitter-markdown]'s, as it is split, a
   block grammar and an inline grammar injected into each block's text. Each extends its Markdown grammar
   (`grammar(markdown, { … })`), so CommonMark and GFM stay theirs. The block grammar adds component lines (a container,
   its closer, a slot, a leaf, their params and continuation lines), the frontmatter's `md` key and component
   definitions; the inline grammar adds inline components, interpolation, wiki links (parsed always: a grammar can't
   read the frontmatter's opt-in) and inline math by Pandoc's rule (spec §2.3). The block grammar's external scanner
   (C) is copied from tree-sitter-markdown and given CompMD's lines: it is the one part that can't be inherited, and
   asking upstream for a hook first would hold the grammar up. Expressions are injected as CEL, from a small
   `tree-sitter-cel` beside it (none published covers CEL as spec §8.2 uses it). Its queries: `highlights.scm`,
   `folds.scm` (a container to its closer, a definition fence), `injections.scm` (generated from the injection table),
   and `textobjects.scm` (a component, its params).

4. **The TextMate grammar** (`compmd.tmLanguage.json`): generated from `syntax/` by a script, on VS Code's Markdown
   grammar as its base: CompMD's patterns come first, and `text.html.markdown` takes the rest, as other Markdown
   dialects' grammars do. A TextMate grammar matches line by line, so it sees less than tree-sitter: a container's
   closer is matched by its name, not by nesting, and a param that continues on the next line is matched by the line
   it starts on. Where it can't tell, it colours less, never wrongly; the language server's semantic tokens fill in.

5. **The language server** (`compmd-lsp`): a Rust crate and binary on the processor crate, in this repository, as
   `compmd-host` is, so it is released with the processor and always parses what the spec says. It gives:
   - **diagnostics**: the processor's (spec §10), with their codes, ranges and the spec's section;
   - **semantic tokens**: by the shared token names, from the processor's tree (which names are components, which are
     unknown, which params are a signature's);
   - **completion and hover**: component names from the document's imports and definitions, a signature's params and
     their types, `md.` paths, the built-ins; hover shows a component's signature and description;
   - **go to definition**: an import to its file or pack, a component to its definition;
   - **the content-language blocks**: each block a renderer draws, its language `id` and range (content languages,
     decision 3), as a custom request (`compmd/blocks`), so an editor package forwards requests inside a block to the
     writer's server for that language;
   - **document symbols and folding**: headings, components and definitions, for an outline.
   It works on the document's files and installed packs as any processor does (the configuration, `@compmd/vite`'s
   packs); a registry pack's signature comes from `@compmd/registry`'s cache, and a pack it can't load is a diagnostic,
   not a failure. It processes as the user types with the processor's incremental processing
   ([incremental.md](../processor/docs/incremental.md)).

6. **The editor packages are thin.** Each wires the grammar, the language server and the file type (`.comp.md`) into
   one editor, and adds what only that editor needs: `compmd-ts-mode`'s major mode, `zed-compmd`'s manifest, the VS Code
   extension's client and its content-language part (decision 7). Neovim and Helix need no package of their own: an
   entry upstream in nvim-treesitter, nvim-lspconfig and Helix's `languages.toml`, with the queries they keep in their
   own repositories.

7. **Content languages, through all three** ([content-languages.md](content-languages.md)):
   - **in a tree-sitter editor**, a renderer's block is injected by its fence's language, as Markdown's are, so the
     editor's own grammar for that language highlights it (the writer installs it as for any language);
   - **in VS Code**, the TextMate grammar embeds a block's language by its scope (`source.mermaid`), so an installed
     grammar for it works; a pack's own grammar (`compmd.languages`) can't be added to VS Code once it is running (an
     extension declares its grammars up front), so the extension tokenizes the block with the pack's grammar itself
     (`vscode-textmate`, in the extension) and gives the result as semantic tokens, by the scope's nearest token type;
   - **the language servers for them**: the server's `compmd/blocks` gives each block's language, range and the
     language's names elsewhere (`aliases`, `languageId`, `extensions`), and the package forwards completion, hover and
     diagnostics inside it to the server the writer has for that language: VS Code by a virtual document per block (its
     embedded-language pattern), the other editors where they support it, and not at all where they don't.

8. **Where they live:**
   - `comp-markdown/compmd-grammars`: both grammars, `tree-sitter-cel` and what they share (`syntax/`, `tests/`). The
     tree-sitter grammar is in `tree-sitter/`; Helix, Zed and nvim-treesitter each take a grammar from a folder of a
     repository. Published as `tree-sitter-compmd` on npm and crates.io (with the bindings `tree-sitter generate`
     writes), and the TextMate grammar as `@compmd/textmate` on npm, for Shiki and the VS Code extension. Its tests
     take the spec's examples and the conformance set from this repository at a release's tag, and the language
     server's semantic tokens from its release.
   - `comp-markdown/compmd`, this repository: the language server, `compmd-lsp`, on crates.io and as release binaries
     for macOS, Linux and Windows.
   - `comp-markdown/vscode-compmd`, `comp-markdown/zed-compmd`, `comp-markdown/compmd-ts-mode`: the editor packages,
     each in its own repository, as their stores expect.
   Each grammar names the spec version it parses (a grammar for version 1).

## What it needs

- Dependencies, in `compmd-grammars`: `tree-sitter-cli` (to generate and test), `@tree-sitter-grammars/tree-sitter-markdown`
  (the grammars it extends), the bindings' own (`node-gyp-build`, the `cc` and `tree-sitter-language` crates), and
  `vscode-textmate` with `vscode-oniguruma` for the TextMate tests. In the language server: `lsp-server` and `lsp-types`
  (rust-analyzer's, synchronous, with no async runtime). In the VS Code extension: `vscode-languageclient` and
  `vscode-textmate`.
- In this repository: the processor's blocks (content languages, decision 3) and the language server.
- In packs: `compmd.languages`, checked (content languages, decision 2).
- CI in `compmd-grammars`: generate, check the generated parser is committed (tree-sitter grammars commit
  `src/parser.c`), and run the test set on both grammars and the released server.

## The order

The shared `syntax/` and `tests/` first, then the tree-sitter grammar and the TextMate grammar (either order), the
processor's blocks and the language server, then the editor packages, each needing its grammar and the server.

## Open

- The language server's packages beyond crates.io and release binaries: an npm package with the binaries (as other
  servers ship them for editors that install from npm), proposed only once an editor package needs it.
- Whether the language server also formats documents (a param list, a table's columns), proposed: later, in a design
  of its own.

[tree-sitter-markdown]: https://github.com/tree-sitter-grammars/tree-sitter-markdown
