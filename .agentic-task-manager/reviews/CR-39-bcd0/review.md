+++
id = "01a1fb58-f859-7639-983d-4dbae78dbcd0"
number = 39
title = "Sizing: a framed use is laid out as in the page"
status = "approved"
tasks = ["CMD-355"]
author = "claude:79612601"
reviewers = []
created = 2026-11-20T23:44:34Z
updated = 2026-11-20T23:53:59Z
+++

[Sizing a component in its box](docs/sizing-components.md), decision 2 and *The mechanism*, in `@compmd/sandbox`: a framed use is laid out as in the page, and its frame no longer scrolls.

## What to check

- **The sizer** (`sizer.ts`, `use.ts`): the box's one row is its whole inside; the sizer fills it and holds an invisible block of the component's natural size; the frame is positioned over the sizer. A frame is no longer as tall as its content.
- **What the frame is sent** (`framed-node.ts`): the use without the box's params, but for `object-fit`, `object-position` and `scroll`, and the box's direction and language as the page has them. `@compmd/render` exports `BOX_PARAMS` and `isRtl` for it.
- **In the frame** (`frame/use-layout.ts`, `frame/runtime.ts`, `build/frames.ts`): `#part` is a grid of one cell, the frame's size; the document never scrolls; the natural size is measured height first, then width with the box laid out as a block.
- **The protocol, version 3**: `size` is a use frame's natural size; `box {width}` from the page, when the frame loads and when the sizer changes size. The schema, `docs/sandbox-protocol.md`, and Q5 of *Running components in the sandbox* say it.

## One change from the design

**`box` carries the width of the box's inside,** and the frame measures its height at that width, not at its own. The prototype measured at the frame's width. The comparison test showed that off screen, Chrome doesn't give a frame its new size, so a box whose width comes from its content (in a row, inline) kept a height of a pixel or so until scrolled to. The design's *The mechanism* should get this line; I left the design as approved and can update it in a round.

## Tests

`test/sizing.test.ts`, in Chrome and WebKit (54 pass): each of the prototype's 21 cases (28 uses) drawn by a trusted component in the page and by the same code in a strict frame, their boxes and what each drew compared; the grid rows after they shrink; all 28 frames off screen (boxes only: a frame's own observers are paused there, so what its component reports is stale); and the four the prototype doesn't run: `object-position` right to left, `scroll`, a `canvas`, and a drawing that doesn't contain its size. Protocol unit tests for `box`, including a `box` with no width or a bad one, refused.

One existing test changed: a fractional natural height (120.4) is now kept, as in the page, where it was rounded up to stop a scroll bar that can no longer appear.

## Two things to know

- A bug I made and fixed on the way: measuring changed an attribute under `body`, which the frame's own `MutationObserver` watches, so each measurement started the next and the frame hung. The width now goes through a CSS variable on the root, with a comment saying why.
- The `row` case timed out once at the old 15 s limit when run with the others, and passed alone and in every run since. I haven't found why. Its limit is now 60 s, with the difference printed if it fails.
