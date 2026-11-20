+++
id = "01a1f9f9-495f-7382-af67-556e44d45934"
number = 29
title = "Wrapping an npm package as a pack: the design"
status = "open"
tasks = ["CMD-8"]
author = "claude:79612601"
reviewers = []
created = 2026-11-20T17:20:26Z
updated = 2026-11-20T19:56:57Z
+++

Round 4: a second design, [sizing a component in its box](docs/sizing-components.md), from challenging "fills the width" in conversation.

What to check in it:

- **Decisions 1 and 2:** a component fits its box (its width, and its height when the box has one, aspect ratio kept and centred, or filled when it has none), and the box a component sees is exactly the use's box: the sandbox applies the box's params twice today, which is the scroll bars.
- **Decision 3, a spec change:** a signature fence takes box defaults (`{width=12rem max-width=100%}`), as a host-language definition fence already does; F036 becomes "both fences style the box".
- **Decision 4, a spec change:** the common params' names, `lang` and `dir` are F013 in a signature, as `class` and `id` are.
- **Decision 5:** "sizes include padding and border". **Decision 6:** no cover, stretch or placement yet.
- **Conformance cases:** every case read; three change (`definitions/fence-params`, `definitions/signature-types`, `resolve/params-differ-in-dash-F013`), four new ones, and the tree schema's signature fence gains `params`.
- **The spike:** each demo has the sizing cases with magenta box borders; `try.ts` measures each drawing against its box and reports frames that scroll. Its README has findings 11 to 13.
- Writing a pack's decision 8 now points to the new design; the wrapping design's mapping and G8 follow it.
