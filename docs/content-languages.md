# Content languages in editors

- **Status:** Approved (2026-11-17). Its parts are built as tasks of their own: the pack's manifest and check, the processor's blocks, the playground, a VS Code extension, the tree-sitter grammar's injections.

A CompMD document holds text in languages that aren't Markdown: a code block a code renderer draws (spec §7.7) is a
diagram, a chart's data, a query, a score, in whatever language that renderer reads. The pack that brings the renderer
is the one thing that knows that language; the editor a writer works in knows only the languages it was built or
configured for. So today such a block is plain text in the editor, with no colours, no folding, no errors and no
completion, while the same text in a file of its own would have them all. A document's other content is no better off
than a plain code block in any Markdown file.

This decides how a pack tells editors about the languages its renderers read, what it ships for them, and what an editor
does with it, so that a writer gets the same help inside a CompMD document as in a file of that language. (A Mermaid pack
is the first example: `renders: mermaid` already says its blocks are Mermaid.)

## Decisions

1. **Where a language is named: a code renderer's `renders`.** A fence's content has the language its renderer names,
   as the spec has it now. A slot whose content is code (`slots: { query: { language: sql } }`) would be a spec change,
   left until a pack needs it; nothing here depends on it.

2. **What a pack ships: grammars, TextMate and tree-sitter, either or both.** A TextMate grammar (`.tmLanguage.json`) is
   what VS Code and Shiki read, so the playground, the Developer Tools and the site's highlighting use it at once; a
   tree-sitter grammar (WebAssembly) is what Neovim, Helix and Zed read, and what [the CompMD grammar](tree-sitter.md)
   injects. The pack declares them in its `package.json`, each a file of the pack:

   ```json
   "compmd": { "languages": [{ "id": "mermaid", "textmate": "grammars/mermaid.tmLanguage.json", "treeSitter": "grammars/mermaid.wasm" }] }
   ```

   `compmd-pack check` checks each: it parses, and its `id` is a language the pack's renderers `renders`.

3. **Language servers: none from CompMD; what an editor needs to choose its own.** CompMD neither ships nor starts a
   language server for a content language. An editor, or its CompMD plugin, decides that, as it does for any language,
   with whatever server the writer has set up for it. What CompMD gives is everything that takes:
   - **the blocks**: for each code block a renderer draws, its language `id` and the range of its content in the
     document, from the processor (positions it has already) and from the tree-sitter grammar's injections, so a
     plugin forwards the requests inside that range to the server it chose, as editors do for an embedded language;
   - **the language's names elsewhere**: a pack's language may give the names editors and servers know it by, its
     `aliases`, a `languageId` (VS Code's), and `extensions` (`.mmd`), so a plugin finds the server it has for it.

   The `compmd.languages` entry above takes these as optional keys.

4. **Who reads them: the playground, a VS Code extension, and the tree-sitter grammar, all three.** The playground and
   the Developer Tools load a pack's TextMate grammar into the frames' Shiki for its renderer's blocks; a VS Code
   extension's CompMD grammar embeds each imported pack's grammar for its fences, found from the document's imports and
   the installed packs, and hands the blocks to the language servers the writer has; the tree-sitter grammar injects by
   the fence's language, as Markdown's does, so an editor with a grammar for it highlights it, from the pack or
   elsewhere. Each is a task of its own when its turn comes, in whatever order they are taken.

5. **A language's `id` is the fence's language as written** (`mermaid`), not scoped to the pack: two packs may give the
   same one, and the renderer the document resolves (spec §6.3) decides whose grammar its blocks use, as it decides how
   they render.

6. **A grammar is data, and runs where the pack's code runs.** A TextMate grammar is regular expressions, and a bad one
   backtracks for ever on some text. Highlighting with a pack's grammar happens in its frame (the playground's Shiki is
   there already), with the frame's timeout, so a grammar can stall at most its own block. In an editor it runs in the
   editor's own highlighter, as any grammar the writer installs does.

## What it changes

- `@compmd/pack`: `compmd.languages` in a pack's manifest, checked; the test page highlights with it.
- `@compmd/sandbox`: the frames' highlighter takes a pack's grammar for its renderer's language.
- The processor: each rendered code block's language and content range, for editors (its positions already have them).
- The Mermaid pack ships a Mermaid grammar, as the first.
- No spec change.
