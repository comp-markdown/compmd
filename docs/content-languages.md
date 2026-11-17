# Content languages in editors (proposed)

A code renderer's signature already says its block's content is Mermaid (`renders: mermaid`, spec §7.7). The same
declaration can tell an editor how to treat that content: an editor that knows CompMD highlights, and maybe completes,
Mermaid in any document that imports the pack, without knowing Mermaid itself, because the pack brings what it takes.
This proposes what a pack declares, what it ships, and who uses it.

## Decisions to make

1. **Where a language is named.**
   - (a) A code renderer's `renders` only, as today: a fence's content has the language its renderer names.
   - (b) Also a slot's content: a slot declaration names a language (`slots: { query: { language: sql } }`), for a
     component whose slot is code, not Markdown. A spec change (§7.3), for a later minor version.
   - *Recommended:* (a) now, (b) when a pack needs it; nothing here depends on (b).

2. **What a pack ships.**
   - (a) A TextMate grammar (`.tmLanguage.json`): what VS Code and Shiki read, so the playground, the Developer Tools
     and the site's highlighting (Shiki, in the frames) use it at once.
   - (b) A tree-sitter grammar (WebAssembly): what Neovim, Helix and Zed read, and [the CompMD grammar](tree-sitter.md)
     can inject.
   - (c) Either, or both.
   - *Recommended:* (c), declared in the pack's `package.json`, `compmd.languages: [{ "id": "mermaid", "textmate":
     "grammars/mermaid.tmLanguage.json", "treeSitter": "grammars/mermaid.wasm" }]`, each a file of the pack, checked
     by `compmd-pack check` (it parses, it names the language its renderer `renders`).

3. **Completion and diagnostics: a language server.**
   - (a) None: grammars only. Highlighting, no completion.
   - (b) A pack ships a language server, which the CompMD language server starts and forwards the block's requests to,
     as editors forward an embedded language's.
   - *Recommended:* (a) by default, and (b) only for packs the writer trusts by name in their editor's settings: a
     server is a program run on the writer's machine, with their files and network, unlike a grammar, which is data.
     It never follows from importing a pack.

4. **Who reads them.**
   - The playground and the Developer Tools: the frames' Shiki loads a pack's TextMate grammar with the pack, for the
     blocks its renderer renders.
   - A VS Code extension (to come): its CompMD grammar embeds each imported pack's grammar for its fences, found from
     the document's imports and the installed packs.
   - The tree-sitter grammar: injects by the fence's language name, as Markdown's does; an editor that has a grammar
     for that name highlights it, from the pack or from elsewhere.
   - *Recommended:* the playground first, since it already has Shiki in its frames and the packs at hand.

5. **Safety of a grammar.** A TextMate grammar is regular expressions, and a bad one backtracks for ever on some text.
   Highlighting with a pack's grammar runs where the pack's code runs, in its frame (the playground's Shiki is there
   already), with the frame's timeout, so a grammar can stall at most its own block.

## What it changes

- `@compmd/pack`: `compmd.languages` in a pack's manifest, checked; the test page highlights with it.
- `@compmd/sandbox`: the frames' highlighter takes a pack's grammar for its renderer's language.
- The Mermaid pack ships a Mermaid grammar, as the first.
- No spec change, unless decision 1 (b).

## Open

- Whether a language's `id` is the fence's language as written (`mermaid`), so two packs may name the same one (the
  document's renderer decides which, as it decides rendering), or scoped to the pack. Proposed: as written.
