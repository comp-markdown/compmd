+++
id = "01a200e6-a5c6-73db-95b9-e9359aabcc7b"
number = 60
title = "Fonts of a document’s own: the spec and the processor"
status = "approved"
tasks = ["CMD-373"]
author = "claude:79612601"
reviewers = []
created = 2026-11-22T01:37:25Z
updated = 2026-11-22T15:39:02Z
+++

CMD-373, [Type in a box](docs/type-params.md) decision 7 (CR-54): fonts of a document's own, the spec and the processor. The renderer, the hosts, the sandbox, the pack tool and the Author's Guide follow in CMD-376.

## Settled in conversation, in decision 7
- `fonts` beside `names` in imports and exports, with what `names` takes (a list, `"*"`, `{name, as}`); an import or export has either or both.
- `fallback` is optional, `body` by default.
- A font name resolves where it is written, an expression's value too: a pack's definition can't be given its caller's font.
- A font file the host can't serve shows the fallback, as a missing image its alt: F026 only for a path leaving its root (the processor's hosts give it text, and it checks no image either). The pack tool checks a pack's font files in CMD-376.

## What to check
- **The spec:** §9.4, a bullet *Fonts of a document's own* (the declaration, its name, files, fallback; a pack's fonts; lexical names; files served as images are, `@font-face` with `font-display: swap`). §2.1 (`md.fonts`), §6 and §6.1–6.3 (`fonts` in imports and exports, `"*"`, names apart from components', the levels and F013, F021, F023), §8.6 (a font's files are relative paths), §9.2 (`font-family`'s row and paragraph), §10.2 (F009, F013, F021, F023, F026), §11 (stage 2's F009), §12.3 (the root's `fonts`, a param naming one by its `family`, `-2`, `-3` for a second font of the same name). The changelog.
- **The processor:**
  - `settings/fonts.rs` reads `md.fonts` with its places; `settings.rs`: `names` and `fonts` both optional on an import and an export (one needed).
  - `resolve/fonts.rs`: the font table, `Fallback`, `Weight`, `Style`, and a document's font names by level, apart from its components'. `resolve/load/fonts.rs`: each declaration checked (F009, F013, F026), imports (F013, F021; a failed or pending one's fonts absent), exports, each font's family in the tree.
  - `validate/params/font_family.rs`: a token or a font of the document the param is written in (F009, F023).
  - `expand/fonts.rs`: a value becomes the font's family (F009 at render for an expression naming none), and the root lists the fonts its params use, files resolved as destinations are (`expand/blocks.rs` `rooted`).
  - `Form::Font` and `Kind::FONT_FAMILY` (`params.json`: `font-family` has the form `font`), `md.fonts` among the known keys, `fonts` in the hints for imports and exports, `fonts` in a frontmatter's imports.
- **Conformance:** `expand/fonts-own-and-a-packs` (a document's own font and a pack's of the same name, renamed on import, a fence using its pack's font, an unused font left out), `expand/font-family-expression-F009`, `resolve/font-declarations` (every check on a declaration), `resolve/font-imports` (F013, F021, F023, a failed import), `resolve/font-imports-pending`; the tree schema's `fonts`; the README.
