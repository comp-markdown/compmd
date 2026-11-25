# compmd-lsp

The [CompMD](https://compmd.dev/) language server ([its design](https://github.com/comp-markdown/compmd/blob/main/docs/language-server.md)): one binary, `compmd-lsp`, which an editor starts and speaks LSP with over its standard input and output (`compmd-lsp --stdio`).

- Each open document has a worker of its own, which keeps the document's processor and applies the writer's edits to it incrementally; the diagnostics are published after each, with their code, severity, related places and a link to the spec's catalog.
- A document's root, configuration and packs are read as its hosts read them (`compmd-project`), so the editor shows what the site renders; its components are analysed too (W104, `compmd-components`).
- The settings are the editor's, under `compmd` (`workspace/configuration`, or `initializationOptions`): `packs.suggest`, `packs.network`, `preview.browser` and `trace`.
- A change to the configuration, a package, a lock file or an imported file reloads the open documents.
- Completion, hover, signature help and go to definition, from the processor's hints (`hint_at`, `signature_at`, `definition_at`): components, params and their values, slots, frontmatter keys, types, `md.` paths, and functions with their documentation from the processor's catalog; a component of the project's folder or installed packs completes with the import that takes it, in the frontmatter's own syntax.
- Document symbols (headings, containers and their slots, leaves, definitions, nested as the document nests them), folding (the frontmatter, containers, definitions, code blocks), and `compmd/blocks`, a request of CompMD's own: each code block a code renderer renders, with its language, its renderer and the range of its text, for an editor package to hand to the writer's server for that language.
- Semantic tokens (`textDocument/semanticTokens/full` and `/range`) by the token names every CompMD grammar shares (`compmd-grammars`' `syntax/tokens.json`), from the processor's `tokens()`; a name that stands for no component has the modifier `unknown`.
- Positions are counted in UTF-32 or UTF-8 when the editor offers them, else UTF-16.

## Status

In development, released with CompMD's packages at the same version. It isn't published on crates.io.

**Versions:**
- **0.11.0:** the first version: documents, their diagnostics, settings, watched files, semantic tokens, completion, hover, signature help and go to definition.
