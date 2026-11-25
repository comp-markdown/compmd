# compmd-lsp

The [CompMD](https://compmd.dev/) language server ([its design](https://github.com/comp-markdown/compmd/blob/main/docs/language-server.md)): one binary, `compmd-lsp`, which an editor starts and speaks LSP with over its standard input and output (`compmd-lsp --stdio`).

- Each open document has a worker of its own, which keeps the document's processor and applies the writer's edits to it incrementally; the diagnostics are published after each, with their code, severity, related places and a link to the spec's catalog.
- A document's root, configuration and packs are read as its hosts read them (`compmd-project`), so the editor shows what the site renders; its components are analysed too (W104, `compmd-components`).
- The settings are the editor's, under `compmd` (`workspace/configuration`, or `initializationOptions`): `packs.suggest`, `packs.network`, `preview.browser` and `trace`.
- A change to the configuration, a package, a lock file or an imported file reloads the open documents.
- Semantic tokens (`textDocument/semanticTokens/full` and `/range`) by the token names every CompMD grammar shares (`compmd-grammars`' `syntax/tokens.json`), from the processor's `tokens()`; a name that stands for no component has the modifier `unknown`.
- Positions are counted in UTF-32 or UTF-8 when the editor offers them, else UTF-16.

## Status

In development, released with CompMD's packages at the same version. It isn't published on crates.io.

**Versions:**
- **0.11.0:** the first version: documents, their diagnostics, settings, watched files and semantic tokens.
