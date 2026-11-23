+++
id = "01a1fef4-82a6-7350-9a26-636a5d2d2f69"
number = 53
title = "An image cropped to its box: the spec, W125, the renderer"
status = "approved"
tasks = ["CMD-366"]
author = "claude:79612601"
reviewers = []
created = 2026-11-21T16:33:19Z
updated = 2026-11-21T16:48:47Z
+++

CMD-366 built from [its approved design](docs/cropping-images.md) (CR-49): an image box fits its image by `object-fit` and `object-position`, so the box's height crops it.

## What to check
- **The spec (§9.2, §10.2):** the image box defined under `object-fit` and `object-position`; W125 on a box that is neither a host-language component's nor an image box. The changelog (breaking only for a box around one image that had the params), `llms.txt`, the Author's Guide (chapters 3 and 20), the Developer's Guide (chapter 22).
- **One change from the design:** W125 is at the box, not at the param, since the expanded tree, where an image box is found, keeps no param's place. The design says so now.
- **The processor:** `expand/image_box.rs` (one paragraph, one image or a link holding only one, white space aside; an image reference counts), W125 in `expand/layout.rs`. Conformance: `validate/image-box-W125` (a square crop, a linked image with a position, one made by a definition around its slot, one with a caption: W125, one with no image: W125); `validate/object-fit-and-position`'s box with text now has W125.
- **The renderer:** `image-box.ts` marks the box `compmd-image-box`, its fit as `--compmd-image-fit` and `--compmd-image-position` (resolved with the box's direction, by `fitOf`), and its base rules make the box one grid cell the image fills. `test/browser/images.test.ts` in Chrome and WebKit: a square and a 16:9 crop inside the padding, `contain` by default, `object-position`, no height giving the image's own, a linked image, a captioned one left alone.
- **Not here: a print test.** The rules have no media query, so they print as they show, but the renderer's browser tests can't emulate print; the gallery's page test (CMD-367) prints cropped images.
