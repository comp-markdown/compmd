+++
id = "01a20440-ff86-73e3-bea9-62191d1814a3"
number = 65
title = "Fonts of a document’s own: rendering, the frames, the pack tool"
status = "approved"
tasks = ["CMD-376"]
author = "claude:79612601"
reviewers = []
created = 2026-11-22T17:14:58Z
updated = 2026-11-22T17:15:46Z
+++

CMD-376, [Type in a box](docs/type-params.md) decision 7, after CMD-373: the expanded tree's fonts shown. Two commits: `302aa2a` (the renderer, host mode, the page frame's files, the pack tool, the Author's Guide) and `5381991` (the frames' policy and component frames, as decided with the maintainer). The range holds other tasks' commits between them (CMD-368, CMD-375, CMD-377), reviewed in their own reviews.

## What to check
- **The renderer** (`fonts.ts`): each font of the root's `fonts` as `@font-face` rules with `font-display: swap`, in a stylesheet of its own, written only when there are fonts. The face names are `compmd-font-<n>-<family>`, apart for each page and example. A custom property on the page's element holds the face and its fallback, which `font-family` uses (`styles.ts`). A pack's file is reached through `packFile`, and through a `fonts` source when there is one. `drawn()` waits for the rules and `print()` for the fonts. Examples have their own fonts (`example-view.ts`). Splices carry the root's fonts (`tree.ts`, and the WebAssembly document's splice). `fontFacesIn` gives the files of the faces a computed family names.
  - **One choice to check:** with no `--compmd-font-*` set, the fallback is `system-ui, sans-serif` (`ui-monospace, monospace` for `mono`). The tokens' own default, `inherit`, can't sit in a font list.
- **Host mode:** fonts load from their URLs; a registry pack's file the host holds loads from a `blob:` URL (`hostFonts`).
- **Frame mode:** the host's page gives a page frame the files of the fonts of the trees and splices it sent, from its own origin only (`sent-fonts.ts`, `PageImages`). The frame loads them from `blob:` URLs (`fontsFrom`).
- **The policy:** `font-src 'self' blob:` in `strict.html` (`build/policy.ts`) and in a permissive frame's second policy (`frame/narrow.ts`); the sandbox design's Q3 table and a note on why.
- **Component frames:** `use.ts` sends the box's type, then again with the files of the fonts its family names (`box-fonts.ts`: the page's faces fetched once, or in a frame the files its parent sent). The `type-style` message has `fonts`, checked by `frameFontsIn`, in the protocol schema. The frame declares them with `FontFace` from their bytes (`frame/frame-fonts.ts`), and keeps them for a frame nested in it.
- **The pack tool:** a declared font file must be there and published (`checkPack`); a warning for one with no licence in its folder (`packWarnings`, printed by `compmd-pack build`).
- **The Author's Guide:** chapter 3's `font-family` row and a section on fonts of a document's own and from a pack.
- **Tests:**
  - The renderer in Chrome and WebKit: a pack's font loads with its fallback after it, a missing file keeps the fallback, an example keeps its own font of the same name, the page takes its rules away.
  - The sandbox in Chrome and WebKit: a page frame's font loads from its host's file (frame mode, which fails without `blob:`), and a component in a box in the font draws in it in its own frame (host mode).
  - Unit tests for the page frame's files, `hostFonts`, `frameFontsIn`, the policies and the pack tool's checks.
- **Fixture:** KaTeX's `KaTeX_Main-Regular.woff2` (MIT, its licence beside it).

## Verified
The renderer's and the sandbox's whole suites pass, as do the pack tool's and the guides' tests. The full `cargo make ci` passed fmt, clippy, coverage and the Rust tests, then stopped red in `compmd-packs/mermaid/test/strict-style.test.ts`, the maintainer's uncommitted work on CMD-374, outside these commits. pnpm stops at the first failing package, so the packages after it didn't run in that ci.
