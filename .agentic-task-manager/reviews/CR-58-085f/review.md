+++
id = "01a1ffbf-1da6-7296-9884-075bdce0085f"
number = 58
title = "A document’s lang and dir reach its page and its examples"
status = "approved"
tasks = ["CMD-371"]
author = "claude:79612601"
reviewers = []
created = 2026-11-21T20:14:37Z
updated = 2026-11-21T20:26:15Z
+++

CMD-371, as you chose: a document's `lang` and `dir` reach its rendered page and an example's output.

## What to check
- **One narrowing:** the root has `lang` and `dir` only when the document writes `md.lang` or `md.dir` (`dir` from the language's script when only `lang` is), not always. Always would have changed all 54 conformance trees, and a document that writes neither keeps where it is shown's, as before.
- **The spec:** §2.1's `dir` row, §12.3 (the root's `lang` and `dir`, the renderer setting them, a host writing HTML keeping them), §7.8's example root; the changelog; the tree schema (both roots).
- **The processor:** `direction.rs` (RTL scripts and languages after CLDR, a script subtag deciding over the language; unit tests), `Settings` reading `md.dir`, the expanded `Root`'s two fields. Conformance: `expand/document-lang-dir` (an Arabic document, an English example, a Hebrew one written `ltr`, one with neither); `value-and-formatting` and `complete-example-expanded`, which write `md.lang: en`, now have `lang` and `dir`.
- **The renderer:** `language.ts` sets them on the page's element and an example's box, and takes back only what it set; `test/browser/language.test.ts` in Chrome and WebKit (an Arabic document's grid right to left, an Arabic example in an English page, a document with neither keeping its host's `dir`).
- **The blog** keeps the element the renderer set them on when a post has them, with a test; the Vite plugin renders in the page, so it needs nothing.
- **The columns demo's** right-to-left example uses `md.lang: ar` now, and its page test still finds the first column on the right.
