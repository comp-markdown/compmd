## (the review) · round 1 · t1
- [x] **claude:a7b49598** · 2026-11-23T16:37:25Z · c1
  The post *The first packs*, with the changes asked for on 2026-11-23 (fa9ec87).

  ## Since round 1
  - **The gallery:** two cards a row from `sm` up, one on a phone (`columns=1 sm:columns=2`), in place of four squeezed into one row. Its titles were larger than the section's own heading: an example is a document of its own, so the cards' titles had no heading above them and became top-level headings. The example now has `### From the harbour` above the gallery, as a real post would, so the titles are small. The paragraph after it explains `sm:` and `columns`.
  - **A figure, beside its text, and columns:** one section in place of two. The orchard's `figure` is in the first column, with a paragraph about the orchard in the wider last one (`split=wide-end`). Right below, three even columns show `:: #middle`. Their text is the opening of *Alice's Adventures in Wonderland* (1865, public domain), a paragraph a column, with no headings or lists, so they don't read as a table. The third paragraph is shortened: a parenthetical left out, and it ends at "ran across the field after it."
  - **The sticky note:**
    - It now uses the pack's own look, from `@comp-markdown/sticky-note` 0.2.0 (CMD-386, CR-76): classic yellow, a red pin, a turned-up corner. The example sets no colours.
    - The walk has a second paragraph, and its box keeps room on its right for the note from `sm` up (`sm:padding-inline-end=13rem`). The note no longer covers the text or gets cut off at the example's bottom.
    - The text says the note is yellow, pinned and turned up, and that a use changes it with `{background=subtle color=accent}`.

  Checked in Chrome at desktop and phone widths, light and dark. `cargo make site` passes.

  ## Before publishing
  As before, the packs it imports must be on npm, now with sticky-note 0.2.0, released with `cargo make release-packs`.
