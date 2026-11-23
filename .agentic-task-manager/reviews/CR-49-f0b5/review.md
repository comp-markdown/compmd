+++
id = "01a1feaf-a323-755c-8b13-29050643f0b5"
number = 49
title = "The design: an image cropped to its box"
status = "approved"
tasks = ["CMD-366"]
author = "claude:79612601"
reviewers = []
created = 2026-11-21T15:18:05Z
updated = 2026-11-21T15:39:01Z
+++

The design for cropping an image to its box (CMD-366, from CR-45), before any spec change or code: `docs/cropping-images.md`.

## To settle
- **Decision 1:** a box whose expanded content is one image fits it by `object-fit` and `object-position`, cropped by the box's height (`aspect-ratio`, `height`, `fill`, a grid row). Rejected: a param list on the image, an `image` component.
- **Decision 2:** found on the expanded tree, so a definition can make one around content it is given (the card's slot, the gallery's item).
- **Decision 3:** `object-fit` and `object-position` on a box that is neither a host-language component's nor an image box: W125.
- **Decisions 4 and 5:** no breakpoints for them, as now; print as shown; a plain reader shows the image whole.
- **Gaps:** a box with several images or an image and text isn't one; raw HTML `<img>` is kept out.

Once it is approved: the spec (§9.2, §10, §12.3, the changelog, conformance), the processor's W125, the renderer's `compmd-image-box`, the Author's Guide; then CMD-367 uses it in the gallery.
