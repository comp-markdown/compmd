+++
id = "01a2044a-d9ec-7265-9e20-0f5b2d98a08f"
number = 66
title = "Gallery: each card’s image cropped to one shape, its cards one height"
status = "approved"
tasks = ["CMD-367"]
author = "claude:79612601"
reviewers = []
created = 2026-11-22T17:25:44Z
updated = 2026-11-22T17:38:31Z
+++

CMD-367: the gallery's images cropped to one shape, and its cards one height (gallery 0.2.0).

## What to check
- **`compmd-packs/gallery/index.comp.md`:**
  - `ratio` (a string, such as `4/3`): each card's image goes in a box with `aspect-ratio`, an image box (CMD-366), which crops it. `image-fit` (`contain | cover | fill`, `cover` by default) and `image-position` (`center` by default) are passed on as its `object-fit` and `object-position`. Without `ratio` the image is as before.
  - The grid has `grid-auto-rows=1fr`, so every card is as tall as the tallest, across rows too (CMD-369).
  - The README, the document's text and the releases note are updated.
- **Two choices to check:**
  - `ratio` is a `string` because no `ratio` type can be declared: §7.3's types don't include it, though `aspect-ratio` takes one. A value that isn't a ratio isn't caught by the processor; the box ignores it. Adding `ratio` to the declarable types would be a spec change.
  - Cards are now always one height. That changes 0.1.0's look for a grid whose descriptions differ in length.
- **The card's image slot** needs nothing of its own: a document crops one with a box around its image, as any image box.
- **The demo:** two new examples with photos of different shapes (a tall lighthouse and a wide shore, beside the harbour): cropped to 4:3, and whole in it with `image-fit=contain image-position="start center"`.
- **Tests:**
  - `test/gallery.test.ts`: the expanded tree, `grid-auto-rows`, the image box's params, and no box without `ratio`.
  - `test/page.test.ts`, new, in Chrome and WebKit: the cards one height, the photos 4:3 and covering, whole with `contain` at the top, and printed cropped (`emulateMedia`). It needs `playwright` and `vite`, the lockfile's only change.

## Verified
The gallery's tests (16, both browsers) and its lint pass. The full `cargo make ci` stopped at the sandbox's lint, in `npm-packages/sandbox/src/refused-styles.ts`, which is the maintainer's uncommitted work, outside this commit; the packages after it didn't run in that ci.
