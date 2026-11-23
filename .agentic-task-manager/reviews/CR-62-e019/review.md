+++
id = "01a20426-e658-70b3-ab68-e89f27eae019"
number = 62
title = "Every pack’s demo as render=both examples in # sections"
status = "approved"
tasks = ["CMD-368"]
author = "claude:79612601"
reviewers = []
created = 2026-11-22T16:46:27Z
updated = 2026-11-22T17:15:57Z
+++

CMD-368, from CR-45 and CR-51: every pack's demo as `compmd {render=both}` examples, each a `#` section named for what it shows, with no demo title and no bold lead-ins.

## What to check
- **Mermaid and wireframe:** their demos rewritten as examples, one per use (a block, a sequence, a diagram from data, ELK, a theme, sizes, a mistake; a phone screen, a settings window, data, a narrow box, every part, a mistake). Each example's frontmatter imports the pack, and its data (`steps`, `tabs`) is its own. The page tests find drawings in the frames only, since the page now shows each example's source; the wireframe test counts six examples and the one framed fallback.
- **card, figure, gallery, link-card, rating, sticky-note:** the title dropped, each bold lead-in a `#` section with its line below.
- **create-pack:** the leaf and renderer templates, and the demo `wrap.ts` writes for a wrapped package, in the same form.
- **The rule** in `docs/writing-a-pack.md`, decision 3.

## Verified
Each changed pack's tests and create-pack's pass, the Mermaid and wireframe page tests in Chrome and WebKit. The full `cargo make ci` was red only in `compmd-packs/mermaid/test/strict-style.test.ts`, which someone else's uncommitted work in the tree changed, outside this commit; pnpm stops at the first failing package, so the packages after it weren't run by that ci.
