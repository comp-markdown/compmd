+++
id = "01a205d1-a723-71cc-9c45-48ed9308990e"
number = 70
title = "A both block: its code folded under one bar, numbered, with Copy"
status = "approved"
tasks = ["CMD-379"]
author = "claude:79612601"
reviewers = []
created = 2026-11-23T00:32:35Z
updated = 2026-11-23T02:12:56Z
+++

Round 2: every code block gets the frame, as the maintainer asked; a `render=both` block's frame folds, as in round 1.

## What changed since round 1
- **`code-frame.ts`** replaces `folded-code.ts`: `CodeFrame` puts any `<pre>` in a frame, a bar over it naming its language and its lines, with **Copy**; with `folds`, **Show code** too. `CodeBlockView` makes one for every code block; a `both` block's folds, and `outputTag` names its output.
- **No text of its own:** the bar's words come from its attributes through CSS (`CODE_FRAME_RULES`, in the base rules and exported), so a page's text, and what a reader selects, stay the document's. Round 1's tests that read "mermaid1 lineCopyShow code" are back to the code alone.
- **Buttons that work in static HTML:** round 1's buttons had listeners of their own, so they did nothing on a blog post built ahead with no script. One listener on the document (`followCodeFrames`) now works them, installed by the renderer and by the site's page script; it marks the document with a registered symbol, so two copies of the renderer in one page (the site's script and a live post's) add one listener, not two that cancel each other.
- **`codeOf`** finds a code block's `<code>` in its frame.
- **The site:** the spec's and the guides' code blocks (Shiki, at build time) are framed by `CodeFrame` in `build/spec.ts`; the page script adopts the frame's rules and follows the frames. The look moved to `base.css`, for every page, the playground's proof too; `.spec pre` no longer styles a framed block; line numbers count Shiki's lines as well as the renderer's. The blog's theme keeps its sheet only for an example that shows its output alone.

## Checked
Unit and browser tests (Chrome, WebKit) for the frame and the listener; a build test that every guide code block is framed. Screenshots of a blog post and a guide page, light and dark. `cargo make ci` and `cargo make site` pass.
